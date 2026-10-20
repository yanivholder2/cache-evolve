// O(1)-amortised GCLOCK cache: a HashMap from key to slot index plus a fixed ring of
// nodes, each carrying a 2-bit saturating frequency counter (0..=FREQ_MAX). A hit only
// bumps the counter, so no nodes are relinked. On eviction the clock hand sweeps the
// ring, decrementing non-zero counters, until it finds a slot with counter 0. That
// slot's old key is removed from the map, then the slot is reused in place. No unsafe,
// no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Saturation ceiling of the per-slot frequency counter (2 bits).
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
	nodes: Vec<Node<K, V>>, // clock ring (arena)
	hand: usize,            // next slot the clock sweep inspects
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
		node.freq += (node.freq < FREQ_MAX) as u8;
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim chosen by the clock sweep.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and bump the counter
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.freq += (node.freq < FREQ_MAX) as u8;
			return;
		}
		if self.nodes.len() < self.capacity {
			// Ring has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
			});
			self.map.insert(key, idx);
		} else {
			// Ring full: GCLOCK sweep. Each full revolution lowers every non-zero counter
			// by one, so the sweep ends within FREQ_MAX * capacity + 1 steps.
			let cap = self.capacity;
			let mut hand = self.hand;
			#[cfg(debug_assertions)]
			let mut steps: usize = 0;
			loop {
				#[cfg(debug_assertions)]
				{
					steps += 1;
					debug_assert!(
						steps <= FREQ_MAX as usize * cap + 1,
						"GCLOCK sweep exceeded FREQ_MAX * capacity + 1 steps"
					);
				}
				let node = &mut self.nodes[hand];
				if node.freq > 0 {
					node.freq -= 1;
					hand = if hand + 1 == cap { 0 } else { hand + 1 };
				} else {
					break;
				}
			}
			let victim = hand;
			// Remove the old key from the map before reusing the slot
			self.map.remove(&self.nodes[victim].key);
			let node = &mut self.nodes[victim];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			self.map.insert(key, victim);
			self.hand = if victim + 1 == cap { 0 } else { victim + 1 };
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
	fn node_is_compact() {
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
	fn eviction_takes_first_zero_count_slot() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // hand at slot 0, freq 0 -> evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_increments_counter() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		assert_eq!(freq_of(&cache, 1), 0);
		cache.get(&1); // key 1 freq 0 -> 1
		assert_eq!(freq_of(&cache, 1), 1);
		cache.put(3, 30); // sweep decrements key 1 to 0, evicts key 2
		assert_eq!(freq_of(&cache, 1), 0);
		assert!(cache.map.get(&2).is_none());

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_update_increments_counter() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update bumps key 1's counter to 1
		assert_eq!(freq_of(&cache, 1), 1);
		cache.put(1, 12); // and to 2
		assert_eq!(freq_of(&cache, 1), 2);
		cache.put(3, 30); // sweep decrements key 1 to 1, evicts key 2
		assert_eq!(freq_of(&cache, 1), 1);

		assert_eq!(*cache.get(&1).unwrap(), 12);
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
		assert_eq!(*cache.get(&1).unwrap(), 10); // freq 1
		cache.put(2, 20); // sweep decrements to 0, wraps, evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20); // freq 1
		for _ in 0..5 {
			cache.get(&2); // saturates at FREQ_MAX
		}
		assert_eq!(freq_of(&cache, 2), FREQ_MAX);
		cache.put(3, 30); // three decrements around the 1-slot ring, then evicts 2
		assert!(cache.map.get(&2).is_none());
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
	fn repeated_get_saturates_at_three_chances() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		for _ in 0..10 {
			cache.get(&1);
		}
		// Ten gets saturate at FREQ_MAX: three chances, not ten.
		assert_eq!(freq_of(&cache, 1), FREQ_MAX);

		cache.put(4, 40); // key 1: 3 -> 2; evicts 2 (slot 1); hand -> 2
		assert!(cache.map.get(&2).is_none());
		cache.put(5, 50); // slot 2 (key 3, freq 0) evicted; hand -> 0
		assert!(cache.map.get(&3).is_none());
		cache.put(6, 60); // key 1: 2 -> 1; evicts 4 (slot 1); hand -> 2
		assert!(cache.map.get(&4).is_none());
		cache.put(7, 70); // evicts 5 (slot 2); hand -> 0
		assert!(cache.map.get(&5).is_none());
		cache.put(8, 80); // key 1: 1 -> 0; evicts 6 (slot 1); hand -> 2
		assert!(cache.map.get(&6).is_none());
		cache.put(9, 90); // evicts 7 (slot 2); hand -> 0
		assert!(cache.map.get(&7).is_none());
		// Key 1 survived three passes of the hand; its counter is now exhausted.
		assert!(cache.map.contains_key(&1));
		assert_eq!(freq_of(&cache, 1), 0);
		cache.put(10, 100); // hand at slot 0 (key 1, freq 0) -> evicts 1
		assert!(cache.map.get(&1).is_none());
		assert_eq!(*cache.get(&8).unwrap(), 80);
		assert_eq!(*cache.get(&9).unwrap(), 90);
		assert_eq!(*cache.get(&10).unwrap(), 100);
	}

	#[test]
	fn get_middle_node_marks_without_reordering() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // slot 2
		cache.get(&2); // bumps slot 1's counter
		cache.put(4, 40); // hand at slot 0 (freq 0) -> evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_with_counter() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3 (freq 0), hand -> 1
		assert_eq!(freq_of(&cache, 3), 0);
		cache.get(&3); // slot 0 freq 1
		cache.get(&3); // slot 0 freq 2
		cache.put(4, 40); // hand at slot 1 (key 2, freq 0) -> slot 1 reused, hand -> 0
		cache.put(5, 50); // slot 0: 2 -> 1, advance; slot 1 (key 4) evicted; hand -> 0
		assert!(cache.map.get(&4).is_none());
		assert_eq!(freq_of(&cache, 3), 1);
		cache.put(6, 60); // slot 0: 1 -> 0, advance; slot 1 (key 5) evicted; hand -> 0
		assert!(cache.map.get(&5).is_none());
		assert_eq!(freq_of(&cache, 3), 0);
		cache.put(7, 70); // slot 0 (key 3, freq 0) evicted
		assert!(cache.map.get(&3).is_none());
		assert_eq!(*cache.get(&6).unwrap(), 60);
		assert_eq!(*cache.get(&7).unwrap(), 70);
	}

	#[test]
	fn full_sweep_when_all_referenced() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // full revolution lowers every counter to 0, evicts slot 0 (key 1)
		assert!(cache.map.get(&1).is_none());
		cache.put(5, 50); // slot 1 (key 2) now freq 0 -> evicted
		assert!(cache.map.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn all_saturated_sweep_terminates_and_evicts_hand_slot() {
		let mut cache = LruCache::new(4);
		for k in 0..4 {
			cache.put(k, k * 10);
		}
		cache.put(4, 40); // evicts slot 0 (key 0); hand -> 1
		assert_eq!(cache.hand, 1);
		for k in 1..=4 {
			for _ in 0..FREQ_MAX + 2 {
				cache.get(&k);
			}
			assert_eq!(freq_of(&cache, k), FREQ_MAX);
		}
		// Every slot is at FREQ_MAX: three full revolutions drain the counters, and the
		// sweep (bounded by the debug_assert) evicts the slot the hand started on.
		let start = cache.hand;
		let victim_key = cache.nodes[start].key;
		cache.put(100, 1000);
		assert!(cache.map.get(&victim_key).is_none());
		assert_eq!(cache.map[&100], start);
		assert_eq!(cache.hand, (start + 1) % 4);
		for i in 0..4 {
			if i != start {
				assert_eq!(cache.nodes[i].freq, 0);
			}
		}
	}

	#[test]
	fn randomized_structure_holds() {
		for &cap in &[1usize, 2, 3, 8, 17] {
			let mut cache: LruCache<i64, i64> = LruCache::new(cap);
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ cap as u64;
			for _ in 0..5000 {
				state = state
					.wrapping_mul(6364136223846793005)
					.wrapping_add(1442695040888963407);
				let r = state >> 33;
				let key = (r % 40) as i64;
				if cache.get(&key).is_none() {
					cache.put(key, key * 7);
				} else if r & 0x100 != 0 {
					cache.put(key, key * 7); // in-place update path
				}
				assert!(cache.map.len() <= cap);
				assert!(cache.nodes.len() <= cap);
				assert_eq!(cache.map.len(), cache.nodes.len());
				assert!(cache.hand < cap);
				for (i, node) in cache.nodes.iter().enumerate() {
					assert_eq!(cache.map[&node.key], i);
					assert!(node.freq <= FREQ_MAX);
					assert_eq!(node.value, node.key * 7);
				}
			}
		}
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
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite bumps slot 2's counter
		cache.put(4, 40); // hand at slot 0 (freq 0) -> evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		let _ = LruCache::<i64, i64>::new(0);
	}
}
