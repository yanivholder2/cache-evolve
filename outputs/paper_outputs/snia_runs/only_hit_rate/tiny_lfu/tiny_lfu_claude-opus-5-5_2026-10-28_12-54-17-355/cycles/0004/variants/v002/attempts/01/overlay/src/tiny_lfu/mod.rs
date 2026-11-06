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
/// Protected segment size as a percentage of the main region (SLRU).
const PROTECTED_PERCENT: usize = 80;
/// Initial hill-climber step as a percentage of capacity.
const INITIAL_STEP_PERCENT: isize = 5;
/// Upper bound on the window as a percentage of capacity.
const MAX_WINDOW_PERCENT: isize = 80;
/// Per-climb multiplicative decay of the step magnitude.
const STEP_DECAY: f64 = 0.98;
/// Hit-rate drop (absolute) that triggers a direction reversal.
const REVERSE_TOLERANCE: f64 = 0.001;
/// Minimum sampling period in `get` calls.
const MIN_SAMPLE_PERIOD: usize = 64;
/// On an admission tie, the (more recent) window candidate wins if its estimate is at
/// least this value; one-hit candidates still lose ties.
const TIE_ADMIT_MIN_FREQ: u32 = 2;

/// Which recency list a node currently lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Window,
	Probation,
	Protected,
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
	probation: List,
	protected: List,
	window_cap: usize,
	main_cap: usize,
	protected_cap: usize,
	capacity: usize,
	sketch: CountMinSketch,
	// Hill-climber state.
	sample_hits: u64,
	sample_count: u64,
	prev_hit_rate: f64,
	step: isize,
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
		let protected_cap = main_cap * PROTECTED_PERCENT / 100;
		let step = (capacity as isize * INITIAL_STEP_PERCENT / 100).max(1);
		let sample_period = capacity.max(MIN_SAMPLE_PERIOD) as u64;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} protected_cap={protected_cap} sketch_width={width} step={step} sample_period={sample_period}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			win: List::default(),
			probation: List::default(),
			protected: List::default(),
			window_cap,
			main_cap,
			protected_cap,
			capacity,
			sketch: CountMinSketch::new(width),
			sample_hits: 0,
			sample_count: 0,
			prev_hit_rate: 0.0,
			step,
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

	/// Hill-climbing step: compare this sample's hit rate to the previous one, reverse the
	/// direction on a drop, decay the step, and retarget `window_cap`/`main_cap`. Nodes are
	/// not moved here; the lists rebalance lazily in `put`/`evict_for_insert`/`promote`.
	fn climb(&mut self) {
		if self.capacity < 2 || self.sample_count == 0 {
			return;
		}
		let hr = self.sample_hits as f64 / self.sample_count as f64;
		if hr < self.prev_hit_rate - REVERSE_TOLERANCE {
			self.step = -self.step;
		}
		let sign: isize = if self.step < 0 { -1 } else { 1 };
		let mag = ((self.step.unsigned_abs() as f64) * STEP_DECAY) as isize;
		self.step = sign * mag.max(1);

		let cap = self.capacity as isize;
		let upper = (cap * MAX_WINDOW_PERCENT / 100).min(cap - 1).max(1);
		let target = (self.window_cap as isize + self.step).clamp(1, upper);
		self.window_cap = target as usize;
		self.main_cap = self.capacity - self.window_cap;
		self.protected_cap = self.main_cap * PROTECTED_PERCENT / 100;
		self.prev_hit_rate = hr;
	}

	#[inline]
	fn main_len(&self) -> usize {
		self.probation.len + self.protected.len
	}

	/// Main-region LRU victim: probation LRU, falling back to protected LRU.
	#[inline]
	fn main_victim(&self) -> Option<usize> {
		self.probation.head.or(self.protected.head)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			// Not full: new node goes to window MRU; spill window LRU to probation while the
			// window is over its (possibly retargeted) cap and main has room.
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
			while self.win.len > self.window_cap && self.main_len() < self.main_cap {
				let cand = self.win.head.expect("TinyLfuCache: overflowing window is empty");
				self.unlink(cand);
				self.push_back(cand, Region::Probation);
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
	/// - `win.len < window_cap` (main over its cap): evict main victim directly; the window
	///   grows by one through the newcomer insert.
	/// - `win.len > window_cap` (main under its cap): spill the window LRU into probation, then
	///   run the usual contest with the new window LRU as candidate.
	/// - `win.len == window_cap`: contest against the probation LRU (protected LRU if
	///   probation is empty).
	fn evict_for_insert(&mut self) -> usize {
		if self.win.len < self.window_cap {
			if let Some(v) = self.main_victim() {
				self.evict(v);
				return v;
			}
		}

		if self.win.len > self.window_cap && self.main_len() < self.main_cap {
			// win.len > window_cap >= 1, so window keeps at least one node after this spill.
			let spill = self.win.head.expect("TinyLfuCache: overflowing window is empty");
			self.unlink(spill);
			self.push_back(spill, Region::Probation);
		}

		let cand = match self.win.head {
			Some(c) => c,
			// Defensive: window empty; evict main victim.
			None => {
				let v = self.main_victim().expect("TinyLfuCache: full cache has no victim");
				self.evict(v);
				return v;
			}
		};

		let victim = match self.main_victim() {
			Some(v) => v,
			None => {
				// main_cap == 0 (capacity 1) or main empty: no contest, candidate leaves.
				self.evict(cand);
				return cand;
			}
		};

		let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
		let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
		// Ties go to the candidate (more recent of the two) unless it is a one-hit key.
		let admit = cand_freq > victim_freq
			|| (cand_freq == victim_freq && cand_freq >= TIE_ADMIT_MIN_FREQ);
		if admit {
			self.evict(victim);
			self.unlink(cand);
			self.push_back(cand, Region::Probation);
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

	/// Window/Protected hit: move to MRU of the same list. Probation hit: move to protected
	/// MRU, demoting protected LRU nodes to probation MRU until protected fits its cap.
	fn promote(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		match region {
			Region::Window | Region::Protected => {
				self.unlink(idx);
				self.push_back(idx, region);
			}
			Region::Probation => {
				self.unlink(idx);
				if self.protected_cap == 0 {
					self.push_back(idx, Region::Probation);
					return;
				}
				self.push_back(idx, Region::Protected);
				// protected_cap >= 1, and idx sits at the protected MRU end, so the loop only
				// demotes other nodes and terminates once len <= protected_cap.
				while self.protected.len > self.protected_cap {
					let d = self.protected.head.expect("TinyLfuCache: overfull protected is empty");
					self.unlink(d);
					self.push_back(d, Region::Probation);
				}
			}
		}
	}

	fn list_mut(&mut self, region: Region) -> &mut List {
		match region {
			Region::Window => &mut self.win,
			Region::Probation => &mut self.probation,
			Region::Protected => &mut self.protected,
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
		// capacity 2: window=1, main=1 (protected_cap=0). After puts 1,2: probation=[1], window=[2].
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
		cache.put(2, 20); // probation=[1], window=[2]
		cache.get(&2);
		cache.get(&2); // 2 hotter than 1
		cache.put(3, 30); // candidate 2 beats victim 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn tie_at_or_above_floor_admits_candidate() {
		// capacity 2: window=1, main=1. After puts 1,2: probation=[1], window=[2].
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // estimate(1) = 2
		cache.get(&2); // estimate(2) = 2
		assert_eq!(cache.sketch.estimate(&1), 2);
		assert_eq!(cache.sketch.estimate(&2), 2);
		cache.put(3, 30); // tie at 2 >= TIE_ADMIT_MIN_FREQ: candidate 2 beats victim 1

		assert!(!cache.map.contains_key(&1));
		assert!(cache.map.contains_key(&2));
		assert!(cache.map.contains_key(&3));
		assert_eq!(cache.nodes[cache.map[&2]].region, Region::Probation);
	}

	#[test]
	fn tie_below_floor_evicts_candidate() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // probation=[1], window=[2], both estimate 1
		assert_eq!(cache.sketch.estimate(&1), 1);
		assert_eq!(cache.sketch.estimate(&2), 1);
		cache.put(3, 30); // tie at 1 < TIE_ADMIT_MIN_FREQ: candidate 2 is evicted

		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&2));
		assert!(cache.map.contains_key(&3));
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

	#[test]
	fn probation_hit_moves_to_protected() {
		let mut cache = TinyLfuCache::<i64, i64>::new(100);
		for k in 0..100 {
			cache.put(k, k * 10);
		}
		// Key 0 spilled out of the window into probation; a hit promotes it to protected.
		assert_eq!(cache.nodes[cache.map[&0]].region, Region::Probation);
		cache.get(&0);
		assert_eq!(cache.nodes[cache.map[&0]].region, Region::Protected);
		check_structure(&cache);
	}

	fn check_structure(cache: &TinyLfuCache<i64, i64>) {
		let total = cache.win.len + cache.probation.len + cache.protected.len;
		assert!(total <= cache.capacity);
		assert_eq!(total, cache.map.len());
		assert!(cache.window_cap >= 1);
		assert_eq!(cache.window_cap + cache.main_cap, cache.capacity);
		assert_eq!(cache.protected_cap, cache.main_cap * PROTECTED_PERCENT / 100);
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
