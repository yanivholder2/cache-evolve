// O(1) second-chance cache using HashMap + arena-based doubly-linked lists, with a ghost-list
// admission filter.
// Nodes are stored in a Vec (arena) indexed by usize, shared by two intrusive lists:
// a probation list (holds newly inserted keys and is the eviction list) and a protected list
// (layout kept from the segmented variant; the referenced-bit hit rule never promotes into it, so
// it stays empty and serves only as an eviction fallback). A hit does not relink the node: it
// sets the node's `referenced` flag. On eviction the probation LRU is examined: a referenced node
// has its flag cleared and is moved to the MRU end of its list (second chance), and the first
// unreferenced node is evicted. The walk is bounded to one full pass; if every node was
// referenced, the original LRU node is evicted. The slot is reused in-place, so no free-list
// management is needed. No unsafe, no external crates.
//
// Ghost admission filter: once the cache is full, a new key is admitted only if it is in the
// bounded ghost set of recently evicted/rejected keys. Otherwise it is recorded in the ghost set
// and not inserted. The ghost set holds at most `capacity` keys in FIFO order (generation-tagged
// deque, so updates are O(1) amortised). While the cache has free slots the filter is inactive.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	protected: bool,
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // probation LRU (eviction candidate)
	tail: Option<usize>,    // probation MRU
	prot_head: Option<usize>, // protected LRU (fallback eviction candidate)
	prot_tail: Option<usize>, // protected MRU
	capacity: usize,
	ghost: HashMap<K, u64>,          // ghost set: key -> generation
	ghost_order: VecDeque<(K, u64)>, // FIFO of (key, generation), bounded by capacity
	ghost_seq: u64,
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
			head: None,
			tail: None,
			prot_head: None,
			prot_tail: None,
			capacity,
			ghost: HashMap::with_capacity(capacity),
			ghost_order: VecDeque::with_capacity(capacity),
			ghost_seq: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. A hit sets the node's referenced bit; the node is
	/// not relinked, so the hit path performs a single key-index probe and no list writes.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)?;
		self.nodes[idx].referenced = true;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, a new key is admitted only if it is in the
	/// ghost set; otherwise it is recorded as a ghost and dropped. Admitted keys evict via
	/// second-chance selection over the probation list (falling back to the protected list).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and set the referenced bit
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].referenced = true;
			return;
		}
		// Admission filter: only active once the cache is full
		if self.nodes.len() >= self.capacity && self.ghost.remove(&key).is_none() {
			self.ghost_record(key);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: second-chance victim, then reuse its slot
			let victim = self.select_victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.ghost_record(evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].referenced = false;
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				protected: false,
				referenced: false,
			});
			idx
		};
		// New entries start in probation (MRU end)
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Second-chance victim selection. Walks the eviction list from its LRU end: a referenced
	/// node has its flag cleared and is moved to the MRU end of the same list; the first
	/// unreferenced node is returned. Bounded to one full pass: if every node was referenced,
	/// the original LRU node (now with a cleared flag) is returned.
	fn select_victim(&mut self) -> usize {
		let in_protected = self.head.is_none();
		let original = self
			.head
			.or(self.prot_head)
			.expect("full arena has a victim in one of the lists");
		for _ in 0..self.nodes.len() {
			let cur = if in_protected {
				self.prot_head
			} else {
				self.head
			};
			let Some(cur) = cur else { break };
			if !self.nodes[cur].referenced {
				return cur;
			}
			self.nodes[cur].referenced = false;
			self.unlink(cur);
			self.push_back(cur, in_protected);
		}
		original
	}

	/// Record `key` in the ghost set as the most recent entry, dropping the oldest ghosts so
	/// that at most `capacity` ghosts are retained. Stale deque entries (superseded by a later
	/// generation or removed on admission) are skipped when trimmed.
	fn ghost_record(&mut self, key: K) {
		if self.ghost.contains_key(&key) {
			return;
		}
		self.ghost_seq += 1;
		let generation = self.ghost_seq;
		self.ghost.insert(key.clone(), generation);
		self.ghost_order.push_back((key, generation));
		while self.ghost_order.len() > self.capacity {
			if let Some((old, gen)) = self.ghost_order.pop_front() {
				if self.ghost.get(&old) == Some(&gen) {
					self.ghost.remove(&old);
				}
			}
		}
	}

	/// Remove node from its current list (protected or probation).
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		let (head, tail) = if protected {
			(&mut self.prot_head, &mut self.prot_tail)
		} else {
			(&mut self.head, &mut self.tail)
		};
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => *head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => *tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of the protected or probation list.
	fn push_back(&mut self, idx: usize, protected: bool) {
		self.nodes[idx].protected = protected;
		let (head, tail) = if protected {
			(&mut self.prot_head, &mut self.prot_tail)
		} else {
			(&mut self.head, &mut self.tail)
		};
		let old_tail = *tail;
		self.nodes[idx].prev = old_tail;
		self.nodes[idx].next = None;
		match old_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => *head = Some(idx),
		}
		*tail = Some(idx);
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
		// Should still have room for a second key without eviction
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (probation head)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // sets key 1's referenced bit: second chance
		cache.put(3, 30); // key 1 is spared and rotated; evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update sets key 1's referenced bit
		cache.put(3, 30); // key 1 is spared and rotated; evicts key 2

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
		cache.put(2, 20); // cache full
		cache.put(1, 11); // overwrite, must NOT evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // key 1 is referenced and spared; key 2 is evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // sets referenced bit on 2
		cache.put(4, 40); // 1 is unreferenced LRU: evicted
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // set referenced bit on key 3 (slot 0)
		cache.put(4, 40); // 3 spared, 2 evicted -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 3 or 4 by second chance; slot reused
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node; get sets its referenced bit
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify list is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite: sets key 3's referenced bit
		// Eviction should take 1 (unreferenced LRU)
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
