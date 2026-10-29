// O(1) segmented-LRU (SLRU, 20% probation / 80% protected) cache using HashMap +
// arena-based doubly-linked lists. Nodes are stored in a Vec (arena) indexed by u32.
// On eviction the slot is reused in-place, so no free-list management is needed.
// No unsafe, no external crates.
//
// Links are `u32` arena indices. To keep `Node<u64, u64>` at exactly 24 bytes, two
// flags live in the top bits of `prev`: bit 31 is the segment flag (probation vs
// protected) and bit 30 is a CLOCK-style reference bit used by the protected segment.
// The low 30 bits hold the index. All link reads/writes go through the accessors on
// `Node`, so the flags never leak into an index. `NIL` (= `IDX_MASK`) is the "no
// link" sentinel for both `prev` (index part) and `next`, and for the segment
// heads/tails.
//
// Policy:
// - A new key enters the probation segment at its MRU end.
// - A hit on a probation entry promotes it to the protected MRU; if the protected
//   segment then exceeds `protected_cap`, `demote_one` runs a bounded CLOCK sweep
//   over the protected LRU end: referenced entries get their bit cleared and are
//   rotated to the protected MRU; the first unreferenced entry (or the head after
//   `MAX_SWEEP` rotations) is demoted to probation MRU.
// - A hit on a protected entry only sets its reference bit in place (no list work).
// - Eviction takes the probation LRU, falling back to the protected LRU (ref bit
//   ignored).
//
// Insert path: `put` performs a single `HashMap::entry` lookup. On eviction the new
// key is inserted before the victim's key is removed, so the map briefly holds
// `capacity + 1` entries; it is pre-sized to `capacity + 1` so this never resizes.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Segment flag stored in bit 31 of `Node::prev`.
const PROT_BIT: u32 = 1 << 31;
/// CLOCK reference bit stored in bit 30 of `Node::prev` (protected segment only).
const REF_BIT: u32 = 1 << 30;
/// All flag bits carried in `Node::prev`.
const FLAGS: u32 = PROT_BIT | REF_BIT;
/// Mask extracting the index part of `Node::prev`.
const IDX_MASK: u32 = (1 << 30) - 1;
/// Sentinel "no link" index. Every valid arena index is strictly below this.
const NIL: u32 = IDX_MASK;
/// Maximum number of referenced protected heads rotated past per demotion.
const MAX_SWEEP: u32 = 8;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Bits 0..30: previous index (NIL if none). Bit 30: reference bit.
	/// Bit 31: protected-segment flag.
	prev: u32,
	/// Next index (NIL if none).
	next: u32,
}

impl<K, V> Node<K, V> {
	#[inline]
	fn prev_idx(&self) -> u32 {
		self.prev & IDX_MASK
	}

	#[inline]
	fn is_protected(&self) -> bool {
		self.prev & PROT_BIT != 0
	}

	#[inline]
	fn is_referenced(&self) -> bool {
		self.prev & REF_BIT != 0
	}

	/// Set the prev index, preserving the flag bits.
	#[inline]
	fn set_prev(&mut self, idx: u32) {
		debug_assert!(idx <= IDX_MASK);
		self.prev = (self.prev & FLAGS) | (idx & IDX_MASK);
	}

	#[inline]
	fn set_protected(&mut self, protected: bool) {
		if protected {
			self.prev |= PROT_BIT;
		} else {
			self.prev &= !PROT_BIT;
		}
	}

	#[inline]
	fn next_idx(&self) -> u32 {
		self.next
	}

