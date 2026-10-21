// O(1) second-chance (CLOCK-like) cache using HashMap + arena-based doubly-linked list.
// Links are u32 indices; hashing uses Fx multiply + fold. `get` does not relink: it sets a
// reference bit stored in the top bit of `prev`. Eviction gives referenced tail-side nodes
// a second chance (clear bit, move to back). No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;
const REF_BIT: u32 = 1 << 31;
const PREV_MASK: u32 = !REF_BIT;
const PREV_NIL: u32 = PREV_MASK;
const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Default, Clone, Copy)]
pub struct FxFoldHasher {
	hash: u64,
}

impl FxFoldHasher {
	#[inline]
	fn add(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxFoldHasher {
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
		let h = self.hash;
		h ^ (h >> 32)
	}
}

type FxBuild = BuildHasherDefault<FxFoldHasher>;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32, // low 31 bits: prev link (PREV_NIL = none); top bit: referenced
	next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	head: u32,                     // eviction side (front)
	tail: u32,                     // newest side (back)
	capacity: usize,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 || capacity >= PREV_NIL as usize {
			log::error!("LruCache.try_new: capacity must be in 1..2^31-1");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	#[inline]
	fn prev_of(&self, idx: u32) -> u32 {
		let p = self.nodes[idx as usize].prev & PREV_MASK;
		if p == PREV_NIL {
			NIL
		} else {
			p
		}
	}

	/// Set prev link, preserving the referenced bit.
	#[inline]
	fn set_prev(&mut self, idx: u32, p: u32) {
		let n = &mut self.nodes[idx as usize];
		let enc = if p == NIL { PREV_NIL } else { p };
		n.prev = (n.prev & REF_BIT) | enc;
	}

	/// Returns the value for `key` if present, marking it referenced (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let n = &mut self.nodes[idx as usize];
		n.prev |= REF_BIT;
		Some(&n.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a non-referenced entry.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let n = &mut self.nodes[idx as usize];
			n.value = value;
			n.prev |= REF_BIT;
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			debug_assert!(self.head != NIL, "head must exist when arena is full");
			let mut victim = self.head;
			let mut budget = self.nodes.len();
			while budget > 0 && self.nodes[victim as usize].prev & REF_BIT != 0 {
				// second chance: clear bit, move to back
				self.nodes[victim as usize].prev &= PREV_MASK;
				self.unlink(victim);
				self.push_back(victim);
				victim = self.head;
				budget -= 1;
			}
			let evicted_key = self.nodes[victim as usize].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let n = &mut self.nodes[victim as usize];
			n.prev = PREV_NIL; // clears referenced bit
			n.key = key.clone();
			n.value = value;
			victim
		} else {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: PREV_NIL,
				next: NIL,
			});
			idx
		};
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: u32) {
		let prev = self.prev_of(idx);
		let next = self.nodes[idx as usize].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.set_prev(next, prev);
		} else {
			self.tail = prev;
		}
		self.set_prev(idx, NIL);
		self.nodes[idx as usize].next = NIL;
	}

	/// Append node to the tail (newest end) of the list.
	fn push_back(&mut self, idx: u32) {
		let t = self.tail;
		self.set_prev(idx, t);
		self.nodes[idx as usize].next = NIL;
		if t != NIL {
			self.nodes[t as usize].next = idx;
		} else {
			self.head = idx;
		}
		self.tail = idx;
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
	fn second_chance() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
