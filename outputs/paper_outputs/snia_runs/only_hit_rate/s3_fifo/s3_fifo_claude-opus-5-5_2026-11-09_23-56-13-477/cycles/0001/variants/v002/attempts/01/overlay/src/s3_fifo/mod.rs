use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Representation: resident entries live in a slab (`Vec<Entry>` + free list);
// `small` / `main` are FIFO queues of slab indices; a hash index maps each key
// to its location (small slot, main slot, or ghost stamp). The ghost queue uses
// lazy deletion: a ghost hit only updates the index, and stale deque slots are
// skipped (stamp mismatch) when popped, or dropped on periodic compaction.
// Policy decisions are identical to the scan-based baseline.
const MAX_FREQ: u8 = 3;

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
	Small(u32),
	Main(u32),
	Ghost(u64),
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Entry<K, V>>,
	free: Vec<u32>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	ghost: VecDeque<(K, u64)>,
	ghost_live: usize,
	next_stamp: u64,
	index: HashMap<K, Loc>,
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
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_live: 0,
			next_stamp: 0,
			index: HashMap::with_capacity(capacity + main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn alloc(&mut self, key: K, value: V) -> u32 {
		match self.free.pop() {
			Some(idx) => {
				self.slab[idx as usize] = Entry::new(key, value);
				idx
			}
			None => {
				self.slab.push(Entry::new(key, value));
				(self.slab.len() - 1) as u32
			}
		}
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = match self.index.get(key)? {
			Loc::Small(i) | Loc::Main(i) => *i,
			Loc::Ghost(_) => return None,
		};
		let entry = &mut self.slab[idx as usize];
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		match self.index.get(&key).copied() {
			Some(Loc::Small(i)) | Some(Loc::Main(i)) => {
				self.slab[i as usize].value = value;
			}
			Some(Loc::Ghost(_)) => {
				// Ghost hit: logically remove from ghost (deque slot left stale).
				self.ghost_live -= 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let idx = self.alloc(key.clone(), value);
				self.main.push_front(idx);
				self.index.insert(key, Loc::Main(idx));
			}
			None => {
				if self.small.len() >= self.small_size {
					self.evict_small();
				}
				let idx = self.alloc(key.clone(), value);
				self.small.push_front(idx);
				self.index.insert(key, Loc::Small(idx));
			}
		}
	}

	fn evict_main(&mut self) {
		while let Some(idx) = self.main.pop_back() {
			let entry = &mut self.slab[idx as usize];
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(idx);
			} else {
				self.index.remove(&entry.key);
				self.free.push(idx);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(idx) = self.small.pop_back() {
			if self.slab[idx as usize].freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(idx);
				let key = self.slab[idx as usize].key.clone();
				self.index.insert(key, Loc::Main(idx));
			} else {
				let key = self.slab[idx as usize].key.clone();
				self.free.push(idx);
				self.ghost_insert(key);
			}
		}
	}

	fn ghost_is_live(index: &HashMap<K, Loc>, key: &K, stamp: u64) -> bool {
		matches!(index.get(key), Some(Loc::Ghost(s)) if *s == stamp)
	}

	fn ghost_insert(&mut self, key: K) {
		if self.ghost_live >= self.main_size {
			// Pop the oldest *live* ghost entry, skipping stale slots.
			while let Some((old_key, stamp)) = self.ghost.pop_back() {
				if Self::ghost_is_live(&self.index, &old_key, stamp) {
					self.index.remove(&old_key);
					self.ghost_live -= 1;
					break;
				}
			}
		}
		let stamp = self.next_stamp;
		self.next_stamp += 1;
		self.ghost.push_front((key.clone(), stamp));
		self.index.insert(key, Loc::Ghost(stamp));
		self.ghost_live += 1;

		if self.ghost.len() > 2 * self.main_size.max(1) {
			let index = &self.index;
			self.ghost.retain(|(k, s)| Self::ghost_is_live(index, k, *s));
		}
	}

	#[cfg(test)]
	fn main_contains(&self, key: &K) -> bool {
		matches!(self.index.get(key), Some(Loc::Main(_)))
	}

	#[cfg(test)]
	fn resident_len(&self) -> usize {
		self.small.len() + self.main.len()
	}

	#[cfg(test)]
	fn test_push_main_back(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key.clone(), value);
		self.slab[idx as usize].freq = freq;
		self.main.push_back(idx);
		self.index.insert(key, Loc::Main(idx));
	}

	#[cfg(test)]
	fn test_push_small_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key.clone(), value);
		self.slab[idx as usize].freq = freq;
		self.small.push_front(idx);
		self.index.insert(key, Loc::Small(idx));
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
		cache.test_push_main_back(1, 10, 0);
		cache.test_push_main_back(2, 20, 2);
		cache.test_push_main_back(3, 30, 0);
		cache.test_push_main_back(4, 40, 1);
		cache.test_push_small_front(5, 50, 2);
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
		assert!(cache.resident_len() <= 4);
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
	fn ghost_live_count_bounded_and_compacts() {
		let mut cache = S3Fifo::new(8);
		for round in 0..50i64 {
			for key in 0..40i64 {
				if cache.get(&(key + round)).is_none() {
					cache.put(key + round, key);
				}
			}
			assert!(cache.ghost_live <= cache.main_size.max(1));
			assert!(cache.ghost.len() <= 2 * cache.main_size.max(1));
			assert!(cache.resident_len() <= 8);
		}
	}
}
