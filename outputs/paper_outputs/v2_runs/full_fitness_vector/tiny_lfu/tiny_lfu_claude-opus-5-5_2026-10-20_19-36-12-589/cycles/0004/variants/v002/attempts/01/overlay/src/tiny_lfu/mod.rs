use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
/// TinyLFU reset period: halve all counters every `SAMPLE_FACTOR * capacity` increments.
const SAMPLE_FACTOR: usize = 10;
const MIN_SAMPLE_SIZE: u32 = 16;

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
		let sample_size = (capacity.saturating_mul(SAMPLE_FACTOR).min(u32::MAX as usize / 2)
			as u32)
			.max(MIN_SAMPLE_SIZE);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} sample_size={sample_size}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
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
		self.sketch.increment_hash(key_hash(key));
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		// Conservative update returns the key's post-increment estimate.
		let incoming_freq = self.sketch.increment_hash(h);
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

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	/// Flattened `depth * width` counters; row `r` occupies `[r * width, (r + 1) * width)`.
	counts: Vec<u32>,
	/// Increments since the last reset (halved, not zeroed, on reset).
	additions: u32,
	/// Reset period: when `additions` reaches this, every counter is halved.
	sample_size: u32,
}

impl CountMinSketch {
	fn new(width: usize, sample_size: u32) -> Self {
		debug_assert!(width.is_power_of_two());
		debug_assert!(sample_size > 0);
		let depth = SKETCH_DEPTH;
		let counts = vec![0; depth * width];
		Self {
			width,
			depth,
			counts,
			additions: 0,
			sample_size,
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	/// Flat index for `row`, derived by double hashing from one 64-bit hash.
	#[inline(always)]
	fn index(&self, h1: u32, h2: u32, row: usize) -> usize {
		let col = (h1.wrapping_add((row as u32).wrapping_mul(h2)) as usize) & (self.width - 1);
		row * self.width + col
	}

	/// Conservative update: read the key's `depth` counters, find their minimum, and bump
	/// only the counters equal to that minimum (branch-free). Counters already above the
	/// minimum are left untouched. Saturates at `MAX_COUNT`: if the minimum is already at
	/// the max, nothing changes. Returns the key's new estimate (`min + 1`, saturating).
	///
	/// Every `sample_size` increments the whole sketch is halved (TinyLFU reset); when that
	/// happens the returned estimate is halved too, so it matches `estimate_hash` afterwards.
	#[inline]
	fn increment_hash(&mut self, h: u64) -> u32 {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let mut idx = [0usize; SKETCH_DEPTH];
		let mut min = MAX_COUNT;
		for (row, slot) in idx.iter_mut().enumerate() {
			let i = self.index(h1, h2, row);
			*slot = i;
			min = min.min(self.counts[i]);
		}
		// 1 unless the minimum is saturated; guards the `c == min` bump against overflow.
		let inc = (min < MAX_COUNT) as u32;
		for &i in idx.iter() {
			let c = &mut self.counts[i];
			*c += ((*c == min) as u32) & inc;
		}
		let est = min + inc;
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
			return est >> 1;
		}
		est
	}

	/// Halve every counter in place and keep half the sample weight (Caffeine-style aging).
	#[cold]
	#[inline(never)]
	fn reset(&mut self) {
		for c in self.counts.iter_mut() {
			*c >>= 1;
		}
		self.additions /= 2;
	}

	#[inline]
	fn estimate_hash(&self, h: u64) -> u32 {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let mut min = u32::MAX;
		for row in 0..self.depth {
			let c = self.counts[self.index(h1, h2, row)];
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

	/// Large sample size so sketch-only tests never trigger a reset.
	const NO_RESET: u32 = u32::MAX / 2;

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
	fn sample_size_is_ten_times_capacity_with_floor() {
		let c = TinyLfuCache::<i64, i64>::new(2048);
		assert_eq!(c.sketch.sample_size, 20480);
		let c = TinyLfuCache::<i64, i64>::new(1);
		assert_eq!(c.sketch.sample_size, MIN_SAMPLE_SIZE);
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16, NO_RESET);
		let h = key_hash(&7u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Inflate one row's counter so it sits above the others.
		let i0 = s.index(h1, h2, 0);
		s.counts[i0] = 5;
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.counts[i0], 5);
		assert_eq!(s.estimate_hash(h), 1);
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(s.counts[i0], 5);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16, NO_RESET);
		for c in s.counts.iter_mut() {
			*c = MAX_COUNT;
		}
		let h = key_hash(&3u64);
		assert_eq!(s.increment_hash(h), MAX_COUNT);
		assert!(s.counts.iter().all(|&c| c == MAX_COUNT));
	}

	#[test]
	fn reset_halves_every_counter_after_sample_size_increments() {
		let sample = 16;
		let mut s = CountMinSketch::new(64, sample);
		// Seed distinct values everywhere, then snapshot after sample-1 increments.
		for (i, c) in s.counts.iter_mut().enumerate() {
			*c = (i as u32) * 3 + 100;
		}
		for k in 0..(sample - 1) as u64 {
			s.increment_hash(key_hash(&k));
		}
		assert_eq!(s.additions, sample - 1);
		let h = key_hash(&999u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Predict the post-increment, pre-halving table for the final increment.
		let mut expected = s.counts.clone();
		let idx: Vec<usize> = (0..SKETCH_DEPTH).map(|r| s.index(h1, h2, r)).collect();
		let min = idx.iter().map(|&i| expected[i]).min().unwrap();
		for &i in &idx {
			if expected[i] == min {
				expected[i] += 1;
			}
		}
		let est = s.increment_hash(h);
		assert_eq!(est, (min + 1) >> 1);
		for (got, want) in s.counts.iter().zip(expected.iter()) {
			assert_eq!(*got, *want >> 1);
		}
		assert_eq!(s.additions, sample / 2);
		assert_eq!(s.estimate_hash(h), est);
	}

	#[test]
	fn estimate_one_drops_to_zero_after_reset() {
		let mut s = CountMinSketch::new(1024, NO_RESET);
		let h = key_hash(&42u64);
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.estimate_hash(h), 1);
		s.reset();
		assert_eq!(s.estimate_hash(h), 0);
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
