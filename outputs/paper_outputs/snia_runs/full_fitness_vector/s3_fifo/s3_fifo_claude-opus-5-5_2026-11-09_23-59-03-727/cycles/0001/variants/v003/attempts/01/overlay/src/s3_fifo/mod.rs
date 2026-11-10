use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Variant H03: the small/main split adapts online (ARC-style). Ghost hits grow
// the small (admission) window; second-chance reinsertions in main shrink it.
// Queue bounds are enforced lazily, with a hard total-capacity bound after
// every put.
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
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
	capacity: usize,
	min_small: usize,
	max_small: usize,
	delta: usize,
	ghost_size: usize,
	adaptive: bool,
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
		let adaptive = capacity > 1;
		let (min_small, max_small) = if adaptive {
			let max_small = (capacity / 2).max(1).min(capacity - 1);
			let min_small = (capacity / 100).max(1).min(max_small);
			(min_small, max_small)
		} else {
			(1, 1)
		};
		let small_size = small_size.max(min_small).min(max_small);
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			small_size,
			main_size,
			capacity,
			min_small,
			max_small,
			delta: (capacity / 1000).max(1),
			ghost_size: main_size,
			adaptive,
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

	fn grow_small(&mut self) {
		if !self.adaptive {
			return;
		}
		self.small_size = (self.small_size + self.delta).min(self.max_small);
		self.main_size = self.capacity - self.small_size;
	}

	fn shrink_small(&mut self) {
		if !self.adaptive {
			return;
		}
		self.small_size = self.small_size.saturating_sub(self.delta).max(self.min_small);
		self.main_size = self.capacity - self.small_size;
	}

	/// Make room in main for one push, honouring the (possibly resized) target.
	fn make_room_main(&mut self) {
		if !self.adaptive {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			return;
		}
		while self.main.len() >= self.main_size && !self.main.is_empty() {
			self.evict_main();
		}
	}

	/// Hard bound: total residents never exceed capacity after a put.
	fn enforce_total(&mut self) {
		if !self.adaptive {
			return;
		}
		while self.small.len() + self.main.len() > self.capacity {
			if self.small.len() > self.small_size || self.main.is_empty() {
				self.evict_small();
			} else {
				self.evict_main();
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost.retain(|ghost_key| ghost_key != &key);
			// The small window was too short to catch this reuse: grow it.
			self.grow_small();
			self.make_room_main();
			self.main.push_front(Entry::new(key, value));
			self.enforce_total();
			return;
		}

		while self.small.len() >= self.small_size && !self.small.is_empty() {
			self.evict_small();
		}
		self.small.push_front(Entry::new(key, value));
		self.enforce_total();
	}

	fn evict_main(&mut self) {
		let mut shrunk = false;
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
				if !shrunk {
					// Main holds useful entries: protect it by shrinking small.
					shrunk = true;
					self.shrink_small();
				}
			} else {
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			if tail.freq > 1 {
				self.make_room_main();
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
	fn adaptive_capacity_bound_under_churn() {
		let cap = 64;
		let mut cache = S3Fifo::new(cap);
		for i in 0..20_000i64 {
			let key = (i * 7919) % 300 + if i % 3 == 0 { 0 } else { i % 17 };
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
			assert!(cache.small.len() + cache.main.len() <= cap);
			assert!(cache.small_size >= cache.min_small && cache.small_size <= cache.max_small);
			assert_eq!(cache.small_size + cache.main_size, cap);
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
