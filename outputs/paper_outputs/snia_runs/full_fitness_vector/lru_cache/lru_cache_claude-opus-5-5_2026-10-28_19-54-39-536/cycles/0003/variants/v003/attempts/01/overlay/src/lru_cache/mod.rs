// O(1) segmented-LRU (SLRU, 20% probation / 80% protected) cache using HashMap +
// arena-based doubly-linked lists, fronted by a TinyLFU-style admission filter.
// Nodes are stored in a Vec (arena) indexed by u32. On eviction the slot is reused
// in-place, so no free-list management is needed. No unsafe, no external crates.
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
// Admission (TinyLFU):
// - Every `put` of a new key (the miss path) increments that key's counters in a
//   4-row count-min sketch of 4-bit (saturating at 15) counters. `get` never
//   touches the sketch.
// - When the arena is full and the victim would be the probation LRU, the candidate
//   is admitted only if its estimated frequency is strictly greater than the
//   victim's (or both estimates are 0). Otherwise the put is dropped: the victim
//   stays and the map, lists and arena are unchanged.
// - If the victim would come from protected (probation empty), or the arena is not
//   yet full, the candidate is always admitted.
// - Aging: after `reset_at = 10 * capacity` additions all counters are halved.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

/// Segment flag stored in bit 31 of `Node::prev`.
const PROT_BIT: u32 = 1 << 31;
/// Mask extracting the index part of `Node::prev`.
const IDX_MASK: u32 = PROT_BIT - 1;
/// Sentinel "no link" index. Every valid arena index is strictly below this.
const NIL: u32 = IDX_MASK;

/// Number of rows in the count-min sketch.
const SKETCH_ROWS: usize = 4;
/// Counter saturation value (4-bit counters stored in a byte).
const SKETCH_MAX: u8 = 15;

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

/// Deterministic 64-bit hash of a key, shared by all sketch operations.
#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

