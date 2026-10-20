// Segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked list,
// extended with a bounded *ghost* list for fast post-eviction re-admission.
//
// The single LRU list of the baseline is split into two segments that share one
// arena: a *probationary* segment (entries seen once) and a *protected* segment
// (entries that earned a second access). Frequently-reused keys are promoted out
// of the probationary segment and so resist eviction, which is the canonical
// first improvement of SLRU over plain LRU on reuse-heavy traces.
//
// On top of that 95/5 SLRU skeleton, this variant tracks the keys most recently
// evicted from the *probationary* segment in a key-only ghost list (its own
// arena-backed LRU, capped at `capacity`, carrying no values). When an incoming
// key is found in the ghost list it has demonstrated genuine reuse — it was
// evicted and re-requested soon after — so it is admitted directly at the
// protected segment's MRU (warmest) end, skipping the probationary "prove
// yourself" lap and getting maximum shielding from recency churn.
//
// Capacity is partitioned so the two resident segment budgets sum to exactly
// `capacity`, leaving the P1 capacity bound identical to the baseline. Ghosts
// hold no value and never appear in `map`, so the resident count and every
// structural invariant are unchanged. Eviction slots are still reused in place,
// so there is no free-list management for residents. No unsafe, no external
// crates.

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
	// `true` when the node currently lives in the protected segment.
	protected: bool,
}

// A ghost entry remembers only a key recently evicted from the probationary
// segment; it carries no value and is never a resident.
#[derive(Debug)]
struct GhostNode<K> {
	key: K,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // shared arena

	// Probationary segment: entries that have been seen exactly once.
	prob_head: Option<usize>, // LRU (front)
	prob_tail: Option<usize>, // MRU (back)
	prob_len: usize,

	// Protected segment: entries that earned a second access.
	prot_head: Option<usize>, // LRU (front)
	prot_tail: Option<usize>, // MRU (back)
	prot_len: usize,

	capacity: usize,
	// Size budget for the protected segment (~95% of capacity, floored). The
	// probationary budget is the remainder, so the two sum to exactly
	// `capacity`. For capacity == 1 this floors to 0, leaving a single usable
	// (probationary) slot — the only case where a segment budget is 0.
	protected_cap: usize,

