use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Maximum number of oldest LRU entries sampled when choosing an eviction
/// victim. Widened from 8 to 64 so the contest can find genuinely cold
/// victims deeper in the LRU order; the scan stops early once an entry with
/// estimate <= 1 is found (no resident can be colder than a one-hit key).
const VICTIM_SAMPLE: usize = 64;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Cheap, non-cryptographic FxHash-style hasher with a splitmix64 avalanche in
/// `finish`. Used both for the count-min sketch and for the key->slot map.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFastHasher {
	h: u64,
}

impl LocalFastHasher {
	#[inline(always)]
	fn add(&mut self, x: u64) {
		self.h = (self.h.rotate_left(5) ^ x).wrapping_mul(FX_SEED);
	}
}

#[inline(always)]
fn splitmix64(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

impl Hasher for LocalFastHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.add(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.add(u64::from_le_bytes(buf) ^ ((rem.len() as u64) << 56));
		}
	}

	#[inline(always)]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u16(&mut self, i: u16) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}

	#[inline(always)]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_i64(&mut self, i: i64) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn finish(&self) -> u64 {
		splitmix64(self.h)
	}
}

type FastBuildHasher = BuildHasherDefault<LocalFastHasher>;

#[inline(always)]
fn hash_key<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut hasher = LocalFastHasher::default();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>,
	tail: Option<usize>,
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
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
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
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let key_hash = hash_key(&key);
		self.sketch.increment_hashed(key_hash);
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

		let (victim_idx, victim_freq) = self.select_victim();
		let incoming_freq = self.sketch.estimate_hashed(key_hash);
		if incoming_freq > victim_freq {
			let victim_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Walk up to `VICTIM_SAMPLE` oldest entries from the LRU head and return
	/// the one with the smallest frequency estimate. Ties keep the entry
	/// closest to the head (strict `<`). The scan stops early once the best
	/// estimate is <= 1, since no resident can be colder than a one-hit key.
	fn select_victim(&self) -> (usize, u32) {
		let head = self.head.expect("TinyLfuCache: full cache has no victim");
		let mut best_idx = head;
		let mut best_freq = self.sketch.estimate(&self.nodes[head].key);
		let mut cursor = self.nodes[head].next;
		let mut seen = 1;
		while seen < VICTIM_SAMPLE && best_freq > 1 {
			let Some(idx) = cursor else { break };
			let freq = self.sketch.estimate(&self.nodes[idx].key);
			if freq < best_freq {
				best_freq = freq;
				best_idx = idx;
			}
			cursor = self.nodes[idx].next;
			seen += 1;
		}
		(best_idx, best_freq)
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

/// Count-min sketch over a flat `depth * width` counter array; row `i` lives at
/// offset `i * width`. Row positions come from one 64-bit hash via
/// Kirsch–Mitzenmacher double hashing.
#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<u32>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![0; depth * width];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	/// Flat indices (row offset already applied) of the key's counter in each row.
	#[inline(always)]
	fn positions(&self, h: u64) -> [usize; SKETCH_DEPTH] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let mask = (self.width - 1) as u32;
		let mut out = [0usize; SKETCH_DEPTH];
		for (i, slot) in out.iter_mut().enumerate() {
			let pos = (h1.wrapping_add((i as u32).wrapping_mul(h2)) & mask) as usize;
			*slot = i * self.width + pos;
		}
		out
	}

	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		for idx in self.positions(h) {
			let counter = &mut self.counts[idx];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	#[inline]
	fn estimate_hashed(&self, h: u64) -> u32 {
		let mut min = MAX_COUNT;
		for idx in self.positions(h) {
			let c = self.counts[idx];
			if c < min {
				min = c;
			}
		}
		min
	}

	#[inline]
	fn increment<K: Hash>(&mut self, key: &K) {
		self.increment_hashed(hash_key(key));
	}

	#[inline]
	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self.estimate_hashed(hash_key(key))
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

	#[test]
	fn victim_is_min_frequency_among_oldest_not_lru_head() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		for _ in 0..5 {
			cache.get(&1); // key 1 is hot (freq 6) ...
		}
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		// ... but is now the LRU head: list = [1, 2, 3, 4].
		assert_eq!(cache.head, cache.map.get(&1).copied());

		cache.get(&5); // warm newcomer: freq 2 after put
		cache.put(5, 50);

		// Pure-LRU victim choice would contest against key 1 and reject 5.
		// Sampling picks key 2 (first of the colder keys) instead.
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.nodes.len() <= 4);
	}
}
