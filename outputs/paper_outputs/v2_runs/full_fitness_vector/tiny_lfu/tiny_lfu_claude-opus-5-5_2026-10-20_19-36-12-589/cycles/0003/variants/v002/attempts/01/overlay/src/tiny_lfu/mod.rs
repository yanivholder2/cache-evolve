use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Sentinel for "no link" in the u32-indexed intrusive list.
const NIL: u32 = u32::MAX;

const FX_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Minimal FxHash-style hasher (local; no external crates).
#[derive(Default, Clone, Copy)]
pub struct FxLikeHasher {
	state: u64,
}

impl FxLikeHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxLikeHasher {
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
			self.add(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u16(&mut self, i: u16) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.state
	}
}

type FxBuild = BuildHasherDefault<FxLikeHasher>;

/// murmur3 fmix64 finaliser for avalanche.
#[inline(always)]
fn fmix64(mut h: u64) -> u64 {
	h ^= h >> 33;
	h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
	h ^= h >> 33;
	h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	h ^= h >> 33;
	h
}

/// Hash a key once with the fast hasher plus a finaliser mix.
#[inline]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut hasher = FxLikeHasher::default();
	key.hash(&mut hasher);
	fmix64(hasher.finish())
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, u32, FxBuild>,
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
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		if capacity >= u32::MAX as usize {
			log::error!(
				"TinyLfuCache.try_new: capacity={capacity} must be < u32::MAX for u32 node indices"
			);
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
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

	/// Heap bytes held by the frequency sketch for this cache: `num_blocks * 64`
	/// (equal to `depth * width * size_of::<u32>()` for power-of-two widths).
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment_hash(key_hash(key));
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		self.sketch.increment_hash(h);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self.head;
		debug_assert!(victim_idx != NIL, "TinyLfuCache: full cache has no victim");
		let victim_freq = self
			.sketch
			.estimate_hash(key_hash(&self.nodes[victim_idx as usize].key));
		let incoming_freq = self.sketch.estimate_hash(h);
		if incoming_freq > victim_freq {
			let victim_key = self.nodes[victim_idx as usize].key.clone();
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx as usize].key = key.clone();
			self.nodes[victim_idx as usize].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
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
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
	}

	fn push_back(&mut self, idx: u32) {
		let i = idx as usize;
		self.nodes[i].prev = self.tail;
		self.nodes[i].next = NIL;
		if self.tail != NIL {
			self.nodes[self.tail as usize].next = idx;
		} else {
			self.head = idx;
		}
		self.tail = idx;
	}
}

/// Counter type held in the sketch.
type Counter = u32;

/// Bytes per sketch block: one cache line.
const BLOCK_BYTES: usize = 64;

/// Counters per 64-byte block.
const COUNTERS_PER_BLOCK: usize = BLOCK_BYTES / std::mem::size_of::<Counter>();

/// Bits of the low hash consumed per row to select a slot within a block.
const SLOT_BITS: u32 = COUNTERS_PER_BLOCK.trailing_zeros();

const SLOT_MASK: u64 = (COUNTERS_PER_BLOCK as u64) - 1;

// Every row's slot field must fit, non-overlapping, in the low 32 bits of the hash
// (the high 32 bits select the block).
const _: () = assert!(COUNTERS_PER_BLOCK.is_power_of_two());
const _: () = assert!(SLOT_BITS as usize * SKETCH_DEPTH <= 32);

/// One cache-line-aligned block of counters; all `depth` counters of a key live in one block.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
struct Block([Counter; COUNTERS_PER_BLOCK]);

const _: () = assert!(std::mem::size_of::<Block>() == BLOCK_BYTES);

#[derive(Debug)]
struct CountMinSketch {
	/// Power-of-two number of 64-byte blocks.
	num_blocks: usize,
	depth: usize,
	/// Blocked counter table: key `h` uses block `(h >> 32) & (num_blocks - 1)`, and row `r`
	/// uses slot `(h >> (r * SLOT_BITS)) & SLOT_MASK` within that block.
	counts: Vec<Block>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let depth = SKETCH_DEPTH;
		let total = depth * width;
		let num_blocks = total.div_ceil(COUNTERS_PER_BLOCK).max(1).next_power_of_two();
		let counts = vec![Block([0; COUNTERS_PER_BLOCK]); num_blocks];
		Self {
			num_blocks,
			depth,
			counts,
		}
	}

	/// Heap bytes backing the counter table: `num_blocks * 64`.
	fn heap_bytes(&self) -> usize {
		self.num_blocks * std::mem::size_of::<Block>()
	}

	#[inline(always)]
	fn block_index(&self, h: u64) -> usize {
		((h >> 32) as usize) & (self.num_blocks - 1)
	}

	#[inline(always)]
	fn slot(h: u64, row: usize) -> usize {
		((h >> (row as u32 * SLOT_BITS)) & SLOT_MASK) as usize
	}

	#[inline]
	fn increment_hash(&mut self, h: u64) {
		let b = self.block_index(h);
		let depth = self.depth;
		let block = &mut self.counts[b].0;
		for row in 0..depth {
			let counter = &mut block[Self::slot(h, row)];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	#[inline]
	fn estimate_hash(&self, h: u64) -> u32 {
		let block = &self.counts[self.block_index(h)].0;
		let mut min = u32::MAX;
		for row in 0..self.depth {
			let c = block[Self::slot(h, row)];
			if c < min {
				min = c;
			}
		}
		if self.depth == 0 {
			0
		} else {
			min
		}
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
	fn node_is_compact() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}

	#[test]
	fn sketch_blocks_are_cache_line_aligned() {
		assert_eq!(std::mem::align_of::<Block>(), 64);
		let cache = TinyLfuCache::<u64, u64>::new(2048);
		let ptr = cache.sketch.counts.as_ptr() as usize;
		assert_eq!(ptr % 64, 0);
		// Heap size unchanged vs. the flat depth * width * 4 layout.
		let width = (2048 * SKETCH_WIDTH_FACTOR).next_power_of_two();
		assert_eq!(cache.sketch_heap_bytes(), SKETCH_DEPTH * width * 4);
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
