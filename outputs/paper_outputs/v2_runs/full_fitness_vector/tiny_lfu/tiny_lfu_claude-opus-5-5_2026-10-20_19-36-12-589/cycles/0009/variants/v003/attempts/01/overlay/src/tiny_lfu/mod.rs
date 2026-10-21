use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
// The unrolled sketch paths below hard-code four rows.
const _: () = assert!(SKETCH_DEPTH == 4);
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
/// Saturation point of the `u16` counters, expressed in the `u32` comparison domain.
const MAX_COUNT: u32 = u16::MAX as u32;
/// `u16` counters per 64-byte block.
const BLOCK_COUNTERS: usize = 32;
/// Counters per row lane inside a block (`BLOCK_COUNTERS / SKETCH_DEPTH`).
const LANE_COUNTERS: usize = BLOCK_COUNTERS / SKETCH_DEPTH;
const _: () = assert!(LANE_COUNTERS == 8);

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
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, u32, FxBuild>,
	/// Resident entries laid out as a CLOCK ring; slot index == ring position.
	nodes: Vec<Node<K, V>>,
	/// One reference bit per slot (`ceil(capacity / 64)` words); set on hit.
	refbits: Vec<u64>,
	/// CLOCK hand: next slot to examine for eviction.
	hand: u32,
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
			refbits: vec![0u64; capacity.div_ceil(64)],
			hand: 0,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u16>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	#[inline(always)]
	fn set_ref(&mut self, idx: u32) {
		let i = idx as usize;
		self.refbits[i >> 6] |= 1u64 << (i & 63);
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment_only(key_hash(key));
		let &idx = self.map.get(key)?;
		self.set_ref(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		// Conservative update returns the key's post-increment estimate.
		let incoming_freq = self.sketch.increment_hash(h);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.set_ref(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			// New entry's ref bit stays clear (it has not been re-referenced yet).
			self.nodes.push(Node {
				key: key.clone(),
				value,
			});
			self.map.insert(key, idx);
			return;
		}

		// CLOCK sweep: clear set ref bits until a slot with a clear bit is found.
		let cap = self.capacity as u32;
		let mut hand = self.hand;
		let mut steps = 0usize;
		loop {
			let i = hand as usize;
			let word = &mut self.refbits[i >> 6];
			let mask = 1u64 << (i & 63);
			if *word & mask == 0 {
				break;
			}
			*word &= !mask;
			hand += 1;
			if hand == cap {
				hand = 0;
			}
			steps += 1;
			debug_assert!(steps <= self.capacity, "TinyLfuCache: CLOCK sweep did not terminate");
		}
		// The hand rests on the victim; it only moves past it if the newcomer is admitted.
		self.hand = hand;
		let victim_idx = hand;

		let victim_freq = self
			.sketch
			.estimate_hash(key_hash(&self.nodes[victim_idx as usize].key));
		if incoming_freq > victim_freq {
			let slot = &mut self.nodes[victim_idx as usize];
			self.map.remove(&slot.key);
			slot.key = key.clone();
			slot.value = value;
			self.map.insert(key, victim_idx);
			// Ref bit is already clear for the victim slot; leave it clear for the newcomer.
			let next = victim_idx + 1;
			self.hand = if next == cap { 0 } else { next };
		}
	}
}

/// One 64-byte, cache-line-aligned block of 32 `u16` counters. Row `r` owns the
/// disjoint lane `[r * 8, (r + 1) * 8)` within the block.
#[derive(Debug, Clone, Copy)]
#[repr(C, align(64))]
struct Block([u16; BLOCK_COUNTERS]);

