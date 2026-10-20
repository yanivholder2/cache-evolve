// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// Link fields use u32 with !0u32 as the null sentinel rather than Option<usize>,
// reducing Node<u64,u64> metadata from 48 → 24 bytes.
//
// v003 changes over v002:
//  1. put() uses the entry API to fold the existence-check and map insert into one hash probe
//     for new keys when the arena is not full (Vacant branch with e.insert()), eliminating the
//     double-probe of get()+insert() on the warmup / not-full miss path.
//     When the arena is full and eviction is required, the VacantEntry is released via
//     into_key() before map.remove() is called (borrow-checker constraint), then map.insert()
//     completes the insertion — same cost as v002 on that path.
//  2. unlink() and push_back() are annotated #[inline(always)] so the compiler always inlines
//     these small hot helpers, removing call overhead and enabling better register allocation.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Null sentinel for u32 link fields: u32::MAX.
const NULL: u32 = !0u32;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: u32,              // LRU (front); NULL when list is empty
	tail: u32,              // MRU (back); NULL when list is empty
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
		if capacity > (u32::MAX as usize - 1) {
			log::error!("LruCache.try_new: capacity exceeds u32 sentinel range");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: NULL,
			tail: NULL,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used entry.
	///
	/// Uses the entry API for a single hash probe:
	/// - Occupied (hit): index is read, entry is dropped by NLL, node updated and promoted.
	/// - Vacant, arena not full: node pushed inline, push_back inlined (avoids &mut self
	///   conflict with the live VacantEntry), then e.insert() completes in one probe.
	/// - Vacant, arena full: VacantEntry is released via into_key() so map.remove() is legal,
	///   then eviction + map.insert() proceeds normally (same cost as v002 on this path).
	pub fn put(&mut self, key: K, value: V) {
		match self.map.entry(key) {
			Entry::Occupied(e) => {
				// Hit path: single hash probe.
				// NLL ends e's borrow of self.map after *e.get(), freeing &mut self.
				let idx = *e.get();
				self.nodes[idx].value = value;
				self.unlink(idx);
				self.push_back(idx);
			}
			Entry::Vacant(e) => {
				if self.nodes.len() < self.capacity {
					// Miss, arena not full: single hash probe (Vacant → e.insert).
					// push_back is inlined here because calling self.push_back() would take
					// &mut self, which conflicts with e's borrow of self.map.
					// Direct field accesses to self.nodes / self.head / self.tail are fine
					// since they are disjoint from self.map.
					let idx = self.nodes.len();
					self.nodes.push(Node {
						key: e.key().clone(),
						value,
						prev: NULL,
						next: NULL,
					});
					// --- inline push_back(idx) ---
					let idx32 = idx as u32;
					self.nodes[idx].prev = self.tail;
					self.nodes[idx].next = NULL;
					if self.tail != NULL {
						self.nodes[self.tail as usize].next = idx32;
					} else {
						self.head = idx32;
					}
					self.tail = idx32;
					// --- end inline push_back ---
					e.insert(idx);
				} else {
					// Miss, arena full: eviction required.
					// map.remove() cannot be called while the VacantEntry borrows self.map,
					// so release the entry first via into_key(), then evict and re-insert.
					let key = e.into_key();
					debug_assert!(self.head != NULL, "head must be valid when arena is full");
					let head_idx = self.head as usize;
					let evicted_key = self.nodes[head_idx].key.clone();
					self.map.remove(&evicted_key);
					self.unlink(head_idx);
					self.nodes[head_idx].key = key.clone();
					self.nodes[head_idx].value = value;
					self.push_back(head_idx);
					self.map.insert(key, head_idx);
				}
			}
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Remove node from its current position in the list.
	#[inline(always)]
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev != NULL { self.nodes[prev as usize].next = next; } else { self.head = next; }
		if next != NULL { self.nodes[next as usize].prev = prev; } else { self.tail = prev; }
		self.nodes[idx].prev = NULL;
		self.nodes[idx].next = NULL;
	}

	/// Append node to the tail (MRU end) of the list.
	#[inline(always)]
	fn push_back(&mut self, idx: usize) {
		let idx32 = idx as u32;
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = NULL;
		if self.tail != NULL { self.nodes[self.tail as usize].next = idx32; } else { self.head = idx32; }
		self.tail = idx32;
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
}
