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
/// Doorkeeper reset period, in sketch increments, per unit of capacity.
const DOORKEEPER_RESET_FACTOR: usize = 16;

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
			sketch: CountMinSketch::new(width, capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: the `depth * width` `u16`
	/// counter table plus the doorkeeper bit array.
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
		// Doorkeeper-gated conservative update returns the key's post-increment estimate.
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

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	/// Flattened `SKETCH_DEPTH * width` saturating `u16` counters; row `r` occupies
	/// `[r * width, (r + 1) * width)`.
	counts: Vec<u16>,
	/// Doorkeeper: one-bit-per-position membership filter of `bits` bits (`bits` is a
	/// power of two). A key's first occurrence only sets its two bits here; counters are
	/// bumped only once the key is already a doorkeeper member.
	doorkeeper: Vec<u64>,
	/// `min(width, 64 * doorkeeper.len())`; power of two.
	dk_mask: u64,
	/// Increments since the last doorkeeper clear.
	ops: u32,
	/// Doorkeeper is cleared every `reset_period` increments (`16 * capacity`).
	reset_period: u32,
}

impl CountMinSketch {
	fn new(width: usize, capacity: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let counts = vec![0u16; SKETCH_DEPTH * width];
		let words = (width / 64).max(1);
		let bits = width.min(words * 64);
		debug_assert!(bits.is_power_of_two());
		let reset_period = capacity
			.saturating_mul(DOORKEEPER_RESET_FACTOR)
			.clamp(1, u32::MAX as usize) as u32;
		Self {
			width,
			counts,
			doorkeeper: vec![0u64; words],
			dk_mask: (bits - 1) as u64,
			ops: 0,
			reset_period,
		}
	}

	/// Heap bytes backing the counter table plus the doorkeeper bit array.
	fn heap_bytes(&self) -> usize {
		SKETCH_DEPTH * self.width * std::mem::size_of::<u16>()
			+ self.doorkeeper.len() * std::mem::size_of::<u64>()
	}

	/// Flat index for `row`, derived by double hashing from one 64-bit hash.
	#[inline(always)]
	fn index(&self, h1: u32, h2: u32, row: usize) -> usize {
		let col = (h1.wrapping_add((row as u32).wrapping_mul(h2)) as usize) & (self.width - 1);
		row * self.width + col
	}

