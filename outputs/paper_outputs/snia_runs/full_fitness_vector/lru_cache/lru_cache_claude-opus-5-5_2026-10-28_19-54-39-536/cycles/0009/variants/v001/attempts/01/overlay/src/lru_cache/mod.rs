// O(1) segmented-LRU (SLRU, 20% probation / 80% protected) cache using HashMap +
// arena-based doubly-linked lists. Nodes are stored in a Vec (arena) indexed by u32.
// On eviction the slot is reused in-place, so no free-list management is needed.
// No unsafe, no external crates.
//
// Links are `u32` arena indices. To keep `Node<u64, u64>` at exactly 24 bytes, two
// flags live in the top bits of `prev`: bit 31 is the segment flag (probation vs
// protected), bit 30 is the second-chance (reprieve) flag. The low 30 bits hold the
// index. `next` is a plain, unmasked index. `NIL` (= `IDX_MASK`) is the "no link"
// sentinel for both `prev` (index part) and `next`, and for the segment heads/tails.
//
// Policy:
// - A new key enters the probation segment at its MRU end.
// - A hit on a probation entry promotes it to the protected MRU (clearing any
//   second-chance flag); if the protected segment then exceeds `protected_cap`, its
//   LRU entry is demoted to probation MRU.
// - A hit on a protected entry moves it to the protected MRU.
// - Eviction takes the probation LRU, falling back to the protected LRU.
//
// Fingerprint reprieve (all capacities):
// - A direct-mapped ghost table of 8-bit tags remembers recently evicted keys.
//   Its length is `capacity.next_power_of_two()` for `capacity <= GHOST_MAX_CAPACITY`
//   and `(capacity / 4).next_power_of_two()` above that, so large caches get a
//   shorter reuse window (about capacity/4 evictions) instead of none.
// - A new key whose tag is found in the ghost is inserted with the second-chance
//   flag set (the ghost slot is cleared).
// - At eviction, if the probation LRU carries the second-chance flag, the flag is
//   cleared and the entry is moved to the probation MRU (at most once per put); the
//   victim is then chosen as usual.
//
// Ghost tags use a cheap one-multiply Fx-style hasher (`GhostHasher`), not the map's
// SipHash. The Fx multiply mixes upward, so the slot comes from the top bits and the
// tag from bits 16..24 (low bit forced to 1, so 0 still means "empty"). Collisions
// only cost an occasional spurious reprieve.
//
// Insert path: `put` performs a single `HashMap::entry` lookup. All ghost work
// (lookup, reprieve, victim-tag recording) runs only in the `Vacant` arm, using
// direct field access while the `VacantEntry` borrows `self.map`. `Occupied` puts do
// no ghost work. On eviction the new key is inserted before the victim's key is
// removed, so the map briefly holds `capacity + 1` entries; it is pre-sized to
// `capacity + 1` so this never resizes.

use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

/// Segment flag stored in bit 31 of `Node::prev`.
const PROT_BIT: u32 = 1 << 31;
/// Second-chance (reprieve) flag stored in bit 30 of `Node::prev`.
const SC_BIT: u32 = 1 << 30;
/// All flag bits of `Node::prev`.
const FLAGS: u32 = PROT_BIT | SC_BIT;
/// Mask extracting the index part of `Node::prev`.
const IDX_MASK: u32 = (1 << 30) - 1;
/// Sentinel "no link" index. Every valid arena index is strictly below this.
const NIL: u32 = IDX_MASK;
/// Largest capacity for which the ghost is full width (`capacity.next_power_of_two()`).
/// Above this the ghost is quarter width (`(capacity / 4).next_power_of_two()`).
const GHOST_MAX_CAPACITY: usize = 32768;

/// Fx multiplier.
const GHOST_K: u64 = 0x517c_c1b7_2722_0a95;

/// One-multiply Fx-style hasher used only for ghost fingerprints.
struct GhostHasher(u64);

impl GhostHasher {
	#[inline]
	fn step(&mut self, x: u64) {
		self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(GHOST_K);
	}
}

impl Hasher for GhostHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut b = [0u8; 8];
			b.copy_from_slice(c);
			self.step(u64::from_le_bytes(b));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut b = [0u8; 8];
			b[..rem.len()].copy_from_slice(rem);
			self.step(u64::from_le_bytes(b));
		}
	}

	#[inline]
	fn write_u8(&mut self, x: u8) {
		self.step(x as u64);
	}

	#[inline]
	fn write_u16(&mut self, x: u16) {
		self.step(x as u64);
	}

	#[inline]
	fn write_u32(&mut self, x: u32) {
		self.step(x as u64);
	}

	#[inline]
	fn write_u64(&mut self, x: u64) {
		self.step(x);
	}

	#[inline]
	fn write_usize(&mut self, x: usize) {
		self.step(x as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}
}

