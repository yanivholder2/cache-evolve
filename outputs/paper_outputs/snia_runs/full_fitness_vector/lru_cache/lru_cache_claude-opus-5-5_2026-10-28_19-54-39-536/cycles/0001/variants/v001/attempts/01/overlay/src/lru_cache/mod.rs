// O(1) segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Two segments share one arena: probation (~20%) and protected (~80%). New keys enter
// probation; a second hit promotes them to protected. Protected overflow demotes the
// protected LRU back to probation MRU. Eviction takes the probation LRU (falling back to
// the protected LRU only if probation is empty). On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	protected: bool, // segment marker: false = probation, true = protected
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	prob_head: Option<usize>, // probation LRU (front)
	prob_tail: Option<usize>, // probation MRU (back)
	prot_head: Option<usize>, // protected LRU (front)
	prot_tail: Option<usize>, // protected MRU (back)
	prob_len: usize,
	prot_len: usize,
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
		let protected_cap = if capacity == 1 {
			0
		} else {
			std::cmp::max(1, capacity * 8 / 10)
		};
		log::info!("LruCache.try_new: capacity={capacity} protected_cap={protected_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prot_head: None,
			prot_tail: None,
			prob_len: 0,
			prot_len: 0,
			protected_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, applying SLRU promotion.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probation LRU entry.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and apply the same promotion as get
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict probation LRU (fallback: protected LRU) and reuse its slot
			let victim = self
				.prob_head
				.or(self.prot_head)
				.expect("non-empty cache has a victim");
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].protected = false;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				protected: false,
			});
			idx
		};
		// Link the new/reused slot at the probation MRU and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
	}

	/// SLRU hit handling: probation -> protected MRU (with overflow demotion);
	/// protected -> protected MRU.
	fn on_hit(&mut self, idx: usize) {
		self.unlink(idx);
		if self.nodes[idx].protected {
			self.push_back(idx);
			return;
		}
		self.nodes[idx].protected = true;
		self.push_back(idx);
		if self.prot_len > self.protected_cap {
			// Demote protected LRU to probation MRU (no eviction)
			let demote = self.prot_head.expect("non-empty protected segment has a head");
			self.unlink(demote);
			self.nodes[demote].protected = false;
			self.push_back(demote);
		}
	}

	/// Remove node from its current segment list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if protected {
					self.prot_head = next
				} else {
					self.prob_head = next
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if protected {
					self.prot_tail = prev
				} else {
					self.prob_tail = prev
				}
			}
		}
		if protected {
			self.prot_len -= 1;
		} else {
			self.prob_len -= 1;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of its segment list.
	fn push_back(&mut self, idx: usize) {
		let protected = self.nodes[idx].protected;
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => {
				if protected {
					self.prot_head = Some(idx)
				} else {
					self.prob_head = Some(idx)
				}
			}
		}
		if protected {
			self.prot_tail = Some(idx);
			self.prot_len += 1;
		} else {
			self.prob_tail = Some(idx);
			self.prob_len += 1;
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
	fn overwrite_does_not_grow_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts probation LRU: key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		// SLRU: get(1) promotes key 1 to protected; key 2 is the probation LRU.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30); // evicts key 2 (probation LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		// SLRU: overwriting key 1 promotes it to protected.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.put(3, 30); // evicts key 2 (probation LRU)

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
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
	fn repeated_get_preserves_eviction_order() {
		// SLRU: key 1 is protected after the first hit; key 2 is the probation LRU.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		// SLRU: get(2) moves key 2 from the middle of probation into protected.
		// Probation: 1 (LRU), 3; protected: 2.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2);
		cache.put(4, 40); // evicts 1 (probation LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		// SLRU (capacity 2, protected_cap 1): a promoted key survives one-touch inserts.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probation
		cache.put(2, 20); // slot 1, probation
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // key 3 promoted to protected
		cache.put(4, 40); // evicts 2 (probation LRU) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (probation LRU) -> slot 1 reused AGAIN for key 5
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(3, 31); // overwrite promotes 3 to protected
		cache.put(4, 40); // evicts 1 (probation LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn scan_does_not_flush_protected() {
		let mut cache = LruCache::new(5);
		cache.put(1, 10);
		cache.get(&1); // protected
		for k in 2..=50 {
			cache.put(k, k * 10); // one-touch scan through probation
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		// capacity 5 -> protected_cap 4
		let mut cache = LruCache::new(5);
		for k in 1..=5 {
			cache.put(k, k * 10);
		}
		for k in 1..=5 {
			cache.get(&k); // 5th promotion demotes key 1 to probation
		}
		assert_eq!(cache.prot_len, 4);
		assert_eq!(cache.prob_len, 1);
		cache.put(6, 60); // evicts key 1 (sole probation entry)
		assert!(cache.get(&1).is_none());
		for k in 2..=6 {
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
