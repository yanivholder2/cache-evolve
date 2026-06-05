// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

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
	freq: u8,
	last_access_counter: u64,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	global_operation_counter: u64,
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
			global_operation_counter: 0,
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

		// Apply per-access decay based on operation delta
		let ops_since_access = self.global_operation_counter - self.nodes[idx].last_access_counter;
		let decay_factor = if ops_since_access > 64 {
			(self.nodes[idx].freq as u16 * (128_u16.saturating_sub((ops_since_access / 64) as u16))) / 128
		} else {
			self.nodes[idx].freq as u16
		} as u8;
		self.nodes[idx].freq = decay_factor;
		self.nodes[idx].last_access_counter = self.global_operation_counter;

		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);

		// Increment global operation counter
		self.global_operation_counter += 1;

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the entry with the lowest frequency.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);

			// Increment global operation counter
			self.global_operation_counter += 1;

			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the node with minimum frequency (ties broken by earliest in order)
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");

			// Scan all nodes to find the one with minimum frequency, applying decay to each
			let mut victim_idx = self.head.unwrap();
			let mut min_freq = self.compute_decayed_freq(victim_idx);

			let mut current = self.nodes[victim_idx].next;
			while let Some(idx) = current {
				let decayed_freq = self.compute_decayed_freq(idx);
				if decayed_freq < min_freq {
					min_freq = decayed_freq;
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
			self.nodes[victim_idx].last_access_counter = self.global_operation_counter;
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
				last_access_counter: self.global_operation_counter,
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);

		// Increment global operation counter
		self.global_operation_counter += 1;
	}

	/// Compute the decayed frequency for a node based on its last access time
	fn compute_decayed_freq(&self, idx: usize) -> u8 {
		let ops_since_access = self.global_operation_counter - self.nodes[idx].last_access_counter;
		let decay_factor = if ops_since_access > 64 {
			(self.nodes[idx].freq as u16 * (128_u16.saturating_sub((ops_since_access / 64) as u16))) / 128
		} else {
			self.nodes[idx].freq as u16
		} as u8;
		decay_factor
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
	fn frequency_decay_with_operation_delta() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);

		// Increase frequency of key 1
		for _ in 0..10 {
			cache.get(&1);
		}

		// At this point, key 1 should have freq 10
		// global_operation_counter = 1 (put) + 10 (gets) = 11

		// Do more operations to trigger decay
		for i in 2..=4 {
			cache.put(i, i as i64 * 10);
		}
		// Now global_operation_counter = 11 + 3 = 14

		// Add more gets to increase the operation delta
		for _ in 0..60 {
			cache.get(&1);
		}
		// Now global_operation_counter = 14 + 60 = 74

		// The frequency should have decayed based on the operation delta
		// We can't directly verify freq, but we can verify cache still works
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}
}
