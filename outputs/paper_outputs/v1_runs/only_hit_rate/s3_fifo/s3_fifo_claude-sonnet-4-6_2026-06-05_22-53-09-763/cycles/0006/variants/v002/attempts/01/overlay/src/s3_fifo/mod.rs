use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// H02: get() now searches small and main separately so that main-queue hits can
// trigger conditional repositioning to the front of main the first time an
// item's frequency saturates at MAX_FREQ.
const MAX_FREQ: u8 = 5;

#[derive(Debug)]
struct Entry<K, V> {
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
	ghost_size: usize,
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
			(capacity / 10).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let ghost_size = (main_size / 2).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(ghost_size),
			small_size,
			main_size,
			ghost_size,
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
		// 1. Search small first — identical behaviour to before.
		if let Some(entry) = self.small.iter_mut().find(|e| &e.key == key) {
			entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
			return Some(&entry.value);
		}

		// 2. Search main by index so we can conditionally reposition.
		if let Some(pos) = self.main.iter().position(|e| &e.key == key) {
			if self.main[pos].freq == MAX_FREQ - 1 && pos != 0 {
				// First saturation event for this item and it is not already at the
				// front: increment to MAX_FREQ and physically move it to the head of
				// main so eviction pressure cannot reach it until the rest of main
				// drains ahead of it.
				self.main[pos].freq = MAX_FREQ;
				let entry = self.main.remove(pos).unwrap();
				self.main.push_front(entry);
				// Return a reference into the entry we just placed at the front.
				return self.main.front().map(|e| &e.value);
			} else {
				// All other cases: increment in place (clamped), no repositioning.
				// Covers:
				//   - freq < MAX_FREQ - 1  → ordinary increment
				//   - freq == MAX_FREQ - 1 && pos == 0 → already at front, just saturate
				//   - freq == MAX_FREQ      → already saturated, clamp keeps it there
				self.main[pos].freq = (self.main[pos].freq.saturating_add(1)).min(MAX_FREQ);
				return Some(&self.main[pos].value);
			}
		}

		// 3. Key not present in either resident queue.
		None
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
				tail.freq -= 1;
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
				if self.ghost.len() >= self.ghost_size {
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
	fn main_item_repositioned_on_first_saturation() {
		// Verify that a main-queue item at a non-zero position is moved to the
		// front exactly on the access that brings its freq from MAX_FREQ-1 to
		// MAX_FREQ, and stays there on subsequent accesses.
		let mut cache = S3Fifo::<i64, i64>::new(10);
		// Manually place two items in main at known positions.
		cache.main.push_back(Entry::new(1, 10)); // position 0
		cache.main.push_back(Entry::new(2, 20)); // position 1
		// Give item-2 a freq of MAX_FREQ-1 so the next get triggers repositioning.
		cache.main.back_mut().unwrap().freq = MAX_FREQ - 1;

		// Before repositioning: item-2 is at back (position 1).
		assert_eq!(cache.main.iter().position(|e| e.key == 2), Some(1));

		// This get should increment item-2 to MAX_FREQ and move it to the front.
		let val = cache.get(&2);
		assert_eq!(*val.unwrap(), 20);
		assert_eq!(cache.main.front().unwrap().key, 2);
		assert_eq!(cache.main.front().unwrap().freq, MAX_FREQ);

		// Subsequent gets must NOT reposition (already at front, freq already MAX).
		cache.get(&2);
		assert_eq!(cache.main.front().unwrap().key, 2);
		assert_eq!(cache.main.front().unwrap().freq, MAX_FREQ);
	}

	#[test]
	fn main_item_at_front_not_repositioned_on_saturation() {
		// An item already at pos 0 with freq MAX_FREQ-1 should just have its freq
		// incremented without any VecDeque surgery.
		let mut cache = S3Fifo::<i64, i64>::new(10);
		cache.main.push_back(Entry::new(1, 10)); // position 0
		cache.main.push_back(Entry::new(2, 20)); // position 1
		cache.main.front_mut().unwrap().freq = MAX_FREQ - 1;

		let val = cache.get(&1);
		assert_eq!(*val.unwrap(), 10);
		// Item-1 must still be at the front.
		assert_eq!(cache.main.front().unwrap().key, 1);
		assert_eq!(cache.main.front().unwrap().freq, MAX_FREQ);
	}
}
