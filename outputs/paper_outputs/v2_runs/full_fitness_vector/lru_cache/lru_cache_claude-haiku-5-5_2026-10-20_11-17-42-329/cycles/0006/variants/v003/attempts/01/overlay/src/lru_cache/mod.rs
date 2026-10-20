// O(1) segmented LRU cache with ghost-based admission.
// Nodes live in a Vec (arena) indexed by usize and are linked into one of two
// intrusive lists: a probationary segment and a protected segment. Eviction takes
// the probationary head (falling back to the protected head). A fixed-size ring of
// fingerprints remembers recently evicted keys; a miss whose key is found in the
// ring is admitted straight to the protected segment. The ring is consulted only
// on puts (misses), never on get. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const PROB: usize = 0;
const PROT: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,   // key -> node index
	nodes: Vec<Node<K, V>>,   // arena
	seg: Vec<bool>,           // arena-parallel: true = node is in the protected segment
	head: [Option<usize>; 2], // per segment: LRU end (front)
	tail: [Option<usize>; 2], // per segment: MRU end (back)
	prot_len: usize,          // number of nodes in the protected segment
	ghost: Vec<u32>,          // ring of evicted-key fingerprints; 0 = empty slot
	ghost_pos: usize,         // next write position in `ghost`
	capacity: usize,
}

/// Cheap multiply-rotate hasher over the `Hash` output. Not cryptographic.
#[derive(Default)]
struct MixHasher(u64);

impl Hasher for MixHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ u64::from(b)).wrapping_mul(0x517c_c1b7_2722_0a95);
		}
	}

	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0.rotate_left(5) ^ i).wrapping_mul(0x517c_c1b7_2722_0a95);
	}
}

/// Nonzero 32-bit fingerprint of a key; zero is reserved for "empty ghost slot".
fn fingerprint<K: Hash + ?Sized>(key: &K) -> u32 {
	let mut h = MixHasher::default();
	key.hash(&mut h);
	let x = h.finish();
	((x ^ (x >> 32)) as u32) | 1
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
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			seg: Vec::with_capacity(capacity),
			head: [None; 2],
			tail: [None; 2],
			prot_len: 0,
			ghost: vec![0; capacity],
			ghost_pos: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to the MRU end of its segment.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let s = usize::from(self.seg[idx]);
		self.unlink(idx);
		self.push_back(idx, s);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim and records its fingerprint.
	/// A new key whose fingerprint is in the ghost ring is admitted to the protected segment.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU of its segment
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let s = usize::from(self.seg[idx]);
			self.unlink(idx);
			self.push_back(idx, s);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			let victim = self.select_victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.ghost_record(&evicted_key);
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			self.seg.push(false);
			idx
		};
		// Returning key that was recently evicted: admit to protected; otherwise probationary.
		let seg = if self.protected_cap() > 0 && self.ghost_take(&key) {
			if self.prot_len >= self.protected_cap() {
				self.demote_prot_head();
			}
			PROT
		} else {
			PROB
		};
		self.push_back(idx, seg);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn protected_cap(&self) -> usize {
		self.capacity / 2
	}

	/// Victim is the probationary LRU if any, else the protected LRU.
	fn select_victim(&self) -> usize {
		self.head[PROB]
			.or(self.head[PROT])
			.expect("full cache has a victim")
	}

	/// Move the protected LRU to the probationary MRU end (it stays resident).
	fn demote_prot_head(&mut self) {
		let h = self.head[PROT].expect("protected segment non-empty when demoting");
		self.unlink(h);
		self.push_back(h, PROB);
	}

	/// Write the fingerprint of an evicted key into the ghost ring.
	fn ghost_record(&mut self, key: &K) {
		self.ghost[self.ghost_pos] = fingerprint(key);
		self.ghost_pos = (self.ghost_pos + 1) % self.capacity;
	}

	/// If `key`'s fingerprint is in the ghost ring, clear that slot and return true.
	fn ghost_take(&mut self, key: &K) -> bool {
		let f = fingerprint(key);
		match self.ghost.iter().position(|&g| g == f) {
			Some(i) => {
				self.ghost[i] = 0;
				true
			}
			None => false,
		}
	}

	/// Remove node from its current segment list.
	fn unlink(&mut self, idx: usize) {
		let s = usize::from(self.seg[idx]);
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head[s] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail[s] = prev,
		}
		if s == PROT {
			self.prot_len -= 1;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of segment `s` and record its segment.
	fn push_back(&mut self, idx: usize, s: usize) {
		self.seg[idx] = s == PROT;
		self.nodes[idx].prev = self.tail[s];
		self.nodes[idx].next = None;
		match self.tail[s] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head[s] = Some(idx),
		}
		self.tail[s] = Some(idx);
		if s == PROT {
			self.prot_len += 1;
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
	fn readmitted_key_is_protected_and_keeps_values() {
		let mut cache = LruCache::new(4);
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		cache.put(5, 50); // evicts 1 -> ghost holds fp(1)
		cache.put(1, 10); // ghost hit: re-admitted to protected, evicts 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn capacity_one_never_protects() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20); // evicts 1
		cache.put(1, 11); // ghost hit, but protected cap is 0
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 11);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
