// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// H03: graft v002's order-preserving short-circuit onto v001's compact
// 24-byte node. v001 unconditionally calls `unlink` + `push_back` on every
// get/put-update, even when the touched node is already the MRU (tail), where
// that work is a no-op. A cheaply predictable `tail == idx` position check lets
// already-MRU touches return without any list surgery, pulling per-op-time
// down while leaving the node layout, hit-rate, and metadata-size untouched.
//
// H03 (inherited): the key map uses an inline FxHash-style integer hasher
// instead of the standard-library default (SipHash 1-3). For the harness's
// 8-byte integer keys a single multiply-xor per key is several times faster
// than SipHash, cutting get/put per-op-time while leaving policy, hit-rate, and
// metadata-size unchanged.
//
// H01: the recency links are encoded as `u32` sentinels (`NIL = u32::MAX`)
// instead of `Option<usize>`. A no-niche `Option<usize>` costs 16 bytes per
// link, so two links dominate `Node`'s footprint; narrowing each to a bare
// `u32` shrinks `Node<u64,u64>` from 48 to 24 bytes. To avoid v002's per-access
// cast penalty the conversions are centralised in two `#[inline]` helpers — a
// link is read once into a `usize`/`Option<usize>` local at the top of each
// routine and written back once per structural mutation, so traversal stays a
// zero-cost zero-extend on the 64-bit target.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod lru_cache_jni;

/// Sentinel for an absent recency link. `u32::MAX` is reserved as "no node";
/// a live arena never reaches `u32::MAX` slots at the harness's capacities.
const NIL: u32 = u32::MAX;

/// Widen a stored link to an arena index, mapping the `NIL` sentinel to `None`.
/// `u32 -> usize` is a zero-extend (free) on the 64-bit target.
#[inline]
fn idx(link: u32) -> Option<usize> {
	if link == NIL {
		None
	} else {
		Some(link as usize)
	}
}

/// Narrow an arena index to a stored link. The single per-mutation narrowing.
#[inline]
fn link(index: usize) -> u32 {
	index as u32
}

/// FxHash-style non-cryptographic hasher tuned for small integer keys.
/// Accumulates input with a single xor + wrapping multiply by an odd constant.
/// Not collision-resistant against adversarial input, but `Eq` still resolves
/// any collisions, so correctness never depends on hash quality.
struct FastHasher(u64);

impl FastHasher {
	const SEED: u64 = 0;
	const K: u64 = 0x517cc1b727220a95;

	#[inline]
	fn fold(&mut self, value: u64) {
		self.0 = (self.0 ^ value).wrapping_mul(Self::K);
	}
}

impl Hasher for FastHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for chunk in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(chunk);
			self.fold(u64::from_le_bytes(buf));
		}
		for &byte in chunks.remainder() {
			self.fold(byte as u64);
		}
	}

	#[inline]
	fn write_u64(&mut self, value: u64) {
		self.fold(value);
	}

	#[inline]
	fn write_usize(&mut self, value: usize) {
		self.fold(value as u64);
	}
}

/// Zero-sized build-hasher producing seeded `FastHasher`s. Being zero-sized, it
/// adds no bytes to `LruCache`'s footprint, so metadata-size is unaffected.
#[derive(Default, Clone)]
struct FastBuildHasher;

impl BuildHasher for FastBuildHasher {
	type Hasher = FastHasher;

	#[inline]
	fn build_hasher(&self) -> FastHasher {
		FastHasher(FastHasher::SEED)
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                  // arena
	head: u32,                               // LRU (front), NIL when empty
	tail: u32,                               // MRU (back), NIL when empty
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
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &index = self.map.get(key)?;
		// Already-MRU fast path: when the node is the tail, the unlink/push_back
		// pair would re-establish the same order, so skip the surgery entirely.
		// `tail == idx` is a cheaply predictable position gate (the recurring hot
		// key sits at the tail), so this branch costs almost nothing.
		if self.tail == link(index) {
			return Some(&self.nodes[index].value);
		}
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(index);
		self.push_back(index);
		Some(&self.nodes[index].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used entry.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&index) = self.map.get(&key) {
			self.nodes[index].value = value;
			// Already-MRU fast path: a value update on the tail leaves recency
			// order unchanged, so skip the no-op relink.
			if self.tail != link(index) {
				self.unlink(index);
				self.push_back(index);
			}
			return;
		}
		// New key — need an arena slot
		let index = if self.nodes.len() >= self.capacity {
			// Arena full: evict LRU (head) and reuse its slot for the new entry
			debug_assert!(self.head != NIL, "head must be set when arena is full");
			let head_idx = idx(self.head).expect("non-empty list has a head");
			let evicted_key = self.nodes[head_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			self.nodes[head_idx].key = key.clone();
			self.nodes[head_idx].value = value;
			head_idx
		} else {
			// Arena has room: allocate a new slot
			let new_idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			new_idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(index);
		self.map.insert(key, index);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, index: usize) {
		// Read both links once; they stay as raw `u32` so the write-backs below
		// are pure moves with no narrowing.
		let prev = self.nodes[index].prev;
		let next = self.nodes[index].next;
		match idx(prev) {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match idx(next) {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[index].prev = NIL;
		self.nodes[index].next = NIL;
	}

	/// Append node to the tail (MRU end) of the list.
	fn push_back(&mut self, index: usize) {
		self.nodes[index].prev = self.tail;
		self.nodes[index].next = NIL;
		match idx(self.tail) {
			Some(t) => self.nodes[t].next = link(index),
			None => self.head = link(index),
		}
		self.tail = link(index);
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
		cache.get(&1); // makes key 1 most recent
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU
		cache.put(3, 30); // evicts key 2 (now LRU)

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
		cache.put(4, 40); // should evict key 2 (LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to MRU
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 3 (slot 0) -> slot 0 reused AGAIN for key 5
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it
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
										// Eviction should take 1 (LRU)
		cache.put(4, 40); // evicts 1
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