	// Ghost list: keys recently evicted from the probationary segment, in their
	// own arena-backed LRU. Bounded by `ghost_cap` entries. Ghosts hold no
	// value and never appear in `map`, so they do not affect the resident count
	// or any capacity bound.
	ghost_map: HashMap<K, usize>,
	ghost_nodes: Vec<GhostNode<K>>,
	ghost_head: Option<usize>, // LRU (front)
	ghost_tail: Option<usize>, // MRU (back)
	ghost_len: usize,
	ghost_free: Vec<usize>, // ghost arena slots freed by re-admission
	ghost_cap: usize,
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
		// floor(0.95 * capacity); probationary budget is capacity - protected_cap,
		// so the two segment budgets sum to exactly `capacity`.
		let protected_cap = (capacity * 19) / 20;
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} probationary_cap={} ghost_cap={capacity}",
			capacity - protected_cap
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			capacity,
			protected_cap,
			ghost_map: HashMap::with_capacity(capacity),
			ghost_nodes: Vec::with_capacity(capacity),
			ghost_head: None,
			ghost_tail: None,
			ghost_len: 0,
			ghost_free: Vec::new(),
			ghost_cap: capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to the protected
	/// segment (most-recently-used end).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss.
		let &idx = self.map.get(key)?;
		// A hit earns protection: detach from its current segment and re-attach
		// at the MRU end of the protected segment.
		self.unlink(idx);
		self.push_back_protected(idx);
		// Keep the protected segment within its budget by demoting its LRU node
		// back to the probationary segment. Net resident count is unchanged.
		self.enforce_protected_budget();
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used
	/// entry (probationary first, then protected).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access,
		// promoting it to the protected segment.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back_protected(idx);
			self.enforce_protected_budget();
			return;
		}
		// Was this key recently evicted from the probationary segment? If so it
		// has proven genuine reuse and earns direct admission to the protected
		// segment's MRU end. Drop it from the ghost list either way.
		let readmit = self.ghost_remove(&key);
		// New key — need an arena slot.
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the LRU of the probationary segment first; only
			// if it is empty fall back to the protected segment's LRU. Reuse the
			// evicted slot in place for the new entry.
			let evict_idx = match self.prob_head {
				Some(head) => head,
				None => self
					.prot_head
					.expect("a full cache always has a victim in some segment"),
			};
			let evicted_key = self.nodes[evict_idx].key.clone();
			let evicted_protected = self.nodes[evict_idx].protected;
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			// Remember probationary evictions as ghosts so a fast re-request can
			// be re-admitted warm. Protected evictions are not ghosted: they
			// already had their second-chance tenure.
			if !evicted_protected {
				self.ghost_insert(evicted_key);
			}
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			evict_idx
		} else {
			// Arena has room: allocate a new slot.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				protected: false,
			});
			idx
		};
		if readmit {
			// Re-admitted ghost: skip probationary, enter protected at the MRU
			// (warmest) end, then restore the protected budget if it overflows.
			self.push_back_protected(idx);
			self.enforce_protected_budget();
		} else {
			// New entries enter at the MRU end of the probationary segment.
			self.push_back_probationary(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
	}

	/// Remove a node from whichever segment it currently occupies.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if protected {
					self.prot_head = next;
				} else {
					self.prob_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if protected {
					self.prot_tail = prev;
				} else {
					self.prob_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		if protected {
			self.prot_len -= 1;
		} else {
			self.prob_len -= 1;
		}
	}

	/// Append a node at the MRU (tail) end of the probationary segment.
	fn push_back_probationary(&mut self, idx: usize) {
		self.nodes[idx].protected = false;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
		self.prob_len += 1;
	}

	/// Append a node at the MRU (tail) end of the protected segment.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].protected = true;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
		self.prot_len += 1;
	}

	/// If the protected segment exceeds its budget, demote its LRU node to the
	/// MRU end of the probationary segment. Only ever one node over budget per
	/// promotion, so a single demotion restores the bound.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.protected_cap {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_back_probationary(lru);
			}
		}
	}

	// --- Ghost list (key-only, bounded by `ghost_cap`) -------------------

	/// Record `key` at the MRU end of the ghost list, evicting the ghost LRU if
	/// the ghost list is full. `key` must not already be a ghost (it is the key
	/// just evicted from the resident probationary segment).
	fn ghost_insert(&mut self, key: K) {
		let idx = if let Some(free) = self.ghost_free.pop() {
			// Reuse a slot freed by a prior re-admission.
			self.ghost_nodes[free].key = key.clone();
			free
		} else if self.ghost_len >= self.ghost_cap {
			// Ghost list full: drop its LRU and reuse that slot in place.
			let evict = self
				.ghost_head
				.expect("a full ghost list always has an LRU head");
			let old_key = self.ghost_nodes[evict].key.clone();
			self.ghost_map.remove(&old_key);
			self.ghost_unlink(evict);
			self.ghost_nodes[evict].key = key.clone();
			evict
		} else {
			// Room to grow the ghost arena.
			let idx = self.ghost_nodes.len();
			self.ghost_nodes.push(GhostNode {
				key: key.clone(),
				prev: None,
				next: None,
			});
			idx
		};
		self.ghost_push_back(idx);
		self.ghost_map.insert(key, idx);
		debug_assert!(self.ghost_len <= self.ghost_cap);
		debug_assert_eq!(self.ghost_len, self.ghost_map.len());
	}

	/// Remove `key` from the ghost list if present, returning whether it was a
	/// ghost. The freed slot is recycled for a future `ghost_insert`.
	fn ghost_remove(&mut self, key: &K) -> bool {
		if let Some(idx) = self.ghost_map.remove(key) {
			self.ghost_unlink(idx);
			self.ghost_free.push(idx);
			true
		} else {
			false
		}
	}

	/// Detach a ghost node from the ghost LRU list.
	fn ghost_unlink(&mut self, idx: usize) {
		let prev = self.ghost_nodes[idx].prev;
		let next = self.ghost_nodes[idx].next;
		match prev {
			Some(p) => self.ghost_nodes[p].next = next,
			None => self.ghost_head = next,
		}
		match next {
			Some(n) => self.ghost_nodes[n].prev = prev,
			None => self.ghost_tail = prev,
		}
		self.ghost_nodes[idx].prev = None;
		self.ghost_nodes[idx].next = None;
		self.ghost_len -= 1;
	}

	/// Append a ghost node at the MRU (tail) end of the ghost list.
	fn ghost_push_back(&mut self, idx: usize) {
		self.ghost_nodes[idx].prev = self.ghost_tail;
		self.ghost_nodes[idx].next = None;
		match self.ghost_tail {
			Some(t) => self.ghost_nodes[t].next = Some(idx),
			None => self.ghost_head = Some(idx),
		}
		self.ghost_tail = Some(idx);
		self.ghost_len += 1;
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
		cache.put(3, 30); // both keys still probationary; evicts probationary LRU (1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promotes key 1 into the protected segment
		cache.put(3, 30); // evicts key 2 (probationary LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update promotes key 1 into the protected segment
		cache.put(3, 30); // evicts key 2 (probationary LRU)

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
		cache.put(4, 40); // key 1 is protected; evicts a probationary LRU, not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // promote key 2 into the protected segment
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn promoted_key_survives_while_probationary_churns() {
		// SLRU: a key promoted to the protected segment is shielded from
		// eviction as long as the probationary segment can supply a victim.
		// Slots are still reused in place across re-evictions.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts probationary LRU (key 1), reuses slot 0
		cache.get(&3); // promote key 3 into the protected segment
		cache.put(4, 40); // evicts probationary LRU (key 2), reuses slot 1
		cache.put(5, 50); // evicts probationary LRU (key 4), reuses slot 1
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survived
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it (into protected).
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify the structure is still intact.
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite promotes key 3 into protected
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn ghost_readmit_enters_protected() {
		// A key evicted from probationary and re-requested soon after is
		// re-admitted directly to the protected segment, where it then resists
		// probationary churn.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full, all probationary
		cache.put(4, 40); // evicts probationary LRU (key 1); key 1 becomes a ghost
		assert!(cache.get(&1).is_none());
		cache.put(1, 11); // ghost hit: re-admitted into the protected segment
		// Key 1 is now protected; subsequent probationary churn must not evict it.
		cache.put(5, 50); // evicts a probationary victim, not protected key 1
		cache.put(6, 60); // evicts a probationary victim, not protected key 1
		assert_eq!(*cache.get(&1).unwrap(), 11);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
