use std::collections::{HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// S3-FIFO with hash-set shadow indexes (`resident`, `ghost_set`) plus a small
// "recent-main" filter: keys evicted from main are remembered in a bounded ring;
// if such a key is re-inserted (into small) and later reaches the small tail with
// freq 0, it gets one reinsertion at the small head instead of being evicted.
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
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	resident: HashSet<K>,
	ghost_set: HashSet<K>,
	recent: VecDeque<K>,
	recent_set: HashSet<K>,
	recent_size: usize,
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
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let recent_size = (capacity / 10).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			resident: HashSet::with_capacity(capacity),
			ghost_set: HashSet::with_capacity(main_size),
			recent: VecDeque::with_capacity(recent_size),
			recent_set: HashSet::with_capacity(recent_size),
			recent_size,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		if !self.resident.contains(key) {
			return None;
		}
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let entry = self.find_entry_mut(key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		if self.ghost_set.remove(&key) {
			self.ghost.retain(|ghost_key| ghost_key != &key);
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.resident.insert(key.clone());
			self.main.push_front(Entry::new(key, value));
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.resident.insert(key.clone());
		self.small.push_front(Entry::new(key, value));
	}

	fn note_main_evicted(&mut self, key: &K) {
		if self.recent_set.insert(key.clone()) {
			self.recent.push_front(key.clone());
			while self.recent.len() > self.recent_size {
				if let Some(old) = self.recent.pop_back() {
					self.recent_set.remove(&old);
				}
			}
		}
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				self.resident.remove(&tail.key);
				self.note_main_evicted(&tail.key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		while let Some(tail) = self.small.pop_back() {
			if tail.freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
			} else if tail.freq == 0 && self.recent_set.remove(&tail.key) {
				// One-shot second chance: flag cleared, so no further retry.
				self.small.push_front(tail);
				continue;
			} else {
				self.resident.remove(&tail.key);
				if self.ghost.len() >= self.main_size {
					if let Some(old) = self.ghost.pop_back() {
						self.ghost_set.remove(&old);
					}
				}
				self.ghost_set.insert(tail.key.clone());
				self.ghost.push_front(tail.key);
			}
			break;
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
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_eq!(cache.resident.len(), cache.small.len() + cache.main.len());
	}
}
