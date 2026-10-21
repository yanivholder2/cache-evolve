// O(1)-amortised CLOCK (second-chance) approximation of LRU, using a HashMap plus a
// fixed-size ring of nodes. Each node has a one-bit reference flag instead of
// doubly-linked list pointers. On a hit the flag is set; on eviction the clock hand
// sweeps the ring, clearing set flags, and evicts the first unreferenced slot.
// Slots are reused in place, so no free-list management is needed.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	referenced: bool,
}

// Metadata budget: key + value + a one-byte reference flag, padded.
const _: () = assert!(std::mem::size_of::<Node<u64, u64>>() <= 24);

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // ring (arena)
	hand: usize,            // clock hand: index into `nodes`
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
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			hand: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, marking it as recently referenced.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx];
		node.referenced = true;
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim chosen by the clock hand.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in place and mark referenced.
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.referenced = true;
			return;
		}
		if self.nodes.len() < self.capacity {
			// Ring has room: allocate a new slot.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				referenced: false,
			});
			self.map.insert(key, idx);
		} else {
			// Ring full: find a victim and reuse its slot in place.
			let victim = self.find_victim();
			self.map.remove(&self.nodes[victim].key);
			let node = &mut self.nodes[victim];
			node.key = key.clone();
			node.value = value;
			node.referenced = false;
			self.map.insert(key, victim);
			self.hand = (victim + 1) % self.capacity;
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Sweep from `hand`, clearing reference bits, until an unreferenced slot is found.
	/// Terminates within `capacity + 1` steps since bits are cleared as the hand passes.
	fn find_victim(&mut self) -> usize {
		debug_assert_eq!(self.nodes.len(), self.capacity);
		loop {
			let node = &mut self.nodes[self.hand];
			if !node.referenced {
				return self.hand;
			}
			node.referenced = false;
			self.hand = (self.hand + 1) % self.capacity;
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
	fn node_is_at_most_24_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 24);
	}

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
	fn eviction_takes_first_unreferenced_at_hand() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // hand at 0, unreferenced -> evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn referenced_entry_survives_one_sweep() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // key 1 referenced
		cache.put(3, 30); // hand clears key 1's bit, evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn overwrite_sets_reference_bit() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // overwrite marks key 1 referenced
		cache.put(3, 30); // key 1 gets a second chance, key 2 evicted

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn all_referenced_full_sweep_evicts_at_hand() {
		// Unlike strict LRU (which would evict key 2), CLOCK clears every bit
		// and wraps back to the slot under the hand.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.get(&2);
		cache.get(&1);
		cache.put(3, 30); // clears both bits, wraps to slot 0 -> evicts key 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
		assert_eq!(*cache.get(&1).unwrap(), 10); // referenced
		cache.put(2, 20); // hand clears bit, wraps to 0, evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		cache.put(3, 30);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
	fn repeated_get_grants_second_chance() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // key 1 referenced -> skipped; key 2 evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn hand_advances_past_victim() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		cache.get(&2); // slot 1 referenced
		cache.put(4, 40); // hand 0 unreferenced -> evicts 1, hand -> 1
		cache.put(5, 50); // slot 1 referenced (cleared), slot 2 unreferenced -> evicts 3
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn slot_reuse_with_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3, hand -> 1
		cache.get(&3); // slot 0 referenced
		cache.put(4, 40); // slot 1 unreferenced -> evicts 2, slot 1 = key 4, hand -> 0
		cache.put(5, 50); // slot 0 referenced (cleared), slot 1 unreferenced -> evicts 4
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
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
	fn overwrite_at_capacity_then_insert() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite marks key 3 referenced
		cache.put(4, 40); // hand at slot 0 (key 1, unreferenced) -> evicts 1
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
