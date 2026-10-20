// Segmented LRU cache using HashMap + arena-based doubly-linked lists.
// Two segments: probationary (new keys) and protected (keys hit at least once).
// Nodes live in a Vec (arena); on eviction the slot is reused in place.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const PROB: usize = 0;
const PROT: usize = 1;

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
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	heads: [Option<usize>; 2], // LRU end per segment
	tails: [Option<usize>; 2], // MRU end per segment
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
			map: HashMap::with_capacity(capacity),
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
			let victim = self.heads[PROB].or(self.heads[PROT]).expect("non-empty list has a head");
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
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
			self.unlink(idx);
			self.push_back(idx, PROT);
			return;
		}
		if self.protected_cap == 0 {
			self.unlink(idx);
			self.push_back(idx, PROB);
			return;
		}
		self.unlink(idx);
		self.push_back(idx, PROT);
		if self.protected_len > self.protected_cap {
			if let Some(h) = self.heads[PROT] {
				self.unlink(h);
				self.push_back(h, PROB);
			}
		}
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
	fn eviction_removes_probationary_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn hit_protects_from_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // protected
		cache.put(3, 30); // evicts 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn scan_does_not_flush_protected() {
		let mut cache = LruCache::new(5); // protected cap 4
		for k in 0..4 {
			cache.put(k, k);
			cache.get(&k);
		}
		for k in 100..200 {
			cache.put(k, k);
		}
		for k in 0..4 {
			assert_eq!(*cache.get(&k).unwrap(), k);
		}
	}

	#[test]
	fn protected_overflow_demotes() {
		let mut cache = LruCache::new(3); // protected cap 2
		cache.put(1, 1);
		cache.put(2, 2);
		cache.put(3, 3);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3); // protected overflow: 1 demoted to probationary
		cache.put(4, 4); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 2);
		assert_eq!(*cache.get(&3).unwrap(), 3);
		assert_eq!(*cache.get(&4).unwrap(), 4);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		for k in 1..=6 {
			cache.put(k, k * 10);
		}
		for k in 1..=3 {
			assert!(cache.get(&k).is_none());
		}
		for k in 4..=6 {
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
	fn slot_reuse_with_protection() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts 1
		cache.get(&3); // 3 protected
		cache.put(4, 40); // evicts 2
		cache.put(5, 50); // evicts 4 (probationary), 3 stays
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn all_protected_evicts_protected_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20);
		cache.get(&2); // protected cap 1: 1 demoted
		cache.put(3, 30); // evicts 1 (probationary)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
