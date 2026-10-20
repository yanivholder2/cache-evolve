use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// Which intrusive list (window or main) this node is currently linked into.
	in_window: bool,
}

/// Anchors of one intrusive LRU list threaded through the shared node arena.
/// `head` is the LRU end (eviction candidate), `tail` the MRU end.
#[derive(Debug, Default)]
struct Lru {
	head: Option<usize>,
	tail: Option<usize>,
	len: usize,
}

/// W-TinyLFU-style cache: a small LRU admission window (about 1% of capacity,
/// at least one slot) in front of a main LRU region, both fronted by the
/// count-min frequency sketch. Every newcomer enters the window; only the
/// window's LRU tail is contested against the main region's victim.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// Arena slots released by evictions/rejections, reused before growing `nodes`.
	free: Vec<usize>,
	window: Lru,
	main: Lru,
	window_capacity: usize,
	main_capacity: usize,
	sketch: CountMinSketch,
}

/// Window slots carved out of the total capacity. Zero only when capacity is 1
/// (a window plus a main region would leave the main region empty).
fn window_capacity_for(capacity: usize) -> usize {
	if capacity < 2 {
		0
	} else {
		(capacity / 100).max(1)
	}
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
		let window_capacity = window_capacity_for(capacity);
		let main_capacity = capacity - window_capacity;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window={window_capacity} main={main_capacity} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			window: Lru::default(),
			main: Lru::default(),
			window_capacity,
			main_capacity,
			sketch: CountMinSketch::new(width),
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
		let in_window = self.nodes[idx].in_window;
		self.unlink(idx);
		self.push_back(idx, in_window);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let in_window = self.nodes[idx].in_window;
			self.unlink(idx);
			self.push_back(idx, in_window);
			return;
		}
		self.put_new(key, value);
	}

	/// Admit a key that is not resident. The newcomer always enters the window;
	/// a full window pushes its LRU tail out as the candidate for the main region.
	fn put_new(&mut self, key: K, value: V) {
		if self.window.len < self.window_capacity {
			self.insert(key, value, true);
			return;
		}

		let Some(cand) = self.window.head else {
			// No window (capacity 1): the newcomer contests the main victim directly.
			if self.main.len < self.main_capacity {
				self.insert(key, value, false);
				return;
			}
			let victim = self.main.head.expect("TinyLfuCache: full main region has no victim");
			if self.admits(&key, victim) {
				self.evict(victim);
				self.insert(key, value, false);
			}
			return;
		};

		// Detach the window's LRU tail; it is now the admission candidate.
		self.unlink(cand);
		if self.main.len < self.main_capacity {
			self.push_back(cand, false);
		} else {
			let victim = self.main.head.expect("TinyLfuCache: full main region has no victim");
			if self.admits(&self.nodes[cand].key, victim) {
				self.evict(victim);
				self.push_back(cand, false);
			} else {
				self.map.remove(&self.nodes[cand].key);
				self.free.push(cand);
			}
		}
		self.insert(key, value, true);
	}

	/// Frequency gate: the incoming key must be strictly hotter than the victim.
	fn admits(&self, incoming: &K, victim: usize) -> bool {
		self.sketch.estimate(incoming) > self.sketch.estimate(&self.nodes[victim].key)
	}

	/// Remove a linked resident and release its arena slot.
	fn evict(&mut self, victim: usize) {
		self.map.remove(&self.nodes[victim].key);
		self.unlink(victim);
		self.free.push(victim);
	}

	/// Insert a non-resident key at the MRU end of the chosen region.
	fn insert(&mut self, key: K, value: V, in_window: bool) {
		let idx = match self.free.pop() {
			Some(idx) => {
				self.nodes[idx] = Node {
					key: key.clone(),
					value,
					prev: None,
					next: None,
					in_window,
				};
				idx
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					prev: None,
					next: None,
					in_window,
				});
				self.nodes.len() - 1
			}
		};
		self.push_back(idx, in_window);
		self.map.insert(key, idx);
	}

	fn list_mut(&mut self, in_window: bool) -> &mut Lru {
		if in_window {
			&mut self.window
		} else {
			&mut self.main
		}
	}

	fn unlink(&mut self, idx: usize) {
		let in_window = self.nodes[idx].in_window;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.list_mut(in_window).head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.list_mut(in_window).tail = prev,
		}
		self.list_mut(in_window).len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize, in_window: bool) {
		let tail = self.list_mut(in_window).tail;
		self.nodes[idx].in_window = in_window;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.list_mut(in_window).head = Some(idx),
		}
		let list = self.list_mut(in_window);
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
	fn window_admits_newcomer_and_rejects_cold_window_tail() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // key 1 leaves the window for the main region
		cache.get(&1); // main victim 1 is now hotter than window tail 2
		cache.put(3, 30); // newcomer enters the window; cold tail 2 is rejected

		assert!(cache.get(&1).is_some());
		assert!(cache.get(&3).is_some());
		assert!(cache.get(&2).is_none());
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
}
