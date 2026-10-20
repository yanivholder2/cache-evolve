use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::mem::MaybeUninit;

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
// Variant cycle/0009/v002 (H02): policy-neutral layout refactor. The resident
// `index` is no longer a hashbrown map but a custom power-of-two
// open-addressing linear-probe table of `(K, u32)` buckets (`ResidentIndex`),
// load factor <= 0.5, empty marked by value word `u32::MAX`, backward-shift
// deletion (no tombstones). The ghost map stays an `FxHashMap`.
const MAX_FREQ: u32 = 3;

/// Packed index value layout: low 30 bits = slot id, top 2 bits = freq.
const FREQ_SHIFT: u32 = 30;
const SLOT_MASK: u32 = (1 << FREQ_SHIFT) - 1;
const FREQ_ONE: u32 = 1 << FREQ_SHIFT;

/// Divisor of the adaptive split's lower clamp: `small_size` never shrinks
/// below `(capacity / SMALL_FLOOR_DIV).max(1)` on a ghost hit.
const SMALL_FLOOR_DIV: usize = 64;

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

/// Plain FxHash of a key.
#[inline]
fn fx_hash<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut h = FxBuildHasher.build_hasher();
	key.hash(&mut h);
	h.finish()
}

/// Ghost fingerprint of a key: FxHash of the key, forced odd so it is never 0
/// (0 marks an empty ring slot).
#[inline]
fn fp<K: Hash + ?Sized>(key: &K) -> u64 {
	fx_hash(key) | 1
}

// ---------------------------------------------------------------------------
// ResidentIndex: power-of-two open-addressing linear-probe table.
// ---------------------------------------------------------------------------

/// Value word marking an empty bucket. A live packed word is
/// `(freq << 30) | slot` with `slot < capacity <= SLOT_MASK`, so it never
/// equals `u32::MAX`.
const EMPTY: u32 = u32::MAX;

/// Resident key -> packed `u32` map. Buckets are `(key, word)`; the key is
/// initialised iff `word != EMPTY`. Load factor <= 0.5 (callers keep at most
/// `capacity` entries in `(2 * capacity).next_power_of_two()` buckets), so
/// every probe sequence terminates on an `EMPTY` bucket.
struct ResidentIndex<K> {
	buckets: Vec<(MaybeUninit<K>, u32)>,
	mask: usize,
	shift: u32,
	len: usize,
}

impl<K> Debug for ResidentIndex<K> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("ResidentIndex")
			.field("buckets", &self.buckets.len())
			.field("len", &self.len)
			.finish()
	}
}

impl<K> Drop for ResidentIndex<K> {
	fn drop(&mut self) {
		if std::mem::needs_drop::<K>() {
			for b in self.buckets.iter_mut() {
				if b.1 != EMPTY {
					b.1 = EMPTY;
					// SAFETY: word != EMPTY means the key is initialised.
					unsafe { b.0.assume_init_drop() };
				}
			}
		}
	}
}

impl<K: Hash + Eq> ResidentIndex<K> {
	fn with_capacity(capacity: usize) -> Self {
		let n = (2 * capacity.max(1)).next_power_of_two();
		let mut buckets = Vec::with_capacity(n);
		buckets.resize_with(n, || (MaybeUninit::uninit(), EMPTY));
		Self { buckets, mask: n - 1, shift: 64 - n.trailing_zeros(), len: 0 }
	}

	/// Home bucket: high bits of a Fibonacci multiply of the FxHash.
	#[inline]
	fn home(&self, key: &K) -> usize {
		(fx_hash(key).wrapping_mul(FX_SEED) >> self.shift) as usize
	}

