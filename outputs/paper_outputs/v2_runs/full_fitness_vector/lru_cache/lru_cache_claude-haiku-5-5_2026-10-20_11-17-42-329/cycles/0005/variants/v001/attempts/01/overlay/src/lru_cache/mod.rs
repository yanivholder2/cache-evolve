// Segmented LRU (SLRU) cache with lazy reference bits, using HashMap + arena-based intrusive lists.
// Nodes are stored in a Vec (arena) indexed by usize. The arena is partitioned into
// two intrusive lists: a probationary segment (keys seen once) and a protected
// segment (keys promoted by a second-chance rescue). Hits only set a reference bit; no list
// pointers are touched on get or on update. On a miss at capacity, the probationary head is
// given one second chance if its reference bit is set. When a protected demotion meets a
// protected head that is referenced, or still holds spare credit, that head is spared in place
// and the next protected node is demoted instead. A referenced head is spared and given two
// spare credits; each later spare consumes one credit, and a head with no credit left is
// demoted. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Sentinel for "no node" in the arena-linked lists.
const NIL: usize = usize::MAX;
/// Node flag: resident in the protected segment (otherwise probationary).
const PROTECTED: u8 = 0b01;
/// Node flag: referenced since it was last inserted, rescued, or demoted.
const REFERENCED: u8 = 0b10;
/// Shift of the 2-bit spare-credit field within `Node.flags` (bits 2-3).
const CREDIT_SHIFT: u8 = 2;
/// Mask of the 2-bit spare-credit field within `Node.flags`.
const CREDIT_MASK: u8 = 0b1100;
/// Spare credit granted to a protected head that is spared because it was referenced.
const SPARE_CREDIT: u8 = 2;

/// Returns the spare credit stored in `flags`.
fn spare_credit(flags: u8) -> u8 {
	(flags & CREDIT_MASK) >> CREDIT_SHIFT
}

