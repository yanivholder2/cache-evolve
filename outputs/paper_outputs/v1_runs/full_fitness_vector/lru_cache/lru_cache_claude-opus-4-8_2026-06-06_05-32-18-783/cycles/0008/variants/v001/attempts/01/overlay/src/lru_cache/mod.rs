// O(1) segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// The recency order is split into two segments that share the same arena:
//   * probationary — where brand-new (one-hit-wonder) entries live; eviction
//     victims are taken from here first.
//   * protected    — where entries that have been referenced again are promoted;
//     these are shielded from probationary eviction.
// A node's segment flag selects which segment its links belong to, so the
// O(1) unlink/push_back helpers operate on whichever segment owns the node.
//
// Hashing uses a zero-sized `FastHasher` (a single xor + wrapping-multiply over
// the 8-byte integer key) instead of SipHash, cutting per-lookup hash cost. A
// fast path on get/put-update avoids redundant list surgery when the touched
// node is already the most-recently-used node of its segment.
//
// Recency links are encoded as bare `u32` indices with a `NIL` sentinel instead
// of `Option<usize>`. A no-niche `Option<usize>` costs 16 bytes; the two links
// dominated the node footprint, so narrowing both to 4-byte words (and packing
// the segment flag into the high bit of `prev`) drops `Node<u64,u64>` from 56
// bytes into the 24-byte class. Arena indices never approach `2^31` at the
// harness's capacities, so 31 bits of index space plus a sentinel is ample.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

// Multiplicative mixing constant (Fibonacci-derived 64-bit odd prime).
const FAST_HASH_MULT: u64 = 0x517cc1b727220a95;

/// "No link" sentinel for the `u32` recency links. Lives in the low-31-bit
/// index space (the high bit is reserved for the segment flag on `prev`), so
/// it can never collide with a real arena index at harness capacities.
const NIL: u32 = u32::MAX >> 1; // 0x7FFF_FFFF

/// Segment flag packed into the high bit of a node's `prev` link:
/// set => protected segment, clear => probationary.
const PROT_MASK: u32 = 1 << 31; // 0x8000_0000

/// Decode a stored link word into an optional arena index, mapping `NIL` to
/// `None`. On the 64-bit target this is a free zero-extend.
#[inline]
fn slot(value: u32) -> Option<usize> {
	if value == NIL {
		None
	} else {
		Some(value as usize)
	}
}

/// Encode an arena index into a stored link word. Indices stay well under
/// `2^31` at harness capacities, so the high (flag) bit is always clear.
#[inline]
fn link(index: usize) -> u32 {
	index as u32
}

/// Zero-sized, non-cryptographic hasher tuned for small integer keys: each
/// written word is xor-folded into the state and mixed with one wrapping
/// multiply. Keys here are 8-byte integers, so a single op suffices.
#[derive(Default)]
pub struct FastHasher {
	state: u64,
}

impl Hasher for FastHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.state
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.state = (self.state ^ b as u64).wrapping_mul(FAST_HASH_MULT);
		}
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.state = (self.state ^ i).wrapping_mul(FAST_HASH_MULT);
	}

	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.state = (self.state ^ i as u64).wrapping_mul(FAST_HASH_MULT);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.state = (self.state ^ i as u64).wrapping_mul(FAST_HASH_MULT);
	}
}

/// `BuildHasher` that constructs a fresh zero-sized `FastHasher` per hash.
pub type FastBuildHasher = BuildHasherDefault<FastHasher>;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	// Recency links as bare `u32` indices (`NIL` == no link). `prev` additionally
	// packs the segment flag in its high bit (`PROT_MASK`): set => protected,
	// clear => probationary. The link portion is the low 31 bits. All access goes
	// through the helpers below so the bodies stay a free zero-extend with no
	// stray `as usize` casts and never clobber the packed flag.
	prev: u32,
	next: u32,
}

impl<K, V> Node<K, V> {
	/// True if this node currently belongs to the protected segment.
	#[inline]
	fn protected(&self) -> bool {
		self.prev & PROT_MASK != 0
	}

	/// Move this node's segment flag without disturbing its `prev` link.
	#[inline]
	fn set_protected(&mut self, protected: bool) {
		if protected {
			self.prev |= PROT_MASK;
		} else {
			self.prev &= !PROT_MASK;
		}
	}

