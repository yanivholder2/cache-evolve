// O(1) cache using HashMap + arena-based doubly-linked list with frequency tracking.
// Uses threshold-based frequency amnesia for eviction decisions.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	freq: u8, // 3-bit quantized frequency bucket (0-7)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	operation_count: u64,   // Track operations for periodic decay
	decay_interval: u64,    // Decay every capacity * 16 operations
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
			operation_count: 0,
			decay_interval: (capacity as u64) * 16,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it and incrementing frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Increment frequency on hit
		if let Some(&idx) = self.map.get(key) {
			// Increment frequency, capped at 7 (3-bit max)
			if self.nodes[idx].freq < 7 {
				self.nodes[idx].freq += 1;
			}
			// Promote to MRU: detach from current position, re-attach at tail
			self.unlink(idx);
			self.push_back(idx);
			// Check for periodic decay
			self.operation_count += 1;
			if self.operation_count >= self.decay_interval {
				self.apply_threshold_amnesia();
				self.operation_count = 0;
			}
			return Some(&self.nodes[idx].value);
		}
		None
	}

	/// Inserts `key`. If at capacity, evicts using two-pass frequency-based selection.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		// Frequency is NOT incremented on put (only on get)
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict using two-pass frequency-based selection
			let victim_idx = self.select_victim();
			let evicted_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].freq = 0; // New item starts with freq 0
			victim_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 0, // New items start with freq 0
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);

		// Check for periodic decay after each put
		self.operation_count += 1;
		if self.operation_count >= self.decay_interval {
			self.apply_threshold_amnesia();
			self.operation_count = 0;
		}
	}

	/// Two-pass victim selection:
	/// 1. Find minimum frequency among all cached items
	/// 2. Among items with min frequency, select the LRU one
	fn select_victim(&self) -> usize {
		debug_assert!(!self.nodes.is_empty(), "cannot select victim from empty cache");

		// Pass 1: Find minimum frequency
		let min_freq = self.nodes.iter().map(|n| n.freq).min().unwrap_or(0);

		// Pass 2: Among items with min frequency, return the LRU (head)
		// Walk from head to find the first node with min frequency
		let mut current = self.head;
		while let Some(idx) = current {
			if self.nodes[idx].freq == min_freq {
				return idx;
			}
			current = self.nodes[idx].next;
		}

		// Fallback: if no node found with min frequency, return head (shouldn't happen)
		self.head.unwrap_or(0)
	}

	/// Threshold-based amnesia: reset frequencies < 2 to 0, keep freq >= 2 unchanged
	fn apply_threshold_amnesia(&mut self) {
		for node in &mut self.nodes {
			if node.freq < 2 {
				node.freq = 0;
			}
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
	fn frequency_tracking() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);

		// Access 1 multiple times to increase its frequency
		cache.get(&1);
		cache.get(&1);
		cache.get(&1);

		// Access 2 once
		cache.get(&2);

		// Now insert 4, which should evict 3 (freq=0) or 2 (freq=1), not 1 (freq=3)
		cache.put(4, 40);

		// 1 should still be there due to high frequency
		assert_eq!(*cache.get(&1).unwrap(), 10);
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
										// Eviction should take 1 (LRU with lowest freq)
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
}
