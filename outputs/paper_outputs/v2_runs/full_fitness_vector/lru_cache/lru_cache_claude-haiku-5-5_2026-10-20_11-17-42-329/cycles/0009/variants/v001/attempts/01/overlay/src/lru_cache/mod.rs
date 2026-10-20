// Segmented LRU (SLRU) cache with lazy reference bits, using HashMap + arena-based intrusive lists.
// Nodes are stored in a Vec (arena) indexed by u32 (capacity is capped below INDEX_MASK in try_new).
// Each node packs its two flags (PROTECTED, REFERENCED) into the top two bits of its `prev` field,
// so a node carries no separate flags byte and is 24 bytes for u64 keys and values.
// The arena is partitioned into two intrusive lists: a probationary segment (keys seen once) and a
// protected segment (keys promoted by a second-chance rescue). Hits only set a reference bit; no list
// pointers are touched on get or on update. On a miss at capacity, the probationary head is
// given one second chance if its reference bit is set. When a protected demotion meets a
// referenced head, that head is spared in place (bit cleared) and the next protected node is
// demoted instead. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
// The key-to-index map uses a cheap FxHash-style hasher (FastBuildHasher) instead of SipHash.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod lru_cache_jni;

/// Mask selecting the arena index from a packed `prev` word (low 30 bits).
const INDEX_MASK: u32 = 0x3FFF_FFFF;
/// Sentinel for "no node" in the arena-linked lists. Equal to INDEX_MASK, so it is never a valid
/// arena index: try_new caps capacity below INDEX_MASK, so indices are at most INDEX_MASK - 2.
const NIL: u32 = INDEX_MASK;
/// Packed flag bit in `prev`: resident in the protected segment (otherwise probationary).
const PROTECTED_BIT: u32 = 1 << 31;
/// Packed flag bit in `prev`: referenced since it was last inserted, rescued, or demoted.
const REFERENCED_BIT: u32 = 1 << 30;

/// Multiplicative constant used by the FxHash-style mixer (same as rustc's FxHasher).
const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Returns the arena index (or NIL) carried in a packed `prev` word, with the flag bits stripped.
/// Every index read goes through here so the masking lives in one place.
#[inline(always)]
fn link_index(prev: u32) -> u32 {
	prev & INDEX_MASK
}

/// Cheap non-cryptographic hasher: each word is folded in with a rotate, xor, and multiply.
/// It does not resist adversarial keys; it only needs to spread the workload's keys.
#[derive(Debug, Default, Clone, Copy)]
pub struct FastHasher(u64);