	#[inline]
	fn set_next(&mut self, idx: u32) {
		debug_assert!(idx <= IDX_MASK);
		self.next = idx;
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	prob_head: u32,         // probation LRU (front), NIL when empty
	prob_tail: u32,         // probation MRU (back), NIL when empty
	prot_head: u32,         // protected LRU (front), NIL when empty
	prot_tail: u32,         // protected MRU (back), NIL when empty
	prob_len: u32,
	prot_len: u32,
	protected_cap: u32,
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
		if capacity >= IDX_MASK as usize {
			log::error!("LruCache.try_new: capacity must be < {IDX_MASK}, got {capacity}");
			return None;
		}
		let protected_cap: u32 = if capacity == 1 {
			0
		} else {
			// capacity < IDX_MASK, so the cast is lossless
			std::cmp::max(1, capacity * 8 / 10) as u32
		};
		log::info!("LruCache.try_new: capacity={capacity} protected_cap={protected_cap}");
		Some(Self {
			// +1: during eviction the new key is inserted before the victim is removed.
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity),
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prob_len: 0,
			prot_len: 0,
			protected_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, applying the SLRU hit policy.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.on_hit(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probation LRU entry
	/// (falling back to the protected LRU). Uses a single map lookup for the key.
	pub fn put(&mut self, key: K, value: V) {
		match self.map.entry(key) {
			Entry::Occupied(e) => {
				// Key already exists: update value in-place and treat as a hit
				let idx = *e.get();
				self.nodes[idx as usize].value = value;
				self.on_hit(idx);
				return;
			}
			Entry::Vacant(v) => {
				if self.nodes.len() < self.capacity {
					// Filling phase: len < capacity < IDX_MASK, so the cast is lossless
					let idx = self.nodes.len() as u32;
					self.nodes.push(Node {
						key: v.key().clone(),
						value,
						prev: NIL,
						next: NIL,
					});
					v.insert(idx);
					self.push_back(idx, false);
				} else {
					// Eviction phase: reuse the victim's slot in place.
					// Fallback to the protected LRU ignores the reference bit.
					let victim = if self.prob_head != NIL {
						self.prob_head
					} else {
						self.prot_head
					};
					debug_assert!(victim != NIL, "a segment must be non-empty when arena is full");
					let old_key = {
						let node = &mut self.nodes[victim as usize];
						node.value = value;
						std::mem::replace(&mut node.key, v.key().clone())
					};
					v.insert(victim);
					// `unlink` reads only links and the segment flag, which are untouched.
					self.unlink(victim);
					// Reused slot is reset to probation (push_back also clears REF_BIT).
					self.nodes[victim as usize].set_protected(false);
					self.map.remove(&old_key);
					self.push_back(victim, false);
				}
			}
		}
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.map.len());
	}

	/// SLRU hit handling.
	#[inline]
	fn on_hit(&mut self, idx: u32) {
		let node = &mut self.nodes[idx as usize];
		let p = node.prev;
		if p & PROT_BIT != 0 {
			// Protected hit: set the CLOCK reference bit in place; no list work.
			if p & REF_BIT == 0 {
				node.prev = p | REF_BIT;
			}
		} else if self.protected_cap == 0 {
			self.unlink(idx);
			self.push_back(idx, false);
		} else {
			// Promote probation -> protected
			self.unlink(idx);
			self.push_back(idx, true);
			if self.prot_len > self.protected_cap {
				self.demote_one();
			}
		}
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.map.len());
	}

	/// Bounded CLOCK sweep over the protected LRU end: rotate referenced heads to
	/// the protected MRU (clearing their bit) and demote the first unreferenced head
	/// to probation MRU. After `MAX_SWEEP` rotations, demote the head regardless.
	#[inline]
	fn demote_one(&mut self) {
		for _ in 0..MAX_SWEEP {
			let h = self.prot_head;
			debug_assert!(h != NIL);
			self.unlink(h);
			if self.nodes[h as usize].is_referenced() {
				// Second chance: push_back clears REF_BIT.
				self.push_back(h, true);
			} else {
				self.push_back(h, false);
				return;
			}
		}
		let h = self.prot_head;
		debug_assert!(h != NIL);
		self.unlink(h);
		self.push_back(h, false);
	}

	/// Remove node from its segment list.
	#[inline]
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let protected = self.nodes[i].is_protected();
		let prev = self.nodes[i].prev_idx();
		let next = self.nodes[i].next_idx();
		if prev == NIL {
			if protected {
				self.prot_head = next;
			} else {
				self.prob_head = next;
			}
		} else {
			self.nodes[prev as usize].set_next(next);
		}
		if next == NIL {
			if protected {
				self.prot_tail = prev;
			} else {
				self.prob_tail = prev;
			}
		} else {
			self.nodes[next as usize].set_prev(prev);
		}
		self.nodes[i].set_prev(NIL);
		self.nodes[i].set_next(NIL);
		if protected {
			self.prot_len -= 1;
		} else {
			self.prob_len -= 1;
		}
	}

	/// Append node to the MRU end of the chosen segment, setting its segment flag
	/// and clearing its reference bit (a newly linked node starts unreferenced).
	#[inline]
	fn push_back(&mut self, idx: u32, protected: bool) {
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		{
			let node = &mut self.nodes[idx as usize];
			node.prev = if protected { PROT_BIT | tail } else { tail };
			node.set_next(NIL);
		}
		if tail == NIL {
			if protected {
				self.prot_head = idx;
			} else {
				self.prob_head = idx;
			}
		} else {
			self.nodes[tail as usize].set_next(idx);
		}
		if protected {
			self.prot_tail = idx;
			self.prot_len += 1;
		} else {
			self.prob_tail = idx;
			self.prob_len += 1;
		}
	}

