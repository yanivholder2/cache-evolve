use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Index of the probation segment in `TinyLfuCache::segments`.
const PROBATION: usize = 0;
/// Index of the protected segment in `TinyLfuCache::segments`.
const PROTECTED: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: usize,
}

/// Head/tail/length of one intrusive LRU list (one segment of the resident set).
#[derive(Debug, Clone, Copy)]
struct Segment {
	head: Option<usize>,
	tail: Option<usize>,
	len: usize,
}

impl Segment {
	const EMPTY: Segment = Segment {
		head: None,
		tail: None,
		len: 0,
	};
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// `segments[PROBATION]` holds newly admitted entries; `segments[PROTECTED]` holds
	/// entries that have been hit at least once while resident.
	segments: [Segment; 2],
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
		let protected_cap = (capacity * 4 / 5).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} protected_cap={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			segments: [Segment::EMPTY; 2],
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
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				segment: PROBATION,
			});
			self.push_back(idx, PROBATION);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self
			.eviction_candidate()
			.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx, PROBATION);
			self.map.insert(key, victim_idx);
		}
	}

	/// Victim selection: the LRU end of probation, falling back to the LRU end of protected
	/// only when probation is empty.
	fn eviction_candidate(&self) -> Option<usize> {
		self.segments[PROBATION]
			.head
			.or(self.segments[PROTECTED].head)
	}

	/// A hit (get or update) moves the entry to the MRU end of protected. If protected then
	/// exceeds its cap, its LRU entry is demoted to the LRU end of probation (stays resident),
	/// so it becomes the next eviction candidate.
	fn touch(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back(idx, PROTECTED);
		if self.segments[PROTECTED].len > self.protected_cap {
			let demoted = self.segments[PROTECTED]
				.head
				.expect("TinyLfuCache: protected over cap but empty");
			self.unlink(demoted);
			self.push_front(demoted, PROBATION);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.segments[seg].head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.segments[seg].tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.segments[seg].len -= 1;
	}

	fn push_back(&mut self, idx: usize, seg: usize) {
		self.nodes[idx].segment = seg;
		self.nodes[idx].prev = self.segments[seg].tail;
		self.nodes[idx].next = None;
		match self.segments[seg].tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.segments[seg].head = Some(idx),
		}
		self.segments[seg].tail = Some(idx);
		self.segments[seg].len += 1;
	}

	/// Links `idx` at the LRU (head) end of `seg`.
	fn push_front(&mut self, idx: usize, seg: usize) {
		self.nodes[idx].segment = seg;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.segments[seg].head;
		match self.segments[seg].head {
			Some(h) => self.nodes[h].prev = Some(idx),
			None => self.segments[seg].tail = Some(idx),
		}
		self.segments[seg].head = Some(idx);
		self.segments[seg].len += 1;
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