#[inline]
fn ghost_hash<K: Hash>(k: &K) -> u64 {
	let mut h = GhostHasher(0);
	k.hash(&mut h);
	h.finish()
}

/// Ghost slot (top `64 - shift` bits of the hash) and non-zero 8-bit tag
/// (bits 16..24 | 1). `shift == 64` (length-1 ghost) yields slot 0.
#[inline]
fn ghost_slot_tag(h: u64, shift: u32) -> (usize, u8) {
	(h.checked_shr(shift).unwrap_or(0) as usize, ((h >> 16) as u8) | 1)
}

/// Ghost table length for a given capacity.
#[inline]
fn ghost_len_for(capacity: usize) -> usize {
	if capacity <= GHOST_MAX_CAPACITY {
		capacity.next_power_of_two()
	} else {
		(capacity / 4).next_power_of_two()
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Bits 0..30: previous index (NIL if none). Bit 30: second-chance flag.
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
	fn has_second_chance(&self) -> bool {
		self.prev & SC_BIT != 0
	}

	/// Set the prev index, preserving the flags.
	#[inline]
	fn set_prev(&mut self, idx: u32) {
		debug_assert!(idx <= IDX_MASK);
		self.prev = (self.prev & FLAGS) | idx;
	}

	/// Set the segment flag. `true` is a single OR (callers must clear `SC_BIT`
	/// beforehand where a flagged node can become protected); `false` clears both
	/// flags.
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

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	ghost: Box<[u8]>,       // evicted-key fingerprints
	prob_head: u32,         // probation LRU (front), NIL when empty
	prob_tail: u32,         // probation MRU (back), NIL when empty
	prot_head: u32,         // protected LRU (front), NIL when empty
	prot_tail: u32,         // protected MRU (back), NIL when empty
	prob_len: u32,
	prot_len: u32,
	protected_cap: u32,
	ghost_shift: u32, // 64 - log2(ghost.len())
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
		let (ghost, ghost_shift): (Box<[u8]>, u32) = {
			let len = ghost_len_for(capacity);
			(vec![0u8; len].into_boxed_slice(), 64 - len.trailing_zeros())
		};
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} ghost_len={}",
			ghost.len()
		);
		Some(Self {
			// +1: during eviction the new key is inserted before the victim is removed.
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity),
			ghost,
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prob_len: 0,
			prot_len: 0,
			protected_cap,
			ghost_shift,
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
				// Key already exists: update value in-place and treat as a hit.
				// No ghost work on this path.
				let idx = *e.get();
				self.nodes[idx as usize].value = value;
				self.on_hit(idx);
				return;
			}
			Entry::Vacant(v) => {
				// `v` borrows `self.map`; below we touch only other fields directly.
				let ghost_on = !self.ghost.is_empty();
				let mut second_chance = false;
				if ghost_on {
					let (slot, tag) = ghost_slot_tag(ghost_hash(v.key()), self.ghost_shift);
					if self.ghost[slot] == tag {
						self.ghost[slot] = 0;
						second_chance = true;
					}
				}
				if self.nodes.len() < self.capacity {
					// Filling phase: len < capacity < IDX_MASK, so the cast is lossless
					let idx = self.nodes.len() as u32;
					self.nodes.push(Node {
						key: v.key().clone(),
						value,
						prev: if second_chance { NIL | SC_BIT } else { NIL },
						next: NIL,
					});
					v.insert(idx);
					self.push_back(idx, false);
				} else {
					if ghost_on {
						// Reprieve: a second-chance probation LRU moves to probation MRU
						// (at most one per put), with its flag cleared.
						let head = self.prob_head;
						if head != NIL && self.nodes[head as usize].has_second_chance() {
							let tail = self.prob_tail;
							if tail == head {
								// Sole probation node: it stays put, just clear the flag.
								self.nodes[head as usize].prev = NIL;
							} else {
								let next = self.nodes[head as usize].next;
								self.nodes[next as usize].set_prev(NIL);
								self.prob_head = next;
								self.nodes[tail as usize].next = head;
								let h = &mut self.nodes[head as usize];
								h.prev = tail; // probation, no flags
								h.next = NIL;
								self.prob_tail = head;
							}
						}
						// Record the victim's fingerprint (same choice as the eviction below).
						let victim = if self.prob_head != NIL {
							self.prob_head
						} else {
							self.prot_head
						};
						debug_assert!(victim != NIL, "a segment must be non-empty when arena is full");
						let (vslot, vtag) =
							ghost_slot_tag(ghost_hash(&self.nodes[victim as usize].key), self.ghost_shift);
						self.ghost[vslot] = vtag;
					}
					// Eviction phase: reuse the victim's slot in place.
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
					// Reused slot is reset to probation (clears both flags).
					self.nodes[victim as usize].set_protected(false);
					self.map.remove(&old_key);
					self.push_back(victim, false);
					if second_chance {
						self.nodes[victim as usize].prev |= SC_BIT;
					}
				}
			}
		}
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
			// Promote probation -> protected, clearing any second-chance flag.
			self.unlink(idx);
			self.nodes[idx as usize].prev &= !SC_BIT;
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
				if protected {
					assert!(!n.has_second_chance(), "protected node {cur} has second-chance flag");
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
		cache.ghost.fill(0);
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
		cache.ghost.fill(0);
		cache.put(4, 40);
		assert!(cache.get(&1).is_none());
		// Reused slot is reset to probation.
		assert!(!cache.nodes[idx1].is_protected());
		assert_eq!(cache.map[&4] as usize, idx1);
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![4]);
		assert_eq!(prot, vec![2, 3]);
		// Next eviction takes 4 (probation), protected keys survive.
		cache.ghost.fill(0);
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
			assert!(cache.map.len() <= 7);
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
		assert_eq!(IDX_MASK, (1u32 << 30) - 1);
		assert!(LruCache::<i64, i64>::try_new(IDX_MASK as usize).is_none());
		assert!(LruCache::<i64, i64>::try_new(IDX_MASK as usize + 1).is_none());
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
	fn header_fits_128_bytes() {
		assert!(std::mem::size_of::<LruCache<u64, u64>>() <= 128);
	}

	#[test]
	fn ghost_size_gating() {
		let c = LruCache::<i64, i64>::new(1000);
		assert_eq!(c.ghost.len(), 1024);
		assert_eq!(c.ghost_shift, 64 - 10);
		let c = LruCache::<i64, i64>::new(1);
		assert_eq!(c.ghost.len(), 1);
		assert_eq!(c.ghost_shift, 64);
		let c = LruCache::<i64, i64>::new(GHOST_MAX_CAPACITY);
		assert_eq!(c.ghost.len(), GHOST_MAX_CAPACITY);
		assert_eq!(c.ghost_shift, 64 - 15);
		let c = LruCache::<i64, i64>::new(GHOST_MAX_CAPACITY + 1);
		let expect = ((GHOST_MAX_CAPACITY + 1) / 4).next_power_of_two();
		assert_eq!(c.ghost.len(), expect);
		assert_eq!(c.ghost_shift, 64 - expect.trailing_zeros());
		let c = LruCache::<i64, i64>::new(131072);
		assert_eq!(c.ghost.len(), 32768);
		assert_eq!(c.ghost_shift, 64 - 15);
	}

	#[test]
	fn ghost_is_one_byte_per_slot() {
		let c = LruCache::<i64, i64>::new(131072);
		assert_eq!(std::mem::size_of_val(&*c.ghost), 32 * 1024);
		let c = LruCache::<i64, i64>::new(16384);
		assert_eq!(std::mem::size_of_val(&*c.ghost), 16 * 1024);
	}

	#[test]
	fn ghost_hasher_is_one_fx_step_for_i64() {
		let k: i64 = 0x1234_5678;
		assert_eq!(ghost_hash(&k), (k as u64).wrapping_mul(GHOST_K));
		// Byte-slice path: 8-byte chunks then zero-padded tail.
		let mut h = GhostHasher(0);
		h.write(&[1, 0, 0, 0, 0, 0, 0, 0, 2]);
		let mut e = GhostHasher(0);
		e.write_u64(1);
		e.write_u64(2);
		assert_eq!(h.finish(), e.finish());
	}

	#[test]
	fn ghost_slot_tag_in_range_and_nonzero() {
		for &len in &[1usize, 2, 1024, GHOST_MAX_CAPACITY, 65536] {
			let shift = 64 - len.trailing_zeros();
			for k in 0..5000i64 {
				let h = ghost_hash(&k);
				let (slot, tag): (usize, u8) = ghost_slot_tag(h, shift);
				assert!(slot < len);
				assert_ne!(tag, 0u8);
				assert_eq!(tag, ((h >> 16) as u8) | 1);
			}
		}
	}

	#[test]
	fn ghost_slots_spread_sequential_keys() {
		// Sequential keys must not pile into a few slots (top-bit slot selection).
		let len = 1024usize;
		let shift = 64 - len.trailing_zeros();
		let mut used = vec![false; len];
		for k in 0..len as i64 {
			used[ghost_slot_tag(ghost_hash(&k), shift).0] = true;
		}
		assert!(used.iter().filter(|&&u| u).count() > len / 2);
	}

	/// Put with the ghost wiped first, so stale fingerprints of other keys can
	/// never grant a spurious (hash-collision) second chance.
	fn put_clean(cache: &mut LruCache<i64, i64>, k: i64) {
		cache.ghost.fill(0);
		cache.put(k, k * 10);
	}

	#[test]
	fn reinserted_key_survives_exactly_one_extra_probation_eviction() {
		let mut cache = LruCache::<i64, i64>::new(4); // all keys stay in probation
		for k in 1..=4 {
			put_clean(&mut cache, k);
		}
		put_clean(&mut cache, 5); // evicts 1, records its fingerprint
		assert!(cache.map.get(&1).is_none());
		cache.put(1, 10); // ghost hit -> second chance; evicts 2
		let idx1 = cache.map[&1] as usize;
		assert!(cache.nodes[idx1].has_second_chance());
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![3, 4, 5, 1]);
		put_clean(&mut cache, 6); // evicts 3
		put_clean(&mut cache, 7); // evicts 4
		put_clean(&mut cache, 8); // evicts 5
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![1, 6, 7, 8]);
		put_clean(&mut cache, 9); // 1 is reprieved to MRU, 6 evicted instead
		assert!(cache.map.get(&6).is_none());
		assert!(!cache.nodes[cache.map[&1] as usize].has_second_chance());
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![7, 8, 1, 9]);
		put_clean(&mut cache, 10); // evicts 7
		put_clean(&mut cache, 11); // evicts 8
		assert!(cache.map.contains_key(&1));
		put_clean(&mut cache, 12); // no second reprieve: 1 evicted
		assert!(cache.map.get(&1).is_none());
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![9, 10, 11, 12]);
	}

	#[test]
	fn promotion_clears_second_chance() {
		let mut cache = LruCache::<i64, i64>::new(4); // protected_cap = 3
		for k in 1..=5 {
			put_clean(&mut cache, k); // 5 evicts 1
		}
		cache.put(1, 10); // second chance
		let idx1 = cache.map[&1] as usize;
		assert!(cache.nodes[idx1].has_second_chance());
		cache.get(&1); // promote
		assert!(cache.nodes[idx1].is_protected());
		assert!(!cache.nodes[idx1].has_second_chance());
		// Overflow protected so 1 gets demoted; flag must stay clear.
		cache.get(&3);
		cache.get(&4);
		cache.get(&5); // protected: 1,3,4,5 > 3 -> 1 demoted
		assert!(!cache.nodes[idx1].is_protected());
		assert!(!cache.nodes[idx1].has_second_chance());
		cache.check_segments();
	}

	#[test]
	fn reprieve_of_sole_probation_node_clears_flag() {
		let mut cache = LruCache::<i64, i64>::new(4); // protected_cap = 3
		for k in 1..=4 {
			put_clean(&mut cache, k);
		}
		put_clean(&mut cache, 5); // evicts 1, records its fingerprint
		cache.put(1, 10); // second chance; evicts 2. probation: 3,4,5,1
		cache.get(&3);
		cache.get(&4);
		cache.get(&5); // protected: 3,4,5; probation: 1 (flagged)
		let (prob, _) = cache.check_segments();
		assert_eq!(prob, vec![1]);
		assert!(cache.nodes[cache.map[&1] as usize].has_second_chance());
		put_clean(&mut cache, 6); // reprieve clears flag, 1 still the victim
		assert!(cache.map.get(&1).is_none());
		let (prob, prot) = cache.check_segments();
		assert_eq!(prob, vec![6]);
		assert_eq!(prot, vec![3, 4, 5]);
	}

	#[test]
	fn occupied_put_leaves_ghost_untouched() {
		let mut cache = LruCache::<i64, i64>::new(4);
		for k in 1..=6 {
			put_clean(&mut cache, k); // evictions populate the ghost
		}
		assert!(cache.map.contains_key(&3) && cache.map.contains_key(&6));
		// Plant the resident key's own fingerprint so a lookup would clear it.
		let (slot, tag): (usize, u8) = ghost_slot_tag(ghost_hash(&3i64), cache.ghost_shift);
		cache.ghost[slot] = tag;
		let before: Box<[u8]> = cache.ghost.clone();
		cache.put(3, 33); // Occupied
		cache.put(6, 66); // Occupied
		assert_eq!(cache.ghost, before);
		assert_eq!(*cache.get(&3).unwrap(), 33);
		cache.check_segments();
	}

	/// Test-only spurious-reprieve counter. Replays a get-miss/put trace against
	/// `cache` while a shadow table records which truly evicted key last wrote each
	/// ghost slot. A ghost tag match on put is genuine if the slot was written by
	/// the eviction of that same key (and not since overwritten or consumed);
	/// otherwise it is spurious (tag collision). Returns (spurious, genuine).
	/// Nothing here touches the release hot path.
	fn count_reprieves(cache: &mut LruCache<i64, i64>, keys: &[i64]) -> (u64, u64) {
		use std::collections::HashSet;
		let mut owner: Vec<Option<i64>> = vec![None; cache.ghost.len()];
		let mut evicted: HashSet<i64> = HashSet::new();
		let (mut spurious, mut genuine) = (0u64, 0u64);
		for &k in keys {
			if cache.get(&k).is_some() {
				continue;
			}
			let shift = cache.ghost_shift;
			let (slot, tag) = ghost_slot_tag(ghost_hash(&k), shift);
			if cache.ghost[slot] == tag {
				if owner[slot] == Some(k) && evicted.contains(&k) {
					genuine += 1;
				} else {
					spurious += 1;
				}
				owner[slot] = None;
			}
			// Predict the victim exactly as `put` will choose it.
			let victim = if cache.nodes.len() < cache.capacity {
				None
			} else {
				let head = cache.prob_head;
				let v = if head != NIL {
					if cache.nodes[head as usize].has_second_chance() && head != cache.prob_tail {
						cache.nodes[head as usize].next
					} else {
						head
					}
				} else {
					cache.prot_head
				};
				Some(cache.nodes[v as usize].key)
			};
			cache.put(k, k);
			evicted.remove(&k);
			if let Some(vk) = victim {
				assert!(!cache.map.contains_key(&vk), "victim prediction wrong");
				let (vslot, _) = ghost_slot_tag(ghost_hash(&vk), shift);
				owner[vslot] = Some(vk);
				evicted.insert(vk);
			}
		}
		(spurious, genuine)
	}

	#[test]
	fn spurious_reprieves_are_rare_with_8bit_tags() {
		let cap = 1024usize;
		let mut cache = LruCache::<i64, i64>::new(cap);
		let distinct = 4 * cap as u64;
		let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
		let keys: Vec<i64> = (0..200_000)
			.map(|_| {
				x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				((x >> 33) % distinct) as i64 * 7919 + 13
			})
			.collect();
		let (spurious, genuine) = count_reprieves(&mut cache, &keys);
		assert!(genuine > 0, "no genuine reprieves observed");
		assert!(
			spurious * 16 < genuine,
			"spurious/genuine reprieve ratio too high: {spurious}/{genuine}"
		);
		cache.check_segments();
	}

	#[test]
	fn large_cache_reinserted_key_gets_second_chance() {
		let cap: usize = 131072;
		let mut cache = LruCache::<i64, i64>::new(cap);
		assert_eq!(cache.ghost.len(), cap / 4);
		assert_eq!(cache.ghost_shift, 64 - (cap / 4).trailing_zeros());
		let c = cap as i64;
		// Filling phase: the ghost is empty, so no second chances are granted.
		for k in 0..c {
			cache.put(k, k);
		}
		assert!(cache.nodes.iter().all(|n| !n.has_second_chance()));
		cache.put(c, c); // evicts 0, records its fingerprint
		assert!(cache.map.get(&0).is_none());
		cache.put(0, 0); // re-insert: ghost hit -> second chance, evicts 1
		assert!(cache.nodes[cache.map[&0] as usize].has_second_chance());
		assert!(cache.map.get(&1).is_none());
		// Probation (all keys): 2..c, c, 0. Evict the c - 1 keys older than 0.
		// Newer keys sit behind 0, so their own (possibly spurious) flags cannot
		// reach the head before 0 does.
		for k in (c + 1)..(c + 1 + (c - 1)) {
			cache.put(k, k);
		}
		assert!(cache.map.contains_key(&0));
		assert_eq!(cache.nodes[cache.prob_head as usize].key, 0);
		// Next eviction reprieves 0 to probation MRU; key c + 1 is evicted instead.
		cache.put(3 * c, 0);
		assert!(cache.map.contains_key(&0));
		assert!(!cache.nodes[cache.map[&0] as usize].has_second_chance());
		assert!(cache.map.get(&(c + 1)).is_none());
		assert_eq!(cache.nodes[cache.prob_tail as usize].key, 3 * c);
		assert_eq!(cache.map.len(), cap);
	}
}