const _: () = assert!(std::mem::size_of::<Block>() == 64);
const _: () = assert!(std::mem::align_of::<Block>() == 64);

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	/// `SKETCH_DEPTH * width / 32` cache-line blocks. All four of a key's counters live
	/// in one block, one per row lane. Flat counter index `i` is `blocks[i / 32].0[i % 32]`.
	blocks: Vec<Block>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let num_blocks = SKETCH_DEPTH * width / BLOCK_COUNTERS;
		debug_assert!(num_blocks >= 1 && num_blocks.is_power_of_two());
		let blocks = vec![Block([0u16; BLOCK_COUNTERS]); num_blocks];
		Self { width, blocks }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u16>()`.
	fn heap_bytes(&self) -> usize {
		SKETCH_DEPTH * self.width * std::mem::size_of::<u16>()
	}

	/// The key's four flat counter indices, one per row lane of a single block.
	/// Block choice uses the low bits of `h`; lane choice uses an independent re-mix.
	#[inline(always)]
	fn indices(&self, h: u64) -> [usize; 4] {
		let m = fmix64(h ^ 0x9e37_79b9_7f4a_7c15);
		let block = (h as usize) & (self.blocks.len() - 1);
		let base = block * BLOCK_COUNTERS;
		let i0 = base + (m as usize & 7);
		let i1 = base + LANE_COUNTERS + ((m >> 8) as usize & 7);
		let i2 = base + 2 * LANE_COUNTERS + ((m >> 16) as usize & 7);
		let i3 = base + 3 * LANE_COUNTERS + ((m >> 24) as usize & 7);
		debug_assert!(i3 < self.blocks.len() * BLOCK_COUNTERS);
		[i0, i1, i2, i3]
	}

	#[inline(always)]
	unsafe fn load(&self, i: usize) -> u32 {
		*self
			.blocks
			.get_unchecked(i / BLOCK_COUNTERS)
			.0
			.get_unchecked(i % BLOCK_COUNTERS) as u32
	}

	#[inline(always)]
	unsafe fn store(&mut self, i: usize, v: u32) {
		*self
			.blocks
			.get_unchecked_mut(i / BLOCK_COUNTERS)
			.0
			.get_unchecked_mut(i % BLOCK_COUNTERS) = v as u16;
	}

	/// Fused single-pass conservative update. Loads the four counters once into
	/// registers, computes the minimum, then writes back from the locals, bumping only
	/// counters equal to the minimum (branch-free). Saturates at `MAX_COUNT`
	/// (`u16::MAX`): if the minimum is already at the max, nothing changes.
	/// Returns the key's new estimate (`min + 1`, saturating).
	#[inline(always)]
	fn increment_hash(&mut self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: the block index is masked with `blocks.len() - 1` (a power of two), so
		// it is in bounds, and each slot is `r * 8 + (bits & 7) < 32`. The four indices
		// fall in the disjoint lanes `[r * 8, (r + 1) * 8)` of that single in-bounds
		// block, so they never alias and the loads/stores below are independent.
		unsafe {
			let c0 = self.load(i0);
			let c1 = self.load(i1);
			let c2 = self.load(i2);
			let c3 = self.load(i3);
			let min = c0.min(c1).min(c2.min(c3));
			// 1 unless the minimum is saturated; guards the `ci == min` bump against overflow.
			let inc = (min < MAX_COUNT) as u32;
			self.store(i0, c0 + (((c0 == min) as u32) & inc));
			self.store(i1, c1 + (((c1 == min) as u32) & inc));
			self.store(i2, c2 + (((c2 == min) as u32) & inc));
			self.store(i3, c3 + (((c3 == min) as u32) & inc));
			min + inc
		}
	}

	/// Same update as `increment_hash`, discarding the estimate (used on `get`).
	#[inline(always)]
	fn increment_only(&mut self, h: u64) {
		let _ = self.increment_hash(h);
	}

	#[inline(always)]
	fn estimate_hash(&self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: indices are in bounds; see `increment_hash`.
		unsafe {
			let c0 = self.load(i0);
			let c1 = self.load(i1);
			let c2 = self.load(i2);
			let c3 = self.load(i3);
			c0.min(c1).min(c2.min(c3))
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

	fn counter(s: &CountMinSketch, i: usize) -> u16 {
		s.blocks[i / BLOCK_COUNTERS].0[i % BLOCK_COUNTERS]
	}

	fn set_counter(s: &mut CountMinSketch, i: usize, v: u16) {
		s.blocks[i / BLOCK_COUNTERS].0[i % BLOCK_COUNTERS] = v;
	}

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
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 16);
	}

	#[test]
	fn sketch_layout_is_blocked() {
		let s = CountMinSketch::new(16);
		assert_eq!(s.heap_bytes(), SKETCH_DEPTH * 16 * 2);
		assert_eq!(s.blocks.len() * 64, s.heap_bytes());
		assert_eq!(s.blocks.as_ptr() as usize % 64, 0);
		for k in 0u64..1000 {
			let idx = s.indices(key_hash(&k));
			let b = idx[0] / BLOCK_COUNTERS;
			for (r, &i) in idx.iter().enumerate() {
				assert_eq!(i / BLOCK_COUNTERS, b);
				assert_eq!((i % BLOCK_COUNTERS) / LANE_COUNTERS, r);
			}
		}
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16);
		let h = key_hash(&7u64);
		// Inflate one row's counter so it sits above the others.
		let i0 = s.indices(h)[0];
		set_counter(&mut s, i0, 5);
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(counter(&s, i0), 5);
		assert_eq!(s.estimate_hash(h), 1);
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(counter(&s, i0), 5);
		s.increment_only(h);
		assert_eq!(s.estimate_hash(h), 3);
		assert_eq!(counter(&s, i0), 5);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16);
		for b in s.blocks.iter_mut() {
			for c in b.0.iter_mut() {
				*c = u16::MAX;
			}
		}
		let h = key_hash(&3u64);
		assert_eq!(s.increment_hash(h), u16::MAX as u32);
		assert!(s.blocks.iter().all(|b| b.0.iter().all(|&c| c == u16::MAX)));
	}

	#[test]
	fn eviction_prefers_hot_items() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2 (and set its CLOCK ref bit)
		cache.get(&3);
		cache.get(&3);
		// CLOCK sweep clears slot 0 (key 1) and picks slot 1 (key 2) as victim.
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
		cache.put(1, 11); // update sets key 1's ref bit, like a hit
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}
}
