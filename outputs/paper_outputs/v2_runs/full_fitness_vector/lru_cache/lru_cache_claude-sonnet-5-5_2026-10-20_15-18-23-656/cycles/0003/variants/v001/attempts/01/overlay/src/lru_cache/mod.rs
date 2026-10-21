// Segmented LRU cache using HashMap (Fx-style hasher) + arena-based doubly-linked lists.
// Two segments: probationary (new keys) and protected (keys hit at least once).
// Nodes live in a Vec (arena); links are u32 indices with u32::MAX as NIL.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const PROB: usize = 0;
const PROT: usize = 1;
const NIL: u32 = u32::MAX;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Fx-style fast hasher.
#[derive(Debug, Default, Clone, Copy)]
pub struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for chunk in bytes.chunks(8) {
			let mut buf = [0u8; 8];
			buf[..chunk.len()].copy_from_slice(chunk);
			self.add(u64::from_le_bytes(buf));
		}
	}
	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.add(i as u64);
	}
	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}
	#[inline]
	fn finish(&self) -> u64 {
		// Avalanche fold: mix high bits down, multiply, fold again.
		let h = self.hash ^ (self.hash >> 32);
		let h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
		h ^ (h >> 29)
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
	seg: bool, // true = protected
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	heads: [u32; 2],               // LRU end per segment
	tails: [u32; 2],               // MRU end per segment
	protected_len: usize,
	protected_cap: usize,
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
		assert!(capacity < u32::MAX as usize, "capacity must be < u32::MAX");
		log::info!("LruCache.try_new: capacity={capacity}");
		let protected_cap = if capacity > 1 { (capacity * 4 / 5).max(1) } else { 0 };
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			heads: [NIL, NIL],
			tails: [NIL, NIL],
			protected_len: 0,
			protected_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.promote(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probationary LRU
	/// (or the protected LRU if probationary is empty).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.promote(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let victim = if self.heads[PROB] != NIL { self.heads[PROB] } else { self.heads[PROT] };
			assert!(victim != NIL, "non-empty list has a head");
			self.unlink(victim);
			let evicted_key = std::mem::replace(&mut self.nodes[victim as usize].key, key.clone());
			self.map.remove(&evicted_key);
			self.nodes[victim as usize].value = value;
			victim
		} else {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				seg: false,
			});
			idx
		};
		self.push_back(idx, PROB);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Handle a hit on `idx`.
	fn promote(&mut self, idx: u32) {
		if self.nodes[idx as usize].seg {
			if self.tails[PROT] == idx {
				return;
			}
			self.unlink(idx);
			self.push_back(idx, PROT);
			return;
		}
		if self.protected_cap == 0 {
			if self.tails[PROB] == idx {
				return;
			}
			self.unlink(idx);
			self.push_back(idx, PROB);
			return;
		}
		self.unlink(idx);
		self.push_back(idx, PROT);
		if self.protected_len > self.protected_cap {
			let h = self.heads[PROT];
			if h != NIL {
				self.unlink(h);
				self.push_back(h, PROB);
			}
		}
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let s = if self.nodes[i].seg { PROT } else { PROB };
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.heads[s] = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tails[s] = prev;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
		if s == PROT {
			self.protected_len -= 1;
		}
	}

	/// Append node to the MRU end of segment `s`.
	fn push_back(&mut self, idx: u32, s: usize) {
		let i = idx as usize;
		let t = self.tails[s];
		self.nodes[i].seg = s == PROT;
		self.nodes[i].prev = t;
		self.nodes[i].next = NIL;
		if t != NIL {
			self.nodes[t as usize].next = idx;
		} else {
			self.heads[s] = idx;
		}
		self.tails[s] = idx;
		if s == PROT {
			self.protected_len += 1;
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
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn hit_protects_from_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