	/// Bucket position holding `key`, if present.
	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		let mut i = self.home(key);
		loop {
			debug_assert!(i < self.buckets.len());
			// SAFETY: `home` < n and `& mask` keeps `i` < n.
			let b = unsafe { self.buckets.get_unchecked(i) };
			if b.1 == EMPTY {
				return None;
			}
			// SAFETY: word != EMPTY means the key is initialised.
			if unsafe { b.0.assume_init_ref() } == key {
				return Some(i);
			}
			i = (i + 1) & self.mask;
		}
	}

	#[inline]
	fn get(&self, key: &K) -> Option<u32> {
		let i = self.find(key)?;
		// SAFETY: `find` returns an in-bounds position.
		Some(unsafe { self.buckets.get_unchecked(i) }.1)
	}

	#[inline]
	fn get_mut(&mut self, key: &K) -> Option<&mut u32> {
		let i = self.find(key)?;
		// SAFETY: `find` returns an in-bounds position.
		Some(&mut unsafe { self.buckets.get_unchecked_mut(i) }.1)
	}

	/// Insert an absent key. Callers guarantee absence.
	#[inline]
	fn insert(&mut self, key: K, word: u32) {
		debug_assert!(self.find(&key).is_none(), "ResidentIndex: duplicate insert");
		debug_assert!(word != EMPTY);
		debug_assert!(self.len < self.buckets.len() / 2 + 1);
		let mut i = self.home(&key);
		loop {
			// SAFETY: `i` < n (see `find`).
			let b = unsafe { self.buckets.get_unchecked_mut(i) };
			if b.1 == EMPTY {
				b.0.write(key);
				b.1 = word;
				self.len += 1;
				return;
			}
			i = (i + 1) & self.mask;
		}
	}

	/// Remove `key` with backward-shift deletion; returns its word.
	#[inline]
	fn remove(&mut self, key: &K) -> Option<u32> {
		let mut hole = self.find(key)?;
		let mask = self.mask;
		let word = self.buckets[hole].1;
		self.buckets[hole].1 = EMPTY;
		// SAFETY: the bucket was occupied; it is now marked EMPTY so the key
		// is read out exactly once.
		drop(unsafe { self.buckets[hole].0.assume_init_read() });
		self.len -= 1;
		let mut j = hole;
		loop {
			j = (j + 1) & mask;
			if self.buckets[j].1 == EMPTY {
				break;
			}
			// SAFETY: occupied bucket.
			let h = self.home(unsafe { self.buckets[j].0.assume_init_ref() });
			// Move j back into the hole iff its home is cyclically at or
			// before the hole, i.e. dist(home -> j) >= dist(hole -> j).
			if (j.wrapping_sub(h) & mask) >= (j.wrapping_sub(hole) & mask) {
				let w = self.buckets[j].1;
				self.buckets[j].1 = EMPTY;
				// SAFETY: bucket j was occupied and is now marked EMPTY; the
				// hole bucket is EMPTY (uninitialised) and receives the key.
				let k = unsafe { self.buckets[j].0.assume_init_read() };
				self.buckets[hole].0.write(k);
				self.buckets[hole].1 = w;
				hole = j;
			}
		}
		Some(word)
	}

	#[inline]
	#[allow(dead_code)]
	fn len(&self) -> usize {
		self.len
	}
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
	/// Smallest `small_size` observed.
	min_small: usize,
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
	index: ResidentIndex<K>,
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
	#[cfg(test)]
	stats: SplitStats,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		// `capacity <= SLOT_MASK` keeps every slot id < SLOT_MASK, so a live
		// packed word never equals the index's `EMPTY` marker.
		if capacity > SLOT_MASK as usize {
			log::error!("S3Fifo.try_new: capacity must fit in 30-bit slot id");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let index = ResidentIndex::with_capacity(capacity);
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
			stats: SplitStats { min_small: small_size, ..SplitStats::default() },
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

	/// Mutable packed index entry of resident slot `i` (one probe).
	#[inline]
	fn packed_mut(&mut self, i: u32) -> &mut u32 {
		let k = self.keys[i as usize].as_ref().expect("S3Fifo: queue references a free slot");
		self.index.get_mut(k).expect("S3Fifo: resident key missing from index")
	}

	/// Store a new resident entry (freq 0) and register it in `index`.
	/// Exactly one key clone and one `index` insert.
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
		if let Some(e) = self.index.get(&key) {
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
				let grow_small =
					adapt && self.adaptive() && main_full && self.small_size < self.capacity / 2;
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
		// Largest live word never collides with EMPTY.
		assert_ne!((MAX_FREQ << FREQ_SHIFT) | (SLOT_MASK - 1), EMPTY);
		assert_eq!(std::mem::size_of::<(MaybeUninit<u64>, u32)>(), 16);
	}

	#[test]
	fn resident_index_sizing() {
		let idx = ResidentIndex::<i64>::with_capacity(2048);
		assert_eq!(idx.buckets.len(), 4096);
		assert_eq!(idx.mask, 4095);
		assert_eq!(idx.shift, 52);
		let one = ResidentIndex::<i64>::with_capacity(1);
		assert_eq!(one.buckets.len(), 2);
		let odd = ResidentIndex::<i64>::with_capacity(3);
		assert_eq!(odd.buckets.len(), 8);
	}

	/// xorshift64* for the stress test.
	fn xs(state: &mut u64) -> u64 {
		let mut x = *state;
		x ^= x >> 12;
		x ^= x << 25;
		x ^= x >> 27;
		*state = x;
		x.wrapping_mul(0x2545_F491_4F6C_DD1D)
	}

	fn stress(cap: usize, key_space: u64, ops: usize, seed: u64) {
		let mut idx = ResidentIndex::<i64>::with_capacity(cap);
		let mut oracle: std::collections::HashMap<i64, u32> = std::collections::HashMap::new();
		let mut st = seed | 1;
		for step in 0..ops {
			let r = xs(&mut st);
			let k = (r % key_space) as i64;
			match (r >> 40) % 3 {
				0 => {
					if !oracle.contains_key(&k) && oracle.len() < cap {
						let w = (step as u32) & SLOT_MASK;
						idx.insert(k, w);
						oracle.insert(k, w);
					}
				}
				1 => {
					assert_eq!(idx.remove(&k), oracle.remove(&k));
				}
				_ => {
					if let Some(w) = idx.get_mut(&k) {
						*w = (*w + 1) & SLOT_MASK;
					}
					if let Some(w) = oracle.get_mut(&k) {
						*w = (*w + 1) & SLOT_MASK;
					}
					assert_eq!(idx.get(&k), oracle.get(&k).copied());
				}
			}
			assert_eq!(idx.len(), oracle.len());
		}
		for k in 0..key_space as i64 {
			assert_eq!(idx.get(&k), oracle.get(&k).copied(), "key {k}");
		}
		let occupied = idx.buckets.iter().filter(|b| b.1 != EMPTY).count();
		assert_eq!(occupied, oracle.len());
	}

	#[test]
	fn resident_index_matches_hashmap_oracle() {
		// Tiny tables force probe chains to wrap around the bucket array.
		for seed in 1..40u64 {
			stress(1, 4, 2_000, seed);
			stress(2, 8, 2_000, seed);
			stress(3, 16, 5_000, seed);
			stress(8, 64, 5_000, seed);
		}
		stress(64, 200, 100_000, 0xDEAD_BEEF);
		stress(2048, 10_000, 300_000, 0x1234_5678);
		// Keys whose home is the last bucket wrap to the front.
		stress(4, 1_000, 50_000, 0xABCD);
	}

	#[test]
	fn resident_index_wraparound_explicit() {
		let mut idx = ResidentIndex::<i64>::with_capacity(4); // 8 buckets
		let last = idx.mask;
		let homed: Vec<i64> = (0..10_000i64).filter(|k| idx.home(k) == last).take(3).collect();
		assert_eq!(homed.len(), 3);
		for (n, &k) in homed.iter().enumerate() {
			idx.insert(k, n as u32);
		}
		// Second and third wrapped into buckets 0 and 1.
		assert_ne!(idx.buckets[0].1, EMPTY);
		assert_ne!(idx.buckets[1].1, EMPTY);
		assert_eq!(idx.remove(&homed[0]), Some(0));
		assert_eq!(idx.get(&homed[1]), Some(1));
		assert_eq!(idx.get(&homed[2]), Some(2));
		assert_eq!(idx.remove(&homed[1]), Some(1));
		assert_eq!(idx.get(&homed[2]), Some(2));
		assert_eq!(idx.buckets[last].1, 2);
		assert_eq!(idx.len(), 1);
	}

	#[test]
	fn resident_index_drops_keys() {
		use std::rc::Rc;
		let probe = Rc::new(());
		{
			let mut idx = ResidentIndex::<(i64, Rc<()>)>::with_capacity(8);
			for k in 0..8i64 {
				idx.insert((k, probe.clone()), k as u32);
			}
			assert_eq!(Rc::strong_count(&probe), 9);
			idx.remove(&(3, probe.clone()));
			assert_eq!(Rc::strong_count(&probe), 8);
		}
		assert_eq!(Rc::strong_count(&probe), 1);
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
					let e = cache.index.get(key).unwrap();
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
		assert_eq!(cache.index.get(&1).unwrap() >> FREQ_SHIFT, MAX_FREQ);
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
		stats: SplitStats,
		ghost_hits: u64,
	}

	fn replay(floor_div: usize, n: usize, alpha: f64, ops: usize) -> Run {
		let cap = 2048;
		let mut cache = S3Fifo::<i64, i64>::new(cap);
		cache.floor_div = floor_div;
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
			assert!(cache.ghost_live <= cache.ghost_cap);
		}
		Run {
			hit_rate: hits as f64 / ops as f64,
			settled_small: cache.small_size,
			min_small: cache.stats.min_small,
			stats: cache.stats,
			ghost_hits: cache.stats.shrinks + cache.stats.floor_blocks,
		}
	}

	/// Run with `cargo test --features test-utils --lib h01_split_report -- --nocapture`.
	#[test]
	fn h01_split_report() {
		for &(n, alpha) in &[(20_000usize, 0.8f64), (20_000, 0.99), (100_000, 0.99)] {
			for &div in &[16usize, 64] {
				let r = replay(div, n, alpha, 400_000);
				println!(
					"H01 n={n} alpha={alpha} floor=cap/{div}={floor}: hit_rate={hr:.4} \
					 settled_small={s} min_small={m} shrinks={sh} floor_blocks={fb} \
					 ({pct:.1}% of ghost hits) grows={g}",
					floor = (2048 / div).max(1),
					hr = r.hit_rate,
					s = r.settled_small,
					m = r.min_small,
					sh = r.stats.shrinks,
					fb = r.stats.floor_blocks,
					pct = if r.ghost_hits == 0 {
						0.0
					} else {
						100.0 * r.stats.floor_blocks as f64 / r.ghost_hits as f64
					},
					g = r.stats.grows,
				);
				assert!(r.settled_small >= (2048 / div).max(1));
			}
		}
	}
}
