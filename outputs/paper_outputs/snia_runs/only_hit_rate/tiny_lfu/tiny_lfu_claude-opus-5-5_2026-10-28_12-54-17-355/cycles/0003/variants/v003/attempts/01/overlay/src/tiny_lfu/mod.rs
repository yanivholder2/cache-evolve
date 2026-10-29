use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
/// Initial window size as a percentage of total capacity (W-TinyLFU admission window).
/// The window is resized afterwards by the hill climber (see `climb`).
const WINDOW_PERCENT: usize = 1;
/// Initial hill-climber step as a percentage of capacity.
const INITIAL_STEP_PERCENT: isize = 5;
/// Upper bound on the window as a percentage of capacity.
const MAX_WINDOW_PERCENT: isize = 80;
/// Per-climb multiplicative decay of the step magnitude.
const STEP_DECAY: f64 = 0.98;
/// Smoothed hit-rate drop (absolute) that triggers a direction reversal.
const REVERSE_TOLERANCE: f64 = 0.001;
/// Sampling period in `get` calls, independent of capacity.
const SAMPLE_PERIOD: u64 = 4096;
/// Weight of the previous EWMA value when folding in a new sample's hit rate.
const EWMA_KEEP: f64 = 0.7;
/// Step-magnitude floor as a fraction of capacity (1 / STEP_FLOOR_DIVISOR).
const STEP_FLOOR_DIVISOR: usize = 1000;

/// Which recency list a node currently lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Window,
	Main,
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: Region,
}

