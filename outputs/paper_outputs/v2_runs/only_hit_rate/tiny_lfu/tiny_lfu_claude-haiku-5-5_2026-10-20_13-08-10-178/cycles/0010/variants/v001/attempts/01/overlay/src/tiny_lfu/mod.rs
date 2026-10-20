use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Protected share of capacity, as a numerator over 10 (9/10 of the main region).
const PROTECTED_NUM: usize = 9;
const PROTECTED_DEN: usize = 10;

/// Number of probation entries examined from the LRU end when choosing an eviction victim.
const TAIL_SCAN: usize = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	in_protected: bool,
}

/// Head (LRU) / tail (MRU) ends of one intrusive list over the arena.
#[derive(Debug, Default, Clone, Copy)]
struct Segment {
	head: Option<usize>,
	tail: Option<usize>,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// New admissions and demoted entries land here; the victim is its head.
	probation: Segment,
	/// Entries that were hit while resident in probation (or hit again here).
	protected_seg: Segment,
	protected_len: usize,
	protected_cap: usize,
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
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		// Leave at least one slot in probation so the admission contest always has a victim.
		let protected_cap = (capacity * PROTECTED_NUM / PROTECTED_DEN).min(capacity - 1);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			probation: Segment::default(),
			protected_seg: Segment::default(),
			protected_len: 0,
			protected_cap,
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
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				in_protected: false,
			});
			self.push_back(idx, false);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self.select_victim();
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx, false);
			self.map.insert(key, victim_idx);
		}
	}

	/// Chooses the eviction candidate among probation's LRU end.
	///
	/// Examines at most `TAIL_SCAN` probation entries, starting at the LRU head and walking
	/// towards the MRU end, and returns the one with the lowest sketch estimate. On an exact tie
	/// the entry reached first (the older one, closer to the head) is kept. If probation is empty
	/// this falls back to the protected head.
	fn select_victim(&self) -> usize {
		let Some(mut cursor) = self.probation.head else {
			return self
				.protected_seg
				.head
				.expect("TinyLfuCache: full cache has no victim");
		};
		let mut best = cursor;
		let mut best_freq = self.sketch.estimate(&self.nodes[cursor].key);
		for _ in 1..TAIL_SCAN {
			let Some(next) = self.nodes[cursor].next else {
				break;
			};
			cursor = next;
			let freq = self.sketch.estimate(&self.nodes[cursor].key);
			if freq < best_freq {
				best = cursor;
				best_freq = freq;
			}
		}
		best
	}

	/// A hit (get or update-put) moves the entry to the MRU end of protected; if protected
	/// overflows, its LRU entry is demoted to the MRU end of probation.
	fn promote(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back(idx, true);
		while self.protected_len > self.protected_cap {
			let demote = self
				.protected_seg
				.head
				.expect("TinyLfuCache: protected overflow with empty segment");
			self.unlink(demote);
			self.push_back(demote, false);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let in_protected = self.nodes[idx].in_protected;
		if in_protected {
			self.protected_len -= 1;
		}
		let seg = if in_protected {
			&mut self.protected_seg
		} else {
			&mut self.probation
		};
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => seg.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => seg.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize, in_protected: bool) {
		self.nodes[idx].in_protected = in_protected;
		if in_protected {
			self.protected_len += 1;
		}
		let seg = if in_protected {
			&mut self.protected_seg
		} else {
			&mut self.probation
		};
		self.nodes[idx].prev = seg.tail;
		self.nodes[idx].next = None;
		match seg.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => seg.head = Some(idx),
		}
		seg.tail = Some(idx);
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
		// Per-row aging: when a counter bumped here reaches the threshold, halve only the
		// counters of that same row. Other rows keep their admission history.
		const SAT_THRESHOLD: u32 = u32::MAX / 2;
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
			let saturated = *counter >= SAT_THRESHOLD;
			if saturated {
				for c in self.counts[row].iter_mut() {
					*c >>= 1;
				}
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
	fn low_frequency_new_item_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_none());
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
