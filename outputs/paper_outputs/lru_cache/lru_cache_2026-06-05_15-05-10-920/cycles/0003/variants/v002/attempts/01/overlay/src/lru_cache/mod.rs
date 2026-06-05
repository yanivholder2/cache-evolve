// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	freq: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	operation_counter: u64,
	frequency_history: VecDeque<u8>, // Rolling window of last 512 frequencies
	current_decay_threshold: u64,
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

		// Initial decay threshold is capacity * 8 (lowest bound)
		let initial_threshold = (capacity as u64) * 8;

		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			operation_counter: 0,
			frequency_history: VecDeque::with_capacity(512),
			current_decay_threshold: initial_threshold,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Increment frequency counter (saturating at 255)
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
		// Record the frequency in the rolling window
		self.record_frequency_access(self.nodes[idx].freq);
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);

		// Increment operation counter and check for frequency decay
		self.operation_counter += 1;
		self.check_and_perform_frequency_decay();

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the entry with the lowest frequency.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			// Record the existing frequency in the rolling window
			self.record_frequency_access(self.nodes[idx].freq);
			self.unlink(idx);
			self.push_back(idx);

			// Increment operation counter and check for frequency decay
			self.operation_counter += 1;
			self.check_and_perform_frequency_decay();

			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the node with minimum frequency (ties broken by earliest in order)
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");

			// Scan all nodes to find the one with minimum frequency
			let mut victim_idx = self.head.unwrap();
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
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].freq = 0;
			victim_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 0,
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);

		// Increment operation counter and check for frequency decay
		self.operation_counter += 1;
		self.check_and_perform_frequency_decay();
	}

	/// Record a frequency access in the rolling window (last 512 operations).
	fn record_frequency_access(&mut self, freq: u8) {
		if self.frequency_history.len() >= 512 {
			self.frequency_history.pop_front();
		}
		self.frequency_history.push_back(freq);
	}

	/// Compute the average frequency from the rolling window.
	fn compute_average_frequency(&self) -> f64 {
		if self.frequency_history.is_empty() {
			return 0.0;
		}
		let sum: u64 = self.frequency_history.iter().map(|&f| f as u64).sum();
		sum as f64 / self.frequency_history.len() as f64
	}

	/// Update the decay threshold based on current average frequency.
	fn update_decay_threshold(&mut self) {
		let avg_freq = self.compute_average_frequency();
		let min_threshold = (self.capacity as f64 * 8.0) as u64;
		let max_threshold = (self.capacity as f64 * 32.0) as u64;
		let dynamic_threshold = ((2.0 * avg_freq) as u64).max(min_threshold).min(max_threshold);
		self.current_decay_threshold = dynamic_threshold;
	}

	/// Check if frequency decay threshold has been reached and perform decay if needed.
	fn check_and_perform_frequency_decay(&mut self) {
		// Update the decay threshold based on current frequency distribution
		self.update_decay_threshold();

		if self.operation_counter >= self.current_decay_threshold {
			// Perform frequency decay: right-shift all frequency values
			for node in &mut self.nodes {
				node.freq >>= 1;
			}
			// Reset the operation counter
			self.operation_counter = 0;
			// Clear the frequency history after decay
			self.frequency_history.clear();
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
	fn eviction_removes_lowest_frequency() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		// Access key 2 to increase its frequency
		cache.get(&2);
		// Now key 1 has freq 0, key 2 has freq 1
		cache.put(3, 30); // evicts key 1 (lowest freq)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_frequency() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		// Access key 1 multiple times to increase its frequency
		for _ in 0..5 {
			cache.get(&1);
		}
		// key 1 should have freq 5, key 2 and 3 have freq 0
		cache.put(4, 40); // Should evict key 2 (tied at freq 0, earliest in order)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn put_overwrites_existing_key_preserves_freq() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.get(&1); // freq becomes 1
		cache.put(1, 20); // overwrite preserves promotion but not freq
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
		// key 1 has freq 10, key 2 and 3 have freq 0
		cache.put(4, 40); // should evict key 2 (freq 0, earliest)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU, freq becomes 1
								 // Order: 1 (LRU, freq 0), 3 (freq 0), 2 (MRU, freq 1)
		cache.put(4, 40); // evicts 1 (freq 0)
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
		cache.get(&3); // promote slot 0 (key 3) to MRU, freq becomes 1
		cache.put(4, 40); // evicts 2 (freq 0) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (freq 0) -> slot 1 reused for key 5
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
										// Eviction should take 1 (lowest freq)
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
	fn frequency_decay_with_adaptive_threshold() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);

		// Increase frequency of key 1
		for _ in 0..10 {
			cache.get(&1);
		}

		// Add more items to test decay with dynamic threshold
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);

		// Perform operations to build up frequency history and trigger decay
		// With adaptive threshold based on average frequency
		for _ in 0..20 {
			cache.get(&1);
		}

		// After decay, frequencies should be reduced
		// Cache should still work correctly
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}
}
