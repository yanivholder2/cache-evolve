// O(1)-amortised CLOCK (second-chance) cache using HashMap + arena ring.
// Nodes are stored in a Vec (arena) indexed by usize; the arena in slot order is the
// clock ring. A hit only sets the node's reference bit (no relinking). On eviction the
// hand sweeps the ring, clearing reference bits, and reuses the first unreferenced slot
// in place. No unsafe, no external crates.
//
// The key -> slot map uses a hand-written FxHash-style multiplicative hasher instead of
// the default SipHash-1-3 `RandomState` (no hash-flooding resistance; fine for a
// simulator workload).

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod lru_cache_jni;

/// rustc-hash / FxHash multiplicative constant.
const SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// FxHash-style hasher: each word `w` does `hash = (hash.rotl(5) ^ w) * SEED`.
#[derive(Debug, Default, Clone, Copy)]
struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline(always)]
	fn add_to_hash(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for chunk in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(chunk);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
		let tail = chunks.remainder();
		if !tail.is_empty() {
			let mut buf = [0u8; 8];
			buf[..tail.len()].copy_from_slice(tail);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
	}

	#[inline(always)]
	fn write_u8(&mut self, i: u8) {
		self.add_to_hash(i as u64);
	}

	#[inline(always)]
	fn write_u16(&mut self, i: u16) {
		self.add_to_hash(i as u64);
	}

	#[inline(always)]
	fn write_u32(&mut self, i: u32) {
		self.add_to_hash(i as u64);
	}

	#[inline(always)]
	fn write_u64(&mut self, i: u64) {
		self.add_to_hash(i);
	}

	#[inline(always)]
	fn write_usize(&mut self, i: usize) {
		self.add_to_hash(i as u64);
	}

	#[inline(always)]
	fn write_i8(&mut self, i: i8) {
		self.write_u8(i as u8);
	}

	#[inline(always)]
	fn write_i16(&mut self, i: i16) {
		self.write_u16(i as u16);
	}

	#[inline(always)]
	fn write_i32(&mut self, i: i32) {
		self.write_u32(i as u32);
	}

	#[inline(always)]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}

	#[inline(always)]
	fn write_isize(&mut self, i: isize) {
		self.write_usize(i as usize);
	}

	/// FxHash leaves weak entropy in the low bits while the high bits are well mixed.
	/// hashbrown uses the top 7 bits as the control tag and the low bits for the bucket
	/// index, so rotate left by 26 (as rustc-hash 2.x does) to bring well-mixed high bits
	/// down into the index bits while keeping good bits in the tag.
	#[inline(always)]
	fn finish(&self) -> u64 {
		self.hash.rotate_left(26)
	}
}

#[derive(Debug, Default, Clone, Copy)]
struct FxBuildHasher;

impl BuildHasher for FxBuildHasher {
	type Hasher = FxHasher;

	#[inline(always)]
	fn build_hasher(&self) -> FxHasher {
		FxHasher { hash: 0 }
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FxBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                 // arena == clock ring in slot order
	hand: usize,                            // clock hand (next eviction candidate)
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
			map: HashMap::with_capacity_and_hasher(capacity, FxBuildHasher),
			nodes: Vec::with_capacity(capacity),
			hand: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, marking it referenced (lazy promotion).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx];
		node.referenced = true;
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts via the CLOCK sweep.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in place and mark referenced
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.referenced = true;
			return;
		}
		if self.nodes.len() < self.capacity {
			// Arena has room: fresh inserts get no second chance until hit
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				referenced: false,
			});
			self.map.insert(key, idx);
		} else {
			// Arena full: run the clock. Terminates within one rotation since each
			// step over a referenced slot clears its bit.
			let mut hand = self.hand;
			while self.nodes[hand].referenced {
				self.nodes[hand].referenced = false;
				hand += 1;
				if hand == self.capacity {
					hand = 0;
				}
			}
			let victim = &mut self.nodes[hand];
			self.map.remove(&victim.key);
			victim.key = key.clone();
			victim.value = value;
			victim.referenced = false;
			self.map.insert(key, hand);
			hand += 1;
			if hand == self.capacity {
				hand = 0;
			}
			self.hand = hand;
		}
		debug_assert!(self.map.len() <= self.capacity);
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
	fn eviction_removes_unreferenced_at_hand() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // hand at slot 0 (key 1, unreferenced) -> evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_grants_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // sets key 1's reference bit
		cache.put(3, 30); // clears key 1's bit, evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_overwrite_grants_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // overwrite sets reference bit
		cache.put(3, 30); // evicts key 2

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
		cache.put(2, 20); // clears 1's bit, wraps, evicts 1
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
	fn repeated_get_sets_single_bit() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // skips key 1 (referenced), evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn hit_on_non_hand_slot_does_not_save_hand_slot() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0 (hand)
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		cache.get(&2);
		cache.put(4, 40); // evicts 1 at the hand
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn clock_victim_order_differs_from_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 = key 3, hand -> 1
		cache.get(&3); // slot 0 referenced
		cache.put(4, 40); // slot 1 (key 2) unreferenced -> evicted, hand -> 0
		cache.put(5, 50); // slot 0 referenced -> cleared; slot 1 (key 4) evicted
		// Strict LRU would have evicted 3 here; CLOCK keeps it.
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn full_rotation_when_all_referenced() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // clears all bits, wraps, evicts key 1 at slot 0
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_at_capacity_then_evict() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(3, 31); // overwrite marks 3 referenced
		cache.put(4, 40); // evicts 1 at the hand
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

	#[test]
	fn fx_hasher_sequential_u64_keys_round_trip() {
		let n: u64 = 10_000;
		let mut cache = LruCache::<u64, u64>::new(n as usize);
		for k in 0..n {
			cache.put(k, k.wrapping_mul(3) + 1);
		}
		for k in 0..n {
			assert_eq!(*cache.get(&k).unwrap(), k.wrapping_mul(3) + 1, "key {k}");
		}
		assert!(cache.get(&n).is_none());
	}
}