	/// Walk both segments and verify flags, links and length counters.
	/// Returns (probation keys LRU->MRU, protected keys LRU->MRU).
	#[cfg(test)]
	fn check_segments(&self) -> (Vec<K>, Vec<K>) {
		let walk = |head: u32, tail: u32, protected: bool, len: u32| -> Vec<K> {
			let mut out = Vec::new();
			let mut prev = NIL;
			let mut cur = head;
			while cur != NIL {
				let n = &self.nodes[cur as usize];
				assert_eq!(n.is_protected(), protected, "segment flag mismatch at {cur}");
				if !protected {
					assert!(!n.is_referenced(), "probation node {cur} has ref bit set");
				}
				assert_eq!(n.prev_idx(), prev, "broken prev link at {cur}");
				assert_eq!(self.map.get(&n.key), Some(&cur), "map/arena mismatch at {cur}");
				out.push(n.key.clone());
				prev = cur;
				cur = n.next_idx();
				assert!(out.len() <= self.nodes.len(), "cycle in list");
			}
			assert_eq!(prev, tail, "tail mismatch");
			assert_eq!(out.len(), len as usize, "length counter mismatch");
			out
		};
		let prob = walk(self.prob_head, self.prob_tail, false, self.prob_len);
		let prot = walk(self.prot_head, self.prot_tail, true, self.prot_len);
		assert_eq!(prob.len() + prot.len(), self.map.len());
		assert!(self.prot_len <= self.protected_cap);
		(prob, prot)
	}

	#[cfg(test)]
	fn referenced(&self, key: &K) -> bool {
		self.nodes[self.map[key] as usize].is_referenced()
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
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
		cache.check_segments();
	}

	#[test]
	fn eviction_removes_probation_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (probation LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		cache.check_segments();
	}

