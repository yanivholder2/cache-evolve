use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Variant v002 (H02): main-queue victims are also recorded in the ghost queue,
// so a key aged out of main skips the small queue on its next reference.
//
// Variant v003 (H03): identical policy decisions to v002, but entries live in a
// slab with a hash index (resident lookups are O(1)), the small/main queues hold
// slab indices, and the ghost queue is backed by a hash index with lazy
// deletion (ghost membership is O(1)).
//
// Cycle 3 v003 (H03): the resident and ghost indexes use an inline Fx-style
// multiplicative hasher instead of SipHash. Policy decisions are unchanged.
//
// Cycle 4 v001 (H01): two policy filters merged in.
//   1. Main victims are ghosted only if they were reused (hit) while in main.
//   2. Graduating small -> main resets freq and the reuse flag, so main-queue
//      second chances and the ghost gate only count reuse inside main.
//
// Cycle 5 v002 (H02): main-queue frequency cap raised from 3 to 7, so a key
// heavily reused inside main can survive up to 7 second-chance sweeps.
//
// Cycle 7 v003 (H03): the ghost FIFO is a fixed ring of `main_size.max(1)`
// slots allocated once. Each ghost insertion overwrites the oldest slot (O(1),
// no scan, no compaction, no reallocation). The window is the last `main_size`
// ghost insertions; slots made stale by ghost hits are reclaimed on wrap.
const MAX_FREQ: u8 = 7;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Fx-style multiplicative hasher (not HashDoS resistant; fine for trace replay).
#[derive(Default, Clone, Copy, Debug)]
pub struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, x: u64) {
		self.hash = (self.hash.rotate_left(5) ^ x).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for chunk in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(chunk);
			self.add(u64::from_le_bytes(buf));
		}
		for &b in chunks.remainder() {
			self.add(b as u64);
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
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
		self.hash
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
	/// Set on any hit; cleared on graduation into main. Gates main-victim ghosting.
	reused: bool,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, freq: 0, reused: false }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	/// Entry storage; slots listed in `free` hold stale data awaiting reuse.
	slab: Vec<Entry<K, V>>,
	free: Vec<u32>,
	/// Resident key -> slab slot (covers both small and main).
	index: HashMap<K, u32, FxBuild>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	/// Fixed ghost ring of (key, seq); a record is live iff `ghost_index[key] == seq`.
	ghost_ring: Vec<Option<(K, u64)>>,
	/// Next ring slot to overwrite (the oldest record).
	ghost_head: usize,
	ghost_index: HashMap<K, u64, FxBuild>,
	ghost_seq: u64,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
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
		let ring_len = main_size.max(1);
		let mut ghost_ring = Vec::with_capacity(ring_len);
		ghost_ring.resize_with(ring_len, || None);
		Some(Self {
			slab: Vec::with_capacity(capacity),
			free: Vec::new(),
			index: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost_ring,
			ghost_head: 0,
			ghost_index: HashMap::with_capacity_and_hasher(ring_len, FxBuild::default()),
			ghost_seq: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let i = *self.index.get(key)? as usize;
		let entry = &mut self.slab[i];
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		entry.reused = true;
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&i) = self.index.get(&key) {
			self.slab[i as usize].value = value;
			return;
		}

		if self.ghost_index.remove(&key).is_some() {
			// The ring slot becomes stale and is reclaimed when the ring wraps.
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let i = self.alloc(key, value);
			self.main.push_front(i);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let i = self.alloc(key, value);
		self.small.push_front(i);
	}

	/// Place a new entry into a free slab slot and index it.
	fn alloc(&mut self, key: K, value: V) -> u32 {
		let i = match self.free.pop() {
			Some(i) => {
				self.slab[i as usize] = Entry::new(key.clone(), value);
				i
			}
			None => {
				let i = self.slab.len() as u32;
				self.slab.push(Entry::new(key.clone(), value));
				i
			}
		};
		self.index.insert(key, i);
		i
	}

	/// Drop slot `i` from residency and return its key.
	fn release(&mut self, i: u32) -> K {
		let key = self.slab[i as usize].key.clone();
		self.index.remove(&key);
		self.free.push(i);
		key
	}

	/// O(1) ghost insert: overwrite the oldest ring slot, expiring its record if live.
	fn push_ghost(&mut self, key: K) {
		let seq = self.ghost_seq;
		self.ghost_seq += 1;
		let slot = &mut self.ghost_ring[self.ghost_head];
		if let Some((old_k, old_s)) = slot.take() {
			if self.ghost_index.get(&old_k) == Some(&old_s) {
				self.ghost_index.remove(&old_k);
			}
		}
		*slot = Some((key.clone(), seq));
		self.ghost_index.insert(key, seq);
		self.ghost_head += 1;
		if self.ghost_head == self.ghost_ring.len() {
			self.ghost_head = 0;
		}
	}

	fn evict_main(&mut self) {
		while let Some(i) = self.main.pop_back() {
			let entry = &mut self.slab[i as usize];
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(i);
			} else {
				let reused = entry.reused;
				let key = self.release(i);
				if reused {
					self.push_ghost(key);
				}
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(i) = self.small.pop_back() {
			if self.slab[i as usize].freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let entry = &mut self.slab[i as usize];
				entry.freq = 0;
				entry.reused = false;
				self.main.push_front(i);
			} else {
				let key = self.release(i);
				self.push_ghost(key);
			}
		}
	}

	#[cfg(test)]
	fn test_insert(&mut self, key: K, value: V, freq: u8, into_main: bool, at_back: bool) {
		let i = self.alloc(key, value);
		self.slab[i as usize].freq = freq;
		self.slab[i as usize].reused = freq > 0;
		let queue = if into_main { &mut self.main } else { &mut self.small };
		if at_back {
			queue.push_back(i);
		} else {
			queue.push_front(i);
		}
	}

	#[cfg(test)]
	fn test_set_reused(&mut self, key: &K, reused: bool) {
		let i = *self.index.get(key).expect("test_set_reused: key not resident") as usize;
		self.slab[i].reused = reused;
	}

	#[cfg(test)]
	fn in_main(&self, key: &K) -> bool {
		self.main.iter().any(|&i| &self.slab[i as usize].key == key)
	}

	#[cfg(test)]
	fn in_ghost(&self, key: &K) -> bool {
		self.ghost_index.contains_key(key)
	}
}

