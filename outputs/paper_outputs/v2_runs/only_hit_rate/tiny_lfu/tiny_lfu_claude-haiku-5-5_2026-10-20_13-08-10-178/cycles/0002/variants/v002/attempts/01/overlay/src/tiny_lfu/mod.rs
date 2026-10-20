use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
/// Share of capacity given to the admission window, in percent (at least one slot).
const WINDOW_PERCENT: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// Which LRU segment this node currently belongs to.
	in_window: bool,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// Slots released by evictions or rejections, reused before `nodes` grows.
	free: Vec<usize>,
	window_head: Option<usize>,
	window_tail: Option<usize>,
	window_len: usize,
	window_cap: usize,
	main_head: Option<usize>,
	main_tail: Option<usize>,
	main_len: usize,
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
		let window_cap = (capacity * WINDOW_PERCENT / 100).max(1);
		let main_cap = capacity - window_cap;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window_cap={window_cap} main_cap={main_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			window_head: None,
			window_tail: None,
			window_len: 0,
			window_cap,
			main_head: None,
			main_tail: None,
			main_len: 0,
			main_cap,
			capacity,
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
		debug_assert!(self.map.len() <= self.capacity);
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let in_window = self.nodes[idx].in_window;
			self.unlink(idx);
			self.push_back(idx, in_window);
			return;
		}

		// New keys always enter the window and are only judged by the sketch once
		// they are pushed out of it.
		let idx = self.alloc(key.clone(), value);
		self.push_back(idx, true);
		self.map.insert(key, idx);
		if self.window_len <= self.window_cap {
			return;
		}

		// Window overflow: its oldest entry becomes the admission candidate.
		let candidate = self
			.window_head
			.expect("TinyLfuCache: overfull window has no head");
		self.unlink(candidate);
		if self.main_len < self.main_cap {
			self.push_back(candidate, false);
			return;
		}
		if let Some(victim) = self.main_head {
			let candidate_freq = self.sketch.estimate(&self.nodes[candidate].key);
			let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
			if candidate_freq > victim_freq {
				self.unlink(victim);
				self.map.remove(&self.nodes[victim].key);
				self.free.push(victim);
				self.push_back(candidate, false);
				return;
			}
		}
		// Candidate loses (or main has no room at all): drop it and free its slot.
		self.map.remove(&self.nodes[candidate].key);
		self.free.push(candidate);
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		let node = Node {
			key,
			value,
			prev: None,
			next: None,
			in_window: true,
		};
		match self.free.pop() {
			Some(idx) => {
				self.nodes[idx] = node;
				idx
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let in_window = self.nodes[idx].in_window;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if in_window {
					self.window_head = next
				} else {
					self.main_head = next
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if in_window {
					self.window_tail = prev
				} else {
					self.main_tail = prev
				}
			}
		}
		if in_window {
			self.window_len -= 1;
		} else {
			self.main_len -= 1;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize, in_window: bool) {
		let tail = if in_window {
			self.window_tail
		} else {
			self.main_tail
		};
		self.nodes[idx].in_window = in_window;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => {
				if in_window {
					self.window_head = Some(idx)
				} else {
					self.main_head = Some(idx)
				}
			}
		}
		if in_window {
			self.window_tail = Some(idx);
			self.window_len += 1;
		} else {
			self.main_tail = Some(idx);
			self.main_len += 1;
		}
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
		cache.put(1, 10); // window: [1]
		cache.put(2, 20); // 1 overflows the window into main; window: [2]
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30); // candidate 2 loses to main victim 1 and is dropped

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn low_frequency_candidate_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10); // window: [1]
		cache.put(2, 20); // 1 moves to main; window: [2]
		cache.get(&1);
		// The new key 3 always enters the window first, so it is not rejected on arrival.
		// Instead the window's oldest entry (2) is the candidate; it loses to main victim 1.
		cache.put(3, 30);

		assert!(cache.get(&1).is_some());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_some());
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