	/// The `prev` link word with the segment flag masked off.
	#[inline]
	fn prev_link(&self) -> u32 {
		self.prev & !PROT_MASK
	}

	/// Overwrite the `prev` link, preserving the packed segment flag.
	#[inline]
	fn set_prev_link(&mut self, value: u32) {
		self.prev = (self.prev & PROT_MASK) | (value & !PROT_MASK);
	}

	/// The `next` link word (never carries the segment flag).
	#[inline]
	fn next_link(&self) -> u32 {
		self.next
	}

	/// Overwrite the `next` link.
	#[inline]
	fn set_next_link(&mut self, value: u32) {
		self.next = value;
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                  // arena
	prot_head: u32,                          // protected LRU (front), NIL if empty
	prot_tail: u32,                          // protected MRU (back), NIL if empty
	prob_head: u32,                          // probationary LRU (front), NIL if empty
	prob_tail: u32,                          // probationary MRU (back), NIL if empty
	prot_len: usize,                         // number of resident protected nodes
	prot_cap: usize,                         // soft cap on protected nodes
	capacity: usize,
	// Single-entry last-key get cache. When the hottest key sits at the
	// protected MRU, a repeated get returns its value directly, skipping the
	// HashMap hash-and-probe. `last_idx` is `NIL` when unprimed. These fields
	// live on the cache struct (not on `Node`), so `Node`'s footprint is
	// unchanged; only the `LruCache` struct grows. No explicit invalidation is
	// needed on eviction/demotion: a stale entry is rejected by the
	// `prot_tail == last_idx` and `nodes[last].key == *key` guards in `get`,
	// which fall through to the normal map lookup.
	last_key: Option<K>,
	last_idx: u32,
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
		// Protected segment gets ~7/8 of capacity, but never the whole cache: at
		// least one slot is reserved for probation (clamp to capacity - 1) and the
		// cap is at least 1 where geometry allows.
		let prot_cap = (capacity * 7 / 8).max(1).min(capacity.saturating_sub(1));
		log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			prot_head: NIL,
			prot_tail: NIL,
			prob_head: NIL,
			prob_tail: NIL,
			prot_len: 0,
			prot_cap,
			capacity,
			last_key: None,
			last_idx: NIL,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it toward most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Single-entry fast path: when the previously primed key is still the
		// protected MRU and its slot has not been reused by eviction, return its
		// value without touching the HashMap. The `prot_tail == last_idx` test
		// guarantees the access would change no links, and `nodes[last].key`
		// guards against the slot having been reused for another key.
		if self.last_idx != NIL && self.last_key.as_ref() == Some(key) {
			let last = self.last_idx as usize;
			if self.prot_tail == self.last_idx && self.nodes[last].key == *key {
				return Some(&self.nodes[last].value);
			}
		}
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		if self.nodes[idx].protected() {
			// Fast path: already the protected MRU, so the access changes
			// nothing — return without touching any links.
			if self.prot_tail == link(idx) {
				// Prime the single-entry cache: this key is the protected MRU.
				self.last_key = Some(key.clone());
				self.last_idx = link(idx);
				return Some(&self.nodes[idx].value);
			}
			// Already protected but not MRU: refresh to the protected MRU.
			self.unlink(idx);
			self.push_back(idx);
		} else {
			// Probationary hit: promote into the protected segment.
			self.unlink(idx);
			self.nodes[idx].set_protected(true);
			self.push_back(idx);
			self.prot_len += 1;
			// Protected segment overflowed its soft cap: demote its LRU back to
			// the probationary MRU so the protected set stays bounded.
			if self.prot_len > self.prot_cap {
				self.demote_protected_lru();
			}
		}
		// The accessed node now sits at the protected MRU (demotion only ever
		// evicts the protected LRU, never the just-pushed node), so prime the
		// single-entry cache to short-circuit the next repeated get.
		self.last_key = Some(key.clone());
		self.last_idx = link(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim (probationary
	/// LRU first, falling back to the protected LRU when probation is empty).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote within its segment.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			// Fast path: if the node is already the MRU of its own segment, its
			// position is unchanged — skip the unlink/push_back pair.
			let seg_mru = if self.nodes[idx].protected() {
				self.prot_tail
			} else {
				self.prob_tail
			};
			if seg_mru != link(idx) {
				self.unlink(idx);
				self.push_back(idx);
			}
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick a victim and reuse its slot for the new entry.
			let victim = if let Some(v) = slot(self.prob_head) {
				// Prefer evicting a probationary (one-hit-wonder) entry.
				v
			} else {
				// Probation empty: fall back to the protected LRU.
				let v = slot(self.prot_head)
					.expect("non-empty list has a protected head when probation is empty");
				self.prot_len -= 1;
				v
			};
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].set_protected(false);
			victim
		} else {
			// Arena has room: allocate a new slot in the probationary segment.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL, // segment flag clear (probationary), no prev link
				next: NIL,
			});
			idx
		};
		// Link the new/reused slot at the probationary MRU and register in the map.
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert!(self.nodes.len() <= self.capacity);
	}

	/// Demote the protected LRU node back into the probationary MRU.
	fn demote_protected_lru(&mut self) {
		if let Some(victim) = slot(self.prot_head) {
			self.unlink(victim);
			self.nodes[victim].set_protected(false);
			self.push_back(victim);
			self.prot_len -= 1;
		}
	}

	/// Remove node from its current position within whichever segment owns it.
	fn unlink(&mut self, idx: usize) {
		// Read each link once into a local; write back once per structural edit.
		let prev = self.nodes[idx].prev_link();
		let next = self.nodes[idx].next_link();
		let protected = self.nodes[idx].protected();
		match slot(prev) {
			Some(p) => self.nodes[p].set_next_link(next),
			None => self.set_head(protected, next),
		}
		match slot(next) {
			Some(n) => self.nodes[n].set_prev_link(prev),
			None => self.set_tail(protected, prev),
		}
		self.nodes[idx].set_prev_link(NIL);
		self.nodes[idx].set_next_link(NIL);
	}

	/// Append node to the MRU end of whichever segment its segment flag selects.
	fn push_back(&mut self, idx: usize) {
		let protected = self.nodes[idx].protected();
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		self.nodes[idx].set_prev_link(tail);
		self.nodes[idx].set_next_link(NIL);
		match slot(tail) {
			Some(t) => self.nodes[t].set_next_link(link(idx)),
			None => self.set_head(protected, link(idx)),
		}
		self.set_tail(protected, link(idx));
	}

	#[inline]
	fn set_head(&mut self, protected: bool, value: u32) {
		if protected {
			self.prot_head = value;
		} else {
			self.prob_head = value;
		}
	}

	#[inline]
	fn set_tail(&mut self, protected: bool, value: u32) {
		if protected {
			self.prot_tail = value;
		} else {
			self.prob_tail = value;
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
		cache.put(3, 30); // evicts key 1 (probationary LRU)

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
		cache.put(1, 11); // update key 1 makes it MRU within probation
		cache.put(3, 30); // evicts key 2 (now probationary LRU)

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
			cache.get(&1); // key 1 becomes protected and stays MRU there
		}
		cache.put(4, 40); // should evict key 2 (probationary LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probationary LRU
		cache.put(2, 20); // probationary middle
		cache.put(3, 30); // probationary MRU
		cache.get(&2); // promote key 2 into protected
		cache.put(4, 40); // evicts 1 (probationary LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn promoted_node_survives_probationary_eviction() {
		// SLRU semantics: a re-referenced (protected) entry is shielded from
		// probationary eviction, so the victim is always taken from probation
		// while a hot key persists.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts 1 (probationary LRU) -> slot 0 reused for key 3
		cache.get(&3); // promote key 3 into protected
		cache.put(4, 40); // evicts 2 (probationary LRU) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (probationary LRU); key 3 stays protected
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get promotes it into the protected segment
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
		cache.put(3, 30); // full; 3 is probationary MRU
		cache.put(3, 31); // overwrite, stays probationary MRU
		cache.put(4, 40); // evicts 1 (probationary LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn repeated_protected_mru_get_is_stable() {
		// Exercises the protected-MRU fast path: repeatedly getting the same
		// hot key must keep it resident and shielded from probationary eviction.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // key 1 -> protected (and protected MRU)
		for _ in 0..5 {
			// Fast path: key 1 is already the protected MRU, no link surgery.
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		cache.put(3, 30); // evicts key 2 (probationary LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
