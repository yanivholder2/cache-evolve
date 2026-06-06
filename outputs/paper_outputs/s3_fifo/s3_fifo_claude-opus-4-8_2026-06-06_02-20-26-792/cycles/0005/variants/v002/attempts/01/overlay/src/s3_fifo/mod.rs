use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
const MAX_FREQ: u8 = 3;

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
			(capacity / 24).max(1).min(capacity - 1)
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
		// 1. Search the small admission queue first. A hit there bumps `freq`
		//    (clamped to MAX_FREQ) without reordering small.
		if let Some(pos) = self.small.iter().position(|entry| &entry.key == key) {
			let entry = &mut self.small[pos];
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			return Some(&self.small[pos].value);
		}

		// 2. Otherwise search the main resident queue. A hit there promotes the
		//    entry to the front (most-recently-used position) in addition to the
		//    `freq` bump, making main LRU-aware while leaving the tail-based
		//    second-chance eviction discipline intact.
		if let Some(pos) = self.main.iter().position(|entry| &entry.key == key) {
			let mut entry = self
				.main
				.remove(pos)
				.expect("S3Fifo::get: position within bounds");
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			self.main.push_front(entry);
			return Some(&self.main.front().expect("S3Fifo::get: just pushed").value);
		}

		// 3. No match.
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
		// Recency-aware tail-window victim selection. Since `get()` keeps hot
		// keys at the front, the tail holds the coldest-by-recency residents.
		// Rather than evicting the strict tail under a second-chance rotation,
		// inspect a small window of the W oldest residents and remove the
		// lowest-`freq` entry within it, sharpening which cold resident the
		// scan reaches. Ties break toward the entry nearest the tail (oldest).
		let len = self.main.len();
		if len <= 1 {
			self.main.pop_back();
			return;
		}

		let window = (len / 16).max(1);
		let start = len - window; // front-most index of the tail window

		// Choose the victim: lowest `freq`, breaking ties toward the tail.
		// Iterating front-to-tail with `<=` lets a later (older) index win ties.
		let mut victim = start;
		let mut victim_freq = self.main[start].freq;
		for idx in (start + 1)..len {
			let f = self.main[idx].freq;
			if f <= victim_freq {
				victim_freq = f;
				victim = idx;
			}
		}

		// Preserve second-chance aging: decrement every other inspected entry
		// with `freq > 0` in place, leaving its position unchanged.
		for idx in start..len {
			if idx != victim && self.main[idx].freq > 0 {
				self.main[idx].freq -= 1;
			}
		}

		// Exactly one removal per call.
		self.main.remove(victim);
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
	fn main_hit_moves_entry_to_front() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.main.push_back(Entry::new(1, 10));
		cache.main.push_back(Entry::new(2, 20));
		cache.main.push_back(Entry::new(3, 30));
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(cache.main.front().unwrap().key, 3);
		assert_eq!(cache.small.len() + cache.main.len(), 3);
	}

	#[test]
	fn main_eviction_removes_exactly_one_within_window() {
		let mut cache = S3Fifo::<i64, i64>::new(40);
		for key in 0..30 {
			cache.main.push_back(Entry::new(key, key));
		}
		let before = cache.main.len();
		cache.evict_main();
		assert_eq!(cache.main.len(), before - 1);
	}
}
