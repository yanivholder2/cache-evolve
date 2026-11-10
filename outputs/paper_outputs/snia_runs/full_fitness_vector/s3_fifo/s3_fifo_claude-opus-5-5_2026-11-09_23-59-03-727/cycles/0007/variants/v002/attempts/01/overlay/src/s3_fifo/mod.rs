use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decay.
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
// Cycle 6 v003 (H03): ghost records carry their source in the low bit of the
// sequence tag. A key re-admitted from a main-victim ghost enters main with
// freq = 1 (one survived sweep); small-demotee ghost hits keep freq = 0.
//
// Cycle 7 v002 (H02): each main-eviction sweep halves freq (`freq >>= 1`)
// instead of decrementing it, so a key at MAX_FREQ survives 3 sweeps
// (7 -> 3 -> 1 -> 0) rather than 7, aging stale hot counts out faster.
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
	/// Ghost FIFO of (key, tag); a tuple is live iff `ghost_index[key] == tag`.
	/// `tag = (seq << 1) | from_main`.
	ghost: VecDeque<(K, u64)>,
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
		Some(Self {
			slab: Vec::with_capacity(capacity),
			free: Vec::new(),
			index: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(2 * main_size + 1),
			ghost_index: HashMap::with_capacity_and_hasher(main_size, FxBuild::default()),
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

		if let Some(tag) = self.ghost_index.remove(&key) {
			// The ghost tuple stays in the deque as a stale entry.
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let i = self.alloc(key, value);
			self.main.push_front(i);
			if tag & 1 == 1 {
				// Main-victim ghost hit: proven reuse earns one survived sweep.
				// `reused` stays false so it must be hit in main to be re-ghosted.
				self.slab[i as usize].freq = 1;
			}
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

	/// Bounded insert into the ghost queue: drop the oldest live ghost when full.
	/// The record's source is encoded in the low bit of its tag.
	fn push_ghost(&mut self, key: K, from_main: bool) {
		if self.ghost_index.len() >= self.main_size {
			while let Some((k, s)) = self.ghost.pop_back() {
				if self.ghost_index.get(&k) == Some(&s) {
					self.ghost_index.remove(&k);
					break;
				}
			}
		}
		let tag = (self.ghost_seq << 1) | from_main as u64;
		self.ghost_seq += 1;
		self.ghost.push_front((key.clone(), tag));
		self.ghost_index.insert(key, tag);

		if self.ghost.len() > 2 * self.main_size {
			let ghost_index = &self.ghost_index;
			self.ghost.retain(|(k, s)| ghost_index.get(k) == Some(s));
		}
	}

	fn evict_main(&mut self) {
		while let Some(i) = self.main.pop_back() {
			let entry = &mut self.slab[i as usize];
			if entry.freq > 0 {
				// Halving decay: 7 -> 3 -> 1 -> 0.
				entry.freq >>= 1;
				self.main.push_front(i);
			} else {
				let reused = entry.reused;
				let key = self.release(i);
				if reused {
					self.push_ghost(key, true);
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
				self.push_ghost(key, false);
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
	fn small_demotee_ghost_hit_readmits_with_zero_freq() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_index[&1] & 1, 0);
		cache.put(1, 10);
		assert!(cache.in_main(&1));
		let i = cache.index[&1] as usize;
		assert_eq!(cache.slab[i].freq, 0);
		assert!(!cache.slab[i].reused);
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
	fn max_freq_main_resident_evicted_on_fourth_sweep() {
		// Main back -> front: key 1 (freq 7), then a freq-0 filler. Each sweep
		// reaches key 1 once, halves its freq, and evicts the filler behind it.
		let mut cache = S3Fifo::new(5);
		cache.test_insert(1, 10, MAX_FREQ, true, true);
		let expected = [3u8, 1, 0];
		for (n, &want) in expected.iter().enumerate() {
			let filler = 100 + n as i64;
			cache.test_insert(filler, filler, 0, true, false);
			cache.evict_main();
			assert!(cache.in_main(&1), "key 1 must survive sweep {}", n + 1);
			assert!(!cache.in_main(&filler));
			let i = cache.index[&1] as usize;
			assert_eq!(cache.slab[i].freq, want);
		}
		// Fourth sweep reaching key 1: freq is 0, so it is evicted.
		cache.test_insert(200, 200, 0, true, false);
		cache.evict_main();
		assert!(!cache.in_main(&1));
		assert!(cache.get(&1).is_none());
		assert!(cache.in_main(&200));
		// It was reused in main, so it is ghosted as a main victim.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_index[&1] & 1, 1);
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
		assert_eq!(cache.ghost_index[&4] & 1, 1);
		cache.put(4, 40);
		assert!(cache.in_main(&4));
		assert!(!cache.in_ghost(&4));
		let i = cache.index[&4] as usize;
		assert_eq!(cache.slab[i].freq, 1);
		assert!(!cache.slab[i].reused);
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
			assert!(cache.ghost.len() <= 2 * cache.main_size);
			for k in cache.index.keys() {
				assert!(!cache.ghost_index.contains_key(k));
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
