// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Links are u32 indices with u32::MAX as NIL; hashing uses Fx multiply + fold.
// On eviction the slot is reused in-place. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;
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
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	head: u32,                     // LRU (front)
	tail: u32,                     // MRU (back)
	capacity: usize,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 || capacity > (u32::MAX - 1) as usize {
			log::error!("LruCache.try_new: capacity must be in 1..=u32::MAX-1");
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

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used entry.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.touch(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			debug_assert!(self.head != NIL, "head must exist when arena is full");
			let head_idx = self.head;
			let evicted_key = self.nodes[head_idx as usize].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			let n = &mut self.nodes[head_idx as usize];
			n.key = key.clone();
			n.value = value;
			head_idx
		} else {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			idx
		};
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Promote to MRU; early return if already the tail.
	#[inline]
	fn touch(&mut self, idx: u32) {
		if self.tail == idx {
			return;
		}
		self.unlink(idx);
		self.push_back(idx);
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: u32) {
		let prev = self.nodes[idx as usize].prev;
		let next = self.nodes[idx as usize].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail = prev;
		}
		self.nodes[idx as usize].prev = NIL;
		self.nodes[idx as usize].next = NIL;
	}

	/// Append node to the tail (MRU end) of the list.
	fn push_back(&mut self, idx: u32) {
		let t = self.tail;
		self.nodes[idx as usize].prev = t;
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
	fn eviction_and_order() {
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
