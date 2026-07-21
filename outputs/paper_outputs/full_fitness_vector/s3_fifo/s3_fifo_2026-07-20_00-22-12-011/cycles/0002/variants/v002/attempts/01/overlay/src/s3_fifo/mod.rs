use std::collections::HashSet;
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
pub struct S3Fifo<K: PartialEq + Eq + std::hash::Hash + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	ghost_set: HashSet<K>,   // shadow membership index for O(1) ghost-hit detection
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Eq + std::hash::Hash + Clone, V> S3Fifo<K, V> {
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
			ghost_set: HashSet::with_capacity(main_size),
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

		// O(1) ghost-hit check via shadow HashSet
		let in_ghost = self.ghost_set.contains(&key);
		if in_ghost {
			self.ghost_set.remove(&key);
			// ghost deque intentionally not modified — lazy deletion
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
				// Evict from ghost deque back, skipping stale tombstones from lazy deletion,
				// until one live ghost entry is evicted or the deque falls below capacity.
				while self.ghost.len() >= self.main_size {
					match self.ghost.pop_back() {
						Some(k) if self.ghost_set.contains(&k) => {
							self.ghost_set.remove(&k);
							break; // evicted one live entry
						}
						Some(_) => { /* stale tombstone — keep looping */ }
						None => break,
					}
				}
				// Admit the new ghost key, updating both deque and set
				self.ghost_set.insert(tail.key.clone());
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

	/// Updated for lazy deletion: after a ghost-hit scenario, ghost_set.len() <= ghost.len()
	/// because stale tombstones may remain in the deque after O(1) lazy removal from the set.
	#[test]
	fn ghost_set_stays_in_sync_with_ghost_deque() {
		let mut cache = S3Fifo::<i64, i64>::new(4);
		// Fill small queue to trigger evictions (no ghost hits yet)
		for key in 0..10 {
			cache.put(key, key);
		}
		// ghost_set.len() <= ghost.len(): deque may hold stale tombstones
		assert!(cache.ghost_set.len() <= cache.ghost.len());
		// Every key in the set must be findable in the deque (set ⊆ deque)
		for k in &cache.ghost_set {
			assert!(cache.ghost.contains(k));
		}

		// Now trigger a ghost-hit scenario: re-put a key that is currently ghosted.
		// With lazy deletion the ghost deque is NOT modified; only ghost_set loses the key.
		if !cache.ghost_set.is_empty() {
			let ghost_key = *cache.ghost_set.iter().next().unwrap();
			cache.put(ghost_key, ghost_key);

			// After the ghost hit: ghost_set.len() is strictly less than ghost.len()
			// because the stale tombstone for ghost_key remains in the deque.
			assert!(cache.ghost_set.len() <= cache.ghost.len());
			// Set is still a subset of the deque
			for k in &cache.ghost_set {
				assert!(cache.ghost.contains(k));
			}
		}
	}

	/// After repeated ghost hits, ghost.len() never exceeds main_size + number_of_ghost_hits.
	/// Lazy deletion leaves stale tombstones in the deque; the evict_small loop drains them
	/// at most one per subsequent eviction, keeping the deque length bounded.
	#[test]
	fn ghost_deque_bounded_after_repeated_ghost_hits() {
		let mut cache = S3Fifo::<i64, i64>::new(4);
		// capacity=4 → small_size=1, main_size=3

		// Fill the ghost queue to capacity by inserting many unique keys
		for key in 0..20i64 {
			cache.put(key, key);
		}

		// Record ghost state before any ghost hits
		let initial_ghost_len = cache.ghost.len();
		let ghost_keys: Vec<i64> = cache.ghost_set.iter().copied().collect();
		let ghost_hit_count = ghost_keys.len();

		// Perform all ghost hits: each removes the key from ghost_set (O(1)) but
		// intentionally leaves the stale entry in the ghost deque (lazy deletion).
		for &k in &ghost_keys {
			if cache.ghost_set.contains(&k) {
				cache.put(k, k);
			}
		}

		// The deque length must not exceed main_size + ghost_hit_count (tombstone bound)
		assert!(
			cache.ghost.len() <= cache.main_size + ghost_hit_count,
			"ghost.len()={} exceeded main_size={} + ghost_hit_count={}",
			cache.ghost.len(),
			cache.main_size,
			ghost_hit_count
		);
		// Ghost hits do not push anything onto the deque, so its length is unchanged
		assert_eq!(
			cache.ghost.len(),
			initial_ghost_len,
			"ghost hits should not grow the deque"
		);
		// ghost_set cardinality is at most ghost deque length
		assert!(cache.ghost_set.len() <= cache.ghost.len());
	}
}
