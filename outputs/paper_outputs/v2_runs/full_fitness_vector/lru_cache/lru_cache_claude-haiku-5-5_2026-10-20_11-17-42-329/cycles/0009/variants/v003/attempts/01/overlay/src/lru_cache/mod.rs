// CLOCK cache with per-slot 2-bit reference counters, using a HashMap plus an arena of slots.
// Nodes are stored in a Vec (arena) indexed by u32. A node holds only its key and value; its
// reference counter lives in a parallel `refs: Vec<u8>` (low two bits, saturating at 3). Hits
// increment the counter. On a miss at capacity, a hand sweeps the ring: a non-zero counter is
// decremented and the hand moves on, and the first slot with counter 0 is evicted and reused
// in-place. No linked lists, no packed flags, no unsafe, no external crates.
// The key-to-index map uses a cheap FxHash-style hasher (FastBuildHasher) instead of SipHash.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod lru_cache_jni;

/// Saturation ceiling for the per-slot reference counter (two bits).
const REF_MAX: u8 = 3;

/// Multiplicative constant used by the FxHash-style mixer (same as rustc's FxHasher).
const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

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
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FastBuildHasher>, // key -> slot index
	nodes: Vec<Node<K, V>>,                // arena (the CLOCK ring)
	refs: Vec<u8>,                         // per-slot reference counter, parallel to `nodes`
	hand: u32,                             // next slot the eviction sweep examines
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
		if capacity > u32::MAX as usize {
			log::error!("LruCache.try_new: capacity={capacity} exceeds the u32 slot index limit");
			return None;
		}
		// Past this check every slot index fits in u32, so the `as u32` casts below are lossless.
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher),
			nodes: Vec::with_capacity(capacity),
			refs: Vec::with_capacity(capacity),
			hand: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, bumping its reference counter (saturating at 3).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.bump(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, the CLOCK hand picks a victim slot.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and bump its reference counter
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.bump(idx);
			return;
		}
		if self.nodes.len() >= self.capacity {
			// Ring full: sweep to a victim, then reuse its slot for the new entry
			let victim = self.select_victim();
			let old_key = self.nodes[victim as usize].key.clone();
			self.map.remove(&old_key);
			self.nodes[victim as usize] = Node {
				key: key.clone(),
				value,
			};
			self.refs[victim as usize] = 0;
			self.map.insert(key, victim);
			self.hand = (victim + 1) % self.nodes.len() as u32;
		} else {
			// Ring has room: allocate a new slot with counter 0
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
			});
			self.refs.push(0);
			self.map.insert(key, idx);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Advances `hand` over the ring. A slot with a non-zero counter is decremented and skipped;
	/// the first slot with counter 0 is returned. Terminates within REF_MAX + 1 sweeps because
	/// every visited non-zero counter strictly decreases.
	fn select_victim(&mut self) -> u32 {
		let len = self.nodes.len();
		loop {
			let idx = self.hand as usize;
			if self.refs[idx] == 0 {
				return idx as u32;
			}
			self.refs[idx] -= 1;
			self.hand = ((idx + 1) % len) as u32;
		}
	}

	/// Saturating increment of a slot's reference counter.
	#[inline]
	fn bump(&mut self, idx: u32) {
		let r = &mut self.refs[idx as usize];
		*r = (*r + 1).min(REF_MAX);
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
		cache.get(&1); // counter of slot 0 -> 1: second chance at eviction
		cache.put(3, 30); // spares key 1 (counter -> 0), evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 bumps its counter
		cache.put(3, 30); // spares key 1, evicts key 2

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
		cache.put(4, 40); // slot 0 counter saturates at 3 -> 2 (spared); key 2 evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		cache.get(&2); // slot 1 counter -> 1
		// Hand at slot 0 (counter 0): key 1 is evicted, key 2 is untouched
		cache.put(4, 40);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3, hand -> slot 1
		cache.get(&3); // slot 0 counter -> 1
		cache.put(4, 40); // slot 1 (counter 0) evicts 2 -> hand -> slot 0
		cache.put(5, 50); // slot 0 spared (counter 1 -> 0), slot 1 evicts 4
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30); // spared, survives
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single slot; get only bumps its counter
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify the entry is still intact after further inserts
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; slot 2 holds the most recent insert
		cache.put(3, 31); // overwrite bumps slot 2's counter
		// Hand at slot 0 (counter 0): eviction takes key 1
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn referenced_key_survives_one_sweep() {
		let mut cache = LruCache::new(4);
		cache.put(100, 1000); // slot 0, counter 0
		cache.get(&100); // counter -> 1: one second chance
		// Three one-hit keys fill slots 1..3; the fourth miss forces the hand past slot 0
		for k in 0..4 {
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
