use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
/// Aging period multiplier: reset (halve) after `SAMPLE_FACTOR * capacity` additions.
const SAMPLE_FACTOR: usize = 10;
/// Four-bit counters per 64-byte block (8 words x 16 nibbles).
const COUNTERS_PER_BLOCK: usize = 8 * 16;
const RESET_MASK: u64 = 0x7777_7777_7777_7777;

/// FxHash-style multiply-rotate hasher with a splitmix64 avalanche finish.
/// Cheap compared to SipHash; the finish step gives good bit dispersion for
/// both the hashbrown control bytes and the sketch index slices.
#[derive(Default, Clone, Copy)]
pub struct FastHasher {
	state: u64,
}

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FastHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

#[inline(always)]
fn splitmix64(mut z: u64) -> u64 {
	z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

impl Hasher for FastHasher {
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
		splitmix64(self.state)
	}
}

type FastBuildHasher = BuildHasherDefault<FastHasher>;

/// Hash a key once with the fast hasher; the result feeds all sketch rows.
#[inline(always)]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut h = FastHasher::default();
	key.hash(&mut h);
	h.finish()
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
			sketch: CountMinSketch::new(width, capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `blocks * 64`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment_hashed(key_hash(key));
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let key_h = key_hash(&key);
		self.sketch.increment_hashed(key_h);
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

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_freq = self
			.sketch
			.estimate_hashed(key_hash(&self.nodes[victim_idx].key));
		let incoming_freq = self.sketch.estimate_hashed(key_h);
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

/// One 64-byte cache line: 8 words x 16 four-bit counters.
#[derive(Debug, Clone, Copy, Default)]
#[repr(align(64))]
struct Block([u64; 8]);

/// Caffeine-style blocked count-min sketch with packed 4-bit counters.
///
/// All four of a key's counters live in a single 64-byte block, so every
/// increment/estimate touches exactly one cache line.
#[derive(Debug)]
struct CountMinSketch {
	table: Vec<Block>,
	block_mask: usize,
	additions: u32,
	sample_size: u32,
}

impl CountMinSketch {
	/// `width` is the legacy per-row width; the blocked table holds at least
	/// `SKETCH_DEPTH * width` four-bit counters.
	fn new(width: usize, capacity: usize) -> Self {
		let counters = SKETCH_DEPTH * width;
		let blocks = ((counters + COUNTERS_PER_BLOCK - 1) / COUNTERS_PER_BLOCK)
			.max(1)
			.next_power_of_two();
		let sample_size = capacity
			.saturating_mul(SAMPLE_FACTOR)
			.clamp(1, u32::MAX as usize) as u32;
		Self {
			table: vec![Block::default(); blocks],
			block_mask: blocks - 1,
			additions: 0,
			sample_size,
		}
	}

	/// Heap bytes backing the counter table: `blocks * 64`.
	fn heap_bytes(&self) -> usize {
		self.table.len() * std::mem::size_of::<Block>()
	}

	#[inline(always)]
	fn block_index(&self, h: u64) -> usize {
		((h >> 32) as usize) & self.block_mask
	}

	/// (word index, bit shift) of row `i`'s counter within the block.
	#[inline(always)]
	fn position(h: u64, i: usize) -> (usize, u32) {
		let s = 8 * i;
		let word = i * 2 + ((h >> s) & 1) as usize;
		let shift = (((h >> (s + 1)) & 0xF) as u32) << 2;
		(word, shift)
	}

	/// Conservative update: increment only the counters equal to the key's
	/// current minimum. All reads stay within the already-loaded block.
	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		let bi = self.block_index(h);
		let block = &mut self.table[bi].0;

		// First pass: locate counters and find the minimum.
		let mut pos = [(0usize, 0u32); SKETCH_DEPTH];
		let mut vals = [0u64; SKETCH_DEPTH];
		let mut min = u64::MAX;
		for i in 0..SKETCH_DEPTH {
			let (w, shift) = Self::position(h, i);
			let c = (block[w] >> shift) & 0xF;
			pos[i] = (w, shift);
			vals[i] = c;
			if c < min {
				min = c;
			}
		}
		if min >= 15 {
			// Saturated: no addition.
			return;
		}

		// Second pass: bump only the minimum counters. Re-read each nibble so a
		// nibble shared by two rows is incremented at most once.
		let mut added = false;
		for i in 0..SKETCH_DEPTH {
			if vals[i] != min {
				continue;
			}
			let (w, shift) = pos[i];
			if (block[w] >> shift) & 0xF > min {
				continue;
			}
			block[w] += 1u64 << shift;
			added = true;
		}

		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.reset();
			}
		}
	}

	#[inline]
	fn estimate_hashed(&self, h: u64) -> u32 {
		let block = &self.table[self.block_index(h)].0;
		let mut min = u32::MAX;
		for i in 0..SKETCH_DEPTH {
			let (w, shift) = Self::position(h, i);
			let c = ((block[w] >> shift) & 0xF) as u32;
			if c < min {
				min = c;
			}
		}
		min
	}

	/// Aging: halve every counter in place and halve the addition count.
	#[cold]
	fn reset(&mut self) {
		for block in self.table.iter_mut() {
			for w in block.0.iter_mut() {
				*w = (*w >> 1) & RESET_MASK;
			}
		}
		self.additions /= 2;
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
	fn sketch_counts_increment() {
		let mut s = CountMinSketch::new(16, 1000);
		let h = key_hash(&7i64);
		s.increment_hashed(h);
		s.increment_hashed(h);
		assert!(s.estimate_hashed(h) >= 2);
	}

	#[test]
	fn sketch_saturates_at_15() {
		let mut s = CountMinSketch::new(16, 1000);
		let h = key_hash(&7i64);
		for _ in 0..40 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 15);
	}

	#[test]
	fn sketch_ages_by_halving() {
		let mut s = CountMinSketch::new(16, 1);
		// sample_size = 10; increments of one key trigger a reset at 10 additions.
		let h = key_hash(&7i64);
		for _ in 0..10 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 5);
		assert_eq!(s.additions, 5);
	}

	#[test]
	fn conservative_update_spares_shared_counter() {
		let mut s = CountMinSketch::new(16, 1000);
		let h_a = key_hash(&7i64);
		for _ in 0..10 {
			s.increment_hashed(h_a);
		}
		assert_eq!(s.estimate_hashed(h_a), 10);

		// h_b shares the block and row 0's position bits with h_a, but flips the
		// word-select bit of rows 1..3, so those rows land on fresh counters.
		let h_b = h_a ^ ((1u64 << 8) | (1u64 << 16) | (1u64 << 24));
		assert_eq!(s.block_index(h_a), s.block_index(h_b));
		assert_eq!(CountMinSketch::position(h_a, 0), CountMinSketch::position(h_b, 0));
		let (w0, sh0) = CountMinSketch::position(h_a, 0);
		let before = (s.table[s.block_index(h_a)].0[w0] >> sh0) & 0xF;

		s.increment_hashed(h_b);

		let after = (s.table[s.block_index(h_a)].0[w0] >> sh0) & 0xF;
		// The shared counter (10) was above B's minimum (0), so it must not move.
		assert_eq!(before, after);
		assert_eq!(s.estimate_hashed(h_a), 10);
		assert_eq!(s.estimate_hashed(h_b), 1);
	}

	#[test]
	fn sketch_heap_within_budget() {
		let cache = TinyLfuCache::<i64, i64>::new(1024);
		assert!(cache.sketch_heap_bytes() <= 65536);
		assert_eq!(std::mem::align_of::<Block>(), 64);
		assert_eq!(std::mem::size_of::<Block>(), 64);
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
