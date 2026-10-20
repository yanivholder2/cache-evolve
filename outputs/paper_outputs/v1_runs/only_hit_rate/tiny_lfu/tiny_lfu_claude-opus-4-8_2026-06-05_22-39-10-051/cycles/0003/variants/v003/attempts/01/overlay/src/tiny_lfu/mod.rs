use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
const DOORKEEPER_SEED: u64 = 0x9E37_79B9_7F4A_7C15;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>,
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
	// Admission-only doorkeeper: a separate bloom-style bit array consulted
	// *only* on the eviction admission path. It never touches the main
	// CountMinSketch counters; it merely requires a churn key to be seen
	// twice within a long clear window before it can displace a resident.
	doorkeeper: Vec<u64>,
	doorkeeper_bits: usize,
	doorkeeper_ops: usize,
	doorkeeper_window: usize,
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
		// Doorkeeper bit array sized to roughly the sketch width (and a power
		// of two so we can mask). At least one 64-bit word.
		let doorkeeper_bits = width.max(64);
		let doorkeeper_words = doorkeeper_bits / 64;
		// Long clear window: a key must reappear within `capacity * 8`
		// admission-path checks to ever be admitted.
		let doorkeeper_window = capacity.saturating_mul(8).max(1);
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width),
			doorkeeper: vec![0u64; doorkeeper_words],
			doorkeeper_bits,
			doorkeeper_ops: 0,
			doorkeeper_window,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		// Eviction admission path. Consult the doorkeeper *before* the
		// frequency comparison. A first sighting within the current clear
		// window only records the key and rejects this admission attempt;
		// only a second sighting is allowed to contend for the victim slot.
		self.doorkeeper_ops += 1;
		if self.doorkeeper_ops >= self.doorkeeper_window {
			for word in self.doorkeeper.iter_mut() {
				*word = 0;
			}
			self.doorkeeper_ops = 0;
		}
		let (word, mask) = self.doorkeeper_index(&key);
		if self.doorkeeper[word] & mask == 0 {
			self.doorkeeper[word] |= mask;
			return;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	fn doorkeeper_index(&self, key: &K) -> (usize, u64) {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(DOORKEEPER_SEED);
		key.hash(&mut hasher);
		let bit = (hasher.finish() as usize) & (self.doorkeeper_bits - 1);
		(bit / 64, 1u64 << (bit % 64))
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = None;
		match self.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
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
	fn doorkeeper_requires_second_sighting_before_admission() {
		// Capacity 2, doorkeeper window = 16. Fill the cache, then a churn
		// key seen only once on the eviction path is rejected; a key seen
		// twice on that path may then contend on frequency.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		// First eviction-path sighting of 3 only records the doorkeeper bit.
		cache.put(3, 30);
		assert!(cache.get(&3).is_none());
	}
}
