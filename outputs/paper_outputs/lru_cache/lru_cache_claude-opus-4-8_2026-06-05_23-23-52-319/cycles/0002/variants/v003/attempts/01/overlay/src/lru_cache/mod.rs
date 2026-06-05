// Segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked list.
//
// The single LRU list of the baseline is split into two segments that share one
// arena: a *probationary* segment (entries seen once) and a *protected* segment
// (entries that earned enough accesses). Frequently-reused keys are promoted out
// of the probationary segment and so resist eviction, which is the canonical
// first improvement of SLRU over plain LRU on reuse-heavy traces.
//
// This variant raises the promotion threshold from one probationary hit to two:
// a probationary node must be accessed `PROMOTE_AT` times before it is promoted
// to the protected segment. Each node carries a `hits` counter for that purpose.
// The intent is to keep the protected tier purer by reserving it for genuinely
// hot keys.
//
// Capacity is partitioned so the two segment budgets sum to exactly `capacity`,
// leaving the P1 capacity bound identical to the baseline. Eviction slots are
// still reused in place, so there is no free-list management. No unsafe, no
// external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Number of probationary hits a node must accumulate before it is promoted to
/// the protected segment.
const PROMOTE_AT: u8 = 2;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// `true` when the node currently lives in the protected segment.
	protected: bool,
	// Number of accesses seen while probationary. Promotion fires once this
	// reaches `PROMOTE_AT`. Demotion leaves this unchanged so a node can
	// re-promote on its next hit.
	hits: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // shared arena

	// Probationary segment: entries that have not yet earned promotion.
	prob_head: Option<usize>, // LRU (front)
	prob_tail: Option<usize>, // MRU (back)
	prob_len: usize,

	// Protected segment: entries that earned enough accesses.
	prot_head: Option<usize>, // LRU (front)
	prot_tail: Option<usize>, // MRU (back)
	prot_len: usize,

	capacity: usize,
	// Size budget for the protected segment (~80% of capacity, floored). The
	// probationary budget is the remainder, so the two sum to exactly
	// `capacity`. For capacity == 1 this floors to 0, leaving a single usable
	// (probationary) slot — the only case where a segment budget is 0.
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
		// floor(0.8 * capacity); probationary budget is capacity - protected_cap,
		// so the two segment budgets sum to exactly `capacity`.
		let protected_cap = (capacity * 4) / 5;
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} probationary_cap={} promote_at={PROMOTE_AT}",
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
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording an access. A
	/// probationary node is promoted only after `PROMOTE_AT` hits; otherwise it
	/// is refreshed to the probationary MRU end.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss.
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used
	/// entry (probationary first, then protected).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
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
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			// Slot is reused for a brand-new key: reset its hit counter.
			self.nodes[evict_idx].hits = 0;
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
				hits: 0,
			});
			idx
		};
		// New entries enter at the MRU end of the probationary segment.
		self.push_back_probationary(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
	}

	/// Record an access to a resident node `idx`.
	///
	/// - Protected node: refresh to the protected MRU end; `hits` is left as is.
	/// - Probationary node: increment `hits`; promote to protected once it
	///   reaches `PROMOTE_AT`, otherwise refresh to the probationary MRU end so
	///   it survives a bit longer while staying unprotected.
	fn touch(&mut self, idx: usize) {
		if self.nodes[idx].protected {
			self.unlink(idx);
			self.push_back_protected(idx);
			return;
		}
		self.nodes[idx].hits = self.nodes[idx].hits.saturating_add(1);
		if self.nodes[idx].hits >= PROMOTE_AT {
			self.unlink(idx);
			self.push_back_protected(idx);
			// Keep the protected segment within its budget by demoting its LRU
			// node back to probationary. Net resident count is unchanged.
			self.enforce_protected_budget();
		} else {
			self.unlink(idx);
			self.push_back_probationary(idx);
		}
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
	/// promotion, so a single demotion restores the bound. The demoted node's
	/// `hits` counter is left unchanged so it can re-promote on its next hit.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.protected_cap {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_back_probationary(lru);
			}
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
		cache.put(3, 30); // both keys still probationary; evicts probationary LRU (1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn promotion_requires_two_probationary_hits() {
		// With PROMOTE_AT == 2, a single get does NOT promote: key 1 stays
		// probationary and remains the LRU victim.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // one hit: refreshed to probationary MRU, NOT promoted
		cache.put(3, 30); // evicts probationary LRU (now key 2)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn two_hits_promote_and_shield() {
		// A second probationary hit promotes key 1 into the protected segment,
		// shielding it from eviction while probationary churns.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // hit 1: probationary MRU
		cache.get(&1); // hit 2: promoted to protected
		cache.put(3, 30); // evicts probationary LRU (key 2)
		cache.put(4, 40); // evicts probationary LRU (key 3)
		assert_eq!(*cache.get(&1).unwrap(), 10); // protected, survived
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
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
	fn slot_reuse_resets_hits() {
		// A slot reused for a new key must start at zero hits, so the new key
		// is not accidentally promoted on its first access.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.get(&1); // key 1 gets one hit
		cache.put(2, 20);
		cache.put(3, 30); // evicts probationary LRU, reusing a slot
		// Whichever slot was reused must have hits reset; a single get should
		// not promote the new occupant.
		cache.get(&3); // one hit only
		cache.put(4, 40); // evicts a probationary LRU
		cache.put(5, 50); // evicts a probationary LRU
		// key 3 was never promoted (only one hit), so it can be evicted.
		assert_eq!(*cache.get(&5).unwrap(), 50);
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