impl FastHasher {
	#[inline]
	fn add_to_hash(&mut self, word: u64) {
		self.0 = (self.0.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FastHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		// Fold the byte stream in 8-byte little-endian words, zero-padding the tail, so that
		// arbitrary `Hash` impls (e.g. str, slices) hash deterministically and equal keys agree.
		let mut chunks = bytes.chunks_exact(8);
		for chunk in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(chunk);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add_to_hash(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add_to_hash(i as u64);
	}
}

/// Build-hasher for the key map; each lookup starts from a zeroed `FastHasher`.
#[derive(Debug, Default, Clone, Copy)]
pub struct FastBuildHasher;

impl BuildHasher for FastBuildHasher {
	type Hasher = FastHasher;

	#[inline]
	fn build_hasher(&self) -> FastHasher {
		FastHasher::default()
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Packed: bit 31 PROTECTED, bit 30 REFERENCED, bits 0..30 the previous arena index (or NIL).
	prev: u32,
	/// Next arena index in the list, or NIL. Carries no flags.
	next: u32,
}

impl<K, V> Node<K, V> {
	#[inline]
	fn prev_idx(&self) -> u32 {
		link_index(self.prev)
	}

	/// Replaces the index part of `prev`, preserving the packed flag bits.
	#[inline]
	fn set_prev_idx(&mut self, idx: u32) {
		self.prev = (self.prev & !INDEX_MASK) | idx;
	}

	#[inline]
	fn is_protected(&self) -> bool {
		self.prev & PROTECTED_BIT != 0
	}

	#[inline]
	fn set_protected(&mut self, protected: bool) {
		if protected {
			self.prev |= PROTECTED_BIT;
		} else {
			self.prev &= !PROTECTED_BIT;
		}
	}

	#[inline]
	fn is_referenced(&self) -> bool {
		self.prev & REFERENCED_BIT != 0
	}

	/// Hit-path reference marking: a single OR on the raw `prev` word, no index access.
	#[inline(always)]
	fn set_referenced(&mut self) {
		self.prev |= REFERENCED_BIT;
	}

	#[inline]
	fn clear_referenced(&mut self) {
		self.prev &= !REFERENCED_BIT;
	}

	/// Clears both packed flag bits, keeping the index part of `prev`.
	#[inline]
	fn clear_flags(&mut self) {
		self.prev = link_index(self.prev);
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FastBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                // arena
	prob_head: u32,                        // probationary LRU (front, eviction candidate)
	prob_tail: u32,                        // probationary MRU (back)
	prot_head: u32,                        // protected LRU (front, demotion candidate)
	prot_tail: u32,                        // protected MRU (back)
	prot_len: usize,                       // number of nodes in the protected segment
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
		if capacity > INDEX_MASK as usize - 1 {
			log::error!("LruCache.try_new: capacity={capacity} exceeds the packed index limit");
			return None;
		}
		// Past this check every arena index fits in the 30-bit packed index, so the `as u32` casts below are lossless.
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher),
			nodes: Vec::with_capacity(capacity),
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prot_len: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its reference bit (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx as usize];
		node.set_referenced();
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a probationary entry (or the
	/// protected LRU when no probationary entry exists).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and set its reference bit
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			node.set_referenced();
			return;
		}
		// New key — need an arena slot
		let idx: u32 = if self.nodes.len() >= self.capacity {
			// Arena full: choose a victim and reuse its slot
			let victim = self.select_victim();
			let evicted_key = self.nodes[victim as usize].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let node = &mut self.nodes[victim as usize];
			node.key = key.clone();
			node.value = value;
			node.clear_flags();
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			idx
		};
		// New entries start probationary, unreferenced, at the MRU end; register in the map
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Maximum number of nodes the protected segment may hold: a 9/10 share of capacity (at least
	/// one). The probationary segment keeps the remaining slots, so it always has room for new keys
	/// once capacity exceeds 2.
	fn protected_cap(&self) -> usize {
		(self.capacity * 9 / 10).max(1)
	}

	/// Picks the arena slot to reuse on a miss at capacity. The probationary head gets at most
	/// one second chance: if its reference bit is set, it moves to the protected tail (first
	/// demoting the protected head if that segment is full). After that, the current
	/// probationary head is evicted regardless of its bit. Falls back to the protected head
	/// when no probationary node exists. O(1) per miss.
	fn select_victim(&mut self) -> u32 {
		if self.prob_head == NIL {
			return self.prot_head;
		}
		let head = self.prob_head;
		if self.nodes[head as usize].is_referenced() {
			self.unlink(head);
			self.nodes[head as usize].clear_referenced();
			if self.prot_len >= self.protected_cap() {
				self.demote_prot_head();
			}
			self.push_back(head, true);
			debug_assert!(self.prot_len <= self.protected_cap());
		}
		if self.prob_head != NIL {
			self.prob_head
		} else {
			self.prot_head
		}
	}

	/// Demotes one protected node to the probationary MRU end. A referenced protected head is
	/// spared once: its reference bit is cleared and it stays in place, and the next protected
	/// node is demoted instead. A spared head is demoted on a later visit only if it is still
	/// unreferenced. There is at most one retry per call, and `prot_len` drops by exactly one.
	fn demote_prot_head(&mut self) {
		let head = self.prot_head;
		let mut victim = head;
		if self.nodes[head as usize].is_referenced() {
			self.nodes[head as usize].clear_referenced();
			let next = self.nodes[head as usize].next;
			if next != NIL {
				victim = next;
			}
		}
		self.unlink(victim);
		self.nodes[victim as usize].clear_referenced();
		self.push_back(victim, false);
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: u32) {
		let prev = link_index(self.nodes[idx as usize].prev);
		let next = self.nodes[idx as usize].next;
		let protected = self.nodes[idx as usize].is_protected();
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else if protected {
			self.prot_head = next;
		} else {
			self.prob_head = next;
		}
		if next != NIL {
			self.nodes[next as usize].set_prev_idx(prev);
		} else if protected {
			self.prot_tail = prev;
		} else {
			self.prob_tail = prev;
		}
		if protected {
			self.prot_len -= 1;
		}
		self.nodes[idx as usize].set_prev_idx(NIL);
		self.nodes[idx as usize].next = NIL;
	}

	/// Append node to the MRU (tail) end of the protected or probationary list.
	fn push_back(&mut self, idx: u32, protected: bool) {
		let node = &mut self.nodes[idx as usize];
		node.set_protected(protected);
		node.next = NIL;
		if protected {
			let tail = self.prot_tail;
			node.set_prev_idx(tail);
			if tail != NIL {
				self.nodes[tail as usize].next = idx;
			} else {
				self.prot_head = idx;
			}
			self.prot_tail = idx;
			self.prot_len += 1;
		} else {
			let tail = self.prob_tail;
			node.set_prev_idx(tail);
			if tail != NIL {
				self.nodes[tail as usize].next = idx;
			} else {
				self.prob_head = idx;
			}
			self.prob_tail = idx;
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
		cache.put(3, 30); // evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // references key 1: second chance at eviction
		cache.put(3, 30); // rescues 1, evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 sets its reference bit
		cache.put(3, 30); // rescues 1, evicts key 2

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
		cache.put(4, 40); // rescues key 1 once, evicts key 2 (probationary LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // reference middle
		// Probationary: 1 (LRU), 2 (referenced), 3
		cache.put(4, 40); // evicts 1 (unreferenced head)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0 (probationary)
		cache.put(2, 20); // slot 1 (probationary)
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // reference slot 0 (key 3)
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // rescues 3 to protected, evicts 4 (slot 1) -> slot 1 reused for key 5
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survives
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node; get only sets its reference bit
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
		cache.put(3, 30); // full; 3 is MRU (tail)
		cache.put(3, 31); // overwrite MRU
		// Eviction should take 1 (LRU, unreferenced)
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn one_hit_scan_does_not_evict_twice_read_key() {
		let mut cache = LruCache::new(4);
		cache.put(100, 1000);
		cache.get(&100); // second reference: rescued to protected on first eviction pass
		// Scan of one-hit keys far larger than capacity
		for k in 0..20 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&100).unwrap(), 1000);
	}

	#[test]
	fn fast_hasher_is_deterministic_and_distinguishes_bytes() {
		let hash_of = |s: &str| {
			let mut h = FastBuildHasher.build_hasher();
			s.hash(&mut h);
			h.finish()
		};
		assert_eq!(hash_of("abc"), hash_of("abc"));
		assert_ne!(hash_of("abc"), hash_of("abd"));
		assert_ne!(hash_of("abcdefghi"), hash_of("abcdefghj"));
	}

	#[test]
	fn string_keys_round_trip() {
		let mut cache = LruCache::new(3);
		cache.put(String::from("alpha"), 1);
		cache.put(String::from("beta"), 2);
		assert_eq!(*cache.get(&String::from("alpha")).unwrap(), 1);
		assert_eq!(*cache.get(&String::from("beta")).unwrap(), 2);
		assert!(cache.get(&String::from("gamma")).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
