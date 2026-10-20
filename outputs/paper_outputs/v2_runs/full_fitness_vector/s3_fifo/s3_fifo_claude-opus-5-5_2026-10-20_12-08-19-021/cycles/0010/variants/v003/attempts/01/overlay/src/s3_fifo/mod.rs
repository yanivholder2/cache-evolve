use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Adaptive split (inherited from cycle/0001/v003): the small/main split adapts
// at runtime (ARC-style). A ghost hit moves one slot from small to main; a
// small-queue graduate arriving while main is full moves one slot from main back
// to small. `small_size + main_size` always equals `capacity`; the ghost bound
// stays fixed at the initial main size.
//
// Slab layout (inherited from cycle/0002/v003): entries live in a slab
// (`slots`) addressed by `u32` indices; `small` / `main` are FIFOs of slot
// indices; `index` maps resident keys to their slot.
//
// Ghost ring (inherited from cycle/0003/v002): the ghost is a fixed-size ring
// of 64-bit key fingerprints (0 = empty slot) plus a fingerprint -> ring-position
// map.
//
// put-path trim (inherited from cycle/0004/v003): one fingerprint per miss,
// `alloc_with_key`, compare-based ring wrap.
//
// Split floor (inherited from cycle/0005/v001): the lower clamp on
// `small_size` is `capacity / 64`.
//
// Hot/cold split (inherited from cycle/0006/v002): the key lives in a parallel
// `keys: Vec<Option<K>>` array indexed by the same `u32` slot id.
//
// Freq out of Entry (inherited from cycle/0007/v001): `Entry` is `{ value }`
// (8 bytes for u64/u64), so the slab is a dense `[V]`.
//
// Packed freq (inherited from cycle/0008/v001): the 2-bit frequency is packed
// into the top bits of the `u32` stored in `index` as `(freq << FREQ_SHIFT) |
// slot`.
//
// Variant cycle/0010/v003 (H03): the upper clamp on `small_size` for the
// grow-small rule is lowered from `capacity / 2` to `capacity / 4` (the initial
// small size), so main always keeps at least 3/4 of the cache.
const MAX_FREQ: u32 = 3;

/// Packed index value layout: low 30 bits = slot id, top 2 bits = freq.
const FREQ_SHIFT: u32 = 30;
const SLOT_MASK: u32 = (1 << FREQ_SHIFT) - 1;
const FREQ_ONE: u32 = 1 << FREQ_SHIFT;

/// Divisor of the adaptive split's lower clamp: `small_size` never shrinks
/// below `(capacity / SMALL_FLOOR_DIV).max(1)` on a ghost hit.
const SMALL_FLOOR_DIV: usize = 64;

/// Divisor of the adaptive split's upper clamp: `small_size` never grows
/// past `(capacity / SMALL_CEIL_DIV).max(small_floor())` on a graduate.
const SMALL_CEIL_DIV: usize = 4;

// ---------------------------------------------------------------------------
// Inline FxHash (rustc-hash) — Cargo.toml cannot change, so no external crate.
// ---------------------------------------------------------------------------

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Debug, Default, Clone, Copy)]
pub struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add_to_hash(&mut self, word: u64) {
		self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for chunk in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(chunk);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
		for &b in chunks.remainder() {
			self.add_to_hash(b as u64);
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_u16(&mut self, i: u16) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add_to_hash(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.hash
	}
}

#[derive(Debug, Default, Clone, Copy)]
pub struct FxBuildHasher;

impl BuildHasher for FxBuildHasher {
	type Hasher = FxHasher;

	#[inline]
	fn build_hasher(&self) -> FxHasher {
		FxHasher::default()
	}
}

type FxHashMap<K, V> = HashMap<K, V, FxBuildHasher>;

/// Ghost fingerprint of a key: FxHash of the key, forced odd so it is never 0
/// (0 marks an empty ring slot).
#[inline]
fn fp<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut h = FxBuildHasher.build_hasher();
	key.hash(&mut h);
	h.finish() | 1
}

// ---------------------------------------------------------------------------

