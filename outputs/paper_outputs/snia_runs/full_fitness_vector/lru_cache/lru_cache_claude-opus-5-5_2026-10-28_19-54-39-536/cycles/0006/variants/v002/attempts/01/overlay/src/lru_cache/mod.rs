// O(1) segmented-LRU (SLRU, 20% probation / 80% protected) cache using a custom
// open-addressing index table + arena-based doubly-linked lists. Nodes are stored in
// a Vec (arena) indexed by u32. On eviction the slot is reused in-place, so no
// free-list management is needed. No unsafe, no external crates.
//
// Links are `u32` arena indices. To keep `Node<u64, u64>` at exactly 24 bytes, the
// segment flag (probation vs protected) lives in bit 31 of `prev`; the low 31 bits
// hold the index. All link reads/writes go through the accessors on `Node`, so the
// flag never leaks into an index. `NIL` (= `IDX_MASK`) is the "no link" sentinel for
// both `prev` (index part) and `next`, and for the segment heads/tails.
//
// Policy:
// - A new key enters the probation segment at its MRU end.
// - A hit on a probation entry promotes it to the protected MRU; if the protected
//   segment then exceeds `protected_cap`, its LRU entry is demoted to probation MRU.
// - A hit on a protected entry moves it to the protected MRU.
// - Eviction takes the probation LRU, falling back to the protected LRU.
//
// Index: `IndexTable` is a dense linear-probing table of packed `u64` slots,
// `(hash high 32 bits, low bit cleared) << 32 | arena index`. `EMPTY = u64::MAX`;
// the cleared fragment bit guarantees a live slot never equals `EMPTY`. A probe
// only dereferences the arena node when the 32-bit fragment matches. The table is
// sized to `(2 * (capacity + 1)).next_power_of_two()` (load factor <= 0.5) and
// never resizes. Deletion uses backward-shift (no tombstones).
//
// Insert path: `put` performs a single probe that yields either the matching slot or
// the first empty slot. On eviction the victim's key is removed from the index first
// (which may shift entries), then the first empty slot for the new key is re-probed.

use std::collections::hash_map::RandomState;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub mod lru_cache_jni;

/// Segment flag stored in bit 31 of `Node::prev`.
const PROT_BIT: u32 = 1 << 31;
/// Mask extracting the index part of `Node::prev`.
const IDX_MASK: u32 = PROT_BIT - 1;
/// Sentinel "no link" index. Every valid arena index is strictly below this.
const NIL: u32 = IDX_MASK;

