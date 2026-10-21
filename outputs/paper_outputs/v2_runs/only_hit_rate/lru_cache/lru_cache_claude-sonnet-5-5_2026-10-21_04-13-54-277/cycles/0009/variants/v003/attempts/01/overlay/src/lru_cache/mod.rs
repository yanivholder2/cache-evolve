// CLOCK cache with a 4-bit conservative-update count-min sketch admission gate.
// Slots live in a Vec; a hand sweeps clearing reference bits. Hits only set the
// reference bit. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const ROWS: usize = 4;
const MAX_COUNT: u8 = 15;

#[derive(Debug)]
struct Sketch {
	table: Vec<u8>, // packed 4-bit counters, ROWS * width counters
	width: usize,   // power of two
	total: usize,
	reset_at: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = (capacity.saturating_mul(2)).next_power_of_two().max(16);
		Sketch {
			table: vec![0u8; (ROWS * width + 1) / 2],
			width,
			total: 0,
			reset_at: capacity.saturating_mul(10).max(1),
		}
	}

	#[inline]
	fn indices<K: Hash>(&self, key: &K) -> [usize; ROWS] {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		let mut x = h.finish();
		let mask = self.width - 1;
		let mut out = [0usize; ROWS];
		for (r, o) in out.iter_mut().enumerate() {
			// splitmix-style remix per row
			x = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
			let mut z = x;
			z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
			z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
			z ^= z >> 31;
			*o = r * self.width + (z as usize & mask);
		}
		out
	}

	#[inline]
	fn get_counter(&self, i: usize) -> u8 {
		let b = self.table[i >> 1];
		if i & 1 == 0 {
			b & 0x0F
		} else {
			b >> 4
		}
	}

	#[inline]
	fn set_counter(&mut self, i: usize, v: u8) {
		let b = &mut self.table[i >> 1];
		if i & 1 == 0 {
			*b = (*b & 0xF0) | v;
		} else {
			*b = (*b & 0x0F) | (v << 4);
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u8 {
		let idx = self.indices(key);
		idx.iter().map(|&i| self.get_counter(i)).min().unwrap_or(0)
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		let idx = self.indices(key);
		let min = idx.iter().map(|&i| self.get_counter(i)).min().unwrap_or(0);
		if min < MAX_COUNT {
			for &i in &idx {
				if self.get_counter(i) == min {
					self.set_counter(i, min + 1);
				}
			}
		}
		self.total += 1;
		if self.total >= self.reset_at {
			self.halve();
		}
	}

	fn halve(&mut self) {
		for b in self.table.iter_mut() {
			// halve both nibbles
			*b = (*b >> 1) & 0x77;
		}
		self.total /= 2;
	}
}

#[derive(Debug)]
struct Slot<K, V> {
	key: K,
	value: V,
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,
	slots: Vec<Slot<K, V>>,
	hand: usize,
	capacity: usize,
	sketch: Sketch,
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
			slots: Vec::with_capacity(capacity),
			hand: 0,
			capacity,
			sketch: Sketch::new(capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its reference bit.
	/// Every access (hit or miss) is recorded in the sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.slots[idx].referenced = true;
		Some(&self.slots[idx].value)
	}

	/// Inserts `key`. If full, CLOCK picks a victim; the new key is admitted
	/// only if its sketch estimate is at least the victim's.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.slots[idx].value = value;
			self.slots[idx].referenced = true;
			return;
		}
		if self.slots.len() < self.capacity {
			let idx = self.slots.len();
			self.slots.push(Slot { key: key.clone(), value, referenced: false });
			self.map.insert(key, idx);
			return;
		}
		// Find victim: first unreferenced slot at/after the hand.
		loop {
			let h = self.hand;
			if self.slots[h].referenced {
				self.slots[h].referenced = false;
				self.hand = (h + 1) % self.capacity;
			} else {
				break;
			}
		}
		let v = self.hand;
		if self.sketch.estimate(&key) < self.sketch.estimate(&self.slots[v].key) {
			return; // rejected
		}
		let old = std::mem::replace(
			&mut self.slots[v],
			Slot { key: key.clone(), value, referenced: false },
		);
		self.map.remove(&old.key);
		self.map.insert(key, v);
		self.hand = (v + 1) % self.capacity;
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
	}

	#[test]
	fn capacity_bound() {
		let mut cache = LruCache::new(3);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert!((0..50i64).filter(|k| cache.get(k).is_some()).count() <= 3);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
