use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and ghost queues for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Variant v002 (H02): main-queue victims are also recorded as ghosts, so a key
// aged out of main skips the small queue on its next reference.
//
// Variant c2/v001 (H01): the ghost is split into two separately bounded
// regions. `ghost_small` holds small-queue demotees at the baseline bound
// (`main_size`); `ghost_main` holds main-queue victims at `main_size / 2`.
// Main victims no longer crowd out demotee history.
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

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost_small: VecDeque<K>,
	ghost_main: VecDeque<K>,
	small_size: usize,
	main_size: usize,
	ghost_main_size: usize,
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
		let ghost_main_size = (main_size / 2).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost_small: VecDeque::with_capacity(main_size),
			ghost_main: VecDeque::with_capacity(ghost_main_size),
			small_size,
			main_size,
			ghost_main_size,
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
		let entry = self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		let ghost_hit = if self.ghost_small.iter().any(|ghost_key| ghost_key == &key) {
			self.ghost_small.retain(|ghost_key| ghost_key != &key);
			true
		} else if self.ghost_main.iter().any(|ghost_key| ghost_key == &key) {
			self.ghost_main.retain(|ghost_key| ghost_key != &key);
			true
		} else {
			false
		};

		if ghost_hit {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(Entry::new(key, value));
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(Entry::new(key, value));
	}

	/// Bounded insert into the small-demotee ghost region (baseline bound).
	fn push_ghost_small(&mut self, key: K) {
		if self.ghost_small.len() >= self.main_size {
			self.ghost_small.pop_back();
		}
		self.ghost_small.push_front(key);
	}

	/// Bounded insert into the main-victim ghost region.
	fn push_ghost_main(&mut self, key: K) {
		if self.ghost_main.len() >= self.ghost_main_size {
			self.ghost_main.pop_back();
		}
		self.ghost_main.push_front(key);
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				self.push_ghost_main(tail.key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			if tail.freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
			} else {
				self.push_ghost_small(tail.key);
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
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().freq = 2;
		cache.main.push_back(Entry::new(3, 30));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(4, 40));
		cache.main.back_mut().unwrap().freq = 1;
		cache.small.push_front(Entry::new(5, 50));
		cache.small.front_mut().unwrap().freq = 2;
		cache.evict_small();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
	}

	#[test]
	fn main_victim_is_ghosted() {
		let mut cache = S3Fifo::new(5);
		cache.main.push_back(Entry::new(1, 10));
		cache.main.push_back(Entry::new(2, 20));
		cache.main.push_back(Entry::new(3, 30));
		cache.main.push_back(Entry::new(4, 40));
		cache.evict_main();
		assert!(!cache.main.iter().any(|entry| entry.key == 4));
		assert!(cache.ghost_main.iter().any(|k| *k == 4));
		assert!(!cache.ghost_small.iter().any(|k| *k == 4));
		cache.put(4, 40);
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.ghost_main.iter().any(|k| *k == 4));
	}

	#[test]
	fn ghost_regions_stay_bounded() {
		let mut cache = S3Fifo::new(8);
		for key in 0..500 {
			if cache.get(&(key % 37)).is_none() {
				cache.put(key % 37, key);
			}
			assert!(cache.ghost_small.len() <= cache.main_size);
			assert!(cache.ghost_main.len() <= cache.ghost_main_size);
			assert!(cache.small.len() + cache.main.len() <= 8);
		}
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
}
