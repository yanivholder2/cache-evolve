use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// S3-FIFO (Yang et al.) with hash indexes: resident entries live in a map keyed
// by key (O(1) get/put lookup); the small/main queues hold keys in FIFO order;
// ghost membership is tracked by a HashSet shadowing the ghost queue.
const MAX_FREQ: u8 = 3;

#[derive(Debug)]
struct Slot<V> {
	value: V,
	freq: u8,
	in_main: bool,
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	index: HashMap<K, Slot<V>>,
	small: VecDeque<K>,
	main: VecDeque<K>,
	ghost: VecDeque<K>,
	ghost_set: HashSet<K>,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			index: HashMap::with_capacity(capacity),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: HashSet::with_capacity(main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = self.index.get_mut(key)?;
		slot.freq = (slot.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&slot.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(slot) = self.index.get_mut(&key) {
			slot.value = value;
			return;
		}

		if self.ghost_set.remove(&key) {
			// Position must be removed from the middle of the ghost queue.
			if let Some(pos) = self.ghost.iter().position(|k| k == &key) {
				self.ghost.remove(pos);
			}
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(key.clone());
			self.index.insert(key, Slot { value, freq: 0, in_main: true });
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(key.clone());
		self.index.insert(key, Slot { value, freq: 0, in_main: false });
	}

	fn evict_main(&mut self) {
		while let Some(tail) = self.main.pop_back() {
			let slot = self.index.get_mut(&tail).expect("main key must be indexed");
			if slot.freq > 0 {
				slot.freq -= 1;
				self.main.push_front(tail);
			} else {
				self.index.remove(&tail);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			let freq = self.index.get(&tail).expect("small key must be indexed").freq;
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				if let Some(slot) = self.index.get_mut(&tail) {
					slot.in_main = true;
				}
				self.main.push_front(tail);
			} else {
				self.index.remove(&tail);
				if self.ghost.len() >= self.main_size {
					if let Some(old) = self.ghost.pop_back() {
						self.ghost_set.remove(&old);
					}
				}
				self.ghost_set.insert(tail.clone());
				self.ghost.push_front(tail);
			}
		}
	}
}

impl<K: Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
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
		assert!(cache.main.iter().any(|k| *k == 1));
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
}