	/// The key's four flat counter indices, one per row.
	#[inline(always)]
	fn indices(&self, h: u64) -> [usize; 4] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let i0 = self.index(h1, h2, 0);
		let i1 = self.index(h1, h2, 1);
		let i2 = self.index(h1, h2, 2);
		let i3 = self.index(h1, h2, 3);
		debug_assert!(i0 < self.counts.len());
		debug_assert!(i1 < self.counts.len());
		debug_assert!(i2 < self.counts.len());
		debug_assert!(i3 < self.counts.len());
		[i0, i1, i2, i3]
	}

	/// The key's two doorkeeper bit positions, taken from the same 64-bit hash.
	#[inline(always)]
	fn dk_bits(&self, h: u64) -> (usize, usize) {
		let b0 = ((h >> 7) & self.dk_mask) as usize;
		let b1 = ((h >> 29) & self.dk_mask) as usize;
		(b0, b1)
	}

	/// Whether the key is a doorkeeper member (both bits set).
	#[inline(always)]
	fn dk_contains(&self, h: u64) -> bool {
		let (b0, b1) = self.dk_bits(h);
		let w0 = self.doorkeeper[b0 >> 6];
		let w1 = self.doorkeeper[b1 >> 6];
		(w0 >> (b0 & 63)) & 1 != 0 && (w1 >> (b1 & 63)) & 1 != 0
	}

	/// Sets both doorkeeper bits for the key.
	#[inline(always)]
	fn dk_insert(&mut self, h: u64) {
		let (b0, b1) = self.dk_bits(h);
		self.doorkeeper[b0 >> 6] |= 1u64 << (b0 & 63);
		self.doorkeeper[b1 >> 6] |= 1u64 << (b1 & 63);
	}

	/// Counts one increment; clears the doorkeeper every `reset_period` increments.
	#[inline(always)]
	fn tick(&mut self) {
		self.ops += 1;
		if self.ops == self.reset_period {
			self.reset_doorkeeper();
		}
	}

	/// Zeroes the doorkeeper. Counters are left untouched (never halved).
	#[cold]
	#[inline(never)]
	fn reset_doorkeeper(&mut self) {
		for w in self.doorkeeper.iter_mut() {
			*w = 0;
		}
		self.ops = 0;
	}

	/// Count-min minimum over the four counters (doorkeeper excluded).
	#[inline(always)]
	fn estimate_counts(&self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: indices are in bounds; see `increment_hash`.
		unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			c0.min(c1).min(c2.min(c3))
		}
	}

	/// Doorkeeper-gated increment. If the key is not yet a doorkeeper member, its two
	/// bits are set and the counters are left untouched (returns `counts_min + 1`).
	/// Otherwise runs the fused single-pass conservative update: loads the four counters
	/// once, computes the minimum, then writes back bumping only counters equal to the
	/// minimum (branch-free), saturating at `MAX_COUNT` (`u16::MAX`). Returns the key's
	/// new estimate (`min + inc + 1`, the `+1` being the doorkeeper bit).
	#[inline(always)]
	fn increment_hash(&mut self, h: u64) -> u32 {
		if !self.dk_contains(h) {
			self.dk_insert(h);
			let est = self.estimate_counts(h) + 1;
			self.tick();
			return est;
		}
		let [i0, i1, i2, i3] = self.indices(h);
		// SAFETY: `index` masks the column with `width - 1` (width is a power of two), so
		// `col < width`, and `row < SKETCH_DEPTH`, giving `row * width + col <
		// SKETCH_DEPTH * width == counts.len()`. Each index lies in a distinct row's
		// disjoint range `[r * width, (r + 1) * width)`, so the four indices never alias
		// and the loads/stores below are independent.
		let est = unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			let min = c0.min(c1).min(c2.min(c3));
			// 1 unless the minimum is saturated; guards the `ci == min` bump against overflow.
			let inc = (min < MAX_COUNT) as u32;
			*self.counts.get_unchecked_mut(i0) = (c0 + (((c0 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i1) = (c1 + (((c1 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i2) = (c2 + (((c2 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i3) = (c3 + (((c3 == min) as u32) & inc)) as u16;
			min + inc + 1
		};
		self.tick();
		est
	}

	/// Same update as `increment_hash`, discarding the estimate (used on `get`).
	#[inline(always)]
	fn increment_only(&mut self, h: u64) {
		let _ = self.increment_hash(h);
	}

	/// Counter minimum plus one if the key is a doorkeeper member.
	#[inline(always)]
	fn estimate_hash(&self, h: u64) -> u32 {
		self.estimate_counts(h) + self.dk_contains(h) as u32
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
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 16);
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16, 1024);
		let h = key_hash(&7u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Inflate one row's counter so it sits above the others.
		let i0 = s.index(h1, h2, 0);
		s.counts[i0] = 5;
		let before = s.counts.clone();
		// First occurrence: doorkeeper only, counters untouched.
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.counts, before);
		assert_eq!(s.estimate_hash(h), 1);
		// Subsequent occurrences: conservative update on the counters.
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(s.counts[i0], 5);
		assert_eq!(s.estimate_hash(h), 2);
		s.increment_only(h);
		assert_eq!(s.estimate_hash(h), 3);
		assert_eq!(s.counts[i0], 5);
	}

	#[test]
	fn doorkeeper_reset_clears_membership_but_keeps_counters() {
		let mut s = CountMinSketch::new(16, 1);
		// reset_period == 16
		let h = key_hash(&11u64);
		s.increment_hash(h); // doorkeeper
		s.increment_hash(h); // counters -> 1
		assert_eq!(s.estimate_hash(h), 2);
		for _ in 2..16 {
			s.increment_hash(h);
		}
		// The 16th increment triggered the reset.
		assert_eq!(s.ops, 0);
		assert!(s.doorkeeper.iter().all(|&w| w == 0));
		assert_eq!(s.estimate_hash(h), 15);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16, 1024);
		for c in s.counts.iter_mut() {
			*c = u16::MAX;
		}
		let h = key_hash(&3u64);
		// Doorkeeper admission first, then a saturated counter update.
		assert_eq!(s.increment_hash(h), u16::MAX as u32 + 1);
		assert_eq!(s.increment_hash(h), u16::MAX as u32 + 1);
		assert!(s.counts.iter().all(|&c| c == u16::MAX));
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