/// Key-free, freq-free slab entry. The key lives in the parallel
/// `S3Fifo::keys` array and the frequency in the packed `S3Fifo::index` value.
#[derive(Debug)]
pub struct Entry<K, V> {
	value: V,
	_k: PhantomData<K>,
}

impl<K, V> Entry<K, V> {
	#[inline]
	fn new(value: V) -> Self {
		Self { value, _k: PhantomData }
	}
}

/// Test-only instrumentation of the adaptive split.
#[cfg(test)]
#[derive(Debug, Default, Clone, Copy)]
struct SplitStats {
	/// Ghost hits that shrank small by one slot.
	shrinks: u64,
	/// Ghost hits on which the floor guard blocked a shrink.
	floor_blocks: u64,
	/// Graduates-while-main-full that grew small by one slot.
	grows: u64,
	/// Graduates-while-main-full on which the ceiling guard blocked a grow.
	ceil_blocks: u64,
	/// Smallest `small_size` observed.
	min_small: usize,
	/// Largest `small_size` observed.
	max_small: usize,
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	/// Slab of entries. A free slot (listed in `free`) keeps its stale value
	/// until the next alloc overwrites it; liveness is tracked by `keys`.
	slots: Vec<Entry<K, V>>,
	/// Parallel to `slots`: the resident key of each slot, `None` if free.
	keys: Vec<Option<K>>,
	free: Vec<u32>,
	/// Resident key -> packed `(freq << FREQ_SHIFT) | slot`.
	index: FxHashMap<K, u32>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	/// Ghost ring of key fingerprints; 0 = empty slot. Fixed length.
	ghost_ring: Vec<u64>,
	/// Next ring write position (wraps).
	ghost_head: usize,
	/// Live fingerprint -> ring position.
	ghost_map: FxHashMap<u64, u32>,
	ghost_live: usize,
	small_size: usize,
	main_size: usize,
	capacity: usize,
	ghost_cap: usize,
	/// Test-only: override of `SMALL_FLOOR_DIV`.
	#[cfg(test)]
	floor_div: usize,
	/// Test-only: override of `SMALL_CEIL_DIV`.
	#[cfg(test)]
	ceil_div: usize,
	#[cfg(test)]
	stats: SplitStats,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		if capacity > SLOT_MASK as usize + 1 {
			log::error!("S3Fifo.try_new: capacity must fit in 30-bit slot id");
			return None;
		}
		// Initial small = capacity / 4, which already equals the grow ceiling.
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let index = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);
		// The ring needs at least one slot so the wrap is defined. Ghost sizing
		// stays pinned to the *initial* main size, independent of the split floor.
		let ring_len = main_size.max(1);
		Some(Self {
			slots: Vec::with_capacity(capacity),
			keys: Vec::with_capacity(capacity),
			free: Vec::with_capacity(capacity),
			index,
			small: VecDeque::with_capacity(capacity),
			main: VecDeque::with_capacity(capacity),
			ghost_ring: vec![0u64; ring_len],
			ghost_head: 0,
			ghost_map: FxHashMap::with_capacity_and_hasher(ring_len + 1, FxBuildHasher),
			ghost_live: 0,
			small_size,
			main_size,
			capacity,
			ghost_cap: main_size,
			#[cfg(test)]
			floor_div: SMALL_FLOOR_DIV,
			#[cfg(test)]
			ceil_div: SMALL_CEIL_DIV,
			#[cfg(test)]
			stats: SplitStats { min_small: small_size, max_small: small_size, ..SplitStats::default() },
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	/// Current `(small_size, main_size)` split.
	#[inline]
	pub fn split(&self) -> (usize, usize) {
		(self.small_size, self.main_size)
	}

	#[inline]
	fn adaptive(&self) -> bool {
		self.capacity >= 4
	}

	/// Lower clamp on `small_size` for ghost-hit shrinking.
	#[inline]
	fn small_floor(&self) -> usize {
		#[cfg(test)]
		let div = self.floor_div;
		#[cfg(not(test))]
		let div = SMALL_FLOOR_DIV;
		(self.capacity / div).max(1)
	}

	/// Upper clamp on `small_size` for graduate-driven growing.
	#[inline]
	fn small_ceil(&self) -> usize {
		#[cfg(test)]
		let div = self.ceil_div;
		#[cfg(not(test))]
		let div = SMALL_CEIL_DIV;
		(self.capacity / div).max(self.small_floor())
	}

	// ---- slab helpers -----------------------------------------------------

	#[cfg(test)]
	#[inline]
	fn slot(&self, i: u32) -> &Entry<K, V> {
		&self.slots[i as usize]
	}

	#[inline]
	fn slot_mut(&mut self, i: u32) -> &mut Entry<K, V> {
		&mut self.slots[i as usize]
	}

	/// Mutable packed index entry of resident slot `i` (one hash probe).
	#[inline]
	fn packed_mut(&mut self, i: u32) -> &mut u32 {
		let k = self.keys[i as usize].as_ref().expect("S3Fifo: queue references a free slot");
		self.index.get_mut(k).expect("S3Fifo: resident key missing from index")
	}

	/// Store a new resident entry (freq 0) and register it in `index`.
	/// Exactly one key clone and one `index` hash operation.
	#[inline]
	fn alloc_with_key(&mut self, key: K, value: V) -> u32 {
		match self.free.pop() {
			Some(i) => {
				self.index.insert(key.clone(), i);
				self.keys[i as usize] = Some(key);
				self.slots[i as usize] = Entry::new(value);
				i
			}
			None => {
				let i = self.slots.len() as u32;
				self.index.insert(key.clone(), i);
				self.keys.push(Some(key));
				self.slots.push(Entry::new(value));
				i
			}
		}
	}

	/// Store an entry with a preset frequency (test helpers only).
	#[cfg(test)]
	fn alloc(&mut self, key: K, value: V, freq: u8) -> u32 {
		let i = self.alloc_with_key(key, value);
		let e = self.packed_mut(i);
		*e = ((freq as u32) << FREQ_SHIFT) | (*e & SLOT_MASK);
		i
	}

	/// Remove a resident slot from `keys` and `index`; return its key. The
	/// value stays in `slots[i]` until the next alloc overwrites it; the packed
	/// freq leaves with the index entry.
	#[inline]
	fn release(&mut self, i: u32) -> K {
		let k = self.keys[i as usize].take().expect("S3Fifo: double release of slot");
		self.index.remove(&k);
		self.free.push(i);
		k
	}

	// ---- ghost helpers ----------------------------------------------------

	/// O(1) ghost membership test + removal by precomputed fingerprint.
	#[inline]
	fn ghost_take_fp(&mut self, f: u64) -> bool {
		if let Some(p) = self.ghost_map.remove(&f) {
			self.ghost_ring[p as usize] = 0;
			self.ghost_live -= 1;
			true
		} else {
			false
		}
	}

	/// Record `key` in the ghost ring, overwriting (evicting) the oldest slot.
	#[inline]
	fn ghost_push(&mut self, key: &K) {
		let f = fp(key);
		let h = self.ghost_head;
		let old = self.ghost_ring[h];
		if old != 0 {
			if self.ghost_map.get(&old) == Some(&(h as u32)) {
				self.ghost_map.remove(&old);
			}
			self.ghost_live -= 1;
		}
		if let Some(p) = self.ghost_map.insert(f, h as u32) {
			// Same fingerprint already live elsewhere: clear its old slot so it
			// is not counted twice.
			if p as usize != h {
				self.ghost_ring[p as usize] = 0;
				self.ghost_live -= 1;
			}
		}
		self.ghost_ring[h] = f;
		self.ghost_live += 1;
		let n = h + 1;
		self.ghost_head = if n == self.ghost_ring.len() { 0 } else { n };
	}

	// ---- public API -------------------------------------------------------

	#[inline]
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let e = self.index.get_mut(key)?;
		if (*e >> FREQ_SHIFT) < MAX_FREQ {
			*e += FREQ_ONE;
		}
		let i = (*e & SLOT_MASK) as usize;
		Some(&self.slots[i].value)
	}

	#[inline]
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&e) = self.index.get(&key) {
			self.slot_mut(e & SLOT_MASK).value = value;
			return;
		}

		let f = fp(&key);
		if self.ghost_take_fp(f) {
			// Ghost hit -> grow main by one slot (shrink small).
			if self.adaptive() && self.small_size > self.small_floor() {
				self.small_size -= 1;
				self.main_size += 1;
				#[cfg(test)]
				{
					self.stats.shrinks += 1;
					self.stats.min_small = self.stats.min_small.min(self.small_size);
				}
				if self.small.len() > self.small_size {
					// No adaptation on this fix-up eviction, so the ghost-driven
					// growth is not immediately undone by the graduate rule.
					self.evict_small_inner(false);
				}
				if self.main.len() > self.main_size {
					self.evict_main();
				}
			} else {
				#[cfg(test)]
				{
					if self.adaptive() {
						self.stats.floor_blocks += 1;
					}
				}
			}
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let i = self.alloc_with_key(key, value);
			self.main.push_front(i);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let i = self.alloc_with_key(key, value);
		self.small.push_front(i);
	}

	#[inline]
	fn evict_main(&mut self) {
		while let Some(i) = self.main.pop_back() {
			let e = self.packed_mut(i);
			if (*e >> FREQ_SHIFT) > 0 {
				*e -= FREQ_ONE;
				self.main.push_front(i);
			} else {
				self.release(i);
				break;
			}
		}
	}

	#[inline]
	fn evict_small(&mut self) {
		self.evict_small_inner(true);
	}

	#[inline]
	fn evict_small_inner(&mut self, adapt: bool) {
		if let Some(i) = self.small.pop_back() {
			if (*self.packed_mut(i) >> FREQ_SHIFT) > 1 {
				let main_full = self.main.len() >= self.main_size;
				let want_grow = adapt && self.adaptive() && main_full;
				let grow_small = want_grow && self.small_size < self.small_ceil();
				#[cfg(test)]
				{
					if want_grow && !grow_small {
						self.stats.ceil_blocks += 1;
					}
				}
				if main_full {
					self.evict_main();
				}
				self.main.push_front(i);
				if grow_small {
					// Main is crowded with reused small-queue keys -> give one
					// slot back to small.
					self.small_size += 1;
					self.main_size -= 1;
					#[cfg(test)]
					{
						self.stats.grows += 1;
						self.stats.max_small = self.stats.max_small.max(self.small_size);
					}
					if self.main.len() > self.main_size {
						self.evict_main();
					}
				}
			} else {
				let key = self.release(i);
				self.ghost_push(&key);
			}
		}
	}

	// ---- test helpers -----------------------------------------------------

	#[cfg(test)]
	fn key_of(&self, i: u32) -> &K {
		self.keys[i as usize].as_ref().expect("S3Fifo: queue references a free slot")
	}

	#[cfg(test)]
	fn in_main(&self, key: &K) -> bool {
		self.main.iter().any(|&i| self.key_of(i) == key)
	}

	#[cfg(test)]
	fn push_main_back_for_test(&mut self, key: K, value: V, freq: u8) {
		let i = self.alloc(key, value, freq);
		self.main.push_back(i);
	}

	#[cfg(test)]
	fn push_small_front_for_test(&mut self, key: K, value: V, freq: u8) {
		let i = self.alloc(key, value, freq);
		self.small.push_front(i);
	}
}

