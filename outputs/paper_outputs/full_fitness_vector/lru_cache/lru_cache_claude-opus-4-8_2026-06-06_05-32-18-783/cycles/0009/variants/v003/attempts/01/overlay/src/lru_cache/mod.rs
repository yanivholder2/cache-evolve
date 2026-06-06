// Approximate-LRU cache using a HashMap + a single-list CLOCK (second-chance) arena.
//
// The doubly-linked recency list of the baseline is gone entirely. Recency is
// approximated by a per-node `referenced` bit and a single `hand` that sweeps
// the arena on eviction. A hit becomes one boolean write — no pointer surgery —
// at the cost of approximating, rather than tracking exactly, the LRU victim.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash, Hasher};

pub mod lru_cache_jni;

/// Zero-sized fast hasher for integer keys. For an `i64`/`u64` key the standard
/// `Hash` impl funnels into `write_i64`/`write_u64`, which we capture directly;
/// `finish` applies a Fibonacci multiplicative mix so the low bits spread across
/// the table's bucket index. This is the proven free hot-path win underlying the
/// champions: it removes SipHash from every map probe.
#[derive(Default)]
pub struct FastHasher {
	state: u64,
}

impl Hasher for FastHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.state.wrapping_mul(0x9E37_79B9_7F4A_7C15)
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		// Fallback for non-integer keys: fold bytes into the state.
		for &byte in bytes {
			self.state = self.state.rotate_left(8) ^ u64::from(byte);
		}
	}

	#[inline]
	fn write_i64(&mut self, value: i64) {
		self.state = value as u64;
	}

	#[inline]
	fn write_u64(&mut self, value: u64) {
		self.state = value;
	}

	#[inline]
	fn write_i32(&mut self, value: i32) {
		self.state = value as u64;
	}

	#[inline]
	fn write_u32(&mut self, value: u32) {
		self.state = u64::from(value);
	}

	#[inline]
	fn write_usize(&mut self, value: usize) {
		self.state = value as u64;
	}
}

/// Zero-sized `BuildHasher` producing [`FastHasher`]. Carries no per-map state,
/// so swapping it in costs nothing in the `LruCache` footprint.
#[derive(Clone, Default)]
pub struct FastBuildHasher;

impl BuildHasher for FastBuildHasher {
	type Hasher = FastHasher;

	#[inline]
	fn build_hasher(&self) -> FastHasher {
		FastHasher::default()
	}
}

/// One arena slot. Holds the key, its value, and the CLOCK reference bit — no
/// recency links at all, so `Node<u64, u64>` collapses to the 24-byte class
/// (16 bytes of data plus the flag padded to the 8-byte alignment).
#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>, // key -> node index
	nodes: Vec<Node<K, V>>,                  // arena
	hand: usize,                             // CLOCK sweep position
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
			hand: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, giving it a CLOCK second chance.
	/// On a hit this is a single boolean write — no list mutation.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)?;
		self.nodes[idx].referenced = true;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, the CLOCK hand selects the
	/// first slot whose reference bit is clear (clearing set bits as it sweeps)
	/// as the eviction victim. `put` is only ever called for a non-resident key,
	/// but the existing-key branch is kept so an update never evicts.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and re-arm its reference bit.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].referenced = true;
			return;
		}
		if self.nodes.len() < self.capacity {
			// Arena has room: allocate a new slot. New entries start unreferenced.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				referenced: false,
			});
			self.map.insert(key, idx);
		} else {
			// Arena full: advance the clock to the first unreferenced slot,
			// clearing reference bits along the way (second chance).
			while self.nodes[self.hand].referenced {
				self.nodes[self.hand].referenced = false;
				self.hand = (self.hand + 1) % self.capacity;
			}
			let victim = self.hand;
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].referenced = false;
			self.map.insert(key, victim);
			// Advance past the freshly filled slot so the next sweep does not
			// immediately reconsider it.
			self.hand = (self.hand + 1) % self.capacity;
		}
		debug_assert!(self.nodes.len() <= self.capacity);
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
	fn overwrite_does_not_consume_capacity() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		// Updates reuse the same slot, so a second key still fits without eviction.
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn fresh_entries_evicted_in_clock_order() {
		// No key has been referenced, so the hand evicts in arena-fill order.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // hand at slot 0 (unreferenced) -> evicts key 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn referenced_entry_gets_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // sets key 1's reference bit
		cache.put(3, 30); // hand clears key 1's bit, then evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn overwrite_arms_reference_bit() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // overwrite sets key 1's reference bit
		cache.put(3, 30); // second chance to key 1 -> evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
	fn capacity_bound_under_overfill() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for key in 0..100i64 {
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
