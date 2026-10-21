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
// Storage: entries live in a slab (`slots`) addressed by `u32` indices; the
// small/main FIFOs hold slot indices; a hash index maps resident keys to slots,
// and a ghost hash set (key -> insertion sequence) gives O(1) ghost membership
// with lazy deletion from the ghost FIFO. Policy behaviour is identical to the
// scan-based baseline.
const MAX_FREQ: u8 = 3;

/// FxHash-style multiply hasher (local; no external crates).
#[derive(Default, Clone, Copy, Debug)]
pub struct FxHasher {
	hash: u64,
}

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

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
	fn write_i64(&mut self, i: i64) {
		self.add_to_hash(i as u64);
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

type FxBuild = BuildHasherDefault<FxHasher>;

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
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slots: Vec<Entry<K, V>>,
	free: Vec<u32>,
	index: HashMap<K, u32, FxBuild>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	ghost: VecDeque<(K, u64)>,
	ghost_set: HashMap<K, u64, FxBuild>,
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
		if capacity >= u32::MAX as usize {
			log::error!("S3Fifo.try_new: capacity too large");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			slots: Vec::with_capacity(capacity),
			free: Vec::with_capacity(capacity),
			index: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: HashMap::with_capacity_and_hasher(main_size, FxBuild::default()),
			ghost_seq: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn alloc(&mut self, key: K, value: V) -> u32 {
		if let Some(i) = self.free.pop() {
			self.slots[i as usize] = Entry::new(key, value);
			i
		} else {
			let i = self.slots.len() as u32;
			self.slots.push(Entry::new(key, value));
			i
		}
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let i = *self.index.get(key)? as usize;
		let entry = &mut self.slots[i];
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&i) = self.index.get(&key) {
			self.slots[i as usize].value = value;
			return;
		}

		if self.ghost_set.remove(&key).is_some() {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let i = self.alloc(key.clone(), value);
			self.index.insert(key, i);
			self.main.push_front(i);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let i = self.alloc(key.clone(), value);
		self.index.insert(key, i);
		self.small.push_front(i);
	}

	fn evict_main(&mut self) {
		while let Some(i) = self.main.pop_back() {
			let entry = &mut self.slots[i as usize];
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(i);
			} else {
				self.index.remove(&entry.key);
				self.free.push(i);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(i) = self.small.pop_back() {
			if self.slots[i as usize].freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(i);
			} else {
				let key = self.slots[i as usize].key.clone();
				self.index.remove(&key);
				self.free.push(i);
				self.push_ghost(key);
			}
		}
	}

	fn push_ghost(&mut self, key: K) {
		if self.ghost_set.len() >= self.main_size {
			// Pop the oldest *live* ghost, draining stale (lazily-deleted) entries.
			while let Some((old, seq)) = self.ghost.pop_back() {
				if self.ghost_set.get(&old) == Some(&seq) {
					self.ghost_set.remove(&old);
					break;
				}
			}
		}
		let seq = self.ghost_seq;
		self.ghost_seq = self.ghost_seq.wrapping_add(1);
		self.ghost_set.insert(key.clone(), seq);
		self.ghost.push_front((key, seq));
		// Bound memory held by stale entries: compact when the deque grows well
		// past the live count (amortised O(1)).
		if self.ghost.len() > 2 * self.main_size.max(1) + 16 {
			let set = &self.ghost_set;
			self.ghost.retain(|(k, s)| set.get(k) == Some(s));
		}
	}

	#[cfg(test)]
	fn main_contains(&self, key: &K) -> bool {
		self.main.iter().any(|&i| &self.slots[i as usize].key == key)
	}

	#[cfg(test)]
	fn test_push(&mut self, to_main: bool, back: bool, key: K, value: V, freq: u8) {
		let i = self.alloc(key.clone(), value);
		self.slots[i as usize].freq = freq;
		self.index.insert(key, i);
		let q = if to_main { &mut self.main } else { &mut self.small };
		if back {
			q.push_back(i);
		} else {
			q.push_front(i);
		}
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
		assert!(cache.main_contains(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main_contains(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.test_push(true, true, 1, 10, 0);
		cache.test_push(true, true, 2, 20, 2);
		cache.test_push(true, true, 3, 30, 0);
		cache.test_push(true, true, 4, 40, 1);
		cache.test_push(false, false, 5, 50, 2);
		cache.evict_small();
		assert!(cache.main_contains(&4));
		assert!(!cache.main_contains(&3));
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
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
	fn index_tracks_queues() {
		let mut cache = S3Fifo::new(8);
		for round in 0..5i64 {
			for key in 0..40i64 {
				if cache.get(&(key * round % 37)).is_none() {
					cache.put(key * round % 37, key);
				}
			}
			assert_eq!(cache.index.len(), cache.small.len() + cache.main.len());
			assert!(cache.ghost_set.len() <= cache.main_size);
		}
	}
}
