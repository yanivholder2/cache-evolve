use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Region tags. Each region is its own doubly-linked recency list over the shared arena.
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const REGIONS: usize = 3;

/// Initial window share of capacity, in percent.
const WINDOW_PERCENT: usize = 1;
/// Upper bound on the adaptive window share of capacity, in percent.
const MAX_WINDOW_PERCENT: usize = 80;
/// Protected share of the main region, in percent.
const PROTECTED_PERCENT: usize = 80;

/// Hill climber: sample period is `SAMPLE_PERIOD_FACTOR * capacity` requests.
const SAMPLE_PERIOD_FACTOR: usize = 10;
/// Hill climber: initial step magnitude as a fraction of capacity.
const INITIAL_STEP_FRACTION: f32 = 0.0625;
/// Hill climber: per-period decay of the step magnitude.
const STEP_DECAY: f32 = 0.98;
/// Maximum lazy region moves (window->probation, protected->probation) per `put`.
const MAX_REBALANCE_MOVES: usize = 8;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: u8,
}

/// Per-region list: head is LRU, tail is MRU.
#[derive(Debug, Clone, Copy, Default)]
struct RegionList {
	head: Option<usize>,
	tail: Option<usize>,
	len: usize,
}

/// W-TinyLFU with an adaptive (hill-climbed) window: a LRU admission window in front of a
/// segmented-LRU main region (probation + protected). The TinyLFU frequency contest runs only
/// between the window evictee (candidate) and the main-region victim (probation LRU, else
/// protected LRU). The window/main split is tuned at runtime by a Caffeine-style hill climber;
/// resizes are applied lazily (bounded moves per `put`, never an eviction).
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	lists: [RegionList; REGIONS],
	capacity: usize,
	window_cap: usize,
	max_window_cap: usize,
	main_cap: usize,
	protected_cap: usize,
	sketch: CountMinSketch,
	// Hill climber state.
	sample_hits: u32,
	sample_misses: u32,
	sample_period: u32,
	prev_hit_rate: f32,
	step: i32,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let window_cap = (capacity * WINDOW_PERCENT / 100).max(1);
		let max_window_cap = (capacity * MAX_WINDOW_PERCENT / 100).max(window_cap);
		let main_cap = capacity - window_cap;
		let protected_cap = main_cap * PROTECTED_PERCENT / 100;
		let sample_period = (capacity.saturating_mul(SAMPLE_PERIOD_FACTOR)).min(u32::MAX as usize) as u32;
		let step = ((capacity as f32 * INITIAL_STEP_FRACTION) as i32).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window={window_cap} max_window={max_window_cap} main={main_cap} protected={protected_cap} sketch_width={width} sample_period={sample_period} step={step}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [RegionList::default(); REGIONS],
			capacity,
			window_cap,
			max_window_cap,
			main_cap,
			protected_cap,
			sketch: CountMinSketch::new(width),
			sample_hits: 0,
			sample_misses: 0,
			sample_period,
			prev_hit_rate: 0.0,
			step,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		match self.map.get(key).copied() {
			Some(idx) => {
				self.sample_hits += 1;
				self.maybe_climb();
				self.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.sample_misses += 1;
				self.maybe_climb();
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		self.put_inner(key, value);
		self.rebalance(MAX_REBALANCE_MOVES);
	}

	fn put_inner(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		// Not full: fresh arena slot at window MRU; overflow the window LRU into probation.
		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				region: WINDOW,
			});
			self.push_back(idx, WINDOW);
			self.map.insert(key, idx);
			if self.lists[WINDOW as usize].len > self.window_cap {
				let candidate = self.lists[WINDOW as usize]
					.head
					.expect("TinyLfuCache: overfull window has no LRU");
				// Window over cap while total <= capacity implies main is not full.
				self.unlink(candidate);
				self.push_back(candidate, PROBATION);
			}
			return;
		}

		let victim = self.lists[PROBATION as usize]
			.head
			.or(self.lists[PROTECTED as usize].head);

		// Window grew (adaptive resize) and is under its cap: fill it naturally by keeping the
		// window intact and evicting the main-region victim to make room for the newcomer.
		if self.lists[WINDOW as usize].len < self.window_cap {
			if let Some(victim) = victim {
				self.unlink(victim);
				self.map.remove(&self.nodes[victim].key);
				self.write_newcomer(victim, key, value);
				return;
			}
		}

		// Full: pick the evictee first, then write the newcomer into its slot.
		let candidate = self.lists[WINDOW as usize]
			.head
			.expect("TinyLfuCache: full cache has empty window");

		let slot = match victim {
			Some(victim) if self.main_cap > 0 => {
				let candidate_freq = self.sketch.estimate(&self.nodes[candidate].key);
				let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
				if candidate_freq > victim_freq {
					// Admit candidate into probation; evict the main-region victim.
					self.unlink(victim);
					self.map.remove(&self.nodes[victim].key);
					self.unlink(candidate);
					self.push_back(candidate, PROBATION);
					victim
				} else {
					self.unlink(candidate);
					self.map.remove(&self.nodes[candidate].key);
					candidate
				}
			}
			// No main region (e.g. capacity == 1): plain LRU window, no contest.
			_ => {
				self.unlink(candidate);
				self.map.remove(&self.nodes[candidate].key);
				candidate
			}
		};

		self.write_newcomer(slot, key, value);
	}

	#[inline]
	fn write_newcomer(&mut self, slot: usize, key: K, value: V) {
		self.nodes[slot].key = key.clone();
		self.nodes[slot].value = value;
		self.push_back(slot, WINDOW);
		self.map.insert(key, slot);
	}

	/// Hill climber: at the end of each sample period, step the window size in the direction
	/// that last improved the hit rate (reverse on a drop), with a decaying step magnitude.
	#[inline]
	fn maybe_climb(&mut self) {
		let total = self.sample_hits + self.sample_misses;
		if total < self.sample_period {
			return;
		}
		self.climb(total);
	}

	#[cold]
	fn climb(&mut self, total: u32) {
		let hr = self.sample_hits as f32 / total as f32;
		if hr < self.prev_hit_rate {
			self.step = -self.step;
		}
		let magnitude = ((self.step.unsigned_abs() as f32 * STEP_DECAY) as i32).max(1);
		self.step = if self.step < 0 { -magnitude } else { magnitude };

		let new_window = (self.window_cap as i64 + self.step as i64)
			.clamp(1, self.max_window_cap as i64) as usize;
		if new_window != self.window_cap {
			self.set_window_cap(new_window);
		}

		self.prev_hit_rate = hr;
		self.sample_hits = 0;
		self.sample_misses = 0;
	}

	fn set_window_cap(&mut self, window_cap: usize) {
		self.window_cap = window_cap;
		self.main_cap = self.capacity - window_cap;
		self.protected_cap = self.main_cap * PROTECTED_PERCENT / 100;
	}

	/// Lazily apply a resize with at most `budget` moves; never evicts. Excess carries over to
	/// later calls.
	fn rebalance(&mut self, budget: usize) {
		let mut moves = 0;
		while moves < budget && self.lists[WINDOW as usize].len > self.window_cap {
			let lru = self.lists[WINDOW as usize]
				.head
				.expect("TinyLfuCache: overfull window has no LRU");
			self.unlink(lru);
			self.push_back(lru, PROBATION);
			moves += 1;
		}
		while moves < budget && self.lists[PROTECTED as usize].len > self.protected_cap {
			let lru = self.lists[PROTECTED as usize]
				.head
				.expect("TinyLfuCache: overfull protected has no LRU");
			self.unlink(lru);
			self.push_back(lru, PROBATION);
			moves += 1;
		}
	}

	/// Region-aware promotion on access.
	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region {
			WINDOW => {
				self.unlink(idx);
				self.push_back(idx, WINDOW);
			}
			PROBATION => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
				if self.lists[PROTECTED as usize].len > self.protected_cap {
					let demoted = self.lists[PROTECTED as usize]
						.head
						.expect("TinyLfuCache: overfull protected has no LRU");
					self.unlink(demoted);
					self.push_back(demoted, PROBATION);
				}
			}
			_ => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
			}
		}
	}

	/// Unlink `idx` from the region list named by its tag.
	fn unlink(&mut self, idx: usize) {
		let region = self.nodes[idx].region as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.lists[region].head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.lists[region].tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.lists[region].len -= 1;
	}

	/// Append `idx` at the MRU end of `region` and retag it.
	fn push_back(&mut self, idx: usize, region: u8) {
		let r = region as usize;
		self.nodes[idx].region = region;
		self.nodes[idx].prev = self.lists[r].tail;
		self.nodes[idx].next = None;
		match self.lists[r].tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.lists[r].head = Some(idx),
		}
		self.lists[r].tail = Some(idx);
		self.lists[r].len += 1;
	}

	/// Structural self-check (tests only). Region caps may be transiently exceeded by a lazy
	/// resize; they must hold once pending moves are drained.
	#[cfg(test)]
	fn check_structure(&mut self) {
		assert!(self.map.len() <= self.capacity);
		assert_eq!(self.nodes.len(), self.map.len());
		assert!(self.window_cap >= 1 && self.window_cap <= self.max_window_cap);
		assert!(self.max_window_cap <= self.capacity);
		assert_eq!(self.main_cap, self.capacity - self.window_cap);
		let mut seen = 0;
		for r in 0..REGIONS {
			let mut count = 0;
			let mut prev = None;
			let mut cur = self.lists[r].head;
			while let Some(i) = cur {
				assert_eq!(self.nodes[i].region as usize, r);
				assert_eq!(self.nodes[i].prev, prev);
				assert_eq!(self.map.get(&self.nodes[i].key), Some(&i));
				prev = Some(i);
				cur = self.nodes[i].next;
				count += 1;
			}
			assert_eq!(self.lists[r].tail, prev);
			assert_eq!(self.lists[r].len, count);
			seen += count;
		}
		assert_eq!(seen, self.map.len());
		self.rebalance(usize::MAX);
		assert!(self.lists[WINDOW as usize].len <= self.window_cap);
		assert!(self.lists[PROTECTED as usize].len <= self.protected_cap);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self
			.counts
			.iter()
			.enumerate()
			.map(|(row, _)| self.counts[row][self.position(key, row as u64)])
			.min()
			.unwrap_or(0)
	}

	fn position<K: Hash>(&self, key: &K, seed: u64) -> usize {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(seed);
		key.hash(&mut hasher);
		(hasher.finish() as usize) & (self.width - 1)
	}
}

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		TinyLfuCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		TinyLfuCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn eviction_prefers_hot_items() {
		// capacity 2: window=1, probation=1, protected=0.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30); // candidate 2 loses to victim 1

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn newcomer_always_enters_window() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30); // cold newcomer still admitted to the window

		assert!(cache.get(&3).is_some());
		assert!(cache.get(&1).is_some());
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn hot_window_candidate_displaces_cold_victim() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // 1 -> probation, 2 in window
		cache.get(&2);
		cache.get(&2);
		cache.put(3, 30); // candidate 2 beats victim 1

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_some());
		assert!(cache.get(&1).is_none());
	}

	#[test]
	fn update_promotes_existing_item_and_admits_hot_arrivals() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn capacity_one_is_plain_window() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		cache.check_structure();
	}

	#[test]
	fn hill_climber_moves_window() {
		let capacity = 200usize;
		let mut cache = TinyLfuCache::<i64, i64>::new(capacity);
		let initial = cache.window_cap;
		// Pure recency-friendly loop over slightly more than capacity: plenty of sample periods.
		for i in 0..(capacity as i64 * SAMPLE_PERIOD_FACTOR as i64 * 3) {
			let key = i % 150;
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
		}
		assert_ne!(cache.window_cap, initial);
		cache.check_structure();
	}

	#[test]
	fn structure_holds_under_workload() {
		for capacity in [1usize, 2, 3, 8, 64, 250] {
			let mut cache = TinyLfuCache::<i64, i64>::new(capacity);
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
			for _ in 0..20_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				// Skewed key mix: half the draws come from a small hot set.
				let key = if x & 1 == 0 { (x % 40) as i64 } else { (x % 600) as i64 };
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
				if let Some(&v) = cache.get(&key) {
					assert_eq!(v, key * 10);
				}
			}
			cache.check_structure();
		}
	}
}
