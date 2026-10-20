use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u8 = u8::MAX;
const NIL: u32 = u32::MAX;
const SEED: u64 = 0x9E37_79B9_7F4A_7C15;

// Node holds no cached hash/frequency (the victim hash is recomputed on the eviction
// path only); links are u32 so the node is key + value + 8 bytes with no padding.
#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FoldBuildHasher;

impl BuildHasher for FoldBuildHasher {
	type Hasher = FoldHasher;
	#[inline]
	fn build_hasher(&self) -> FoldHasher {
		FoldHasher(SEED, true)
	}
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	// Map values are u32 indices (half the size of usize) to shrink per-entry footprint.
	map: HashMap<K, u32, FoldBuildHasher>,
	nodes: Vec<Node<K, V>>,
	head: u32,
	tail: u32,
	capacity: usize,
	sketch: CountMinSketch,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 || capacity >= u32::MAX as usize {
			log::error!("TinyLfuCache.try_new: capacity must be in 1..u32::MAX");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FoldBuildHasher),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width` bytes (u8 counters).
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Hit check first; the sketch is incremented exactly once on either branch.
		match self.map.get(key) {
			Some(&idx) => {
				self.sketch.increment(CountMinSketch::mix(key));
				if idx != self.tail {
					self.unlink(idx as usize);
					self.push_back(idx as usize);
				}
				Some(&self.nodes[idx as usize].value)
			}
			None => self.record_miss(key),
		}
	}

	#[cold]
	#[inline(never)]
	fn record_miss(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(CountMinSketch::mix(key));
		None
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = CountMinSketch::mix(&key);
		self.sketch.increment(h);
		if let Some(&idx) = self.map.get(&key) {
			let idx = idx as usize;
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
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx);
			self.map.insert(key, idx as u32);
			return;
		}

		assert!(self.head != NIL, "TinyLfuCache: full cache has no victim");
		let victim_idx = self.head as usize;
		// Victim hash recomputed from the key only on the eviction path.
		let victim_h = CountMinSketch::mix(&self.nodes[victim_idx].key);
		let victim_freq = self.sketch.estimate(victim_h);
		let incoming_freq = self.sketch.estimate(h);
		if incoming_freq > victim_freq {
			self.map.remove(&self.nodes[victim_idx].key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx as u32);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	fn push_back(&mut self, idx: usize) {
		let tail = self.tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx as u32;
		} else {
			self.head = idx as u32;
		}
		self.tail = idx as u32;
	}
}

/// Cheap multiplicative-fold hasher (replaces SipHash). The bool selects an extra
/// xorshift fold in `finish` (map use); sketch use keeps the raw value for splitmix64.
#[derive(Debug)]
pub struct FoldHasher(u64, bool);

impl Hasher for FoldHasher {
	#[inline]
	fn finish(&self) -> u64 {
		if self.1 {
			self.0 ^ (self.0 >> 32)
		} else {
			self.0
		}
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517C_C1B7_2722_0A95);
		}
	}

	#[inline]
	fn write_u64(&mut self, v: u64) {
		self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517C_C1B7_2722_0A95);
	}

	#[inline]
	fn write_i64(&mut self, v: i64) {
		self.write_u64(v as u64);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	bits: u32,
	counts: Vec<u8>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		Self {
			width,
			depth,
			bits: width.trailing_zeros(),
			counts: vec![0; depth * width],
		}
	}

	fn heap_bytes(&self) -> usize {
		self.depth * self.width
	}

	#[inline(always)]
	fn mix<K: Hash>(key: &K) -> u64 {
		let mut hasher = FoldHasher(SEED, false);
		key.hash(&mut hasher);
		let mut z = hasher.finish();
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}

	#[inline(always)]
	fn index(&self, h: u64, row: usize) -> usize {
		let slice = h.rotate_right((row as u32 * self.bits) & 63) as usize;
		row * self.width + (slice & (self.width - 1))
	}

	#[inline(always)]
	fn increment(&mut self, h: u64) {
		let mut idx = [0usize; SKETCH_DEPTH];
		let mut min = MAX_COUNT;
		for row in 0..SKETCH_DEPTH {
			idx[row] = self.index(h, row);
			min = min.min(self.counts[idx[row]]);
		}
		if min == MAX_COUNT {
			return;
		}
		for &i in idx.iter() {
			if self.counts[i] == min {
				self.counts[i] += 1;
			}
		}
	}

	#[inline(always)]
	fn estimate(&self, h: u64) -> u32 {
		(0..SKETCH_DEPTH)
			.map(|row| self.counts[self.index(h, row)])
			.min()
			.unwrap_or(0) as u32
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
}
