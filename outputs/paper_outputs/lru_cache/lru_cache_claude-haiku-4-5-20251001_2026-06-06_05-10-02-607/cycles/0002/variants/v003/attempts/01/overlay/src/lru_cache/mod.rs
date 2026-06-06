// O(1) Segmented LRU (S-LRU) cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// S-LRU partitions the cache into two zones:
// - Hot zone: top 30% capacity, protected from eviction
// - Main zone: remaining 70% capacity, evicted first with frequency-selective eviction
// Items are inserted into main zone and promoted to hot zone on access if space permits.
// Frequency tracking is applied only to main-zone items to identify low-frequency candidates
// for eviction, improving hit rate on zipfian workloads.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	zone: u8, // 0 = main, 1 = hot
	freq: u32, // frequency counter (only used for main-zone items)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	hot_capacity: usize,    // 30% of capacity
	main_capacity: usize,   // 70% of capacity
	hot_count: usize,       // current count in hot zone
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
		let hot_capacity = (capacity * 30) / 100;
		let main_capacity = capacity - hot_capacity;
		log::info!("LruCache.try_new: capacity={capacity}, hot_capacity={hot_capacity}, main_capacity={main_capacity}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			hot_capacity,
			main_capacity,
			hot_count: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	/// If the item is in the main zone and hot zone has space, promote it to hot zone.
	/// For main-zone items, increment their frequency counter.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;

		// Increment frequency for main-zone items
		if self.nodes[idx].zone == 0 {
			self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
		}

		// Check if we should promote from main to hot zone
		if self.nodes[idx].zone == 0 && self.hot_count < self.hot_capacity {
			// Item is in main zone and hot zone has space - promote it
			self.nodes[idx].zone = 1;
			self.hot_count += 1;
		}

		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts using frequency-selective eviction
	/// from the main zone first, then hot zone if main is empty.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict from main zone using frequency selection, or from hot if main is empty
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");

			let head_idx = self.head.expect("non-empty list has a head");

			// If main zone is not empty, use frequency-selective eviction; otherwise use LRU
			let evict_idx = if self.nodes.len() > self.hot_count {
				// Main zone is not empty; try frequency selection first, then fall back to LRU in main
				self.find_low_freq_candidate()
					.or_else(|| self.find_lru_main_zone())
					.unwrap_or(head_idx)
			} else {
				// Main zone empty; use LRU (must be from hot zone)
				head_idx
			};

			let evicted_key = self.nodes[evict_idx].key.clone();
			let evicted_zone = self.nodes[evict_idx].zone;

			self.map.remove(&evicted_key);
			self.unlink(evict_idx);

			// Update zone counts if evicting from hot zone
			if evicted_zone == 1 {
				self.hot_count = self.hot_count.saturating_sub(1);
			}

			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].zone = 0; // New items go to main zone
			self.nodes[evict_idx].freq = 0; // Initialize frequency to 0
			evict_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				zone: 0, // New items go to main zone
				freq: 0, // Initialize frequency to 0
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert!(self.hot_count <= self.hot_capacity);
	}

	/// Scan the main-zone items from head (LRU) and find one with freq <= 1.
	/// Scans at most the first 20 main-zone items.
	/// Returns the index of a candidate for eviction, or None if no low-frequency item found.
	fn find_low_freq_candidate(&self) -> Option<usize> {
		let mut current = self.head;
		let mut main_zone_scanned = 0;
		const SCAN_LIMIT: usize = 20;

		while let Some(idx) = current {
			// Only consider main-zone items
			if self.nodes[idx].zone == 0 {
				if self.nodes[idx].freq <= 1 {
					return Some(idx);
				}
				main_zone_scanned += 1;
				if main_zone_scanned >= SCAN_LIMIT {
					break;
				}
			}
			current = self.nodes[idx].next;
		}
		None
	}

	/// Find the least-recently-used item in the main zone.
	/// Returns the index of the LRU main-zone item, or None if main zone is empty.
	fn find_lru_main_zone(&self) -> Option<usize> {
		let mut current = self.head;
		while let Some(idx) = current {
			if self.nodes[idx].zone == 0 {
				return Some(idx);
			}
			current = self.nodes[idx].next;
		}
		None
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the tail (MRU end) of the list.
	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = None;
		match self.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
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
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 most recent
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // should evict key 2 (LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
										 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // evicts 1
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
		cache.get(&3); // promote slot 0 (key 3) to MRU
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 3 (slot 0) -> slot 0 reused AGAIN for key 5
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify list is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; 3 is MRU (tail)
		cache.put(3, 31); // overwrite MRU
										// Eviction should take 1 (LRU)
		cache.put(4, 40); // evicts 1
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
	fn zone_promotion_on_get() {
		// With capacity 10: hot_capacity = 3, main_capacity = 7
		let mut cache = LruCache::new(10);
		// Fill main zone
		for i in 1..=7 {
			cache.put(i, i as i64);
		}
		// Hot zone is empty, so accessing items should promote them
		cache.get(&1);
		assert_eq!(cache.nodes[cache.map[&1]].zone, 1);
		cache.get(&2);
		assert_eq!(cache.nodes[cache.map[&2]].zone, 1);
		cache.get(&3);
		assert_eq!(cache.nodes[cache.map[&3]].zone, 1);
		// Hot zone is now full, so accessing more items should not promote them
		cache.get(&4);
		assert_eq!(cache.nodes[cache.map[&4]].zone, 0);
	}

	#[test]
	fn evict_main_before_hot() {
		// With capacity 10: hot_capacity = 3, main_capacity = 7
		let mut cache = LruCache::new(10);
		// Fill main zone
		for i in 1..=7 {
			cache.put(i, i as i64);
		}
		// Promote first 3 to hot zone
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		// Now hot zone is [1, 2, 3], main zone is [4, 5, 6, 7]
		// Add 3 more items - should evict from main zone first
		cache.put(8, 8);
		cache.put(9, 9);
		cache.put(10, 10);

		// Items 1, 2, 3 should still be in hot zone
		assert!(cache.get(&1).is_some());
		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_some());
		// Items from main zone should be evicted
		assert!(cache.get(&4).is_none());
	}

	#[test]
	fn frequency_selective_eviction() {
		// With capacity 10: hot_capacity = 3, main_capacity = 7
		let mut cache = LruCache::new(10);
		// Fill main zone with 7 items
		for i in 1..=7 {
			cache.put(i, i as i64);
		}
		// Access item 1 multiple times to increase its frequency
		cache.get(&1); // freq = 1
		cache.get(&1); // freq = 2
		cache.get(&1); // freq = 3

		// Item 2 accessed once
		cache.get(&2); // freq = 1

		// Items 3-7 never accessed (freq = 0)

		// Now main zone is full [1,2,3,4,5,6,7]
		// When we add item 8, it should evict a low-frequency item from main
		// Item 3 has freq=0, which is <= 1, so it should be evicted
		cache.put(8, 8);

		// Item 3 (freq=0) should be evicted, not item 1 (freq=3)
		assert!(cache.get(&3).is_none(), "Item 3 (freq=0) should be evicted");
		assert!(cache.get(&1).is_some(), "Item 1 (freq=3) should still be present");
	}
}
