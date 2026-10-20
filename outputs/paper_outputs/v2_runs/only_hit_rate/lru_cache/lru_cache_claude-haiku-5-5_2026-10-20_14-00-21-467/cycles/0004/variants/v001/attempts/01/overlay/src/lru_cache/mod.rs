// O(1) segmented LRU cache using HashMap + arena-based doubly-linked lists, with a ghost-list
// admission filter.
// Nodes are stored in a Vec (arena) indexed by usize, shared by two intrusive lists:
// a protected segment (~80% of capacity, holds keys re-referenced at least once) and a
// probation segment (the rest, holds newly inserted keys). New keys enter probation and are
// evicted first, so one-hit-wonders cannot displace the reused working set. A hit on a
// probation node promotes it to the protected segment; overflow demotes the protected LRU
// back to probation. On eviction the slot is reused in-place, so no free-list management is
// needed. No unsafe, no external crates.
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
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // probation LRU (eviction candidate)
	tail: Option<usize>,    // probation MRU
	prot_head: Option<usize>, // protected LRU (demotion candidate)
	prot_tail: Option<usize>, // protected MRU
	prot_len: usize,
	prot_cap: usize,
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
			prot_len: 0,
			prot_cap: (capacity.saturating_mul(4) / 5).max(1),
			capacity,
			ghost: HashMap::with_capacity(capacity),
			ghost_order: VecDeque::with_capacity(capacity),
			ghost_seq: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it within the segmented order.
	/// Exactly one key-index probe is made on the hit path; the node is then addressed by its
	/// arena index for the (skipped or applied) recency update.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)?;
		// Already the protected MRU: a relink would leave both lists unchanged, so skip it.
		if !(self.nodes[idx].protected && self.prot_tail == Some(idx)) {
			self.touch(idx);
		}
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, a new key is admitted only if it is in the
	/// ghost set; otherwise it is recorded as a ghost and dropped. Admitted keys evict from the
	/// probation LRU (falling back to the protected LRU when probation is empty).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and apply the hit rule
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		// Admission filter: only active once the cache is full
		if self.nodes.len() >= self.capacity && self.ghost.remove(&key).is_none() {
			self.ghost_record(key);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict probation head (or protected head) and reuse its slot
			let victim = self
				.head
				.or(self.prot_head)
				.expect("full arena has a victim in one of the lists");
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.ghost_record(evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
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
			});
			idx
		};
		// New entries start in probation (MRU end)
		self.push_back(idx, false);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
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

	/// Apply the hit rule: a probation node is promoted to the protected MRU, a protected
	/// node is moved to the protected MRU. Protected overflow demotes its LRU to probation.
	fn touch(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back(idx, true);
		while self.prot_len > self.prot_cap {
			let demoted = self
				.prot_head
				.expect("protected overflow implies a protected head");
			self.unlink(demoted);
			self.push_back(demoted, false);
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
		if protected {
			self.prot_len -= 1;
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
		if protected {
			self.prot_len += 1;
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
		cache.get(&1); // promotes key 1 to protected
		cache.put(3, 30); // evicts key 2 (probation, not protected key 1)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 promotes it to protected
		cache.put(3, 30); // evicts key 2 (probation)

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
		cache.put(4, 40); // should evict key 2 (probation LRU), not protected key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probation LRU
		cache.put(2, 20); // probation middle
		cache.put(3, 30); // probation MRU
		cache.get(&2); // promotes 2 to protected
		// Probation order: 1, 3. Protected: 2.
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3 (probation)
		cache.get(&3); // promote slot 0 (key 3) to protected
		cache.put(4, 40); // evicts 2 (probation) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (probation) -> slot 1 reused AGAIN for key 5
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survives
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single node; get promotes it and re-attaches it
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
		cache.put(3, 30); // full; 3 is probation MRU
		cache.put(3, 31); // overwrite: promotes 3 to protected
		// Eviction should take 1 (probation LRU)
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
