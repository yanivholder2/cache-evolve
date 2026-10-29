// O(1) segmented-LRU (SLRU, 20% probation / 80% protected) cache using HashMap +
// arena-based doubly-linked lists, with a fingerprint ghost giving re-inserted keys
// a one-shot "second chance" at probation eviction. Nodes are stored in a Vec (arena)
// indexed by u32. On eviction the slot is reused in-place, so no free-list management
// is needed. No unsafe, no external crates.
//
// Links are `u32` arena indices. To keep `Node<u64, u64>` at exactly 24 bytes, the
// segment flag (probation vs protected) lives in bit 31 of `prev` and the
// second-chance flag lives in bit 31 of `next`; the low 31 bits of each hold the
// index. All link reads/writes go through the accessors on `Node`, so neither flag
// ever leaks into an index. `NIL` (= `IDX_MASK`) is the "no link" sentinel for both
// links and for the segment heads/tails.
//
// Policy:
// - A new key enters the probation segment at its MRU end. If its 16-bit fingerprint
//   is found in the direct-mapped ghost array (i.e. it was recently evicted), its
//   second-chance bit is set and the ghost slot is cleared.
// - A hit on a probation entry promotes it to the protected MRU (clearing the
//   second-chance bit); if the protected segment then exceeds `protected_cap`, its
//   LRU entry is demoted to probation MRU.
// - A hit on a protected entry moves it to the protected MRU.
// - Eviction takes the probation LRU, falling back to the protected LRU. If the
//   probation LRU has its second-chance bit, the bit is cleared and that node is
//   moved to probation MRU once; the new probation LRU is then evicted regardless.
// - The evicted key's fingerprint is written to its ghost slot (lossy, bounded).
// - The ghost is never touched on `get`.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub mod lru_cache_jni;

