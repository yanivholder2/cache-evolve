use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The per-entry frequency counter lives in a parallel array (`small_freq` /
// `main_freq`) kept in lockstep with its queue: index i of the freq deque is the
// counter for index i of the entry deque. Entries therefore hold only key and
// value, which keeps `Entry<u64, u64>` at 16 bytes.
//
// The ghost queue stores 64-bit key fingerprints rather than keys, so eviction
// never clones a key. `ghost_set` holds the live fingerprints; `ghost` may hold
// stale copies of fingerprints that were promoted out of the ghost, and
// `ghost_copies` counts the queue copies of each fingerprint so stale ones can
// be discarded when they reach the back of the queue.
const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value }
	}
}

#[inline]
fn bump_freq(freq: &mut u8) {
	*freq = (freq.saturating_add(1)).min(MAX_FREQ);
}

#[inline]
fn fingerprint<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	small_freq: VecDeque<u8>,
	main: VecDeque<Entry<K, V>>,
	main_freq: VecDeque<u8>,
	ghost: VecDeque<u64>,
	ghost_set: HashSet<u64>,
	ghost_copies: HashMap<u64, u32>,
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
			(capacity / 5).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			small_freq: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			main_freq: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: HashSet::with_capacity(main_size),
			ghost_copies: HashMap::with_capacity(main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		if let Some(i) = self.small.iter().position(|entry| &entry.key == key) {
			bump_freq(&mut self.small_freq[i]);
			return Some(&self.small[i].value);
		}
		let i = self.main.iter().position(|entry| &entry.key == key)?;
		bump_freq(&mut self.main_freq[i]);
		Some(&self.main[i].value)
	}

	fn evict_main(&mut self) {
		while let (Some(tail), Some(mut freq)) = (self.main.pop_back(), self.main_freq.pop_back()) {
			if freq > 0 {
				freq -= 1;
				self.main.push_front(tail);
				self.main_freq.push_front(freq);
			} else {
				break;
			}
		}
	}

	fn ghost_push(&mut self, fp: u64) {
		if self.ghost_set.len() >= self.main_size {
			self.ghost_pop_oldest();
		}
		self.ghost.push_front(fp);
		self.ghost_set.insert(fp);
		*self.ghost_copies.entry(fp).or_insert(0) += 1;
		if self.ghost.len() > 2 * (self.main_size + 1) {
			self.ghost_compact();
		}
	}

	// Drops the oldest live ghost fingerprint, discarding stale copies on the way.
	// The live copy of a fingerprint is always its newest queue copy, so it is the
	// last copy reached from the back.
	fn ghost_pop_oldest(&mut self) {
		while let Some(fp) = self.ghost.pop_back() {
			let remaining = match self.ghost_copies.get_mut(&fp) {
				Some(c) => {
					*c -= 1;
					*c
				}
				None => 0,
			};
			if remaining == 0 {
				self.ghost_copies.remove(&fp);
				if self.ghost_set.remove(&fp) {
					return;
				}
			}
		}
	}

	// Rebuilds the ghost queue with only the live copy of each fingerprint, keeping
	// recency order. Bounds the queue's length to the live ghost count.
	fn ghost_compact(&mut self) {
		let mut seen: HashSet<u64> = HashSet::with_capacity(self.ghost_set.len());
		let mut kept: VecDeque<u64> = VecDeque::with_capacity(self.ghost_set.len());
		for &fp in self.ghost.iter() {
			if self.ghost_set.contains(&fp) && seen.insert(fp) {
				kept.push_back(fp);
			}
		}
		self.ghost = kept;
		self.ghost_copies = self.ghost.iter().map(|&fp| (fp, 1)).collect();
	}
}

impl<K: PartialEq + Clone + Hash, V> S3Fifo<K, V> {
	pub fn put(&mut self, key: K, value: V) {
		if let Some(i) = self.small.iter().position(|entry| entry.key == key) {
			self.small[i].value = value;
			return;
		}
		if let Some(i) = self.main.iter().position(|entry| entry.key == key) {
			self.main[i].value = value;
			return;
		}

		let fp = fingerprint(&key);
		if self.ghost_set.remove(&fp) {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(Entry::new(key, value));
			self.main_freq.push_front(0);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(Entry::new(key, value));
		self.small_freq.push_front(0);
	}

	fn evict_small(&mut self) {
		if let (Some(tail), Some(freq)) = (self.small.pop_back(), self.small_freq.pop_back()) {
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
				self.main_freq.push_front(freq);
			} else {
				let fp = fingerprint(&tail.key);
				self.ghost_push(fp);
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
		cache.main_freq.push_back(0);
		cache.main.push_back(Entry::new(2, 20));
		cache.main_freq.push_back(2);
		cache.main.push_back(Entry::new(3, 30));
		cache.main_freq.push_back(0);
		cache.main.push_back(Entry::new(4, 40));
		cache.main_freq.push_back(1);
		cache.small.push_front(Entry::new(5, 50));
		cache.small_freq.push_front(2);
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
	fn freq_deques_stay_in_lockstep() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			if cache.get(&(key % 6)).is_none() {
				cache.put(key % 6, key);
			}
			assert_eq!(cache.small.len(), cache.small_freq.len());
			assert_eq!(cache.main.len(), cache.main_freq.len());
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
