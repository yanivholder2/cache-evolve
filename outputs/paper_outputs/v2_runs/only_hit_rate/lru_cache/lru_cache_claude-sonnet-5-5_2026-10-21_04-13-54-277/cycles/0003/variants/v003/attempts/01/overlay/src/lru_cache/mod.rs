// Cache with 2-bit saturating-frequency CLOCK eviction.
// Slots live in a Vec ring. New entries start at freq 0; hits increment freq (cap 3).
// On eviction the hand sweeps: freq > 0 -> decrement and advance; freq == 0 -> evict.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	freq: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> slot index
	nodes: Vec<Node<K, V>>, // ring of slots
	hand: usize,
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

	/// Returns the value for `key` if present, bumping its frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let n = &mut self.nodes[idx];
		if n.freq < MAX_FREQ {
			n.freq += 1;
		}
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts via the clock hand.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let n = &mut self.nodes[idx];
			n.value = value;
			if n.freq < MAX_FREQ {
				n.freq += 1;
			}
			return;
		}
		if self.nodes.len() >= self.capacity {
			// Sweep the hand until a freq-0 victim is found (terminates within 4 laps).
			let victim = loop {
				let h = self.hand;
				self.hand = if h + 1 >= self.nodes.len() { 0 } else { h + 1 };
				let n = &mut self.nodes[h];
				if n.freq == 0 {
					break h;
				}
				n.freq -= 1;
			};
			let old = std::mem::replace(
				&mut self.nodes[victim],
				Node {
					key: key.clone(),
					value,
					freq: 0,
				},
			);
			self.map.remove(&old.key);
			self.map.insert(key, victim);
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
			});
			self.map.insert(key, idx);
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
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn hot_key_survives_scan() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		for _ in 0..3 {
			cache.get(&1);
		}
		for k in 2..4 {
			cache.put(k, k);
		}
		cache.put(4, 4);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