	#[test]
	fn hit_promotes_and_protects() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // 1 -> protected
		cache.put(3, 30); // evicts key 2 (probation LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		cache.check_segments();
	}

	#[test]
	fn put_existing_counts_as_hit() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // hit -> protected
		cache.put(3, 30); // evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		cache.check_segments();
	}

	#[test]
	fn protected_survives_probation_churn() {
		// SLRU scan resistance: a protected key survives a scan of one-hit keys.
		let mut cache = LruCache::new(5); // protected_cap = 4
		cache.put(1, 10);
		cache.get(&1); // protected
		for k in 100..200 {
			cache.put(k, k);
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.check_segments();
	}

	#[test]
	fn protected_overflow_demotes_unreferenced_head() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		cache.get(&1);
		cache.get(&2);
		cache.get(&3); // protected: 1,2,3 (all unreferenced) ; probation: 4
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![4]);
		assert_eq!(prot, vec![1, 2, 3]);
		cache.get(&4); // promote 4; protected overflows -> unreferenced head 1 demoted
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![1]);
		assert_eq!(prot, vec![2, 3, 4]);
		cache.put(5, 50); // evicts 1 (only probation entry)
		assert!(cache.get(&1).is_none());
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![5]);
		assert_eq!(prot, vec![2, 3, 4]);
	}

	#[test]
	fn protected_overflow_rotates_past_referenced_head() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		cache.get(&1);
		cache.get(&2);
		cache.get(&3); // protected: 1,2,3
		cache.get(&1); // protected hit: sets ref bit on 1, order unchanged
		assert!(cache.referenced(&1));
		let (_, prot) = cache.check_segments();
		assert_eq!(prot, vec![1, 2, 3]);
		cache.get(&4); // overflow: 1 referenced -> rotated (bit cleared); 2 demoted
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![2]);
		assert_eq!(prot, vec![3, 4, 1]);
		assert!(!cache.referenced(&1));
	}

	#[test]
	fn protected_hit_sets_ref_bit_without_relinking() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		for k in 1..=4 {
			cache.put(k, k);
		}
		cache.get(&1);
		cache.get(&2);
		assert!(!cache.referenced(&1));
		cache.get(&1); // protected hit: order stays 1, 2; 1 referenced
		let (_, prot) = cache.check_segments();
		assert_eq!(prot, vec![1, 2]);
		assert!(cache.referenced(&1));
		assert!(!cache.referenced(&2));
		cache.get(&1); // idempotent
		assert!(cache.referenced(&1));
		let (_, prot) = cache.check_segments();
		assert_eq!(prot, vec![1, 2]);
	}

	#[test]
	fn sweep_bound_forces_demotion() {
		let mut cache = LruCache::new(12); // protected_cap = 9
		assert_eq!(cache.protected_cap, 9);
		for k in 1..=12 {
			cache.put(k, k);
		}
		for k in 1..=9 {
			cache.get(&k); // promote 1..9
		}
		for k in 1..=9 {
			cache.get(&k); // reference all 9 protected entries
			assert!(cache.referenced(&k));
		}
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![10, 11, 12]);
		assert_eq!(prot, (1..=9).collect::<Vec<_>>());
		cache.get(&10); // promote 10; sweep rotates 1..8, then demotes 9 regardless
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![11, 12, 9]);
		assert_eq!(prot, vec![10, 1, 2, 3, 4, 5, 6, 7, 8]);
		for k in 1..=8 {
			assert!(!cache.referenced(&k));
		}
		assert!(!cache.referenced(&9));
		assert!(!cache.referenced(&10));
	}

	#[test]
	fn flag_cycle_probation_protected_demoted() {
		// One key cycles probation -> protected -> demoted; flags and links stay
		// consistent and eviction order is correct afterwards.
		let mut cache = LruCache::new(3); // protected_cap = 2
		cache.put(1, 10);
		let idx1 = cache.map[&1] as usize;
		assert!(!cache.nodes[idx1].is_protected());
		cache.get(&1); // protected
		assert!(cache.nodes[idx1].is_protected());
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // protected: 1, 2
		cache.get(&3); // protected overflows: 1 demoted to probation MRU
		assert!(!cache.nodes[idx1].is_protected());
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![1]);
		assert_eq!(prot, vec![2, 3]);
		assert_eq!(cache.nodes[idx1].prev_idx(), NIL);
		assert_eq!(cache.nodes[idx1].next_idx(), NIL);
		// Eviction takes the demoted key 1 first.
		cache.put(4, 40);
		assert!(cache.get(&1).is_none());
		// Reused slot is reset to probation.
		assert!(!cache.nodes[idx1].is_protected());
		assert_eq!(cache.map[&4] as usize, idx1);
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![4]);
		assert_eq!(prot, vec![2, 3]);
		// Next eviction takes 4 (probation), protected keys survive.
		cache.put(5, 50);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		cache.check_segments();
	}

	#[test]
	fn eviction_fallback_clears_ref_bit() {
		// capacity 2: protected_cap = 1. Reused slots and demoted/rotated nodes must
		// come back with the reference bit cleared.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // 1 protected
		cache.get(&1); // 1 referenced
		assert!(cache.referenced(&1));
		cache.put(3, 30); // evicts 2
		assert!(!cache.referenced(&3));
		cache.get(&3); // promote 3; protected overflow -> 1 rotated, then 3 demoted
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![3]);
		assert_eq!(prot, vec![1]);
		assert!(!cache.referenced(&1));
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
		cache.check_segments();
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1); // protected_cap = 0
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.check_segments();
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		cache.check_segments();
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		cache.check_segments();
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2); // protected_cap = 1
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // key 3 -> protected
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (probation LRU), slot 1 reused again
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		cache.check_segments();
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.check_segments();
	}

	#[test]
	fn random_workload_keeps_segments_consistent() {
		for &cap in &[1usize, 2, 3, 7, 16] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 12345;
			for _ in 0..5000 {
				x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let k = ((x >> 33) % 40) as i64;
				if cache.get(&k).is_none() {
					cache.put(k, k * 10);
				}
				assert!(cache.map.len() <= cap);
			}
			cache.check_segments();
		}
	}

	#[test]
	fn churn_does_not_resize_map() {
		for &cap in &[1usize, 2, 64, 2048] {
			let mut cache = LruCache::<i64, i64>::new(cap);
			for k in 0..cap as i64 {
				cache.put(k, k);
			}
			// Promote some entries so eviction exercises both segments.
			for k in 0..(cap as i64 / 2) {
				cache.get(&k);
			}
			let warm_cap = cache.map.capacity();
			assert!(warm_cap >= cap + 1);
			let base = cap as i64;
			for k in base..base + 10 * cap as i64 {
				cache.put(k, k);
				assert!(cache.map.len() <= cap);
			}
			assert_eq!(cache.map.capacity(), warm_cap, "map resized during churn (cap={cap})");
			assert_eq!(cache.map.len(), cap);
			cache.check_segments();
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn capacity_at_index_limit_rejected() {
		assert_eq!(IDX_MASK, (1 << 30) - 1);
		assert!(LruCache::<i64, i64>::try_new(IDX_MASK as usize).is_none());
		// The previous 31-bit limit is now also out of range.
		assert!(LruCache::<i64, i64>::try_new(((1u64 << 31) - 1) as usize).is_none());
	}

	#[test]
	fn flags_do_not_overlap_index() {
		assert_eq!(FLAGS & IDX_MASK, 0);
		assert_eq!(FLAGS | IDX_MASK, u32::MAX);
	}

	#[test]
	fn protected_cap_formula() {
		assert_eq!(LruCache::<i64, i64>::new(1).protected_cap, 0);
		assert_eq!(LruCache::<i64, i64>::new(2).protected_cap, 1);
		assert_eq!(LruCache::<i64, i64>::new(10).protected_cap, 8);
		assert_eq!(LruCache::<i64, i64>::new(2048).protected_cap, 1638);
	}

	#[test]
	fn node_u64_is_24_bytes() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}
}
