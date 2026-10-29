// O(1)-amortised GCLOCK (generalised CLOCK) cache using HashMap + arena ring.
// Nodes are stored in a Vec (arena) indexed by usize; the arena in slot order is the
// clock ring. A hit only bumps the node's 2-bit saturating frequency counter (no
// relinking). On eviction the hand sweeps the ring, decrementing counters, and reuses
// the first slot whose counter is zero in place. The counter lives in the padding
// byte the CLOCK reference bit used, so `Node<u64, u64>` stays 24 bytes.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Saturation ceiling of the per-node frequency counter (2 bits).
const FREQ_MAX: u8 = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	freq: u8,
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

	/// Returns the value for `key` if present, bumping its saturating frequency counter.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx];
		// Branch keeps a saturated hot key from dirtying its line on every hit.
		if node.freq < FREQ_MAX {
			node.freq += 1;
		}
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts via the GCLOCK sweep.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in place and bump its counter
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			if node.freq < FREQ_MAX {
				node.freq += 1;
			}
			return;
		}
		if self.nodes.len() < self.capacity {
			// Arena has room: fresh inserts start at zero until hit
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
			});
			self.map.insert(key, idx);
		} else {
			// Arena full: run the GCLOCK sweep. Each step over a non-zero slot
			// decrements its counter by one, so the loop would terminate on its own
			// within FREQ_MAX rotations. Liveness/latency bound: the sweep is capped
			// at 2 * capacity steps; if the cap is hit, the slot currently under the
			// hand is evicted regardless of its remaining count. This bounds the
			// worst-case put (p99) at two rotations.
			let limit = 2 * self.capacity;
			let mut steps = 0usize;
			let mut hand = self.hand;
			while self.nodes[hand].freq > 0 && steps < limit {
				self.nodes[hand].freq -= 1;
				hand += 1;
				if hand == self.capacity {
					hand = 0;
				}
				steps += 1;
			}
			let victim = &mut self.nodes[hand];
			self.map.remove(&victim.key);
			victim.key = key.clone();
			victim.value = value;
			victim.freq = 0;
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

	fn freq_of(cache: &LruCache<i64, i64>, key: i64) -> u8 {
		cache.nodes[cache.map[&key]].freq
	}

	#[test]
	fn node_u64_is_24_bytes() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
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
	fn eviction_removes_zero_count_at_hand() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // hand at slot 0 (key 1, freq 0) -> evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_grants_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // key 1 freq -> 1
		cache.put(3, 30); // decrements key 1 to 0, evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_overwrite_grants_second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // overwrite bumps counter
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
		cache.put(2, 20); // decrements 1 to 0, wraps, evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn capacity_one_saturated_hits_cap() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		for _ in 0..5 {
			cache.get(&1);
		}
		cache.put(2, 20); // cap (2 steps) reached, evicts slot under hand
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
	fn repeated_get_saturates_at_freq_max() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(freq_of(&cache, 1), 0);
		cache.get(&1);
		assert_eq!(freq_of(&cache, 1), 1);
		for _ in 0..10 {
			cache.get(&1);
		}
		assert_eq!(freq_of(&cache, 1), FREQ_MAX);
		cache.put(1, 11);
		assert_eq!(freq_of(&cache, 1), FREQ_MAX);
	}

	#[test]
	fn hit_thrice_outlives_hit_once() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		for _ in 0..3 {
			cache.get(&1); // freq 3
		}
		cache.get(&2); // freq 1
		// Sweep 1: key1 3->2, key2 1->0, evicts key 3 at slot 2. hand -> 0.
		cache.put(4, 40);
		assert!(cache.get(&3).is_none());
		assert_eq!(freq_of(&cache, 1), 2);
		assert_eq!(freq_of(&cache, 2), 0);
		// Sweep 2: key1 2->1, evicts key 2 at slot 1 (hit once: survived one pass).
		cache.put(5, 50);
		assert!(!cache.map.contains_key(&2));
		// Key 1 (hit three times) has survived two passes.
		assert!(cache.map.contains_key(&1));
		assert_eq!(freq_of(&cache, 1), 1);
	}

	#[test]
	fn hit_thrice_survives_three_passes() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		for _ in 0..3 {
			cache.get(&1);
		}
		cache.put(4, 40); // pass 1 over key 1 (3->2), evicts 2 at slot 1, hand -> 2
		cache.put(5, 50); // evicts 3 at slot 2, hand -> 0
		cache.put(6, 60); // pass 2 over key 1 (2->1), evicts 4 at slot 1, hand -> 2
		cache.put(7, 70); // evicts 5 at slot 2, hand -> 0
		cache.put(8, 80); // pass 3 over key 1 (1->0), evicts 6 at slot 1, hand -> 2
		assert!(cache.map.contains_key(&1));
		assert_eq!(freq_of(&cache, 1), 0);
		cache.put(9, 90); // evicts 7 at slot 2, hand -> 0
		cache.put(10, 100); // key 1 now has freq 0 -> evicted
		assert!(!cache.map.contains_key(&1));
		assert!(cache.map.contains_key(&8));
		assert!(cache.map.contains_key(&9));
		assert!(cache.map.contains_key(&10));
	}

	#[test]
	fn sweep_cap_evicts_under_hand() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		for _ in 0..3 {
			cache.get(&1);
			cache.get(&2);
		}
		// 4-step cap: both decremented twice; slot 0 (key 1) evicted despite freq 1.
		cache.put(3, 30);
		assert!(!cache.map.contains_key(&1));
		assert_eq!(freq_of(&cache, 2), 1);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(cache.map.len(), 2);
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
		cache.get(&3); // slot 0 freq 1
		cache.put(4, 40); // slot 1 (key 2) freq 0 -> evicted, hand -> 0
		cache.put(5, 50); // slot 0 freq 1 -> 0; slot 1 (key 4) evicted
		// Strict LRU would have evicted 3 here; GCLOCK keeps it.
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn full_rotation_when_all_hit_once() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // decrements all to 0, wraps, evicts key 1 at slot 0
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
		cache.put(3, 31); // overwrite bumps 3's counter
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
