// Segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked list.
//
// The single LRU list of the baseline is split into two segments that share one
// arena: a *probationary* segment (entries seen once) and a *protected* segment
// (entries that earned a second access). Frequently-reused keys are promoted out
// of the probationary segment and so resist eviction, which is the canonical
// first improvement of SLRU over plain LRU on reuse-heavy traces.
//
// Capacity is partitioned so the two segment budgets sum to exactly `capacity`,
// leaving the P1 capacity bound identical to the baseline. Eviction slots are
// still reused in place, so there is no free-list management. No unsafe, no
// external crates.
//
// Variant H02 (adaptive geometry): the protected/probationary boundary is no
// longer a fixed floor(0.95 * capacity) constant. Instead a mutable target
// `prot_target` floats within a clamp band of roughly
// [floor(0.90 * capacity), floor(0.98 * capacity)], steered by ghost-hit
// feedback. A single bounded ghost structure records the *keys* (never values,
// so residency and the P1 bound are untouched) of nodes evicted from either
// segment, tagged with whether the evicted node had been demoted out of the
// protected segment:
//
//   * A returning *probationary* ghost (a one-hitter we discarded whose reuse
//     distance just exceeded the probationary window) means a larger
//     probationary segment would have caught it — so `prot_target` is nudged
//     *down* one step within the clamp.
//   * A returning *demoted-then-evicted* ghost (a once-protected key that we
//     demoted, then evicted, that has come back) means a larger protected
//     segment would have kept it — so `prot_target` is nudged *up* one step.
//
// Ghosts are used here only as a resize *signal*, never for re-admission, so the
// resident set and the `prob_len + prot_len == map.len()` accounting are exactly
// the parent's. The demote-to-front behavior of the parent is preserved.

use std::collections::HashMap;
use std::collections::VecDeque;
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
	// `true` when this node currently sits in the probationary segment only
	// because it was demoted out of the protected segment (not because it is a
	// fresh arrival). Used to tag ghosts so a return can distinguish a
	// probationary one-hitter from a demoted-then-evicted key.
	demoted: bool,
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
	// Mutable target size for the protected segment. Initialized to
	// floor(0.95 * capacity) and clamped to [prot_lo, prot_hi] so the
	// probationary floor can neither vanish nor dominate. The probationary
	// budget is always the remainder (capacity - prot_len), so the two segments
	// still sum to at most `capacity`.
	prot_target: usize,
	prot_lo: usize, // floor(0.90 * capacity)
	prot_hi: usize, // floor(0.98 * capacity)

	// Bounded ghost directory: keys (no values) of recently evicted nodes,
	// mapping each to `was_demoted` (true if the evicted node had been demoted
	// out of the protected segment). `ghost_order` is a FIFO of insertion order
	// used to bound the directory to ~capacity live entries; stale entries left
	// after a ghost hit are reclaimed lazily.
	ghosts: HashMap<K, bool>,
	ghost_order: VecDeque<K>,
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
		// floor(0.95 * capacity) target, clamped within
		// [floor(0.90 * capacity), floor(0.98 * capacity)].
		let prot_target = (capacity * 19) / 20;
		let prot_lo = (capacity * 9) / 10;
		let prot_hi = (capacity * 49) / 50;
		log::info!(
			"LruCache.try_new: capacity={capacity} prot_target={prot_target} band=[{prot_lo},{prot_hi}]"
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
			prot_target,
			prot_lo,
			prot_hi,
			ghosts: HashMap::with_capacity(capacity),
			ghost_order: VecDeque::with_capacity(capacity),
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
		// Brand-new key. Before allocating, consult the ghost directory: a hit
		// there is a resize signal (never a re-admission). The key is removed
		// from the directory once consumed.
		if let Some(was_demoted) = self.ghosts.remove(&key) {
			if was_demoted {
				// A once-protected key we demoted and then evicted has returned:
				// a larger protected segment would have kept it. Grow protected.
				self.nudge_target_up();
			} else {
				// A probationary one-hitter we discarded has returned: its reuse
				// distance just exceeded the probationary window, so a larger
				// probationary segment would have caught it. Shrink protected.
				self.nudge_target_down();
			}
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
			let evicted_demoted = self.nodes[evict_idx].demoted;
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			// Record the evicted key as a ghost so a later return can steer the
			// boundary. Values are never retained.
			self.record_ghost(evicted_key, evicted_demoted);
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
				demoted: false,
			});
			idx
		};
		// New entries enter at the MRU end of the probationary segment.
		self.push_back_probationary(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
	}

	/// Record an evicted key in the bounded ghost directory, tagged with whether
	/// the evicted node had been demoted out of the protected segment. When the
	/// directory is full the oldest live ghost is dropped (FIFO); stale entries
	/// left behind by a ghost hit are skipped and reclaimed here.
	fn record_ghost(&mut self, key: K, was_demoted: bool) {
		if self.ghost_cap == 0 {
			return;
		}
		// Already tracked: refresh its tag without changing FIFO position.
		if self.ghosts.contains_key(&key) {
			self.ghosts.insert(key, was_demoted);
			return;
		}
		// Evict oldest live ghosts until there is room for one more.
		while self.ghosts.len() >= self.ghost_cap {
			match self.ghost_order.pop_front() {
				Some(old) => {
					// `remove` returns None for a stale (already-consumed) entry;
					// those cost nothing and simply free a FIFO slot.
					self.ghosts.remove(&old);
				}
				None => break,
			}
		}
		self.ghost_order.push_back(key.clone());
		self.ghosts.insert(key, was_demoted);
	}

	/// Nudge the protected target up by one within the clamp band.
	fn nudge_target_up(&mut self) {
		if self.prot_target < self.prot_hi {
			self.prot_target += 1;
		}
	}

	/// Nudge the protected target down by one within the clamp band.
	fn nudge_target_down(&mut self) {
		if self.prot_target > self.prot_lo {
			self.prot_target -= 1;
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
		self.nodes[idx].demoted = false;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
		self.prob_len += 1;
	}

	/// Prepend a node at the LRU (head) end of the probationary segment. This is
	/// the head-end mirror of `push_back_probationary`; a node placed here is the
	/// next probationary eviction victim, since eviction always takes `prob_head`.
	/// Used only for demotion, so the node is tagged `demoted`.
	fn push_front_probationary(&mut self, idx: usize) {
		self.nodes[idx].protected = false;
		self.nodes[idx].demoted = true;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.prob_head;
		match self.prob_head {
			Some(h) => self.nodes[h].prev = Some(idx),
			None => self.prob_tail = Some(idx),
		}
		self.prob_head = Some(idx);
		self.prob_len += 1;
	}

	/// Append a node at the MRU (tail) end of the protected segment.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].protected = true;
		// Earning (re-)protection clears any pending demoted tag.
		self.nodes[idx].demoted = false;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
		self.prot_len += 1;
	}

	/// If the protected segment exceeds its (now adaptive) target, demote its LRU
	/// node to the LRU (front) end of the probationary segment. Only ever one
	/// node over budget per promotion, so a single demotion restores the bound.
	/// Placing the demoted node at the front makes it the next eviction victim,
	/// reserving the probationary reuse window for fresh arrivals.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.prot_target {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_front_probationary(lru);
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
