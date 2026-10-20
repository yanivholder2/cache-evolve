use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
const MAX_FREQ: u8 = 3;
// Per-slot state byte: the low seven bits hold the access frequency
// (0..=MAX_FREQ) and the high bit is the resident bit, set while the slot lives
// in `small` or `main` and cleared when the slot leaves the cache.
const FREQ_MASK: u8 = 0x7F;
const RESIDENT_BIT: u8 = 0x80;
// Counting filter over the ghost queue. Each ghost key contributes one count to
// the bucket chosen by `ghost_bucket`; a zero bucket proves the key is absent
// from `ghost`, so the linear ghost scan can be skipped.
const GHOST_FILTER_BUCKETS: usize = 64;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	state: u8,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, state: RESIDENT_BIT }
	}

	fn freq(&self) -> u8 {
		self.state & FREQ_MASK
	}

	fn set_freq(&mut self, freq: u8) {
		self.state = (self.state & RESIDENT_BIT) | (freq & FREQ_MASK);
	}

	fn is_resident(&self) -> bool {
		(self.state & RESIDENT_BIT) != 0
	}

	fn clear_resident(&mut self) {
		self.state &= FREQ_MASK;
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	// Counting filter mirroring `ghost`. Counts are u32 rather than u8 so that a
	// bucket holding more than 255 ghost keys (possible at large capacities) can
	// neither overflow nor wrap to zero, which would hide a live ghost key.
	ghost_filter: [u32; GHOST_FILTER_BUCKETS],
	// Residency probe outcome carried from a `get` miss to the immediately
	// following `put` of the same key. A miss mutates nothing, so the key is still
	// known to be absent when `put` runs; `put` then skips its duplicate probe.
	// Cleared by any `get` hit and consumed by every `put`.
	last_miss: Option<K>,
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
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_filter: [0; GHOST_FILTER_BUCKETS],
			last_miss: None,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn ghost_bucket(key: &K) -> usize {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		(hasher.finish() & (GHOST_FILTER_BUCKETS as u64 - 1)) as usize
	}

	// Reads the resident bit carried by each queued slot.
	fn contains_resident(&self, key: &K) -> bool {
		self
			.small
			.iter()
			.chain(self.main.iter())
			.any(|entry| entry.is_resident() && &entry.key == key)
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		if !self.contains_resident(key) {
			self.last_miss = Some(key.clone());
			return None;
		}
		self.last_miss = None;
		let entry = self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)?;
		let freq = entry.freq().saturating_add(1).min(MAX_FREQ);
		entry.set_freq(freq);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		// Reuse the residency probe from a immediately preceding `get` miss on the
		// same key instead of probing the resident bits a second time.
		let known_absent = match self.last_miss.take() {
			Some(missed) => missed == key,
			None => false,
		};
		if !known_absent && self.contains_resident(&key) {
			if let Some(entry) = self.find_entry_mut(&key) {
				entry.value = value;
			}
			return;
		}

		let bucket = Self::ghost_bucket(&key);
		// A zero bucket means no ghost key hashes here, so the scan is skipped.
		let in_ghost = self.ghost_filter[bucket] != 0
			&& self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			let before = self.ghost.len();
			self.ghost.retain(|ghost_key| ghost_key != &key);
			self.ghost_filter[bucket] -= (before - self.ghost.len()) as u32;
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
			if tail.freq() > 0 {
				let freq = tail.freq() - 1;
				tail.set_freq(freq);
				self.main.push_front(tail);
			} else {
				tail.clear_resident();
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(mut tail) = self.small.pop_back() {
			if tail.freq() > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
			} else {
				if self.ghost.len() >= self.main_size {
					if let Some(old) = self.ghost.pop_back() {
						self.ghost_filter[Self::ghost_bucket(&old)] -= 1;
					}
				}
				tail.clear_resident();
				self.ghost_filter[Self::ghost_bucket(&tail.key)] += 1;
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
		cache.main.back_mut().unwrap().set_freq(0);
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().set_freq(2);
		cache.main.push_back(Entry::new(3, 30));
		cache.main.back_mut().unwrap().set_freq(0);
		cache.main.push_back(Entry::new(4, 40));
		cache.main.back_mut().unwrap().set_freq(1);
		cache.small.push_front(Entry::new(5, 50));
		cache.small.front_mut().unwrap().set_freq(2);
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
		assert_eq!(cache.small.front().unwrap().freq(), MAX_FREQ);
	}

	#[test]
	fn resident_bit_mirrors_queues() {
		use std::collections::HashSet;

		let mut cache = S3Fifo::<i64, i64>::new(8);
		let check = |cache: &S3Fifo<i64, i64>| {
			let from_queues: HashSet<i64> = cache
				.small
				.iter()
				.chain(cache.main.iter())
				.map(|entry| entry.key)
				.collect();
			assert!(cache
				.small
				.iter()
				.chain(cache.main.iter())
				.all(|entry| entry.is_resident()));
			for key in 0..40i64 {
				assert_eq!(cache.contains_resident(&key), from_queues.contains(&key));
			}
		};
		for key in 0..40i64 {
			cache.put(key, key);
			check(&cache);
			cache.get(&(key / 2));
			check(&cache);
			if key % 3 == 0 {
				cache.put(key / 2, key);
				check(&cache);
			}
		}
	}

	#[test]
	fn miss_then_put_matches_direct_put() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		for key in 0..40i64 {
			assert!(cache.get(&key).is_none());
			cache.put(key, key);
			assert_eq!(*cache.get(&key).unwrap(), key);
		}
		assert!(cache.small.len() + cache.main.len() <= 8);
	}

	#[test]
	fn ghost_filter_tracks_ghost_queue() {
		fn filter_total(cache: &S3Fifo<i64, i64>) -> usize {
			cache.ghost_filter.iter().map(|&c| c as usize).sum()
		}
		let mut cache = S3Fifo::<i64, i64>::new(8);
		for key in 0..30i64 {
			cache.put(key, key);
			assert_eq!(filter_total(&cache), cache.ghost.len());
			if key >= 2 {
				// `key - 2` was evicted from `small` into the ghost queue, so this put
				// is a ghost re-admission that exercises the filter decrement.
				cache.put(key - 2, key);
				assert_eq!(filter_total(&cache), cache.ghost.len());
			}
		}
	}
}
