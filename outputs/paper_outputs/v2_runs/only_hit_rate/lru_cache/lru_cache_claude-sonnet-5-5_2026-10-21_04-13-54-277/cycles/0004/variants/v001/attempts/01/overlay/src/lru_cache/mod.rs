// Segmented LRU cache: HashMap + arena shared by two doubly-linked lists
// (probation = 0, protected = 1). New keys enter probation; a hit promotes to protected.
// Eviction takes the probation LRU (fallback: protected LRU) and reuses its arena slot.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const PROBATION: usize = 0;
const PROTECTED: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: bool, // false = probation, true = protected
}

#[derive(Debug, Clone, Copy, Default)]
struct List {
	head: Option<usize>, // LRU end
	tail: Option<usize>, // MRU end
	len: usize,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	lists: [List; 2],
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
		// 95% protected share, at least 1, always leaving >= 1 probation slot.
		let protected_cap = if capacity > 1 {
			(capacity * 19 / 20).max(1).min(capacity - 1)
		} else {
			0
		};
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::default(); 2],
			protected_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the hit.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts from probation (else protected).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let seg = if self.lists[PROBATION].head.is_some() { PROBATION } else { PROTECTED };
			let victim = self.lists[seg].head.expect("non-empty cache has a victim");
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
				segment: false,
			});
			idx
		};
		self.push_back(idx, PROBATION);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Handle a hit on `idx`: probation -> protected MRU (with demotion), protected -> protected MRU.
	fn touch(&mut self, idx: usize) {
		let was_protected = self.nodes[idx].segment;
		self.unlink(idx);
		self.push_back(idx, PROTECTED);
		if !was_protected {
			while self.lists[PROTECTED].len > self.protected_cap {
				let demote = self.lists[PROTECTED].head.expect("protected non-empty");
				self.unlink(demote);
				self.push_back(demote, PROBATION);
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.lists[seg].head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.lists[seg].tail = prev,
		}
		self.lists[seg].len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of segment `seg`.
	fn push_back(&mut self, idx: usize, seg: usize) {
		let tail = self.lists[seg].tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		self.nodes[idx].segment = seg == PROTECTED;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.lists[seg].head = Some(idx),
		}
		self.lists[seg].tail = Some(idx);
		self.lists[seg].len += 1;
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
	fn scan_does_not_evict_hot_key() {
		let mut cache = LruCache::new(5);
		cache.put(1, 10);
		cache.get(&1); // promoted to protected
		for k in 2..20 {
			cache.put(k, k);
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
