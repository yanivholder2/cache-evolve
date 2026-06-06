use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// H03: a dedicated short `hot` queue is scanned first in `get`/`find_entry_mut`.
// When a `main`-resident entry saturates its frequency (`freq == MAX_FREQ`) it is
// promoted into `hot`, so the common median hit resolves after a tiny `hot` scan
// instead of a full `main` walk. The `hot` budget is taken out of `main`, so the
// total resident budget (`small_size + hot_size + main_size == capacity`) is
// unchanged and hit-rate stays neutral. `hot` is not an eviction source.
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
	hot: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	small_size: usize,
	hot_size: usize,
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
			(capacity / 32).max(1).min(capacity - 1)
		};
		// Reserve a short `hot` segment out of the original `main` budget so the
		// total resident budget still equals `capacity` (hit-rate neutral).
		let remaining = capacity - small_size;
		let hot_size = if remaining == 0 {
			0
		} else {
			(remaining / 8).max(1).min(remaining)
		};
		let main_size = remaining - hot_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			hot: VecDeque::with_capacity(hot_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size + hot_size),
			small_size,
			hot_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.hot
			.iter_mut()
			.chain(self.main.iter_mut())
			.chain(self.small.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Scan the short `hot` segment first: the median hit lands here and
		// resolves after a bounded `hot_size` walk instead of a full `main` walk.
		if let Some(idx) = self.hot.iter().position(|entry| &entry.key == key) {
			let freq = (self.hot[idx].freq.saturating_add(1)).min(MAX_FREQ);
			self.hot[idx].freq = freq;
			return Some(&self.hot[idx].value);
		}

		// Then `main`. A `main`-resident entry that saturates its frequency is
		// promoted into `hot` so subsequent hits skip the `main` walk entirely.
		if let Some(idx) = self.main.iter().position(|entry| &entry.key == key) {
			let freq = (self.main[idx].freq.saturating_add(1)).min(MAX_FREQ);
			self.main[idx].freq = freq;
			if freq == MAX_FREQ && self.hot_size > 0 {
				let entry = self.main.remove(idx).expect("main index valid");
				// Keep `hot` bounded: demote its coldest tail back to `main`.
				if self.hot.len() >= self.hot_size {
					if let Some(demoted) = self.hot.pop_back() {
						self.main.push_front(demoted);
					}
				}
				self.hot.push_front(entry);
				return Some(&self.hot[0].value);
			}
			return Some(&self.main[idx].value);
		}

		// Finally the short admission queue.
		if let Some(idx) = self.small.iter().position(|entry| &entry.key == key) {
			let freq = (self.small[idx].freq.saturating_add(1)).min(MAX_FREQ);
			self.small[idx].freq = freq;
			return Some(&self.small[idx].value);
		}

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
				if self.ghost.len() >= (self.main_size / 2).max(1) {
					self.ghost.pop_back();
				}
				self.ghost.push_front(tail.key);
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
				if self.ghost.len() >= (self.main_size / 2).max(1) {
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
	fn main_entry_promotes_to_hot_at_max_freq() {
		let mut cache = S3Fifo::<i64, i64>::new(64);
		cache.main.push_front(Entry::new(1, 10));
		for _ in 0..MAX_FREQ {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert!(cache.hot.iter().any(|entry| entry.key == 1));
		assert!(!cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.hot.len() + cache.main.len() <= 4);
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
