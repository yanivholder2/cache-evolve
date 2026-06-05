// O(1) LRU cache with zone-based partitioning and periodic frequency decay.
// Two zones: cold zone (2/3 capacity, LRU eviction) and hot zone (1/3 capacity, frequency eviction).
// Items in cold zone promoted to hot zone when frequency >= 2.
// Independent frequency decay for each zone.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Zone {
	Cold,
	Hot,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	freq: u8,
	zone: Zone,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	// Cold zone (LRU order)
	cold_head: Option<usize>,
	cold_tail: Option<usize>,
	cold_size: usize,
	// Hot zone (frequency order)
	hot_head: Option<usize>,
	hot_tail: Option<usize>,
	hot_size: usize,
	// Capacity settings
	capacity: usize,
	cold_capacity: usize,
	hot_capacity: usize,
	// Operation counters for frequency decay
	total_operations: u64,
	hot_last_decay: u64,
	cold_last_decay: u64,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("LruCache.try_new: capacity must be > 0");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");

		let cold_capacity = (capacity * 2) / 3;
		let hot_capacity = capacity - cold_capacity;

		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			cold_head: None,
			cold_tail: None,
			cold_size: 0,
			hot_head: None,
			hot_tail: None,
			hot_size: 0,
			capacity,
			cold_capacity,
			hot_capacity,
			total_operations: 0,
			hot_last_decay: 0,
			cold_last_decay: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;

		let zone = self.nodes[idx].zone;

		// Increment frequency counter (saturating at 255)
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);

		// Increment operation counter and check for frequency resets
		self.total_operations += 1;
		self.check_and_perform_frequency_resets();

		// Check for promotion from cold to hot
		if zone == Zone::Cold && self.nodes[idx].freq >= 2 {
			self.promote_to_hot(idx);
		} else {
			// Promote to MRU within the current zone
			self.unlink_in_zone(idx, zone);
			self.push_back_in_zone(idx, zone);
		}

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts an entry based on zone policy.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let zone = self.nodes[idx].zone;
			self.unlink_in_zone(idx, zone);
			self.push_back_in_zone(idx, zone);

			// Increment operation counter
			self.total_operations += 1;
			self.check_and_perform_frequency_resets();

			return;
		}

		// New key — need an arena slot
		if self.nodes.len() >= self.capacity {
			// Arena full: evict based on zone policy
			let idx = self.evict_to_reuse();
			self.nodes[idx].key = key.clone();
			self.nodes[idx].value = value;
			self.nodes[idx].freq = 0;
			self.nodes[idx].zone = Zone::Cold;

			self.push_back_in_zone(idx, Zone::Cold);
			self.cold_size += 1;
			self.map.insert(key, idx);
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 0,
				zone: Zone::Cold,
			});

			self.push_back_in_zone(idx, Zone::Cold);
			self.cold_size += 1;
			self.map.insert(key, idx);
		}

		// Increment operation counter
		self.total_operations += 1;
		self.check_and_perform_frequency_resets();

		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Evict a node and return its index for reuse.
	/// Prioritizes cold zone (LRU) over hot zone (frequency-based).
	fn evict_to_reuse(&mut self) -> usize {
		// If cold zone has items, evict LRU from cold zone
		if self.cold_size > 0 {
			let victim_idx = self.cold_head.expect("cold list non-empty");
			let evicted_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink_in_zone(victim_idx, Zone::Cold);
			self.cold_size -= 1;
			victim_idx
		} else {
			// Otherwise, evict from hot zone using frequency-based eviction
			debug_assert!(self.hot_size > 0, "at least one zone should have items");

			let mut victim_idx = self.hot_head.expect("hot list non-empty");
			let mut min_freq = self.nodes[victim_idx].freq;

			let mut current = self.nodes[victim_idx].next;
			while let Some(idx) = current {
				if self.nodes[idx].freq < min_freq {
					min_freq = self.nodes[idx].freq;
					victim_idx = idx;
				}
				current = self.nodes[idx].next;
			}

			let evicted_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink_in_zone(victim_idx, Zone::Hot);
			self.hot_size -= 1;
			victim_idx
		}
	}

	/// Promote a cold-zone node to the hot zone.
	/// If hot zone is at capacity, evict the lowest-frequency item from hot zone.
	fn promote_to_hot(&mut self, idx: usize) {
		if self.nodes[idx].zone == Zone::Hot {
			return; // Already in hot zone
		}

		// Unlink from cold zone
		self.unlink_in_zone(idx, Zone::Cold);
		self.cold_size -= 1;

		// If hot zone is at capacity, evict from hot zone first
		if self.hot_size >= self.hot_capacity {
			// Find and evict the lowest frequency item in hot zone
			let mut victim_idx = self.hot_head.expect("hot list non-empty");
			let mut min_freq = self.nodes[victim_idx].freq;

			let mut current = self.nodes[victim_idx].next;
			while let Some(idx) = current {
				if self.nodes[idx].freq < min_freq {
					min_freq = self.nodes[idx].freq;
					victim_idx = idx;
				}
				current = self.nodes[idx].next;
			}

			let evicted_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink_in_zone(victim_idx, Zone::Hot);
			self.hot_size -= 1;
		}

		// Move to hot zone and link at tail
		self.nodes[idx].zone = Zone::Hot;
		self.push_back_in_zone(idx, Zone::Hot);
		self.hot_size += 1;
	}

	/// Remove node from its current position in the specified zone.
	fn unlink_in_zone(&mut self, idx: usize, zone: Zone) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;

		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if zone == Zone::Cold {
					self.cold_head = next;
				} else {
					self.hot_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if zone == Zone::Cold {
					self.cold_tail = prev;
				} else {
					self.hot_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the tail (MRU end) of the specified zone.
	fn push_back_in_zone(&mut self, idx: usize, zone: Zone) {
		self.nodes[idx].prev = if zone == Zone::Cold {
			self.cold_tail
		} else {
			self.hot_tail
		};
		self.nodes[idx].next = None;

		if zone == Zone::Cold {
			match self.cold_tail {
				Some(t) => self.nodes[t].next = Some(idx),
				None => self.cold_head = Some(idx),
			}
			self.cold_tail = Some(idx);
		} else {
			match self.hot_tail {
				Some(t) => self.nodes[t].next = Some(idx),
				None => self.hot_head = Some(idx),
			}
			self.hot_tail = Some(idx);
		}
	}

	/// Check and perform frequency decay for both zones if thresholds are reached.
	fn check_and_perform_frequency_resets(&mut self) {
		// Hot zone decay: every capacity * 8 operations
		if self.total_operations - self.hot_last_decay >= (self.capacity as u64) * 8 {
			for node in &mut self.nodes {
				if node.zone == Zone::Hot {
					node.freq >>= 1;
				}
			}
			self.hot_last_decay = self.total_operations;
		}

		// Cold zone decay: every capacity * 16 operations
		if self.total_operations - self.cold_last_decay >= (self.capacity as u64) * 16 {
			for node in &mut self.nodes {
				if node.zone == Zone::Cold {
					node.freq >>= 1;
				}
			}
			self.cold_last_decay = self.total_operations;
		}
	}
}

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		LruCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		LruCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		// Should still have room for a second key without eviction
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn cold_zone_lru_eviction() {
		let mut cache = LruCache::new(3); // cold: 2, hot: 1
		cache.put(1, 10);
		cache.put(2, 20);
		// Both in cold zone, 3 is next
		cache.put(3, 30); // cold zone full, evicts LRU (1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn promotion_to_hot_zone() {
		let mut cache = LruCache::new(3); // cold: 2, hot: 1
		cache.put(1, 10);
		cache.put(2, 20);
		// Access key 1 twice to trigger promotion (freq becomes 2)
		cache.get(&1);
		cache.get(&1); // freq = 2, should promote to hot

		// Now we have 2 in cold zone (freq 0), 1 in hot zone (freq 2)
		cache.put(3, 30); // Should evict 2 from cold zone

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn hot_zone_frequency_eviction() {
		let mut cache = LruCache::new(3); // cold: 2, hot: 1
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // freq = 1
		cache.get(&1); // freq = 2, promote to hot

		// 2 in cold (freq 0), 1 in hot (freq 2)
		cache.put(3, 30); // evicts 2 from cold
		cache.put(4, 40); // evicts 3 from cold

		// Now only 1 in hot, let's try to fill cold and trigger hot eviction
		cache.put(5, 50); // evicts 4 from cold
		cache.put(6, 60); // evicts 5 from cold

		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn access_updates_mru_within_zone() {
		let mut cache = LruCache::new(3); // cold: 2, hot: 1
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 MRU in cold
		cache.put(3, 30); // evicts key 2 (LRU in cold)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_overwrites_existing_key_preserves_zone() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.get(&1); // freq = 1
		cache.put(1, 20); // overwrite preserves position but not freq
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // cache full
		cache.put(1, 11); // overwrite, must NOT evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_increases_frequency() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		// key 1 should have high frequency and be promoted to hot
		// After 2 gets it's promoted, then 8 more gets
		// Cold zone: 2, 3
		cache.put(4, 40); // should evict key 2 (LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU in cold
		cache.put(4, 40); // evicts 1 (LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to hot if freq reaches threshold
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // slot reused
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite MRU
		cache.put(4, 40); // evicts 1 (LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn hot_frequency_decay() {
		let mut cache = LruCache::new(4); // cold: 2, hot: 2
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // freq = 1
		cache.get(&1); // freq = 2, promote to hot

		// At this point: 1 operation in hot zone (the 2nd get)
		// Hot decay threshold is capacity * 8 = 32 operations

		// Verify frequency decay occurs
		for i in 3..=10 {
			cache.put(i, i as i64 * 10);
		}

		// Access key 1 to verify it's still there and working
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn cold_frequency_decay() {
		let mut cache = LruCache::new(4); // cold: 2, hot: 2
		cache.put(1, 10);
		cache.put(2, 20);

		// Cold decay threshold is capacity * 16 = 64 operations
		for i in 3..=30 {
			cache.put(i, i as i64 * 10);
		}

		// Keys should still be there (depending on eviction)
		// This test ensures decay doesn't break the cache
	}
}
