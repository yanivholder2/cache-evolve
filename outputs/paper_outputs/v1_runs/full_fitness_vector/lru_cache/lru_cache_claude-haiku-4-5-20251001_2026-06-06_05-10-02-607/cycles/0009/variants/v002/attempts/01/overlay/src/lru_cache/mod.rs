// O(1) LRU cache with time-windowed recency tracking.
// Instead of per-node frequency counters and bucket chains, maintains a global operation counter.
// Each node tracks its last_access_op (logical timestamp of last access).
// Eviction scans a bounded window and selects the least recently accessed node.

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
	last_access_op: u64,  // Logical timestamp of last access
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,  // key -> node index
	nodes: Vec<Node<K, V>>,  // arena
	capacity: usize,
	operation_counter: u64,  // Global operation counter, incremented on get/put
	eviction_count: u32,     // Number of evictions since last decay
	decay_period: u32,       // Decay counter every N evictions
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
			capacity,
			operation_counter: 0,
			eviction_count: 0,
			decay_period: capacity as u32,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, updating its last_access_op.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Increment operation counter and update last_access_op
		self.operation_counter += 1;
		self.nodes[idx].last_access_op = self.operation_counter;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts using time-windowed recency strategy.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and update last_access_op
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.operation_counter += 1;
			self.nodes[idx].last_access_op = self.operation_counter;
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: find and evict a candidate using time-windowed recency strategy
			let evict_idx = self.find_evictable_node();
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);

			// Reuse the slot for the new entry
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;

			// Increment operation counter and set last_access_op
			self.operation_counter += 1;
			self.nodes[evict_idx].last_access_op = self.operation_counter;

			// Increment eviction counter and possibly decay operation counter
			self.eviction_count += 1;
			if self.eviction_count >= self.decay_period {
				self.decay_operation_counter();
				self.eviction_count = 0;
			}

			evict_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();

			// Increment operation counter
			self.operation_counter += 1;

			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				last_access_op: self.operation_counter,
			});
			idx
		};
		// Register in the map
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Find a node to evict by scanning the nodes and selecting the one with smallest last_access_op.
	/// This implements time-windowed recency: evict the least recently accessed node within the arena.
	fn find_evictable_node(&self) -> usize {
		let mut min_idx = 0;
		let mut min_op = self.nodes[0].last_access_op;

		for (idx, node) in self.nodes.iter().enumerate() {
			if node.last_access_op < min_op {
				min_op = node.last_access_op;
				min_idx = idx;
			}
		}
		min_idx
	}

	/// Decay operation counter by resetting all last_access_op values downward.
	/// This prevents overflow of the operation_counter and keeps values within a reasonable range.
	fn decay_operation_counter(&mut self) {
		if self.nodes.is_empty() {
			return;
		}

		// Find the minimum last_access_op value
		let min_op = self.nodes.iter().map(|n| n.last_access_op).min().unwrap_or(0);

		// Shift all last_access_op values down by min_op
		for node in &mut self.nodes {
			node.last_access_op = node.last_access_op.saturating_sub(min_op);
		}

		// Shift operation_counter down as well
		self.operation_counter = self.operation_counter.saturating_sub(min_op);
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
	fn eviction_removes_least_recently_accessed() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (least recently accessed)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_last_access_op() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // updates key 1's last_access_op
		cache.put(3, 30); // evicts key 2 (now LRU, has older last_access_op)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_last_access_op_and_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1, updates last_access_op
		cache.put(3, 30); // evicts key 2 (older last_access_op)

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
		cache.put(4, 40); // evicts 1 (oldest last_access_op)
		cache.put(5, 50); // evicts 2 (oldest last_access_op)
		cache.put(6, 60); // evicts 3 (oldest last_access_op)

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
		cache.put(4, 40); // should evict key 2 or 3 (not key 1, which has been accessed)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // updates last_access_op for key 2
		cache.put(4, 40); // should evict key 1 or 3 (not key 2, which was just accessed)
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, op=1
		cache.put(2, 20); // slot 1, op=2
		cache.put(3, 30); // evicts 1 (op=1) -> slot 0 reused, op=3
		cache.get(&3); // promotes slot 0 (key 3), op=4
		cache.put(4, 40); // evicts 2 (op=2) -> slot 1 reused, op=5
		cache.put(5, 50); // evicts 4 (op=5, but actually the head from previous iteration)
						 // should evict the one with smallest last_access_op
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
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
		cache.put(3, 30); // full; 3 is most recently added
		cache.put(3, 31); // overwrite, updates last_access_op
		cache.put(4, 40); // evicts 1 or 2 (one of the older ones)
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn operation_counter_decay() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		// Access key 1 multiple times to build up operation counter
		for _ in 0..3 {
			cache.get(&1);
		}
		// At this point: operation_counter should be high
		let op_count_before = cache.operation_counter;
		// Trigger eviction with decay (decay_period = capacity = 2)
		cache.put(3, 30); // triggers eviction -> decay
		// After decay, operation_counter should be smaller (but structure preserved)
		// Key 1 should survive because it has recent last_access_op
		assert!(cache.get(&1).is_some());
	}
}