/// Compact count-min sketch: 4 rows of `width` saturating 4-bit counters (one per
/// byte), stored row-major in a single flat `Vec<u8>`.
#[derive(Debug)]
struct FreqSketch {
	table: Vec<u8>,
	mask: usize,
	additions: u32,
	reset_at: u32,
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.max(1).next_power_of_two();
		let reset_at = capacity.saturating_mul(10).min(u32::MAX as usize).max(1) as u32;
		Self {
			table: vec![0u8; SKETCH_ROWS * width],
			mask: width - 1,
			additions: 0,
			reset_at,
		}
	}

	/// Flat-table position of the counter for hash `h` in row `r`.
	#[inline]
	fn slot(&self, h: u64, r: usize) -> usize {
		let mixed = (h.rotate_left(16 * r as u32) ^ (h >> 32)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		let col = (mixed >> 32) as usize & self.mask;
		r * (self.mask + 1) + col
	}

	/// Increment all 4 counters for `h` (saturating at 15), then age if due.
	fn increment(&mut self, h: u64) {
		for r in 0..SKETCH_ROWS {
			let s = self.slot(h, r);
			let c = &mut self.table[s];
			if *c < SKETCH_MAX {
				*c += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.reset_at {
			self.halve();
		}
	}

	/// Count-min estimate: the minimum of the 4 counters for `h`.
	#[inline]
	fn estimate(&self, h: u64) -> u8 {
		let mut m = SKETCH_MAX;
		for r in 0..SKETCH_ROWS {
			let c = self.table[self.slot(h, r)];
			if c < m {
				m = c;
			}
		}
		m
	}

	fn halve(&mut self) {
		for c in self.table.iter_mut() {
			*c >>= 1;
		}
		self.additions /= 2;
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
	sketch: FreqSketch,
	/// Test-only switch to bypass admission so pure-SLRU mechanics can be tested
	/// deterministically.
	#[cfg(test)]
	admit_all: bool,
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
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prob_len: 0,
			prot_len: 0,
			protected_cap,
			capacity,
			sketch: FreqSketch::new(capacity),
			#[cfg(test)]
			admit_all: false,
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

	#[cfg(test)]
	#[inline]
	fn admission_enabled(&self) -> bool {
		!self.admit_all
	}

	#[cfg(not(test))]
	#[inline(always)]
	fn admission_enabled(&self) -> bool {
		true
	}

	/// Inserts or updates `key`. If at capacity, the candidate must win the
	/// TinyLFU admission check against the probation LRU; if admitted, that entry
	/// (or the protected LRU when probation is empty) is evicted.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a hit (no sketch).
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.on_hit(idx);
			return;
		}
		// New key (miss path): record its frequency.
		let h = hash_key(&key);
		self.sketch.increment(h);
		// Need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			let victim = if self.prob_head != NIL {
				let v = self.prob_head;
				if self.admission_enabled() {
					let cand_est = self.sketch.estimate(h);
					let victim_est = self.sketch.estimate(hash_key(&self.nodes[v as usize].key));
					if cand_est <= victim_est && !(cand_est == 0 && victim_est == 0) {
						// Rejected: cache state is left untouched.
						return;
					}
				}
				v
			} else {
				self.prot_head
			};
			debug_assert!(victim != NIL, "a segment must be non-empty when arena is full");
			let evicted_key = self.nodes[victim as usize].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let node = &mut self.nodes[victim as usize];
			node.key = key.clone();
			node.value = value;
			// Reused slot is reset to probation.
			node.set_protected(false);
			victim
		} else {
			// len < capacity < IDX_MASK, so the cast is lossless
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
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

	/// A cache with admission bypassed, for testing the pure SLRU mechanics
	/// (eviction order, promotion, demotion, slot reuse) deterministically.
	fn slru(capacity: usize) -> LruCache<i64, i64> {
		let mut c = LruCache::new(capacity);
		c.admit_all = true;
		c
	}

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
		let mut cache = slru(2);
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
		let mut cache = slru(2);
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
		let mut cache = slru(2);
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
		let mut cache = slru(4); // protected_cap = 3
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
		let mut cache = slru(3); // protected_cap = 2
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
	fn eviction_chain() {
		let mut cache = slru(3);
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
		let mut cache = slru(1); // protected_cap = 0
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
		let mut cache = slru(2); // protected_cap = 1
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
				let before = cache.map.len();
				cache.put(k, k * 10);
				// Either admitted (+1 or evict-and-replace) or rejected (unchanged).
				assert!(cache.map.len() == before || cache.map.len() == before + 1);
			}
			assert!(cache.map.len() <= 7);
			cache.check_segments();
		}
		cache.check_segments();
	}

	#[test]
	fn random_workload_without_admission_stays_consistent() {
		let mut cache = slru(7);
		let mut x: u64 = 999;
		for _ in 0..5000 {
			x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let k = ((x >> 33) % 20) as i64;
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.map.len() <= 7);
		}
		cache.check_segments();
	}

	#[test]
	fn one_hit_scan_cannot_displace_frequent_key() {
		let cap = 64;
		let mut cache = LruCache::new(cap);
		for k in 0..cap as i64 {
			cache.put(k, k);
		}
		// Key 0 is the probation LRU. Give it a history of many puts (it was put,
		// evicted and re-put several times in the past).
		let h0 = hash_key(&0i64);
		for _ in 0..SKETCH_MAX {
			cache.sketch.increment(h0);
		}
		assert_eq!(cache.sketch.estimate(h0), SKETCH_MAX);
		let (prob, _) = cache.check_segments();
		assert_eq!(prob[0], 0);
		// A one-hit scan: each candidate's estimate stays below key 0's.
		for k in 1000..1100i64 {
			let before = cache.map.len();
			cache.put(k, k);
			assert_eq!(cache.map.len(), before, "rejection must leave map.len unchanged");
			assert!(cache.get(&k).is_none(), "scan key {k} displaced the frequent key");
			cache.check_segments();
		}
		assert_eq!(*cache.get(&0).unwrap(), 0);
		cache.check_segments();
	}

	#[test]
	fn rejection_leaves_state_unchanged() {
		let mut cache = LruCache::new(16);
		for k in 0..16i64 {
			cache.put(k, k);
		}
		let h0 = hash_key(&0i64);
		for _ in 0..SKETCH_MAX {
			cache.sketch.increment(h0);
		}
		let (prob_before, prot_before) = cache.check_segments();
		let len_before = cache.map.len();
		cache.put(500, 500);
		let (prob_after, prot_after) = cache.check_segments();
		assert_eq!(cache.map.len(), len_before);
		assert_eq!(prob_before, prob_after);
		assert_eq!(prot_before, prot_after);
		assert!(cache.get(&500).is_none());
	}

	#[test]
	fn cold_cache_always_admits() {
		// While the arena is not full there is no victim, so every key is admitted.
		// (When full, probation always holds >= capacity - protected_cap >= 1
		// entries, so the protected-victim fallback is a defensive path.)
		let mut cold = LruCache::new(8);
		for k in 0..8i64 {
			cold.put(k, k);
			assert_eq!(*cold.get(&k).unwrap(), k, "cold cache must admit {k}");
		}
		cold.check_segments();
	}

	#[test]
	fn sketch_halves_after_reset_at_additions() {
		let mut s = FreqSketch::new(4);
		assert_eq!(s.reset_at, 40);
		let h = hash_key(&7i64);
		for _ in 0..39 {
			s.increment(h);
		}
		assert_eq!(s.estimate(h), SKETCH_MAX);
		assert_eq!(s.additions, 39);
		s.increment(h); // 40th addition triggers aging
		assert_eq!(s.estimate(h), SKETCH_MAX >> 1);
		assert_eq!(s.additions, 20);
		assert!(s.table.iter().all(|&c| c <= SKETCH_MAX >> 1));
	}

	#[test]
	fn sketch_counters_saturate() {
		let mut s = FreqSketch::new(1024);
		let h = hash_key(&42i64);
		for _ in 0..100 {
			s.increment(h);
		}
		assert_eq!(s.estimate(h), SKETCH_MAX);
		assert!(s.table.iter().all(|&c| c <= SKETCH_MAX));
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
}
