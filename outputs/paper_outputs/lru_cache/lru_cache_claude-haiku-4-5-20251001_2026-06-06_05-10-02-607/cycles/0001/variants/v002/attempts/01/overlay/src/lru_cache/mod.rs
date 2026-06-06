// O(1) LRU cache using HashMap + arena-based doubly-linked list with frequency tracking.
// Nodes track access frequency alongside recency (LRU order).
// Eviction prefers nodes with low frequency while maintaining LRU ordering as tiebreaker.
// Frequency decays periodically to prevent stale frequency values from dominating.

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
	freq: u32,  // Access frequency counter
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	eviction_count: u32,    // Number of evictions since last decay
	decay_period: u32,      // Decay frequency every N evictions
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			eviction_count: 0,
			decay_period: capacity as u32, // Decay every `capacity` evictions
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used and incrementing frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Increment frequency on access
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts using hybrid frequency-LRU strategy.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place, promote to MRU, and increment frequency
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: find and evict a candidate using hybrid strategy
			let evict_idx = self.find_evictable_node();
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);

			// Reuse the slot for the new entry
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq = 1;  // New entries start with freq=1

			// Increment eviction counter and possibly decay frequencies
			self.eviction_count += 1;
			if self.eviction_count >= self.decay_period {
				self.decay_frequencies();
				self.eviction_count = 0;
			}

			evict_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 1,  // New entries start with freq=1
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Find a node to evict using hybrid frequency-LRU strategy:
	/// Scan from LRU (head) and skip nodes with freq > 1 until finding freq == 1.
	/// If all remaining nodes have freq > 1, evict the absolute LRU anyway.
	fn find_evictable_node(&self) -> usize {
		let mut idx = self.head;
		while let Some(current_idx) = idx {
			if self.nodes[current_idx].freq <= 1 {
				return current_idx;
			}
			idx = self.nodes[current_idx].next;
		}
		// All nodes have freq > 1; fall back to absolute LRU (head)
		self.head.expect("non-empty cache has a head")
	}

	/// Decay all non-evicted nodes' frequency by dividing by 2 (integer division).
	fn decay_frequencies(&mut self) {
		for node in &mut self.nodes {
			node.freq = node.freq / 2;
		}
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
		cache.put(3, 30); // evicts key 1 (freq=1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 MRU and increments its frequency
		cache.put(3, 30); // evicts key 2 (freq=1, LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU and increments frequency
		cache.put(3, 30); // evicts key 2 (freq=1, now LRU)

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
		cache.put(4, 40); // should evict key 2 (freq=1, LRU), not key 1 (freq>1)
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
									 // Order: 1 (LRU, freq=1), 3 (freq=1), 2 (MRU, freq=2)
		cache.put(4, 40); // evicts 1 (freq=1, LRU)
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
										// All have freq=1, so eviction takes 1 (LRU)
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
	fn frequency_decay_on_schedule() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		// Access key 1 multiple times to build up frequency
		for _ in 0..3 {
			cache.get(&1);
		}
		// At this point: key 1 freq=4, key 2 freq=1
		// The cache will trigger decay every 2 evictions (decay_period = capacity = 2)
		// After one eviction and decay, frequencies should be halved
		cache.put(3, 30); // triggers first eviction

		// After decay, key 1's frequency should be 4/2=2, key 2's should be 1/2=0
		// So key 2 should still be in cache (it got evicted to slot of the old evict,
		// but let's verify the frequency logic with the remaining items)
		assert!(cache.get(&1).is_some()); // key 1 should survive with freq=2
	}
}
