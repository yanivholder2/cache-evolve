// Segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Nodes are stored in a Vec (arena) indexed by usize. The arena is partitioned into
// two intrusive lists: a probationary segment (keys seen once) and a protected
// segment (keys re-referenced at least once). Eviction takes the probationary head
// first, so one-hit keys cannot displace re-referenced keys. On eviction the slot is
// reused in-place, so no free-list management is needed. No unsafe, no external crates.

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

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	prob_head: Option<usize>, // probationary LRU (front, eviction candidate)
	prob_tail: Option<usize>, // probationary MRU (back)
	prot_head: Option<usize>, // protected LRU (front, demotion candidate)
	prot_tail: Option<usize>, // protected MRU (back)
	prot_len: usize,          // number of nodes in the protected segment
	capacity: usize,
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
			prob_head: None,
			prob_tail: None,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to the protected segment.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		self.promote(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a probationary entry (or the
	/// protected LRU when no probationary entry exists).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a hit
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the probationary head, else the protected head, and reuse its slot
			let victim = self
				.prob_head
				.or(self.prot_head)
				.expect("full arena has an eviction victim");
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			victim
		} else {
			// Arena has room: allocate a new slot
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
		// New entries start probationary at the MRU end, and are registered in the map
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Maximum number of nodes the protected segment may hold.
	fn protected_cap(&self) -> usize {
		(self.capacity * 4 / 5).max(1)
	}

	/// Record a reference to `idx`: move it to the protected MRU end, demoting the
	/// protected LRU to the probationary MRU end if the protected segment overflows.
	fn promote(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back(idx, true);
		if self.prot_len > self.protected_cap() {
			let demoted = self
				.prot_head
				.expect("over-full protected segment has a head");
			self.unlink(demoted);
			self.push_back(demoted, false);
		}
		debug_assert!(self.prot_len <= self.protected_cap());
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].in_protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None if protected => self.prot_head = next,
			None => self.prob_head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None if protected => self.prot_tail = prev,
			None => self.prob_tail = prev,
		}
		if protected {
			self.prot_len -= 1;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU (tail) end of the protected or probationary list.
	fn push_back(&mut self, idx: usize, protected: bool) {
		self.nodes[idx].in_protected = protected;
		self.nodes[idx].next = None;
		if protected {
			self.nodes[idx].prev = self.prot_tail;
			match self.prot_tail {
				Some(t) => self.nodes[t].next = Some(idx),
				None => self.prot_head = Some(idx),
			}
			self.prot_tail = Some(idx);
			self.prot_len += 1;
		} else {
			self.nodes[idx].prev = self.prob_tail;
			match self.prob_tail {
				Some(t) => self.nodes[t].next = Some(idx),
				None => self.prob_head = Some(idx),
			}
			self.prob_tail = Some(idx);
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
		cache.put(4, 40); // should evict key 2 (LRU probationary), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // promote middle to protected
									 // Probationary: 1 (LRU), 3
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0 (probationary)
		cache.put(2, 20); // slot 1 (probationary)
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to protected
		cache.put(4, 40); // evicts 2 (slot 1, the only probationary) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (slot 1, probationary) -> slot 1 reused AGAIN for key 5
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survives
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node; get moves it to the protected segment
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
	fn one_hit_scan_does_not_evict_twice_read_key() {
		let mut cache = LruCache::new(4);
		cache.put(100, 1000);
		cache.get(&100); // second reference: promotes 100 to protected
		// Scan of one-hit keys far larger than capacity
		for k in 0..20 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&100).unwrap(), 1000);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
