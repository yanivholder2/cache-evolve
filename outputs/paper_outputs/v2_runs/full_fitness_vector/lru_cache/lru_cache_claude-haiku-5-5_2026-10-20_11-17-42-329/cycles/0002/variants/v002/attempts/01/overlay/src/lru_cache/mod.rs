// O(1) LRU-approximate cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// Variant v002/H02: a hit no longer relinks the node. It sets a per-node `referenced`
// bit (CLOCK-style lazy promotion). The bit is folded into the top bit of `Node::prev`
// (no padded bool field). On a miss, the eviction scan from the list head gives at most
// MAX_REPRIEVES referenced nodes a second chance (unlink and push to the tail). If the
// head is still referenced after that, its bit is cleared and it is evicted. A miss
// therefore inspects at most MAX_REPRIEVES + 1 head nodes.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Sentinel for "no node" in the link fields. Its complement (the top bit) is reserved for `REFERENCED`.
const NIL: usize = usize::MAX >> 1;
/// Top bit of `Node::prev`: set when the node was hit since it was last rescued or inserted.
const REFERENCED: usize = !NIL;
/// Maximum number of referenced heads rescued (moved to the tail) during one eviction scan.
const MAX_REPRIEVES: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Low bits: previous node index, or NIL. Top bit: referenced flag.
	prev: usize,
	/// Next node index, or NIL.
	next: usize,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // eviction candidate (front)
	tail: Option<usize>,    // most recently inserted/rescued (back)
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
			head: None,
			tail: None,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its referenced bit (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.set_referenced(idx, true);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim chosen by the bounded second-chance scan.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and mark referenced
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.set_referenced(idx, true);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick a victim (bounded second chance) and reuse its slot for the new entry
			let victim = self.find_victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.set_referenced(victim, false);
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			idx
		};
		// Link the new/reused slot at the tail and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Bounded CLOCK second-chance scan from `head`. Each referenced head has its bit cleared and
	/// is moved to the tail, for at most `MAX_REPRIEVES` rescues. If the head is still referenced
	/// after those rescues, its bit is cleared and it is returned as the victim anyway. Thus at
	/// most `MAX_REPRIEVES + 1` head nodes are inspected per eviction. No `get`/`put` runs during the scan.
	fn find_victim(&mut self) -> usize {
		debug_assert!(self.head.is_some(), "head must be Some when arena is full");
		let mut reprieves = 0;
		loop {
			let head_idx = self.head.expect("non-empty list has a head");
			if !self.is_referenced(head_idx) {
				return head_idx;
			}
			self.set_referenced(head_idx, false);
			if reprieves == MAX_REPRIEVES {
				return head_idx;
			}
			self.unlink(head_idx);
			self.push_back(head_idx);
			reprieves += 1;
		}
	}

	#[inline]
	fn prev_of(&self, idx: usize) -> Option<usize> {
		let p = self.nodes[idx].prev & NIL;
		(p != NIL).then_some(p)
	}

	#[inline]
	fn next_of(&self, idx: usize) -> Option<usize> {
		let n = self.nodes[idx].next;
		(n != NIL).then_some(n)
	}

	/// Sets the prev link of `idx`, preserving its referenced bit.
	#[inline]
	fn set_prev(&mut self, idx: usize, prev: Option<usize>) {
		let flag = self.nodes[idx].prev & REFERENCED;
		self.nodes[idx].prev = flag | prev.unwrap_or(NIL);
	}

	#[inline]
	fn set_next(&mut self, idx: usize, next: Option<usize>) {
		self.nodes[idx].next = next.unwrap_or(NIL);
	}

	#[inline]
	fn is_referenced(&self, idx: usize) -> bool {
		self.nodes[idx].prev & REFERENCED != 0
	}

	#[inline]
	fn set_referenced(&mut self, idx: usize, on: bool) {
		if on {
			self.nodes[idx].prev |= REFERENCED;
		} else {
			self.nodes[idx].prev &= NIL;
		}
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.prev_of(idx);
		let next = self.next_of(idx);
		match prev {
			Some(p) => self.set_next(p, next),
			None => self.head = next,
		}
		match next {
			Some(n) => self.set_prev(n, prev),
			None => self.tail = prev,
		}
		self.set_prev(idx, None);
		self.set_next(idx, None);
	}

	/// Append node to the tail (back) of the list.
	fn push_back(&mut self, idx: usize) {
		let tail = self.tail;
		self.set_prev(idx, tail);
		self.set_next(idx, None);
		match tail {
			Some(t) => self.set_next(t, Some(idx)),
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
		cache.get(&1); // sets key 1 referenced: second chance at eviction
		cache.put(3, 30); // head (1) is referenced -> spared; evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 sets referenced bit
		cache.put(3, 30); // head (1) spared; evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn referenced_head_survives_one_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // referenced head
		cache.put(3, 30); // 1 gets second chance, unreferenced 2 is evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn bounded_scan_evicts_head_after_two_reprieves() {
		let mut cache = LruCache::new(4);
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		for k in 1..=4 {
			cache.get(&k); // all referenced
		}
		// Rescue 1 and 2 to the tail; head 3 is still referenced after two reprieves,
		// so its bit is cleared and it is evicted.
		cache.put(5, 50);
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn node_has_no_padded_flag() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 48);
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
		cache.put(2, 20); // referenced 1 spared once, then evicted
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
		cache.put(4, 40); // head 1 spared (referenced); evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // head, unreferenced
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // sets referenced bit only; list order unchanged
		cache.put(4, 40); // evicts 1 (unreferenced head)
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
		cache.get(&3); // slot 0 (key 3) referenced
		cache.put(4, 40); // head is key 2 (unreferenced): evicts 2 -> slot 1 reused for 4
		cache.put(5, 50); // head is key 3 (referenced): spared once; key 4 (unreferenced) evicted -> slot 1 reused for 5
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node: get sets its referenced bit
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
