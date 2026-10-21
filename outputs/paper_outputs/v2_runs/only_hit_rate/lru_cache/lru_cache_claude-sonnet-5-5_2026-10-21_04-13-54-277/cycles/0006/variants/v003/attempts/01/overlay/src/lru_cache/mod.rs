// CLOCK (1-bit) cache with a TinyLFU-style count-min sketch admission gate.
// Slots live in a Vec; a hand sweeps them. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const ROWS: usize = 4;
const SEEDS: [u64; ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0x85EB_CA77_C2B2_AE63,
];

/// Count-min sketch with 4 rows of 4-bit counters (two per byte) and periodic halving.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>, // ROWS rows, each row_bytes long
	row_bytes: usize,
	bits: u32,
	additions: usize,
	sample: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.next_power_of_two().max(16);
		Self {
			table: vec![0u8; ROWS * width / 2],
			row_bytes: width / 2,
			bits: width.trailing_zeros(),
			additions: 0,
			sample: capacity.saturating_mul(10).max(1),
		}
	}

	#[inline]
	fn index(&self, h: u64, row: usize) -> usize {
		let x = (h ^ (h >> 29)).wrapping_mul(SEEDS[row]);
		(x >> (64 - self.bits)) as usize
	}

	#[inline]
	fn get_counter(&self, row: usize, i: usize) -> u8 {
		let b = self.table[row * self.row_bytes + (i >> 1)];
		if i & 1 == 0 {
			b & 0x0F
		} else {
			b >> 4
		}
	}

	fn estimate(&self, h: u64) -> u8 {
		let mut m = u8::MAX;
		for r in 0..ROWS {
			m = m.min(self.get_counter(r, self.index(h, r)));
		}
		m
	}

	fn increment(&mut self, h: u64) {
		for r in 0..ROWS {
			let i = self.index(h, r);
			let pos = r * self.row_bytes + (i >> 1);
			let b = self.table[pos];
			if i & 1 == 0 {
				if b & 0x0F < 15 {
					self.table[pos] = b + 1;
				}
			} else if b >> 4 < 15 {
				self.table[pos] = b + 0x10;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for b in self.table.iter_mut() {
				*b = (*b >> 1) & 0x77;
			}
			self.additions = 0;
		}
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
	map: HashMap<K, usize>, // key -> slot index
	nodes: Vec<Node<K, V>>, // CLOCK ring
	hand: usize,
	capacity: usize,
	sketch: Sketch,
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
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
			sketch: Sketch::new(capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its referenced bit.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(hash_of(key));
		let &idx = self.map.get(key)?;
		self.nodes[idx].referenced = true;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, the hand's victim candidate is
	/// compared against the new key by sketch frequency; the new key is dropped
	/// when its estimate is strictly lower.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].referenced = true;
			return;
		}
		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				referenced: false,
			});
			self.map.insert(key, idx);
			return;
		}
		// Sweep: clear referenced bits until an unreferenced victim is found.
		let n = self.nodes.len();
		while self.nodes[self.hand].referenced {
			self.nodes[self.hand].referenced = false;
			self.hand = if self.hand + 1 == n { 0 } else { self.hand + 1 };
		}
		let victim = self.hand;
		let new_est = self.sketch.estimate(hash_of(&key));
		let vic_est = self.sketch.estimate(hash_of(&self.nodes[victim].key));
		if new_est < vic_est {
			return; // admission rejected
		}
		let old_key = std::mem::replace(&mut self.nodes[victim].key, key.clone());
		self.map.remove(&old_key);
		self.nodes[victim].value = value;
		self.nodes[victim].referenced = false;
		self.map.insert(key, victim);
		self.hand = if victim + 1 == n { 0 } else { victim + 1 };
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
	fn capacity_bound() {
		let mut cache = LruCache::<i64, i64>::new(3);
		for k in 0..50 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert!((0..50).filter(|k| cache.get(k).is_some()).count() <= 3);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