impl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
	#[inline]
	fn put(&mut self, key: K, value: V) {
		S3Fifo::put(self, key, value);
	}

	#[inline]
	fn get(&mut self, key: &K) -> Option<&V> {
		S3Fifo::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn entry_is_8_bytes() {
		assert_eq!(std::mem::size_of::<Entry<u64, u64>>(), 8);
		assert_eq!(std::mem::size_of::<Entry<i64, i64>>(), 8);
		// Index bucket stays narrow: (K, packed u32) for u64 keys.
		assert_eq!(std::mem::size_of::<(u64, u32)>(), 16);
	}

	#[test]
	fn packed_layout_fits_max_freq() {
		assert_eq!(SLOT_MASK, (1u32 << 30) - 1);
		assert!(MAX_FREQ <= (u32::MAX >> FREQ_SHIFT));
		assert!(S3Fifo::<i64, i64>::try_new(0).is_none());
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = S3Fifo::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_finds_value() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_existing_updates_value() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn small_eviction_promotes_to_main_after_multiple_hits() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1);
		cache.put(2, 20);
		assert!(cache.in_main(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.in_main(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.push_main_back_for_test(1, 10, 0);
		cache.push_main_back_for_test(2, 20, 2);
		cache.push_main_back_for_test(3, 30, 0);
		cache.push_main_back_for_test(4, 40, 1);
		cache.push_small_front_for_test(5, 50, 2);
		cache.evict_small();
		assert!(cache.in_main(&4));
		assert!(!cache.in_main(&3));
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_eq!(cache.index.len(), cache.small.len() + cache.main.len());
	}

	#[test]
	fn adaptive_split_keeps_bounds() {
		let cap = 64;
		let mut cache = S3Fifo::new(cap);
		for round in 0..50i64 {
			for key in 0..200i64 {
				let k = (key * 7 + round) % 150;
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
				assert!(cache.small.len() + cache.main.len() <= cap);
				assert_eq!(cache.small_size + cache.main_size, cap);
				assert!(cache.small_size >= 1 && cache.main_size >= 1);
				assert!(cache.small_size >= cache.small_floor());
				assert!(cache.small_size <= cache.small_ceil());
				assert!(cache.ghost_live <= cache.ghost_cap);
				assert_eq!(cache.ghost_live, cache.ghost_map.len());
				assert_eq!(cache.slots.len(), cache.keys.len());
				assert!(cache.slots.len() <= cap);
			}
		}
	}

	#[test]
	fn small_floor_is_capacity_over_64() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_floor(), 32);
		assert_eq!(cache.ghost_cap, 1536);
		assert_eq!(cache.ghost_ring.len(), 1536);
		assert_eq!(cache.split(), (512, 1536));
		let tiny = S3Fifo::<i64, i64>::new(4);
		assert_eq!(tiny.small_floor(), 1);
	}

	#[test]
	fn small_ceil_is_capacity_over_4() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_ceil(), 512);
		// Initial small already sits at the ceiling.
		assert_eq!(cache.split().0, cache.small_ceil());
		let tiny = S3Fifo::<i64, i64>::new(4);
		assert_eq!(tiny.small_ceil(), 1);
		assert!(tiny.small_ceil() >= tiny.small_floor());
	}

	#[test]
	fn index_tracks_residency() {
		let cap = 64;
		let mut cache = S3Fifo::new(cap);
		for round in 0..50i64 {
			for key in 0..200i64 {
				let k = (key * 7 + round) % 150;
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
				assert_eq!(cache.index.len(), cache.small.len() + cache.main.len());
				for &i in cache.small.iter().chain(cache.main.iter()) {
					let key = cache.key_of(i);
					let e = *cache.index.get(key).unwrap();
					assert_eq!(e & SLOT_MASK, i);
					assert!((e >> FREQ_SHIFT) <= MAX_FREQ);
					assert_eq!(cache.slot(i).value, *key);
					assert!(!cache.ghost_map.contains_key(&fp(key)));
				}
				let live = cache.keys.iter().filter(|k| k.is_some()).count();
				assert_eq!(live, cache.index.len());
				assert_eq!(live + cache.free.len(), cache.keys.len());
			}
		}
	}

	#[test]
	fn ghost_ring_wraps() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		for key in 0..1000i64 {
			cache.put(key, key);
			assert!(cache.ghost_head < cache.ghost_ring.len());
		}
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert_eq!(cache.index[&1] >> FREQ_SHIFT, MAX_FREQ);
	}

	// ---- split instrumentation (inherited from cycle/0005/v001) ------------

	/// Deterministic Zipf-like key stream (inverse-CDF over `n` ranks, skew
	/// `alpha`, xorshift64* PRNG), ranks scrambled so key order != popularity.
	struct Zipf {
		cdf: Vec<f64>,
		state: u64,
	}

	impl Zipf {
		fn new(n: usize, alpha: f64, seed: u64) -> Self {
			let mut cdf = Vec::with_capacity(n);
			let mut acc = 0.0;
			for r in 1..=n {
				acc += 1.0 / (r as f64).powf(alpha);
				cdf.push(acc);
			}
			for c in cdf.iter_mut() {
				*c /= acc;
			}
			Self { cdf, state: seed | 1 }
		}

		fn next_u64(&mut self) -> u64 {
			let mut x = self.state;
			x ^= x >> 12;
			x ^= x << 25;
			x ^= x >> 27;
			self.state = x;
			x.wrapping_mul(0x2545_F491_4F6C_DD1D)
		}

		fn next_key(&mut self) -> i64 {
			let u = (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64;
			let rank = self.cdf.partition_point(|&c| c < u).min(self.cdf.len() - 1);
			((rank as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 1) as i64
		}
	}

	struct Run {
		hit_rate: f64,
		settled_small: usize,
		min_small: usize,
		max_small: usize,
		stats: SplitStats,
		ghost_hits: u64,
	}

	fn replay(floor_div: usize, ceil_div: usize, n: usize, alpha: f64, ops: usize) -> Run {
		let cap = 2048;
		let mut cache = S3Fifo::<i64, i64>::new(cap);
		cache.floor_div = floor_div;
		cache.ceil_div = ceil_div;
		let mut z = Zipf::new(n, alpha, 0xC0FF_EE12_3456_789B);
		let mut hits = 0u64;
		for _ in 0..ops {
			let k = z.next_key();
			if cache.get(&k).is_some() {
				hits += 1;
			} else {
				cache.put(k, k);
			}
			assert_eq!(cache.small_size + cache.main_size, cap);
			assert!(cache.small_size >= cache.small_floor());
			assert!(cache.small_size <= cache.small_ceil());
			assert!(cache.ghost_live <= cache.ghost_cap);
		}
		Run {
			hit_rate: hits as f64 / ops as f64,
			settled_small: cache.small_size,
			min_small: cache.stats.min_small,
			max_small: cache.stats.max_small,
			stats: cache.stats,
			ghost_hits: cache.stats.shrinks + cache.stats.floor_blocks,
		}
	}

	/// Run with `cargo test --features test-utils --lib h01_split_report -- --nocapture`.
	#[test]
	fn h01_split_report() {
		for &(n, alpha) in &[(20_000usize, 0.8f64), (20_000, 0.99), (100_000, 0.99)] {
			for &div in &[16usize, 64] {
				for &cdiv in &[2usize, 4] {
					let r = replay(div, cdiv, n, alpha, 400_000);
					println!(
						"H03 n={n} alpha={alpha} floor=cap/{div}={floor} ceil=cap/{cdiv}={ceil}: \
						 hit_rate={hr:.4} settled_small={s} min_small={m} max_small={mx} \
						 shrinks={sh} floor_blocks={fb} ({pct:.1}% of ghost hits) grows={g} \
						 ceil_blocks={cb}",
						floor = (2048 / div).max(1),
						ceil = (2048 / cdiv).max((2048 / div).max(1)),
						hr = r.hit_rate,
						s = r.settled_small,
						m = r.min_small,
						mx = r.max_small,
						sh = r.stats.shrinks,
						fb = r.stats.floor_blocks,
						pct = if r.ghost_hits == 0 {
							0.0
						} else {
							100.0 * r.stats.floor_blocks as f64 / r.ghost_hits as f64
						},
						g = r.stats.grows,
						cb = r.stats.ceil_blocks,
					);
					assert!(r.settled_small >= (2048 / div).max(1));
					assert!(r.max_small <= (2048 / cdiv).max((2048 / div).max(1)));
				}
			}
		}
	}
}