/// Empty index-table slot.
const EMPTY: u64 = u64::MAX;
/// Bits of the hash kept as the slot fragment: high 32 bits with bit 32 cleared, so
/// an occupied slot can never equal `EMPTY`.
const FRAG_MASK: u64 = 0xFFFF_FFFE_0000_0000;
/// Bits of a slot holding the arena index.
const SLOT_IDX_MASK: u64 = 0xFFFF_FFFF;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Bits 0..31: previous index (NIL if none). Bit 31: protected-segment flag.
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

	/// Set the prev index, preserving the segment flag.
	#[inline]
	fn set_prev(&mut self, idx: u32) {
		debug_assert!(idx <= IDX_MASK);
		self.prev = (self.prev & PROT_BIT) | (idx & IDX_MASK);
	}

	#[inline]
	fn set_protected(&mut self, protected: bool) {
		if protected {
			self.prev |= PROT_BIT;
		} else {
			self.prev &= IDX_MASK;
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

/// Dense open-addressing key -> arena-index table with packed `u64` slots.
#[derive(Debug)]
struct IndexTable {
	slots: Box<[u64]>,
	mask: u32,
	hasher: RandomState,
}

impl IndexTable {
	fn with_capacity(capacity: usize) -> Self {
		// capacity < IDX_MASK (2^31 - 1), so n <= 2^32 and n - 1 fits in u32.
		let n = (2 * (capacity + 1)).next_power_of_two();
		Self {
			slots: vec![EMPTY; n].into_boxed_slice(),
			mask: (n - 1) as u32,
			hasher: RandomState::new(),
		}
	}

	#[inline]
	fn hash<K: Hash>(&self, key: &K) -> u64 {
		self.hasher.hash_one(key)
	}

	#[inline]
	fn home(&self, h: u64) -> usize {
		h as usize & self.mask as usize
	}

	#[inline]
	fn pack(h: u64, idx: u32) -> u64 {
		(h & FRAG_MASK) | idx as u64
	}

	/// Probe for `key` with precomputed hash `h`. Returns `Ok((slot, idx))` when the
	/// key is present, or `Err(slot)` with the first empty slot on the probe path.
	#[inline]
	fn probe<K: Eq, V>(&self, h: u64, key: &K, nodes: &[Node<K, V>]) -> Result<(usize, u32), usize> {
		let mask = self.mask as usize;
		let frag = h & FRAG_MASK;
		let mut i = self.home(h);
		loop {
			let s = self.slots[i];
			if s == EMPTY {
				return Err(i);
			}
			if s & FRAG_MASK == frag {
				let idx = (s & SLOT_IDX_MASK) as u32;
				if nodes[idx as usize].key == *key {
					return Ok((i, idx));
				}
			}
			i = (i + 1) & mask;
		}
	}

	/// Look up `key`; returns `(slot, arena index)` if present.
	#[inline]
	fn find<K: Hash + Eq, V>(&self, key: &K, nodes: &[Node<K, V>]) -> Option<(usize, u32)> {
		self.probe(self.hash(key), key, nodes).ok()
	}

	/// First empty slot on the probe path of hash `h` (key known to be absent).
	#[inline]
	fn first_empty(&self, h: u64) -> usize {
		let mask = self.mask as usize;
		let mut i = self.home(h);
		while self.slots[i] != EMPTY {
			i = (i + 1) & mask;
		}
		i
	}

	/// Fill an empty slot with `(h, idx)`.
	#[inline]
	fn insert_vacant(&mut self, slot: usize, h: u64, idx: u32) {
		debug_assert_eq!(self.slots[slot], EMPTY);
		self.slots[slot] = Self::pack(h, idx);
	}

	/// Remove `key` from the table using backward-shift deletion. Returns the arena
	/// index it mapped to, if present. `nodes` must hold the current key of every
	/// indexed entry (used to recompute the home slot of shifted entries).
	fn remove_by_key<K: Hash + Eq, V>(&mut self, key: &K, nodes: &[Node<K, V>]) -> Option<u32> {
		let (slot, idx) = self.find(key, nodes)?;
		self.remove_slot(slot, nodes);
		Some(idx)
	}

	/// Empty `hole` and backward-shift the following cluster so that every remaining
	/// entry stays reachable from its home slot.
	fn remove_slot<K: Hash, V>(&mut self, hole: usize, nodes: &[Node<K, V>]) {
		let mask = self.mask as usize;
		let mut hole = hole;
		let mut j = hole;
		loop {
			j = (j + 1) & mask;
			let s = self.slots[j];
			if s == EMPTY {
				break;
			}
			let idx = (s & SLOT_IDX_MASK) as usize;
			let home = self.home(self.hash(&nodes[idx].key));
			// Entry at j stays put iff its home lies cyclically in (hole, j].
			let dist_home = j.wrapping_sub(home) & mask;
			let dist_hole = j.wrapping_sub(hole) & mask;
			if dist_home >= dist_hole {
				self.slots[hole] = s;
				hole = j;
			}
		}
		self.slots[hole] = EMPTY;
	}

	#[cfg(test)]
	fn occupied(&self) -> usize {
		self.slots.iter().filter(|&&s| s != EMPTY).count()
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	index: IndexTable,      // key -> node index
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
			index: IndexTable::with_capacity(capacity),
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
		let (_, idx) = self.index.find(key, &self.nodes)?;
		self.on_hit(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probation LRU entry
	/// (falling back to the protected LRU). Uses a single index probe for the key.
	pub fn put(&mut self, key: K, value: V) {
		let h = self.index.hash(&key);
		match self.index.probe(h, &key, &self.nodes) {
			Ok((_, idx)) => {
				// Key already exists: update value in-place and treat as a hit
				self.nodes[idx as usize].value = value;
				self.on_hit(idx);
				return;
			}
			Err(slot) => {
				if self.nodes.len() < self.capacity {
					// Filling phase: len < capacity < IDX_MASK, so the cast is lossless
					let idx = self.nodes.len() as u32;
					self.nodes.push(Node {
						key,
						value,
						prev: NIL,
						next: NIL,
					});
					self.index.insert_vacant(slot, h, idx);
					self.push_back(idx, false);
				} else {
					// Eviction phase: reuse the victim's slot in place.
					let victim = if self.prob_head != NIL {
						self.prob_head
					} else {
						self.prot_head
					};
					debug_assert!(victim != NIL, "a segment must be non-empty when arena is full");
					// Remove the victim from the index first (its node still holds its key).
					let removed = self.index.remove_by_key(&self.nodes[victim as usize].key, &self.nodes);
					debug_assert_eq!(removed, Some(victim));
					// The backward shift may have moved entries; re-probe for an empty slot.
					let slot = self.index.first_empty(h);
					{
						let node = &mut self.nodes[victim as usize];
						node.key = key;
						node.value = value;
					}
					self.index.insert_vacant(slot, h, victim);
					// `unlink` reads only links and the segment flag, which are untouched.
					self.unlink(victim);
					// Reused slot is reset to probation.
					self.nodes[victim as usize].set_protected(false);
					self.push_back(victim, false);
				}
			}
		}
		debug_assert!(self.nodes.len() <= self.capacity);
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.nodes.len());
	}

	/// SLRU hit handling.
	#[inline]
	fn on_hit(&mut self, idx: u32) {
		if self.nodes[idx as usize].is_protected() {
			self.unlink(idx);
			self.push_back(idx, true);
		} else if self.protected_cap == 0 {
			self.unlink(idx);
			self.push_back(idx, false);
		} else {
			// Promote probation -> protected
			self.unlink(idx);
			self.push_back(idx, true);
			if self.prot_len > self.protected_cap {
				// Demote protected LRU -> probation MRU
				let demoted = self.prot_head;
				debug_assert!(demoted != NIL);
				self.unlink(demoted);
				self.push_back(demoted, false);
			}
		}
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.nodes.len());
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

	/// Append node to the MRU end of the chosen segment, setting its flag.
	#[inline]
	fn push_back(&mut self, idx: u32, protected: bool) {
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		{
			let node = &mut self.nodes[idx as usize];
			node.set_protected(protected);
			node.set_prev(tail);
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

	/// Test-only: arena index of `key`, if resident.
	#[cfg(test)]
	fn index_of(&self, key: &K) -> Option<u32> {
		self.index.find(key, &self.nodes).map(|(_, idx)| idx)
	}

	/// Test-only: verify the index table against the arena. Every resident key's
	/// `find` returns its own arena index, and every occupied slot is owned by a
	/// resident node (no orphans, no duplicates).
	#[cfg(test)]
	fn check_index(&self) {
		let mut seen = vec![false; self.nodes.len()];
		let mut occupied = 0usize;
		for (slot, &s) in self.index.slots.iter().enumerate() {
			if s == EMPTY {
				continue;
			}
			occupied += 1;
			let idx = (s & SLOT_IDX_MASK) as usize;
			assert!(idx < self.nodes.len(), "orphan slot {slot} -> idx {idx}");
			assert!(!seen[idx], "duplicate slot for idx {idx}");
			seen[idx] = true;
			let key = &self.nodes[idx].key;
			let h = self.index.hash(key);
			assert_eq!(s & FRAG_MASK, h & FRAG_MASK, "fragment mismatch at slot {slot}");
			assert_eq!(self.index.find(key, &self.nodes), Some((slot, idx as u32)), "unreachable slot {slot}");
		}
		assert_eq!(occupied, self.nodes.len(), "index/arena size mismatch");
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
				assert_eq!(n.prev_idx(), prev, "broken prev link at {cur}");
				assert_eq!(self.index_of(&n.key), Some(cur), "index/arena mismatch at {cur}");
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
		assert_eq!(prob.len() + prot.len(), self.index.occupied());
		assert!(self.prot_len <= self.protected_cap);
		self.check_index();
		(prob, prot)
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
	fn protected_overflow_demotes_to_probation_mru() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		cache.get(&1);
		cache.get(&2);
		cache.get(&3); // protected: 1,2,3 ; probation: 4
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![4]);
		assert_eq!(prot, vec![1, 2, 3]);
		cache.get(&4); // promote 4; protected overflows -> demote 1 to probation MRU
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
	fn protected_hit_moves_to_protected_mru() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		for k in 1..=4 {
			cache.put(k, k);
		}
		cache.get(&1);
		cache.get(&2);
		cache.get(&1); // protected order: 2, 1
		let (_, prot) = cache.check_segments();
		assert_eq!(prot, vec![2, 1]);
	}

	#[test]
	fn flag_cycle_probation_protected_demoted() {
		// One key cycles probation -> protected -> demoted; flags and links stay
		// consistent and eviction order is correct afterwards.
		let mut cache = LruCache::new(3); // protected_cap = 2
		cache.put(1, 10);
		let idx1 = cache.index_of(&1).unwrap() as usize;
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
		assert_eq!(cache.index_of(&4).unwrap() as usize, idx1);
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
		let mut cache = LruCache::new(7);
		let mut x: u64 = 12345;
		for _ in 0..5000 {
			x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let k = ((x >> 33) % 20) as i64;
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.nodes.len() <= 7);
		}
		cache.check_segments();
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
			let table_len = cache.index.slots.len();
			assert!(table_len >= 2 * (cap + 1));
			assert!(table_len.is_power_of_two());
			let base = cap as i64;
			for k in base..base + 10 * cap as i64 {
				cache.put(k, k);
				assert!(cache.nodes.len() <= cap);
			}
			assert_eq!(cache.index.slots.len(), table_len, "index resized during churn (cap={cap})");
			assert_eq!(cache.index.occupied(), cap);
			cache.check_segments();
		}
	}

	#[test]
	fn randomized_index_consistency_100k() {
		// Mixed put/get over a small key space; after every op, every resident key's
		// `find` returns its arena index and no slot is orphaned.
		let cap = 16usize;
		let mut cache = LruCache::<i64, i64>::new(cap);
		let mut x: u64 = 0xDEADBEEF;
		for _ in 0..100_000 {
			x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let k = ((x >> 33) % 48) as i64;
			if (x >> 20) & 1 == 0 {
				if cache.get(&k).is_none() {
					cache.put(k, k * 3);
				}
			} else {
				cache.put(k, k * 3);
			}
			for (idx, n) in cache.nodes.iter().enumerate() {
				assert_eq!(cache.index_of(&n.key), Some(idx as u32));
			}
			cache.check_index();
		}
		cache.check_segments();
	}

	#[test]
	fn live_slot_never_equals_empty() {
		assert_ne!(IndexTable::pack(u64::MAX, IDX_MASK - 1), EMPTY);
		assert_ne!(IndexTable::pack(u64::MAX, u32::MAX), EMPTY);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn capacity_at_index_limit_rejected() {
		assert!(LruCache::<i64, i64>::try_new(IDX_MASK as usize).is_none());
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

	#[test]
	fn header_not_larger_than_parent() {
		// Parent (HashMap-based) header was 112 bytes.
		assert!(std::mem::size_of::<LruCache<u64, u64>>() <= 112);
	}
}
