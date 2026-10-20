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
// Residency and per-key state live in a keyed index (`resident`). The small and
// main queues hold only keys, so get/put locate a resident entry in O(1) rather
// than scanning the queues. Eviction order is unchanged from the linear-scan
// version.
const MAX_FREQ: u8 = 3;

/// Legacy per-entry record. The cache no longer stores this on its data path
/// (residency lives in `S3Fifo::resident` as a `Slot`). It is retained because
/// the fixed metadata-size measurement binary names `Entry<K, V>` directly.
#[allow(dead_code)]
#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
}

#[derive(Debug)]
struct Slot<V> {
	value: V,
	freq: u8,
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
	resident: HashMap<K, Slot<V>>,
	small: VecDeque<K>,
	main: VecDeque<K>,
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
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
			resident: HashMap::with_capacity(capacity),
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

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = self.resident.get_mut(key)?;
		slot.freq = slot.freq.saturating_add(1).min(MAX_FREQ);
		Some(&slot.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(slot) = self.resident.get_mut(&key) {
			slot.value = value;
			return;
		}

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost.retain(|ghost_key| ghost_key != &key);
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(key.clone());
			self.resident.insert(key, Slot { value, freq: 0 });
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(key.clone());
		self.resident.insert(key, Slot { value, freq: 0 });
	}

	fn evict_main(&mut self) {
		while let Some(key) = self.main.pop_back() {
			let freq = match self.resident.get_mut(&key) {
				Some(slot) => {
					let freq = slot.freq;
					if freq > 0 {
						slot.freq -= 1;
					}
					freq
				}
				None => 0,
			};
			if freq > 0 {
				self.main.push_front(key);
			} else {
				self.resident.remove(&key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		let Some(key) = self.small.pop_back() else {
			return;
		};
		let freq = self.resident.get(&key).map_or(0, |slot| slot.freq);
		if freq > 1 {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(key);
		} else {
			self.resident.remove(&key);
			if self.ghost.len() >= self.main_size {
				self.ghost.pop_back();
			}
			self.ghost.push_front(key);
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
		assert!(cache.main.iter().any(|key| *key == 1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|key| *key == 1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		for (key, freq) in [(1i64, 0u8), (2, 2), (3, 0), (4, 1)] {
			cache.main.push_back(key);
			cache.resident.insert(key, Slot { value: key * 10, freq });
		}
		cache.small.push_front(5);
		cache.resident.insert(5, Slot { value: 50, freq: 2 });
		cache.evict_small();
		assert!(cache.main.iter().any(|key| *key == 4));
		assert!(!cache.main.iter().any(|key| *key == 3));
		assert!(!cache.resident.contains_key(&3));
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_eq!(cache.resident.len(), cache.small.len() + cache.main.len());
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