impl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
	fn put(&mut self, key: K, value: V) {
		S3Fifo::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		S3Fifo::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn entry_fits_in_padding() {
		assert_eq!(std::mem::size_of::<Entry<u64, u64>>(), 24);
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
	fn graduation_resets_freq_and_reused() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1);
		cache.put(2, 20);
		assert!(cache.in_main(&1));
		let i = cache.index[&1] as usize;
		assert_eq!(cache.slab[i].freq, 0);
		assert!(!cache.slab[i].reused);
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
		cache.test_insert(1, 10, 0, true, true);
		cache.test_insert(2, 20, 2, true, true);
		cache.test_insert(3, 30, 0, true, true);
		cache.test_insert(4, 40, 1, true, true);
		cache.test_insert(5, 50, 2, false, false);
		cache.evict_small();
		assert!(cache.in_main(&4));
		assert!(!cache.in_main(&3));
	}

	#[test]
	fn main_victim_is_ghosted() {
		let mut cache = S3Fifo::new(5);
		cache.test_insert(1, 10, 0, true, true);
		cache.test_insert(2, 20, 0, true, true);
		cache.test_insert(3, 30, 0, true, true);
		cache.test_insert(4, 40, 0, true, true);
		cache.test_set_reused(&4, true);
		cache.evict_main();
		assert!(!cache.in_main(&4));
		assert!(cache.in_ghost(&4));
		cache.put(4, 40);
		assert!(cache.in_main(&4));
		assert!(!cache.in_ghost(&4));
	}

	#[test]
	fn unreused_main_victim_is_not_ghosted() {
		let mut cache = S3Fifo::new(5);
		cache.test_insert(1, 10, 0, true, true);
		cache.test_insert(2, 20, 0, true, true);
		cache.test_insert(3, 30, 0, true, true);
		cache.test_insert(4, 40, 0, true, true);
		cache.evict_main();
		assert!(!cache.in_main(&4));
		assert!(cache.get(&4).is_none());
		assert!(!cache.in_ghost(&4));
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
	fn ghost_live_bound_and_disjoint_from_resident() {
		let mut cache = S3Fifo::new(8);
		for round in 0..50i64 {
			for key in 0..40i64 {
				let k = (key * 7 + round) % 37;
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
			assert!(cache.ghost_index.len() <= cache.main_size);
			assert_eq!(cache.ghost_ring.len(), cache.main_size.max(1));
			for k in cache.index.keys() {
				assert!(!cache.ghost_index.contains_key(k));
			}
		}
	}

	#[test]
	fn ghost_ring_overwrite_expires_oldest() {
		// capacity 4 -> small 1, main 3 -> ring of 3.
		let mut cache = S3Fifo::<i64, i64>::new(4);
		for k in 0..3 {
			cache.push_ghost(k);
		}
		assert!((0..3).all(|k| cache.in_ghost(&k)));
		cache.push_ghost(3);
		assert!(!cache.in_ghost(&0));
		assert!(cache.in_ghost(&3));
		assert_eq!(cache.ghost_index.len(), 3);
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
	}

	#[test]
	fn repeated_get_saturates_at_max_freq() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			cache.get(&1);
		}
		let i = cache.index[&1] as usize;
		assert_eq!(cache.slab[i].freq, MAX_FREQ);
		assert_eq!(MAX_FREQ, 7);
	}
}
