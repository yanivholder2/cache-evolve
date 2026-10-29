use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
/// Window size as a percentage of total capacity (W-TinyLFU admission window).
const WINDOW_PERCENT: usize = 1;
/// Sketch aging period: halve all counters every `SAMPLE_FACTOR * capacity` increments.
const SAMPLE_FACTOR: usize = 10;
const MIN_SAMPLE_SIZE: usize = 16;

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
		let sample_size = capacity.saturating_mul(SAMPLE_FACTOR).max(MIN_SAMPLE_SIZE) as u64;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} sketch_width={width} sample_size={sample_size}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			win: List::default(),
			main: List::default(),
			window_cap,
			main_cap,
			capacity,
			sketch: CountMinSketch::new(width, sample_size),
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
		let &idx = self.map.get(key)?;
		self.promote(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			// Not full: new node goes to window MRU; spill window LRU to main if over cap.
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
			if self.win.len > self.window_cap {
				let cand = self.win.head.expect("TinyLfuCache: overflowing window is empty");
				// Not full overall, so main must have room.
				debug_assert!(self.main.len < self.main_cap);
				self.unlink(cand);
				self.push_back(cand, Region::Main);
			}
			return;
		}

		// Full: resolve the window overflow first (candidate = window LRU) to free one slot,
		// then reuse that slot for the newcomer at window MRU.
		let freed = self.evict_for_insert();
		self.nodes[freed].key = key.clone();
		self.nodes[freed].value = value;
		self.push_back(freed, Region::Window);
		self.map.insert(key, freed);
	}

	/// Called when the cache is full. Runs the W-TinyLFU admission step for the window LRU
	/// candidate and evicts exactly one key, returning its (unlinked) slot index.
	fn evict_for_insert(&mut self) -> usize {
		let cand = match self.win.head {
			Some(c) => c,
			// Defensive: window empty (cannot happen when full with window_cap >= 1); evict main LRU.
			None => {
				let v = self.main.head.expect("TinyLfuCache: full cache has no victim");
				self.evict(v);
				return v;
			}
		};

		if self.main.len < self.main_cap {
			// Unreachable when full (both regions are at cap); if it ever occurs, no contest
			// is possible without a main victim, so the candidate leaves.
			self.evict(cand);
			return cand;
		}

		let victim = match self.main.head {
			Some(v) => v,
			None => {
				// main_cap == 0 (capacity 1): no contest, candidate leaves.
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
	/// Increments (one per `increment` call) since the last reset, halved on each reset.
	additions: u64,
	/// Aging period: once `additions` reaches this, every counter is halved.
	sample_size: u64,
}

impl CountMinSketch {
	fn new(width: usize, sample_size: u64) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self {
			width,
			depth,
			counts,
			additions: 0,
			sample_size: sample_size.max(1),
		}
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
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	/// TinyLFU aging: halve every counter and the addition count.
	fn reset(&mut self) {
		for row in self.counts.iter_mut() {
			for c in row.iter_mut() {
				*c >>= 1;
			}
		}
		self.additions /= 2;
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

	#[test]
	fn sketch_halves_counters_after_sample_size_increments() {
		// capacity 2 -> sample_size = max(10 * 2, 16) = 20.
		let cache = TinyLfuCache::<i64, i64>::new(2);
		let sample_size = cache.sketch.sample_size;
		assert_eq!(sample_size, 20);

		let mut sketch = CountMinSketch::new(cache.sketch.width, sample_size);
		for _ in 0..sample_size - 1 {
			sketch.increment(&7i64);
		}
		assert_eq!(sketch.estimate(&7i64) as u64, sample_size - 1);
		sketch.increment(&7i64); // reaches sample_size -> reset fires
		assert_eq!(sketch.estimate(&7i64) as u64, sample_size / 2);
		assert_eq!(sketch.additions, sample_size / 2);
	}
}
