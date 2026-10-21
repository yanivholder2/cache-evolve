// Segmented LRU (probation + protected) over a HashMap + arena-based doubly-linked lists.
// Links are u32 indices with NIL = 0x7FFF_FFFF; bit 31 of `next` stores the segment flag
// (0 = probation, 1 = protected). Hashing uses Fx multiply + fold.
// On eviction the slot is reused in-place. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const FLAG: u32 = 0x8000_0000;
const MASK: u32 = 0x7FFF_FFFF;
const NIL: u32 = MASK;
const MAX_CAPACITY: usize = (MASK - 1) as usize;
const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[inline(always)]
fn next_of(n: u32) -> u32 {
	n & MASK
}

#[inline(always)]
fn set_next(n: u32, v: u32) -> u32 {
	(n & FLAG) | v
}

#[inline(always)]
fn seg_of(n: u32) -> usize {
	(n >> 31) as usize
}

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
	next: u32, // low 31 bits: link; bit 31: segment flag
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	head: [u32; 2],                // LRU end: [probation, protected]
	tail: [u32; 2],                // MRU end: [probation, protected]
	prot_len: usize,
	prot_cap: usize,
	capacity: usize,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 || capacity > MAX_CAPACITY {
			log::error!("LruCache.try_new: capacity must be in 1..=2^31-2");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			head: [NIL; 2],
			tail: [NIL; 2],
			prot_len: 0,
			prot_cap: (capacity / 5 * 4 + (capacity % 5) * 4 / 5).max(1),
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts from probation (else protected).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.touch(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let victim = if self.head[0] != NIL { self.head[0] } else { self.head[1] };
			debug_assert!(victim != NIL, "a list must be non-empty when arena is full");
			let evicted_key = self.nodes[victim as usize].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let n = &mut self.nodes[victim as usize];
			n.key = key.clone();
			n.value = value;
			victim
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
		self.push_back(idx, 0);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Hit handling: protected hit -> move to protected MRU (early return if already there);
	/// probation hit -> promote to protected, demoting protected LRU if over cap.
	#[inline]
	fn touch(&mut self, idx: u32) {
		let seg = seg_of(self.nodes[idx as usize].next);
		if seg == 1 {
			if self.tail[1] == idx {
				return;
			}
			self.unlink(idx);
			self.push_back(idx, 1);
		} else {
			self.unlink(idx);
			self.push_back(idx, 1);
			if self.prot_len > self.prot_cap {
				let d = self.head[1];
				self.unlink(d);
				self.push_back(d, 0);
			}
		}
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: u32) {
		let prev = self.nodes[idx as usize].prev;
		let raw = self.nodes[idx as usize].next;
		let next = next_of(raw);
		let seg = seg_of(raw);
		if prev != NIL {
			let p = &mut self.nodes[prev as usize];
			p.next = set_next(p.next, next);
		} else {
			self.head[seg] = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail[seg] = prev;
		}
		self.prot_len -= seg;
		let n = &mut self.nodes[idx as usize];
		n.prev = NIL;
		n.next = NIL;
	}

	/// Append node to the MRU end of segment `seg`.
	fn push_back(&mut self, idx: u32, seg: usize) {
		let t = self.tail[seg];
		let n = &mut self.nodes[idx as usize];
		n.prev = t;
		n.next = NIL | ((seg as u32) << 31);
		if t != NIL {
			let tn = &mut self.nodes[t as usize];
			tn.next = set_next(tn.next, idx);
		} else {
			self.head[seg] = idx;
		}
		self.tail[seg] = idx;
		self.prot_len += seg;
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
