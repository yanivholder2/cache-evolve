// O(1) segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// The recency order is split into two segments that share the same arena:
//   * probationary — where brand-new (one-hit-wonder) entries live; eviction
//     victims are taken from here first.
//   * protected    — where entries that have been referenced again are promoted;
//     these are shielded from probationary eviction.
// A node's `protected` flag selects which segment its links belong to, so the
// O(1) unlink/push_back helpers operate on whichever segment owns the node.
//
// Hashing uses a zero-sized `FastHasher` (a single xor + wrapping-multiply over
// the 8-byte integer key) instead of SipHash, cutting per-lookup hash cost. A
// fast path on get/put-update avoids redundant list surgery when the touched
// node is already the most-recently-used node of its segment.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

// Multiplicative mixing constant (Fibonacci-derived 64-bit odd prime).
const FAST_HASH_MULT: u64 = 0x517cc1b727220a95;

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
	prev: Option<usize>,
	next: Option<usize>,
	// Which segment this node currently belongs to: true => protected, false => probationary.
	protected: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                  // arena
	prot_head: Option<usize>,                // protected LRU (front)
	prot_tail: Option<usize>,                // protected MRU (back)
	prob_head: Option<usize>,                // probationary LRU (front)
	prob_tail: Option<usize>,                // probationary MRU (back)
	prot_len: usize,                         // number of resident protected nodes
	prot_cap: usize,                         // soft cap on protected nodes
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
		// Protected segment gets ~80% of capacity, but never the whole cache: at
		// least one slot is reserved for probation (clamp to capacity - 1) and the
		// cap is at least 1 where geometry allows.
		let prot_cap = (capacity * 4 / 5).max(1).min(capacity.saturating_sub(1));
		log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			prot_head: None,
			prot_tail: None,
			prob_head: None,
			prob_tail: None,
			prot_len: 0,
			prot_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it toward most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		if self.nodes[idx].protected {
			// Fast path: already the protected MRU, so the access changes
			// nothing — return without touching any links.
			if self.prot_tail == Some(idx) {
				return Some(&self.nodes[idx].value);
			}
			// Already protected but not MRU: refresh to the protected MRU.
			self.unlink(idx);
			self.push_back(idx);
		} else {
			// Probationary hit: promote into the protected segment.
			self.unlink(idx);
			self.nodes[idx].protected = true;
			self.push_back(idx);
			self.prot_len += 1;
			// Protected segment overflowed its soft cap: demote its LRU back to
			// the probationary MRU so the protected set stays bounded.
			if self.prot_len > self.prot_cap {
				self.demote_protected_lru();
			}
		}
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
			let seg_mru = if self.nodes[idx].protected {
				self.prot_tail
			} else {
				self.prob_tail
			};
			if seg_mru != Some(idx) {
				self.unlink(idx);
				self.push_back(idx);
			}
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick a victim and reuse its slot for the new entry.
			let victim = if let Some(v) = self.prob_head {
				// Prefer evicting a probationary (one-hit-wonder) entry.
				v
			} else {
				// Probation empty: fall back to the protected LRU.
				let v = self
					.prot_head
					.expect("non-empty list has a protected head when probation is empty");
				self.prot_len -= 1;
				v
			};
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].protected = false;
			victim
		} else {
			// Arena has room: allocate a new slot in the probationary segment.
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
		// Link the new/reused slot at the probationary MRU and register in the map.
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert!(self.nodes.len() <= self.capacity);
	}

	/// Demote the protected LRU node back into the probationary MRU.
	fn demote_protected_lru(&mut self) {
		if let Some(victim) = self.prot_head {
			self.unlink(victim);
			self.nodes[victim].protected = false;
			self.push_back(victim);
			self.prot_len -= 1;
		}
	}

	/// Remove node from its current position within whichever segment owns it.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.set_head(protected, next),
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.set_tail(protected, prev),
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of whichever segment its `protected` flag selects.
	fn push_back(&mut self, idx: usize) {
		let protected = self.nodes[idx].protected;
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.set_head(protected, Some(idx)),
		}
		self.set_tail(protected, Some(idx));
	}

	#[inline]
	fn set_head(&mut self, protected: bool, value: Option<usize>) {
		if protected {
			self.prot_head = value;
		} else {
			self.prob_head = value;
		}
	}

	#[inline]
	fn set_tail(&mut self, protected: bool, value: Option<usize>) {
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