/// Segment flag stored in bit 31 of `Node::prev`.
const PROT_BIT: u32 = 1 << 31;
/// Second-chance flag stored in bit 31 of `Node::next`.
const SC_BIT: u32 = 1 << 31;
/// Mask extracting the index part of `Node::prev` / `Node::next`.
const IDX_MASK: u32 = PROT_BIT - 1;
/// Sentinel "no link" index. Every valid arena index is strictly below this.
const NIL: u32 = IDX_MASK;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Bits 0..31: previous index (NIL if none). Bit 31: protected-segment flag.
	prev: u32,
	/// Bits 0..31: next index (NIL if none). Bit 31: second-chance flag.
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
		self.next & IDX_MASK
	}

	/// Set the next index, preserving the second-chance flag.
	#[inline]
	fn set_next(&mut self, idx: u32) {
		debug_assert!(idx <= IDX_MASK);
		self.next = (self.next & SC_BIT) | (idx & IDX_MASK);
	}

	#[inline]
	fn has_second_chance(&self) -> bool {
		self.next & SC_BIT != 0
	}

	#[inline]
	fn set_second_chance(&mut self, sc: bool) {
		if sc {
			self.next |= SC_BIT;
		} else {
			self.next &= IDX_MASK;
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	/// Direct-mapped fingerprint ghost of recently evicted keys. 0 = empty.
	ghost: Box<[u16]>,
	prob_head: u32, // probation LRU (front), NIL when empty
	prob_tail: u32, // probation MRU (back), NIL when empty
	prot_head: u32, // protected LRU (front), NIL when empty
	prot_tail: u32, // protected MRU (back), NIL when empty
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
		let ghost_len = capacity.next_power_of_two();
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} ghost_len={ghost_len}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			ghost: vec![0u16; ghost_len].into_boxed_slice(),
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

	/// Ghost (slot, tag) for a key hash. The tag is never 0.
	#[inline]
	fn ghost_slot_tag(&self, h: u64) -> (usize, u16) {
		let slot = (h as usize) & (self.ghost.len() - 1);
		let tag = ((h >> 48) as u16) | 1;
		(slot, tag)
	}

	/// Returns the value for `key` if present, applying the SLRU hit policy.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.on_hit(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probation LRU entry
	/// (falling back to the protected LRU), honouring a one-shot second chance.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a hit
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.on_hit(idx);
			return;
		}
		// Fresh key: consult the ghost (miss path only).
		let (slot, tag) = self.ghost_slot_tag(self.map.hasher().hash_one(&key));
		let second_chance = self.ghost[slot] == tag;
		if second_chance {
			self.ghost[slot] = 0;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			if self.prob_head != NIL && self.nodes[self.prob_head as usize].has_second_chance() {
				// One-shot reprieve: move to probation MRU; skip at most one node.
				let h = self.prob_head;
				self.nodes[h as usize].set_second_chance(false);
				self.unlink(h);
				self.push_back(h, false);
			}
			let victim = if self.prob_head != NIL {
				self.prob_head
			} else {
				self.prot_head
			};
			debug_assert!(victim != NIL, "a segment must be non-empty when arena is full");
			let evicted_key = self.nodes[victim as usize].key.clone();
			let (eslot, etag) = self.ghost_slot_tag(self.map.hasher().hash_one(&evicted_key));
			self.ghost[eslot] = etag;
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let node = &mut self.nodes[victim as usize];
			node.key = key.clone();
			node.value = value;
			// Reused slot is reset to probation; second chance set from the ghost.
			node.set_protected(false);
			node.set_second_chance(second_chance);
			victim
		} else {
			// len < capacity < IDX_MASK, so the cast is lossless
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: if second_chance { NIL | SC_BIT } else { NIL },
			});
			idx
		};
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.map.len());
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
		debug_assert_eq!((self.prob_len + self.prot_len) as usize, self.map.len());
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
	/// Promotion to protected clears the second-chance bit.
	#[inline]
	fn push_back(&mut self, idx: u32, protected: bool) {
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		{
			let node = &mut self.nodes[idx as usize];
			node.set_protected(protected);
			node.set_prev(tail);
			if protected {
				node.next = NIL; // clears SC_BIT
			} else {
				node.set_next(NIL);
			}
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
		let n_nodes = self.nodes.len() as u32;
		for n in &self.nodes {
			let (p, nx) = (n.prev_idx(), n.next_idx());
			assert!(p == NIL || p < n_nodes, "prev index out of range: {p}");
			assert!(nx == NIL || nx < n_nodes, "next index out of range: {nx}");
		}
		let walk = |head: u32, tail: u32, protected: bool, len: u32| -> Vec<K> {
			let mut out = Vec::new();
			let mut prev = NIL;
			let mut cur = head;
			while cur != NIL {
				assert!(cur < n_nodes, "link index out of range: {cur}");
				let n = &self.nodes[cur as usize];
				assert_eq!(n.is_protected(), protected, "segment flag mismatch at {cur}");
				if protected {
					assert!(!n.has_second_chance(), "protected node with SC bit at {cur}");
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
	fn ghost_of(&self, key: &K) -> (usize, u16) {
		self.ghost_slot_tag(self.map.hasher().hash_one(key))
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
		// Re-insert 1 (likely ghost hit): with one slot, SC cannot save the head.
		cache.put(1, 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 11);
		cache.put(3, 30);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
	fn reinserted_key_gets_second_chance_once() {
		let mut cache = LruCache::new(3); // protected_cap = 2
		// Protect 100 and 200 so probation holds a single slot.
		cache.put(100, 0);
		cache.get(&100);
		cache.put(200, 0);
		cache.get(&200);
		cache.put(1, 10); // probation: [1]
		cache.put(2, 20); // evicts 1 -> ghost
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![2]);
		// Re-insert 1: evicts 2 (no SC); 1 enters probation with SC.
		cache.put(1, 11);
		let idx1 = cache.map[&1] as usize;
		assert!(cache.nodes[idx1].has_second_chance());
		assert!(!cache.nodes[idx1].is_protected());
		// With a single probation node the reprieve cannot help: 1 is the only
		// candidate, so it is evicted anyway (bit cleared first).
		cache.put(3, 30);
		assert!(cache.get(&1).is_none());
		cache.check_segments();
	}

	#[test]
	fn second_chance_survives_exactly_one_extra_round() {
		let mut cache = LruCache::new(4); // protected_cap = 3
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(5, 50); // evicts 1 -> ghost
		assert!(cache.map.get(&1).is_none());
		cache.put(1, 11); // evicts 2; 1 enters probation MRU with SC
		let idx1 = cache.map[&1] as usize;
		assert!(cache.nodes[idx1].has_second_chance());
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![3, 4, 5, 1]);
		cache.put(6, 60); // evicts 3
		cache.put(7, 70); // evicts 4
		cache.put(8, 80); // evicts 5
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![1, 6, 7, 8]);
		// 1 is now probation LRU with SC: reprieved, 6 evicted instead.
		cache.put(9, 90);
		assert!(cache.map.get(&6).is_none());
		assert!(!cache.nodes[cache.map[&1] as usize].has_second_chance());
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![7, 8, 1, 9]);
		cache.put(10, 100); // evicts 7
		cache.put(11, 110); // evicts 8
		cache.put(12, 120); // now evicts 1 (no second reprieve)
		assert!(cache.map.get(&1).is_none());
		cache.check_segments();
	}

	#[test]
	fn ghost_tag_cleared_on_use() {
		let mut cache = LruCache::new(64);
		for k in 0..64 {
			cache.put(k, k);
		}
		cache.put(1000, 0); // evicts 0
		let (slot, tag) = cache.ghost_of(&0);
		assert_eq!(cache.ghost[slot], tag);
		cache.put(0, 1); // ghost hit (evicts 1, writes 1's tag elsewhere or here)
		let idx0 = cache.map[&0] as usize;
		assert!(cache.nodes[idx0].has_second_chance());
		assert_ne!(cache.ghost[slot], tag, "ghost tag must be cleared on use");
		cache.check_segments();
	}

	#[test]
	fn promotion_clears_second_chance() {
		let mut cache = LruCache::new(4);
		for k in 1..=4 {
			cache.put(k, k);
		}
		cache.put(5, 5); // evicts 1
		cache.put(1, 1); // SC
		let idx1 = cache.map[&1] as usize;
		assert!(cache.nodes[idx1].has_second_chance());
		cache.get(&1); // promoted
		assert!(cache.nodes[idx1].is_protected());
		assert!(!cache.nodes[idx1].has_second_chance());
		cache.check_segments();
	}

	#[test]
	fn random_workload_keeps_segments_consistent() {
		for &cap in &[1usize, 2, 7, 33] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 12345 + cap as u64;
			for i in 0..20000 {
				x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let k = ((x >> 33) % (cap as u64 * 3 + 2)) as i64;
				match cache.get(&k) {
					Some(&v) => assert_eq!(v, k * 10),
					None => cache.put(k, k * 10),
				}
				assert!(cache.map.len() <= cap);
				if i % 997 == 0 {
					cache.check_segments();
				}
			}
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
	fn ghost_len_is_power_of_two() {
		assert_eq!(LruCache::<i64, i64>::new(1).ghost.len(), 1);
		assert_eq!(LruCache::<i64, i64>::new(5).ghost.len(), 8);
		assert_eq!(LruCache::<i64, i64>::new(2048).ghost.len(), 2048);
	}

	#[test]
	fn node_u64_is_24_bytes() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}
}