/// Returns `flags` with its spare-credit field set to `credit` (must be at most 3).
fn with_spare_credit(flags: u8, credit: u8) -> u8 {
	(flags & !CREDIT_MASK) | (credit << CREDIT_SHIFT)
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	flags: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	prob_head: usize,       // probationary LRU (front, eviction candidate)
	prob_tail: usize,       // probationary MRU (back)
	prot_head: usize,       // protected LRU (front, demotion candidate)
	prot_tail: usize,       // protected MRU (back)
	prot_len: usize,        // number of nodes in the protected segment
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
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prot_len: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its reference bit (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.nodes[idx].flags |= REFERENCED;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a probationary entry (or the
	/// protected LRU when no probationary entry exists).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and set its reference bit
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.flags |= REFERENCED;
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: choose a victim and reuse its slot
			let victim = self.select_victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let node = &mut self.nodes[victim];
			node.key = key.clone();
			node.value = value;
			node.flags = 0;
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				flags: 0,
			});
			idx
		};
		// New entries start probationary, unreferenced, at the MRU end; register in the map
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Maximum number of nodes the protected segment may hold.
	fn protected_cap(&self) -> usize {
		(self.capacity * 4 / 5).max(1)
	}

	/// Picks the arena slot to reuse on a miss at capacity. The probationary head gets at most
	/// one second chance: if its reference bit is set, it moves to the protected tail (first
	/// demoting the protected head if that segment is full). After that, the current
	/// probationary head is evicted regardless of its bit. Falls back to the protected head
	/// when no probationary node exists. O(1) per miss.
	fn select_victim(&mut self) -> usize {
		if self.prob_head == NIL {
			return self.prot_head;
		}
		let head = self.prob_head;
		if self.nodes[head].flags & REFERENCED != 0 {
			self.unlink(head);
			// Rescue clears the reference bit and the spare credit
			self.nodes[head].flags &= !(REFERENCED | CREDIT_MASK);
			if self.prot_len >= self.protected_cap() {
				self.demote_prot_head();
			}
			self.push_back(head, true);
			debug_assert!(self.prot_len <= self.protected_cap());
		}
		if self.prob_head != NIL {
			self.prob_head
		} else {
			self.prot_head
		}
	}

	/// Demotes one protected node to the probationary MRU end. The protected head is spared
	/// (and the next protected node demoted instead) when it is referenced, which clears its
	/// reference bit and grants it `SPARE_CREDIT` spare credits, or when it still holds spare
	/// credit, which consumes one credit. A head with no credit and no reference is demoted.
	/// If the head is the only protected node there is no skip target, so it is always demoted.
	/// There is at most one skip per call, and `prot_len` drops by exactly one.
	fn demote_prot_head(&mut self) {
		let head = self.prot_head;
		let next = self.nodes[head].next;
		let mut victim = head;
		if next != NIL {
			if self.nodes[head].flags & REFERENCED != 0 {
				self.nodes[head].flags &= !REFERENCED;
				self.nodes[head].flags = with_spare_credit(self.nodes[head].flags, SPARE_CREDIT);
				victim = next;
			} else {
				let credit = spare_credit(self.nodes[head].flags);
				if credit > 0 {
					self.nodes[head].flags = with_spare_credit(self.nodes[head].flags, credit - 1);
					victim = next;
				}
			}
		}
		self.unlink(victim);
		// Demotion clears the reference bit and the spare credit
		self.nodes[victim].flags &= !(REFERENCED | CREDIT_MASK);
		self.push_back(victim, false);
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].flags & PROTECTED != 0;
		if prev != NIL {
			self.nodes[prev].next = next;
		} else if protected {
			self.prot_head = next;
		} else {
			self.prob_head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else if protected {
			self.prot_tail = prev;
		} else {
			self.prob_tail = prev;
		}
		if protected {
			self.prot_len -= 1;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU (tail) end of the protected or probationary list.
	fn push_back(&mut self, idx: usize, protected: bool) {
		let node = &mut self.nodes[idx];
		node.flags = (node.flags & !PROTECTED) | if protected { PROTECTED } else { 0 };
		node.next = NIL;
		if protected {
			let tail = self.prot_tail;
			node.prev = tail;
			if tail != NIL {
				self.nodes[tail].next = idx;
			} else {
				self.prot_head = idx;
			}
			self.prot_tail = idx;
			self.prot_len += 1;
		} else {
			let tail = self.prob_tail;
			node.prev = tail;
			if tail != NIL {
				self.nodes[tail].next = idx;
			} else {
				self.prob_head = idx;
			}
			self.prob_tail = idx;
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
		cache.get(&1); // references key 1: second chance at eviction
		cache.put(3, 30); // rescues 1, evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 sets its reference bit
		cache.put(3, 30); // rescues 1, evicts key 2

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
		cache.put(4, 40); // rescues key 1 once, evicts key 2 (probationary LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // reference middle
		// Probationary: 1 (LRU), 2 (referenced), 3
		cache.put(4, 40); // evicts 1 (unreferenced head)
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
		cache.get(&3); // reference slot 0 (key 3)
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // rescues 3 to protected, evicts 4 (slot 1) -> slot 1 reused for key 5
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survives
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node; get only sets its reference bit
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
		// Eviction should take 1 (LRU, unreferenced)
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
		cache.get(&100); // second reference: rescued to protected on first eviction pass
		// Scan of one-hit keys far larger than capacity
		for k in 0..20 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&100).unwrap(), 1000);
	}

	#[test]
	fn spared_protected_head_survives_two_credit_passes() {
		let mut cache = LruCache::new(10);
		for k in 1..=5 {
			cache.put(k, k * 10);
		}
		// Move all five nodes to the protected list in order 0..5; node 0 is the protected head.
		for idx in 0..5 {
			cache.unlink(idx);
			cache.push_back(idx, true);
		}
		assert_eq!(cache.prot_len, 5);
		cache.nodes[0].flags |= REFERENCED;

		// Pass 1: the referenced head is spared with two credits; node 1 is demoted.
		cache.demote_prot_head();
		assert_eq!(cache.prot_head, 0);
		assert_eq!(spare_credit(cache.nodes[0].flags), 2);
		assert_eq!(cache.prot_len, 4);

		// Pass 2: no reference, so one credit is spent and node 2 is demoted.
		cache.demote_prot_head();
		assert_eq!(cache.prot_head, 0);
		assert_eq!(spare_credit(cache.nodes[0].flags), 1);
		assert_eq!(cache.prot_len, 3);

		// Pass 3: the second credit is spent and node 3 is demoted.
		cache.demote_prot_head();
		assert_eq!(cache.prot_head, 0);
		assert_eq!(spare_credit(cache.nodes[0].flags), 0);
		assert_eq!(cache.prot_len, 2);

		// Pass 4: no credit and no reference left, so the head itself is demoted.
		cache.demote_prot_head();
		assert_eq!(cache.prot_head, 4);
		assert_eq!(cache.nodes[0].flags & PROTECTED, 0);
		assert_eq!(cache.nodes[0].flags & REFERENCED, 0);
		assert_eq!(spare_credit(cache.nodes[0].flags), 0);
		assert_eq!(cache.prot_len, 1);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
