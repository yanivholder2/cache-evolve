// O(1)-amortised CLOCK (second-chance) cache using HashMap + arena ring.
// Nodes are stored in a Vec (arena) indexed by usize; the arena in slot order is the
// clock ring. A hit only sets the node's reference bit (no relinking). On eviction the
// hand sweeps the ring, clearing reference bits, and reuses the first unreferenced slot
// in place. No unsafe, no external crates.

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

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena == clock ring in slot order
	hand: usize,            // clock hand (next eviction candidate)
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
}
