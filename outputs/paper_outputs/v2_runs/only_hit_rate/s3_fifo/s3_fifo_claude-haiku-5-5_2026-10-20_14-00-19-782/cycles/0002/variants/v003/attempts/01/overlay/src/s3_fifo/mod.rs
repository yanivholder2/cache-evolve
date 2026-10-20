use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys are re-admitted through the small queue, so
// they must earn promotion again with a hit; main eviction uses a second-chance-
// style frequency decrement.
//
// Frequencies live in parallel `u8` deques (`small_freq`, `main_freq`) rather
// than inside `Entry`, so `Entry<i64, i64>` is 16 bytes instead of 24. Invariants:
// `small_freq.len() == small.len()` and `main_freq.len() == main.len()` between
// operations; index `i` in a queue and its frequency deque always describe the
// same entry.
const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	small_freq: VecDeque<u8>,
	main: VecDeque<Entry<K, V>>,
	main_freq: VecDeque<u8>,
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			small_freq: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			main_freq: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		if let Some(i) = self.small.iter().position(|entry| &entry.key == key) {
			let freq = &mut self.small_freq[i];
			*freq = freq.saturating_add(1).min(MAX_FREQ);
			return Some(&self.small[i].value);
		}
		let j = self.main.iter().position(|entry| &entry.key == key)?;
		let freq = &mut self.main_freq[j];
		*freq = freq.saturating_add(1).min(MAX_FREQ);
		Some(&self.main[j].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost.retain(|ghost_key| ghost_key != &key);
			if self.small.len() >= self.small_size {
				self.evict_small();
			}
			self.small.push_front(Entry::new(key, value));
			self.small_freq.push_front(0);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(Entry::new(key, value));
		self.small_freq.push_front(0);
	}

	fn evict_main(&mut self) {
		while let Some(tail) = self.main.pop_back() {
			let freq = self
				.main_freq
				.pop_back()
				.expect("S3Fifo: main_freq out of sync with main");
			if freq > 0 {
				self.main.push_front(tail);
				self.main_freq.push_front(freq - 1);
			} else {
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			let freq = self
				.small_freq
				.pop_back()
				.expect("S3Fifo: small_freq out of sync with small");
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
				self.main_freq.push_front(freq);
			} else {
				if self.ghost.len() >= self.main_size {
					self.ghost.pop_back();
				}
				self.ghost.push_front(tail.key);
			}
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

	fn assert_parallel<K: PartialEq + Clone, V>(cache: &S3Fifo<K, V>) {
		assert_eq!(cache.small.len(), cache.small_freq.len());
		assert_eq!(cache.main.len(), cache.main_freq.len());
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
		assert!(cache.main.iter().any(|entry| entry.key == 1));
		assert_parallel(&cache);
	}

	#[test]
	fn ghost_hit_readmits_to_small() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.small.iter().any(|entry| entry.key == 1));
		assert!(!cache.main.iter().any(|entry| entry.key == 1));
		assert_parallel(&cache);
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.main.push_back(Entry::new(1, 10));
		cache.main_freq.push_back(0);
		cache.main.push_back(Entry::new(2, 20));
		cache.main_freq.push_back(2);
		cache.main.push_back(Entry::new(3, 30));
		cache.main_freq.push_back(0);
		cache.main.push_back(Entry::new(4, 40));
		cache.main_freq.push_back(1);
		cache.small.push_front(Entry::new(5, 50));
		cache.small_freq.push_front(2);
		cache.evict_small();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
		assert_parallel(&cache);
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_parallel(&cache);
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert_eq!(cache.small_freq[0], MAX_FREQ);
	}
}
