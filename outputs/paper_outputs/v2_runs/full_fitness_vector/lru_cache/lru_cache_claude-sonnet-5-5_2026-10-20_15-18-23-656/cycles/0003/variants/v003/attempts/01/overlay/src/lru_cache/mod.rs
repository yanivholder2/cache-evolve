// Segmented LRU cache using HashMap (Fx-style hasher) + arena-based doubly-linked lists.
// Two segments: probationary (new keys) and protected (keys hit at least once).
// Protected overflow is tolerated on get and repaired lazily (one demotion) on eviction in put.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const PROB: usize = 0;
const PROT: usize = 1;

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
		// Fold high bits down so hashbrown's low-bit/top-7-bit usage both get entropy.
		self.hash ^ (self.hash >> 32)
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: bool, // true = protected
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,          // arena
	heads: [Option<usize>; 2],       // LRU end per segment
	tails: [Option<usize>; 2],       // MRU end per segment
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
		log::info!("LruCache.try_new: capacity={capacity}");
		let protected_cap = if capacity > 1 { (capacity * 4 / 5).max(1) } else { 0 };
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			heads: [None, None],
			tails: [None, None],
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
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probationary LRU
	/// (or the protected LRU if probationary is empty).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			// Lazy repair: demote exactly one protected tail node if over cap.
			if self.protected_len > self.protected_cap {
				if let Some(h) = self.heads[PROT] {
					self.unlink(h);
					self.push_back(h, PROB);
				}
			}
			let victim = self.heads[PROB].or(self.heads[PROT]).expect("non-empty list has a head");
			self.unlink(victim);
			// Take the evicted key by move (no clone), swapping in the new key.
			let evicted_key = std::mem::replace(&mut self.nodes[victim].key, key.clone());
			self.map.remove(&evicted_key);
			self.nodes[victim].value = value;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				seg: false,
			});
			idx
		};
		self.push_back(idx, PROB);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Handle a hit on `idx`.
	fn promote(&mut self, idx: usize) {
		if self.nodes[idx].seg {
			// Fast path: already MRU of protected segment.
			if self.tails[PROT] == Some(idx) {
				return;
			}
			self.unlink(idx);
			self.push_back(idx, PROT);
			return;
		}
		if self.protected_cap == 0 {
			// Fast path: already MRU of probationary segment.
			if self.tails[PROB] == Some(idx) {
				return;
			}
			self.unlink(idx);
			self.push_back(idx, PROB);
			return;
		}
		// Promote without immediate demotion; overflow is repaired in put.
		self.unlink(idx);
		self.push_back(idx, PROT);
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: usize) {
		let s = if self.nodes[idx].seg { PROT } else { PROB };
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.heads[s] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tails[s] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		if s == PROT {
			self.protected_len -= 1;
		}
	}

	/// Append node to the MRU end of segment `s`.
	fn push_back(&mut self, idx: usize, s: usize) {
		self.nodes[idx].seg = s == PROT;
		self.nodes[idx].prev = self.tails[s];
		self.nodes[idx].next = None;
		match self.tails[s] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.heads[s] = Some(idx),
		}
		self.tails[s] = Some(idx);
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
