use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

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
// stays fixed (set once at construction).
//
// Slab layout (inherited from cycle/0002/v003): entries live in a slab
// (`slots`) addressed by `u32` indices; `small` / `main` are FIFOs of slot
// indices; `index` maps resident keys to their slot.
//
// Ghost ring (inherited from cycle/0003/v002): the ghost is a fixed-size ring
// of 64-bit key fingerprints (0 = empty slot) plus a fingerprint ->
// ring-position map. A `ghost_take` leaves a 0 hole that still consumes a ring
// position, so the ghost remembers the last `ghost_cap` pushes.
//
// Variant cycle/0004/v001 (H01): the ghost ring length is halved to half the
// *initial* main size (`(main_size / 2).max(1)`), and `ghost_cap` equals that
// ring length. Nothing else changes.
//
// Where `small_size` settles: it moves only between the clamps
// `[max(capacity/16, 1), capacity/2)`. A shorter ghost yields fewer ghost hits,
// so the shrink-small pressure falls and `small_size` is expected to settle
// higher than under the parent (driven mainly by the grow-small graduate
// rule). `small_size_for_test` exposes the value for unit-test inspection.
const MAX_FREQ: u8 = 3;

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

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, freq: 0 }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	/// Slab of entries; `None` marks a free slot (listed in `free`).
	slots: Vec<Option<Entry<K, V>>>,
	free: Vec<u32>,
	/// Resident key -> slot index.
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
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		if capacity > u32::MAX as usize {
			log::error!("S3Fifo.try_new: capacity must fit in u32");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let index = FxHashMap::with_capacity_and_hasher(capacity, FxBuildHasher);
		// H01: ghost ring is half the initial main size. It needs at least one
		// slot so the modulo is defined (capacity 1 -> ring_len 1).
		let ring_len = (main_size / 2).max(1);
		Some(Self {
			slots: Vec::with_capacity(capacity),
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
			ghost_cap: ring_len,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn adaptive(&self) -> bool {
		self.capacity >= 4
	}

	// ---- slab helpers -----------------------------------------------------

	#[inline]
	fn slot(&self, i: u32) -> &Entry<K, V> {
		self.slots[i as usize].as_ref().expect("S3Fifo: queue references a free slot")
	}

	#[inline]
	fn slot_mut(&mut self, i: u32) -> &mut Entry<K, V> {
		self.slots[i as usize].as_mut().expect("S3Fifo: queue references a free slot")
	}

	/// Store a new resident entry and register it in `index`.
	#[inline]
	fn alloc(&mut self, entry: Entry<K, V>) -> u32 {
		let key = entry.key.clone();
		let i = match self.free.pop() {
			Some(i) => {
				self.slots[i as usize] = Some(entry);
				i
			}
			None => {
				let i = self.slots.len() as u32;
				self.slots.push(Some(entry));
				i
			}
		};
		self.index.insert(key, i);
		i
	}

	/// Remove a resident entry from the slab and `index`; return it.
	#[inline]
	fn release(&mut self, i: u32) -> Entry<K, V> {
		let entry = self.slots[i as usize].take().expect("S3Fifo: double release of slot");
		self.index.remove(&entry.key);
		self.free.push(i);
		entry
	}

	// ---- ghost helpers ----------------------------------------------------

	/// O(1) ghost membership test + removal.
	#[inline]
	fn ghost_take(&mut self, key: &K) -> bool {
		let f = fp(key);
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
		self.ghost_head = (h + 1) % self.ghost_ring.len();
	}

	// ---- public API -------------------------------------------------------

	#[inline]
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let i = *self.index.get(key)? as usize;
		match &mut self.slots[i] {
			Some(entry) => {
				entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
				Some(&entry.value)
			}
			None => None,
		}
	}

	#[inline]
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&i) = self.index.get(&key) {
			self.slot_mut(i).value = value;
			return;
		}

		if self.ghost_take(&key) {
			// Ghost hit -> grow main by one slot (shrink small).
			if self.adaptive() && self.small_size > (self.capacity / 16).max(1) {
				self.small_size -= 1;
				self.main_size += 1;
				if self.small.len() > self.small_size {
					// No adaptation on this fix-up eviction, so the ghost-driven
					// growth is not immediately undone by the graduate rule.
					self.evict_small_inner(false);
				}
				if self.main.len() > self.main_size {
					self.evict_main();
				}
			}
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let i = self.alloc(Entry::new(key, value));
			self.main.push_front(i);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let i = self.alloc(Entry::new(key, value));
		self.small.push_front(i);
	}

	fn evict_main(&mut self) {
		while let Some(i) = self.main.pop_back() {
			let entry = self.slot_mut(i);
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(i);
			} else {
				self.release(i);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		self.evict_small_inner(true);
	}

	fn evict_small_inner(&mut self, adapt: bool) {
		if let Some(i) = self.small.pop_back() {
			if self.slot(i).freq > 1 {
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
					if self.main.len() > self.main_size {
						self.evict_main();
					}
				}
			} else {
				let entry = self.release(i);
				self.ghost_push(&entry.key);
			}
		}
	}

	// ---- test helpers -----------------------------------------------------

	#[cfg(test)]
	fn in_main(&self, key: &K) -> bool {
		self.main.iter().any(|&i| &self.slot(i).key == key)
	}

	#[cfg(test)]
	fn push_main_back_for_test(&mut self, entry: Entry<K, V>) {
		let i = self.alloc(entry);
		self.main.push_back(i);
	}

	#[cfg(test)]
	fn push_small_front_for_test(&mut self, entry: Entry<K, V>) {
		let i = self.alloc(entry);
		self.small.push_front(i);
	}

	/// Current adaptive small-queue target, for inspecting where it settles.
	#[cfg(test)]
	fn small_size_for_test(&self) -> usize {
		self.small_size
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
		// H01: with capacity 4 the ring is a single slot (main 3 / 2 = 1), so
		// re-put the most recently demoted key (3), which is still ghosted.
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(3, 30);
		assert!(cache.in_main(&3));
	}

	#[test]
	fn ghost_ring_is_half_initial_main() {
		let cache = S3Fifo::<i64, i64>::new(64);
		// small 16, main 48 -> ring 24.
		assert_eq!(cache.ghost_ring.len(), 24);
		assert_eq!(cache.ghost_cap, 24);
		let tiny = S3Fifo::<i64, i64>::new(1);
		assert_eq!(tiny.ghost_ring.len(), 1);
		assert_eq!(tiny.ghost_cap, 1);
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(1, 10) });
		cache.push_main_back_for_test(Entry { freq: 2, ..Entry::new(2, 20) });
		cache.push_main_back_for_test(Entry { freq: 0, ..Entry::new(3, 30) });
		cache.push_main_back_for_test(Entry { freq: 1, ..Entry::new(4, 40) });
		cache.push_small_front_for_test(Entry { freq: 2, ..Entry::new(5, 50) });
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
				assert!(cache.ghost_live <= cache.ghost_cap);
				assert_eq!(cache.ghost_live, cache.ghost_map.len());
				assert_eq!(cache.ghost_cap, cache.ghost_ring.len());
			}
		}
		let s = cache.small_size_for_test();
		assert!(s >= (cap / 16).max(1) && s <= cap / 2);
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
					let e = cache.slot(i);
					assert_eq!(cache.index.get(&e.key), Some(&i));
					assert!(!cache.ghost_map.contains_key(&fp(&e.key)));
				}
			}
		}
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
	}
}