/// Head/tail/len of one intrusive recency list over the shared `nodes` arena.
/// Head is LRU, tail is MRU.
#[derive(Debug, Default, Clone, Copy)]
struct List {
	head: Option<usize>,
	tail: Option<usize>,
	len: usize,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	win: List,
	main: List,
	window_cap: usize,
	main_cap: usize,
	capacity: usize,
	sketch: CountMinSketch,
	// Hill-climber state.
	sample_hits: u64,
	sample_count: u64,
	/// Previous climb's smoothed (EWMA) hit rate.
	prev_hit_rate: f64,
	/// Smoothed per-sample hit rate.
	ewma_hit_rate: f64,
	ewma_seeded: bool,
	step: isize,
	step_floor: isize,
	sample_period: u64,
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
		let window_cap = (capacity * WINDOW_PERCENT / 100).max(1).min(capacity);
		let main_cap = capacity - window_cap;
		let step = (capacity as isize * INITIAL_STEP_PERCENT / 100).max(1);
		let step_floor = (capacity / STEP_FLOOR_DIVISOR).max(1) as isize;
		let sample_period = SAMPLE_PERIOD;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} sketch_width={width} step={step} step_floor={step_floor} sample_period={sample_period}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			win: List::default(),
			main: List::default(),
			window_cap,
			main_cap,
			capacity,
			sketch: CountMinSketch::new(width),
			sample_hits: 0,
			sample_count: 0,
			prev_hit_rate: 0.0,
			ewma_hit_rate: 0.0,
			ewma_seeded: false,
			step,
			step_floor,
			sample_period,
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
		let hit = self.map.get(key).copied();

		self.sample_count += 1;
		if hit.is_some() {
			self.sample_hits += 1;
		}
		if self.sample_count >= self.sample_period {
			self.climb();
			self.sample_hits = 0;
			self.sample_count = 0;
		}

		let idx = hit?;
		self.promote(idx);
		Some(&self.nodes[idx].value)
	}

	/// Hill-climbing step: fold this sample's hit rate into an EWMA, compare it to the previous
	/// EWMA, reverse (and halve) the step on a drop, decay the step down to a capacity-relative
	/// floor, and retarget `window_cap`/`main_cap`. Nodes are not moved here; the lists rebalance
	/// lazily in `put`/`evict_for_insert`.
	fn climb(&mut self) {
		if self.capacity < 2 || self.sample_count == 0 {
			return;
		}
		let hr = self.sample_hits as f64 / self.sample_count as f64;
		if self.ewma_seeded {
			self.ewma_hit_rate = EWMA_KEEP * self.ewma_hit_rate + (1.0 - EWMA_KEEP) * hr;
		} else {
			self.ewma_hit_rate = hr;
			self.ewma_seeded = true;
		}
		let ewma = self.ewma_hit_rate;

		let mut mag = self.step.unsigned_abs() as f64;
		let mut sign: isize = if self.step < 0 { -1 } else { 1 };
		if ewma < self.prev_hit_rate - REVERSE_TOLERANCE {
			sign = -sign;
			mag *= 0.5;
		}
		mag *= STEP_DECAY;
		let mag = (mag as isize).max(self.step_floor).max(1);
		self.step = sign * mag;

		let cap = self.capacity as isize;
		let upper = (cap * MAX_WINDOW_PERCENT / 100).min(cap - 1).max(1);
		let target = (self.window_cap as isize + self.step).clamp(1, upper);
		self.window_cap = target as usize;
		self.main_cap = self.capacity - self.window_cap;
		self.prev_hit_rate = ewma;
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			// Not full: new node goes to window MRU; spill window LRU to main while the window
			// is over its (possibly retargeted) cap and main has room.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				region: Region::Window,
			});
			self.push_back(idx, Region::Window);
			self.map.insert(key, idx);
			while self.win.len > self.window_cap && self.main.len < self.main_cap {
				let cand = self.win.head.expect("TinyLfuCache: overflowing window is empty");
				self.unlink(cand);
				self.push_back(cand, Region::Main);
			}
			return;
		}

		// Full: free one slot (lazy rebalance + admission step), then reuse that slot for the
		// newcomer at window MRU.
		let freed = self.evict_for_insert();
		self.nodes[freed].key = key.clone();
		self.nodes[freed].value = value;
		self.push_back(freed, Region::Window);
		self.map.insert(key, freed);
	}

	/// Called when the cache is full. Evicts exactly one key and returns its (unlinked) slot.
	///
	/// - `win.len < window_cap` (main over its cap): evict main LRU directly; the window grows
	///   by one through the newcomer insert.
	/// - `win.len > window_cap` (main under its cap): spill the window LRU into main (main has
	///   room), then run the usual contest with the new window LRU as candidate. Net effect: the
	///   window shrinks by one per insert until it reaches its cap.
	/// - `win.len == window_cap`: v001's contest unchanged.
	fn evict_for_insert(&mut self) -> usize {
		if self.win.len < self.window_cap {
			if let Some(v) = self.main.head {
				self.evict(v);
				return v;
			}
		}

		if self.win.len > self.window_cap && self.main.len < self.main_cap {
			// win.len > window_cap >= 1, so window keeps at least one node after this spill.
			let spill = self.win.head.expect("TinyLfuCache: overflowing window is empty");
			self.unlink(spill);
			self.push_back(spill, Region::Main);
		}

		let cand = match self.win.head {
			Some(c) => c,
			// Defensive: window empty; evict main LRU.
			None => {
				let v = self.main.head.expect("TinyLfuCache: full cache has no victim");
				self.evict(v);
				return v;
			}
		};

		let victim = match self.main.head {
			Some(v) => v,
			None => {
				// main_cap == 0 (capacity 1) or main empty: no contest, candidate leaves.
				self.evict(cand);
				return cand;
			}
		};

		let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
		let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
		if cand_freq > victim_freq {
			self.evict(victim);
			self.unlink(cand);
			self.push_back(cand, Region::Main);
			victim
		} else {
			self.evict(cand);
			cand
		}
	}

	/// Removes `idx` from its list and from the map. The slot stays in the arena for reuse.
	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		let k = self.nodes[idx].key.clone();
		self.map.remove(&k);
	}

	/// Moves `idx` to the MRU end of whichever list it currently lives in.
	fn promote(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		self.unlink(idx);
		self.push_back(idx, region);
	}

	fn list_mut(&mut self, region: Region) -> &mut List {
		match region {
			Region::Window => &mut self.win,
			Region::Main => &mut self.main,
		}
	}

	fn unlink(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.list_mut(region).head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.list_mut(region).tail = prev,
		}
		self.list_mut(region).len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize, region: Region) {
		let tail = self.list_mut(region).tail;
		self.nodes[idx].region = region;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.list_mut(region).head = Some(idx),
		}
		let list = self.list_mut(region);
		list.tail = Some(idx);
		list.len += 1;
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
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn newcomer_enters_window_and_cold_candidate_loses_contest() {
		// capacity 2: window=1, main=1. After puts 1,2: main=[1], window=[2].
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		// Newcomer 3 is admitted unconditionally into the window; the window evictee (2)
		// loses the contest against the hotter main victim (1).
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn hot_candidate_wins_contest() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // main=[1], window=[2]
		cache.get(&2);
		cache.get(&2); // 2 hotter than 1
		cache.put(3, 30); // candidate 2 beats victim 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn capacity_one_keeps_most_recent() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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

	fn check_structure(cache: &TinyLfuCache<i64, i64>) {
		assert!(cache.win.len + cache.main.len <= cache.capacity);
		assert_eq!(cache.win.len + cache.main.len, cache.map.len());
		assert!(cache.window_cap >= 1);
		assert_eq!(cache.window_cap + cache.main_cap, cache.capacity);
		if cache.capacity >= 2 {
			assert!(cache.window_cap <= cache.capacity - 1);
		}
	}

	#[test]
	fn adaptive_window_keeps_structure_sound() {
		for &cap in &[1usize, 2, 3, 8, 100] {
			let mut cache = TinyLfuCache::<i64, i64>::new(cap);
			// Phase-shifting workload so the climber moves in both directions.
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
			for i in 0..20_000i64 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let key = if (i / 2_000) % 2 == 0 {
					(x % (cap as u64 * 3 + 1)) as i64
				} else {
					i % (cap as i64 * 2 + 5)
				};
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
				check_structure(&cache);
			}
			for (k, &idx) in &cache.map {
				assert_eq!(cache.nodes[idx].key, *k);
				assert_eq!(cache.nodes[idx].value, *k * 10);
			}
		}
	}
}
