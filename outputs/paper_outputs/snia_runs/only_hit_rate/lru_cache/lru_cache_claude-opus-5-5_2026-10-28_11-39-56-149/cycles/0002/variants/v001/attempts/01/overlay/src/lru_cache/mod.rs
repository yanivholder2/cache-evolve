// O(1) Segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists,
// extended with a bounded key-only ghost history of recently evicted probation keys.
// Two lists share one arena: a probationary segment (~20%) that receives new entries,
// and a protected segment (~80%) that only re-referenced entries reach. One-touch
// scan traffic therefore only churns the probationary segment.
// A key evicted from probation is remembered (key only, no value) in a FIFO ghost of
// `capacity` entries; if it is put again while still remembered, it is admitted
// straight into the protected segment, so reuse distances longer than the probation
// list can still be detected.
// Nodes are stored in a Vec (arena) indexed by u32. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Sentinel for "no node" in the u32-indexed linked lists.
const NIL: u32 = u32::MAX;

/// Segment identifiers (index into the `head`/`tail` arrays).
const PROB: u8 = 0;
const PROT: u8 = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
	segment: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: [u32; 2],         // LRU (front) per segment: [probation, protected]
	tail: [u32; 2],         // MRU (back) per segment
	prot_len: usize,
	prot_cap: usize,
	capacity: usize,
	// Ghost history (keys only) of recently evicted probation entries.
	ghost_map: HashMap<K, u64>,       // key -> sequence number assigned at eviction (truth)
	ghost_fifo: VecDeque<(K, u64)>,   // insertion order; may contain stale entries
	ghost_seq: u64,
	ghost_cap: usize,
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
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity {capacity} exceeds u32 index range");
			return None;
		}
		let prot_cap = if capacity == 1 { 0 } else { std::cmp::max(1, capacity * 8 / 10) };
		let ghost_cap = if prot_cap == 0 { 0 } else { capacity };
		log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap} ghost_cap={ghost_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: [NIL; 2],
			tail: [NIL; 2],
			prot_len: 0,
			prot_cap,
			capacity,
			ghost_map: HashMap::with_capacity(ghost_cap),
			ghost_fifo: VecDeque::with_capacity(ghost_cap + 1),
			ghost_seq: 0,
			ghost_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it per SLRU rules.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`. New keys enter the probationary segment, unless they are
	/// remembered in the ghost history, in which case they enter the protected segment.
	/// If at capacity, evicts the probationary LRU (or protected LRU when probation is empty).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a re-reference
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.touch(idx);
			return;
		}
		// Ghost check first (before eviction can push more keys into the ghost).
		let ghost_hit = self.prot_cap != 0 && self.ghost_map.remove(&key).is_some();
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick victim from probation LRU, else protected LRU
			let victim = if self.head[PROB as usize] != NIL {
				self.head[PROB as usize]
			} else {
				self.head[PROT as usize]
			};
			debug_assert!(victim != NIL, "a list must be non-empty when arena is full");
			let vi = victim as usize;
			let evicted_key = self.nodes[vi].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			if self.nodes[vi].segment == PROT {
				self.prot_len -= 1;
			} else if self.prot_cap != 0 {
				self.ghost_insert(evicted_key);
			}
			let node = &mut self.nodes[vi];
			node.key = key.clone();
			node.value = value;
			node.segment = PROB;
			victim
		} else {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				segment: PROB,
			});
			idx
		};
		// Slot filled; now choose its segment.
		if ghost_hit {
			self.nodes[idx as usize].segment = PROT;
			self.push_back(idx);
			self.prot_len += 1;
			if self.prot_len > self.prot_cap {
				self.demote_protected_lru();
			}
		} else {
			// Link at the probationary MRU end
			self.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Record an evicted probation key in the bounded ghost history.
	fn ghost_insert(&mut self, key: K) {
		self.ghost_seq += 1;
		let seq = self.ghost_seq;
		self.ghost_map.insert(key.clone(), seq);
		self.ghost_fifo.push_back((key, seq));
		while self.ghost_fifo.len() > self.ghost_cap {
			if let Some((old_key, old_seq)) = self.ghost_fifo.pop_front() {
				// Lazy deletion: only drop the map entry if it is this exact ghosting.
				if self.ghost_map.get(&old_key) == Some(&old_seq) {
					self.ghost_map.remove(&old_key);
				}
			}
		}
	}

	/// Move the protected LRU node to the probation MRU end.
	fn demote_protected_lru(&mut self) {
		let demote = self.head[PROT as usize];
		debug_assert!(demote != NIL);
		self.unlink(demote);
		self.nodes[demote as usize].segment = PROB;
		self.push_back(demote);
		self.prot_len -= 1;
	}

	/// Handle a re-reference: promote probation -> protected (demoting protected LRU on
	/// overflow), or move a protected node to protected MRU.
	fn touch(&mut self, idx: u32) {
		let i = idx as usize;
		self.unlink(idx);
		if self.nodes[i].segment == PROB {
			if self.prot_cap == 0 {
				// Degenerate (capacity 1): everything stays in probation
				self.push_back(idx);
				return;
			}
			self.nodes[i].segment = PROT;
			self.push_back(idx);
			self.prot_len += 1;
			if self.prot_len > self.prot_cap {
				self.demote_protected_lru();
			}
		} else {
			self.push_back(idx);
		}
	}

	/// Remove node from its segment's list.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let seg = self.nodes[i].segment as usize;
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head[seg] = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail[seg] = prev;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
	}

	/// Append node to the MRU end of its segment's list.
	fn push_back(&mut self, idx: u32) {
		let i = idx as usize;
		let seg = self.nodes[i].segment as usize;
		let tail = self.tail[seg];
		self.nodes[i].prev = tail;
		self.nodes[i].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx;
		} else {
			self.head[seg] = idx;
		}
		self.tail[seg] = idx;
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
	fn node_fits_baseline_size() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 48);
	}

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
	fn eviction_removes_probation_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (probation LRU)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_promotes_to_protected() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // 1 -> protected
		cache.put(3, 30); // evicts key 2 (probation LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_existing_promotes_to_protected() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // 1 -> protected
		cache.put(3, 30); // evicts key 2

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
		cache.put(4, 40);
		cache.put(5, 50);
		cache.put(6, 60);

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
	fn capacity_one_skips_ghost() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20); // evicts 1, not ghosted
		assert!(cache.ghost_map.is_empty());
		assert!(cache.ghost_fifo.is_empty());
		cache.put(1, 11); // evicts 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(cache.prot_len, 0);
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
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // evicts key 2 (probation LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // 2 -> protected; probation: 1, 3
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		// capacity 2 -> prot_cap 1
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3; probation: 2, 3
		cache.get(&3); // 3 -> protected; probation: 2
		cache.put(4, 40); // evicts 2 -> slot 1 reused for key 4; probation: 4
		cache.put(5, 50); // evicts 4 (probation LRU) -> slot 1 reused again; 3 stays protected
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		// capacity 2 -> prot_cap 1
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // protected: 1; probation: 2
		cache.get(&2); // protected: 2; 1 demoted -> probation: 1
		cache.put(3, 30); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn scan_does_not_flush_protected() {
		let mut cache = LruCache::new(10); // prot_cap 8
		for k in 0..8 {
			cache.put(k, k);
			cache.get(&k); // promote working set
		}
		for k in 100..200 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for k in 0..8 {
			assert_eq!(*cache.get(&k).unwrap(), k);
		}
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
		cache.put(3, 31); // 3 -> protected
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn ghost_hit_enters_protected_and_survives_scan() {
		let cap = 10; // prot_cap 8
		let mut cache = LruCache::new(cap);
		for k in 0..cap as i64 {
			cache.put(k, k);
		}
		cache.put(100, 100); // evicts key 0 from probation -> ghosted
		assert!(cache.get(&0).is_none());
		assert!(cache.ghost_map.contains_key(&0));
		cache.put(0, 7); // ghost hit -> protected
		assert!(!cache.ghost_map.contains_key(&0));
		let idx = cache.map[&0];
		assert_eq!(cache.nodes[idx as usize].segment, PROT);
		assert_eq!(cache.prot_len, 1);
		// Scan of `capacity` new keys must not evict the protected key.
		for k in 1000..1000 + cap as i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&0).unwrap(), 7);
	}

	#[test]
	fn ghost_hit_overflow_demotes_protected_lru() {
		// capacity 2 -> prot_cap 1
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.get(&1); // protected: 1
		cache.put(2, 20); // probation: 2
		cache.put(3, 30); // evicts 2 (ghosted); probation: 3
		cache.put(2, 21); // ghost hit: evicts 3 (ghosted), 2 -> protected, 1 demoted
		assert_eq!(cache.prot_len, 1);
		let i2 = cache.map[&2];
		let i1 = cache.map[&1];
		assert_eq!(cache.nodes[i2 as usize].segment, PROT);
		assert_eq!(cache.nodes[i1 as usize].segment, PROB);
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 21);
	}

	#[test]
	fn ghost_hits_consume_ghost_entries() {
		// capacity 2 -> prot_cap 1; each ghost hit removes its entry, each probation
		// eviction adds one.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts 1 (probation) -> ghost {1}
		cache.put(1, 11); // ghost hit; evicts 2 (probation) -> ghost {2}; 1 protected
		cache.put(2, 21); // ghost hit; evicts 3 (prob) -> ghost {3}; 2 protected, 1 demoted
		assert_eq!(cache.ghost_map.len(), 1);
		assert!(cache.ghost_map.contains_key(&3));
	}

	#[test]
	fn ghost_is_bounded_with_lazy_deletion() {
		let cap = 4;
		let mut cache = LruCache::new(cap);
		for k in 0..200i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			if cache.get(&(k / 3)).is_none() {
				cache.put(k / 3, k / 3);
			}
			assert!(cache.ghost_fifo.len() <= cache.ghost_cap);
			assert!(cache.ghost_map.len() <= cache.ghost_cap);
			assert!(cache.map.len() <= cap);
			assert!(cache.prot_len <= cache.prot_cap);
			for k in cache.map.keys() {
				assert!(!cache.ghost_map.contains_key(k), "resident key also in ghost");
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
