// Segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Two segments share one arena: a probation segment (newly inserted entries)
// and a protected segment (entries hit at least once). Eviction takes the LRU
// entry of probation when it is non-empty, otherwise the LRU entry of protected.
// Protected is capped at 80% of capacity; overflow demotes its LRU entry to the
// MRU end of probation. Nodes are stored in a Vec (arena) indexed by usize.
// No unsafe, no external crates.

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
	in_protected: bool,
}

#[derive(Debug, Clone, Copy)]
struct Seg {
	head: Option<usize>, // LRU (front)
	tail: Option<usize>, // MRU (back)
	len: usize,
}

const EMPTY_SEG: Seg = Seg {
	head: None,
	tail: None,
	len: 0,
};

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	probation: Seg,
	protected: Seg,
	capacity: usize,
	protected_cap: usize,
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
			probation: EMPTY_SEG,
			protected: EMPTY_SEG,
			capacity,
			protected_cap: capacity.saturating_mul(4) / 5,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it (probation -> protected).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts from probation (else protected).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a hit
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			let victim = self.victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				in_protected: false,
			});
			idx
		};
		// New entries enter probation at the MRU end
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Index of the eviction victim: LRU of probation, else LRU of protected.
	fn victim(&self) -> usize {
		self.probation
			.head
			.or(self.protected.head)
			.expect("full cache has a victim")
	}

	/// Record a hit on `idx`. Probation hits move to protected; protected hits
	/// move to protected MRU. Protected overflow demotes its LRU to probation MRU.
	fn touch(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back(idx, true);
		while self.protected.len > self.protected_cap {
			let Some(h) = self.protected.head else { break };
			self.unlink(h);
			self.push_back(h, false);
		}
	}

	/// Remove node from its current segment.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let seg = if self.nodes[idx].in_protected {
			&mut self.protected
		} else {
			&mut self.probation
		};
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => seg.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => seg.tail = prev,
		}
		seg.len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of the given segment.
	fn push_back(&mut self, idx: usize, in_protected: bool) {
		let seg = if in_protected {
			&mut self.protected
		} else {
			&mut self.probation
		};
		let tail = seg.tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		self.nodes[idx].in_protected = in_protected;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => seg.head = Some(idx),
		}
		seg.tail = Some(idx);
		seg.len += 1;
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
		cache.put(3, 30); // evicts key 1 (probation LRU)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promotes key 1 to protected
		cache.put(3, 30); // evicts key 2 (probation LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 promotes it to protected
		cache.put(3, 30); // evicts key 2 (probation LRU)

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
		cache.put(4, 40); // should evict key 2 (probation LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probation LRU
		cache.put(2, 20); // probation middle
		cache.put(3, 30); // probation MRU
		cache.get(&2); // promote 2 to protected
		cache.put(4, 40); // evicts 1 (probation LRU)
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
		cache.get(&3); // promote slot 0 (key 3) to protected
		cache.put(4, 40); // evicts 2 (slot 1, probation) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (slot 1, probation) -> slot 1 reused AGAIN for key 5
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node: get promotes it to protected
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
		cache.put(3, 30); // full; 3 is probation MRU
		cache.put(3, 31); // overwrite: promotes 3 to protected
		// Eviction should take 1 (probation LRU)
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
