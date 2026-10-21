use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decay.
//
// H03 (cycle 3): main-queue frequency cap raised from 3 to 7 so heavily reused
// keys survive more second-chance laps in main.
// H02 (cycle 4): frequency cap raised further from 7 to 15 to test whether the
// retention gain keeps rising past 7.
// H01 (cycle 5): evict_main now halves a rotated entry's freq (freq >>= 1)
// instead of decrementing it, so a saturated key survives at most 4 laps
// (15 -> 7 -> 3 -> 1 -> 0). evict_main still terminates: every rotation
// strictly decreases some entry's freq, bounding rotations by main.len() * 4.
const MAX_FREQ: u8 = 15;

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
		// Small queue is 5% of capacity (inherited from cycle/0002/v003).
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
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

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost.retain(|ghost_key| ghost_key != &key);
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

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				// H01: halve credit on each rotation (log-scale decay).
				tail.freq >>= 1;
				self.main.push_front(tail);
			} else {
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
		// Main (front..back): 1(f0), 2(f2), 3(f0), 4(f1). The sweep halves
		// key 4 (1 -> 0) and rotates it, then evicts key 3 (f0).
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
		assert_eq!(cache.main.iter().find(|e| e.key == 4).unwrap().freq, 0);
		assert_eq!(cache.main.iter().find(|e| e.key == 2).unwrap().freq, 2);
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
	fn small_queue_is_five_percent_of_capacity() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_size, 102);
		assert_eq!(cache.main_size, 1946);
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
	fn main_resident_frequency_saturates_at_fifteen() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.main.push_front(Entry::new(1, 10));
		for _ in 0..20 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		let entry = cache.main.iter().find(|entry| entry.key == 1).unwrap();
		assert_eq!(entry.freq, 15);
		assert_eq!(MAX_FREQ, 15);
	}

	#[test]
	fn single_sweep_halves_frequency_six_to_three() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		// Cold entry at front, freq-6 entry at the tail.
		cache.main.push_back(Entry::new(2, 20));
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 6;
		cache.evict_main();
		assert!(!cache.main.iter().any(|entry| entry.key == 2));
		assert_eq!(cache.main.iter().find(|e| e.key == 1).unwrap().freq, 3);
	}

	#[test]
	fn max_freq_key_survives_four_main_passes_then_evicted_on_fifth() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		// Hot key sits at main's tail with a saturated frequency.
		cache.main.push_back(Entry::new(1, 10));
		for _ in 0..20 {
			cache.get(&1);
		}
		assert_eq!(cache.main.back().unwrap().freq, MAX_FREQ);

		// Each pass: a fresh cold (freq 0) entry sits just ahead of the hot
		// key, so evict_main rotates the hot key once (freq >>= 1) and then
		// evicts the cold entry. The hot key is not re-accessed.
		let expected = [7u8, 3, 1, 0];
		for (pass, want) in expected.iter().enumerate() {
			let cold = 100 + pass as i64;
			cache.main.push_front(Entry::new(cold, cold));
			cache.evict_main();
			let hot = cache.main.iter().find(|entry| entry.key == 1);
			assert!(hot.is_some(), "hot key evicted on pass {}", pass + 1);
			assert_eq!(hot.unwrap().freq, *want);
			assert!(!cache.main.iter().any(|entry| entry.key == cold));
		}

		// Fifth pass: hot key is at the tail with freq 0 and is evicted.
		cache.main.push_front(Entry::new(200, 200));
		assert_eq!(cache.main.back().unwrap().key, 1);
		cache.evict_main();
		assert!(!cache.main.iter().any(|entry| entry.key == 1));
		assert!(cache.main.iter().any(|entry| entry.key == 200));
	}
}
