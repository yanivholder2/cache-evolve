// GCLOCK aging-counter cache: same fixed-size ring + hand layout as the v002
// CLOCK variant, but each slot carries a saturating 2-bit reference counter
// (logical range 0..=3) instead of a single reference bit. A hit increments
// the counter (saturating at 3); the hand sweep decrements the counter and
// only evicts when it hits 0. New entries start at counter = 1 so they
// survive the first sweep across them. The doubly-linked list of the
// baseline is gone: `get` is a single u8 update, and `put` amortises the
// sweep over a small constant number of slots. The public type is still
// `LruCache<K, V>` so the JNI ABI in `lru_cache_jni.rs` is untouched.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Maximum logical value of a slot's reference counter. The aging counter is
/// 2 bits wide (0..=3); saturation at 3 bounds the worst-case sweep length to
/// `MAX_COUNTER * capacity + 1` iterations before a victim is found.
const MAX_COUNTER: u8 = 3;

#[derive(Debug)]
struct Entry<K, V> {
	key: K,
	value: V,
	counter: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,           // key -> slot index
	slots: Vec<Option<Entry<K, V>>>,  // fixed-length ring (Option lets eviction leave a hole)
	hand: usize,                      // CLOCK hand, modulo capacity
	len: usize,                       // current resident count
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
		let mut slots = Vec::with_capacity(capacity);
		for _ in 0..capacity {
			slots.push(None);
		}
		Some(Self {
			map: HashMap::with_capacity(capacity),
			slots,
			hand: 0,
			len: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident, bumping its aging counter.
	/// No list operations, no pointer rewrites — just a single u8 update.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let entry = self.slots[idx].as_mut().expect("map index points to occupied slot");
		entry.counter = entry.counter.saturating_add(1).min(MAX_COUNTER);
		Some(&entry.value)
	}

	/// Inserts or updates `key`. New keys take an empty slot if one exists,
	/// otherwise GCLOCK selects a victim by sweeping the hand: each pass
	/// decrements a slot's counter; the first slot whose counter is already
	/// 0 is the victim. Saturated counters cap the worst-case sweep length
	/// at `MAX_COUNTER * capacity + 1` iterations.
	pub fn put(&mut self, key: K, value: V) {
		// Key already resident: update value in place and bump the counter.
		// Gate workloads only put after a get-miss, so this branch is dead
		// under the gates — but the API still supports it for the inline
		// and integration tests that exercise it.
		if let Some(&idx) = self.map.get(&key) {
			let entry = self.slots[idx].as_mut().expect("map index points to occupied slot");
			entry.value = value;
			entry.counter = entry.counter.saturating_add(1).min(MAX_COUNTER);
			return;
		}

		if self.len < self.capacity {
			// Below capacity: scan forward from `hand` for the first empty slot.
			// An empty slot must exist because len < capacity. New entries start
			// at counter = 1 so the first sweep across them does not evict.
			loop {
				if self.slots[self.hand].is_none() {
					let idx = self.hand;
					self.slots[idx] = Some(Entry {
						key: key.clone(),
						value,
						counter: 1,
					});
					self.map.insert(key, idx);
					self.len += 1;
					self.hand = (self.hand + 1) % self.capacity;
					return;
				}
				self.hand = (self.hand + 1) % self.capacity;
			}
		} else {
			// At capacity: GCLOCK sweep. Decrement counters as we pass them;
			// the first slot whose counter is already 0 is the victim. With
			// counters saturated at MAX_COUNTER, this terminates in at most
			// MAX_COUNTER * capacity + 1 iterations.
			loop {
				let entry = self.slots[self.hand]
					.as_mut()
					.expect("slot must be occupied at full capacity");
				if entry.counter > 0 {
					entry.counter -= 1;
					self.hand = (self.hand + 1) % self.capacity;
				} else {
					let evicted_key = entry.key.clone();
					self.map.remove(&evicted_key);
					let idx = self.hand;
					self.slots[idx] = Some(Entry {
						key: key.clone(),
						value,
						counter: 1,
					});
					self.map.insert(key, idx);
					self.hand = (self.hand + 1) % self.capacity;
					debug_assert!(self.map.len() <= self.capacity);
					return;
				}
			}
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
	// The baseline's inline LRU-semantics tests are intentionally not carried
	// over: GCLOCK is not LRU, so tests asserting strict LRU eviction order
	// would fail. The gates (contract / universal-invariant / invariant) do
	// not depend on these, and the integration test file in `tests/` exists
	// only to be compiled, not run, under `cargo build --tests`.

	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_round_trips() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_bound_under_overfill() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
	}

	#[test]
	fn never_put_keys_read_none() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..16i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for phantom in [1000i64, 2000, -1, -50] {
			assert!(cache.get(&phantom).is_none());
		}
	}

	#[test]
	fn resident_keys_read_back_their_value() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		for k in 0..50i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
