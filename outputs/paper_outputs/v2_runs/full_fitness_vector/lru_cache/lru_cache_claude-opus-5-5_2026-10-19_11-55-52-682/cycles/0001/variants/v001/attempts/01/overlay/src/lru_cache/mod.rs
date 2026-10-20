// O(1) Segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists.
// Nodes live in a Vec (arena) indexed by u32. Two intrusive lists share the arena:
// a probationary segment (new / once-touched keys) and a protected segment (keys
// that have been hit at least once since insertion). Protected is capped at 80% of
// capacity; overflow demotes the protected LRU back to probationary MRU.
// Eviction takes the probationary LRU, so one-touch scans cannot flush reused keys.
// On eviction the slot is reused in place, so no free-list is needed. No unsafe.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;
const PROB: u8 = 0;
const PROT: u8 = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
	seg: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	prob_head: u32,         // probationary LRU
	prob_tail: u32,         // probationary MRU
	prot_head: u32,         // protected LRU
	prot_tail: u32,         // protected MRU
	prot_len: usize,
	prot_cap: usize, // 0 => promotion disabled (capacity 1: plain LRU)
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
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity {capacity} exceeds u32 index range");
			return None;
		}
		let prot_cap = if capacity > 1 {
			std::cmp::max(1, capacity * 4 / 5)
		} else {
			0
		};
		log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: NIL,
			prob_tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			prot_len: 0,
			prot_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, applying the SLRU promotion rule.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.on_hit(idx as u32);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the probationary LRU entry.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in place and apply the hit rule.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx as u32);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: victim is the probationary LRU; fall back to protected LRU.
			let victim = if self.prob_head != NIL {
				self.prob_head
			} else {
				self.prot_head
			};
			debug_assert!(victim != NIL, "a full cache must have a victim");
			let v = victim as usize;
			let evicted_key = self.nodes[v].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[v].key = key.clone();
			self.nodes[v].value = value;
			victim
		} else {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				seg: PROB,
			});
			idx
		};
		self.push_back(idx, PROB);
		self.map.insert(key, idx as usize);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// SLRU hit rule: probationary -> protected MRU (demoting protected LRU on
	/// overflow); protected -> protected MRU.
	fn on_hit(&mut self, idx: u32) {
		let seg = self.nodes[idx as usize].seg;
		if seg == PROT {
			self.unlink(idx);
			self.push_back(idx, PROT);
			return;
		}
		self.unlink(idx);
		if self.prot_cap == 0 {
			// Capacity 1: no protected segment; behave as plain LRU.
			self.push_back(idx, PROB);
			return;
		}
		self.push_back(idx, PROT);
		if self.prot_len > self.prot_cap {
			let demote = self.prot_head;
			self.unlink(demote);
			self.push_back(demote, PROB);
		}
	}

	/// Remove node from whichever segment list it is in.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
		let seg = self.nodes[i].seg;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else if seg == PROT {
			self.prot_head = next;
		} else {
			self.prob_head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else if seg == PROT {
			self.prot_tail = prev;
		} else {
			self.prob_tail = prev;
		}
		if seg == PROT {
			self.prot_len -= 1;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
	}

	/// Append node to the MRU end of segment `seg`, tagging it with that segment.
	fn push_back(&mut self, idx: u32, seg: u8) {
		let i = idx as usize;
		let tail = if seg == PROT {
			self.prot_tail
		} else {
			self.prob_tail
		};
		self.nodes[i].seg = seg;
		self.nodes[i].prev = tail;
		self.nodes[i].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx;
		}
		if seg == PROT {
			if tail == NIL {
				self.prot_head = idx;
			}
			self.prot_tail = idx;
			self.prot_len += 1;
		} else {
			if tail == NIL {
				self.prob_head = idx;
			}
			self.prob_tail = idx;
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

	/// Walk both lists and verify structural consistency.
	fn check_structure(c: &LruCache<i64, i64>) {
		let mut seen = 0usize;
		for (head, seg) in [(c.prob_head, PROB), (c.prot_head, PROT)] {
			let mut cur = head;
			let mut prev = NIL;
			let mut n = 0usize;
			while cur != NIL {
				let node = &c.nodes[cur as usize];
				assert_eq!(node.seg, seg);
				assert_eq!(node.prev, prev);
				assert_eq!(c.map.get(&node.key), Some(&(cur as usize)));
				prev = cur;
				cur = node.next;
				n += 1;
			}
			if seg == PROT {
				assert_eq!(n, c.prot_len);
				assert_eq!(c.prot_tail, prev);
				assert!(c.prot_len <= c.prot_cap);
			} else {
				assert_eq!(c.prob_tail, prev);
			}
			seen += n;
		}
		assert_eq!(seen, c.map.len());
		assert!(c.map.len() <= c.capacity);
	}

	#[test]
	fn node_fits_in_32_bytes_for_i64() {
		assert!(std::mem::size_of::<Node<i64, i64>>() <= 32);
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
	fn eviction_removes_probationary_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (probationary LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn hit_promotes_out_of_probation() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // key 1 -> protected
		cache.put(3, 30); // evicts key 2 (only probationary entry)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_update_promotes_out_of_probation() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update promotes key 1
		cache.put(3, 30); // evicts key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(cache.prot_len, 0);
		check_structure(&cache);
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
	fn repeated_get_keeps_key_protected() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // evicts key 2 (probationary LRU)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // prob [1,3], prot [2]
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn slot_reuse_prefers_probation_victims() {
		let mut cache = LruCache::new(2); // prot_cap = 1
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // key 3 -> protected
		cache.put(4, 40); // evicts 2 (slot 1) -> key 4 probationary
		cache.put(5, 50); // evicts 4 (probationary LRU), key 3 survives
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
		check_structure(&cache);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		check_structure(&cache);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(3, 31);
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		let mut cache = LruCache::new(5); // prot_cap = 4
		for k in 1..=5 {
			cache.put(k, k * 10);
		}
		for k in 1..=5 {
			cache.get(&k); // 5th promotion demotes key 1
		}
		assert_eq!(cache.prot_len, 4);
		check_structure(&cache);
		cache.put(6, 60); // evicts key 1 (only probationary entry)
		assert!(cache.get(&1).is_none());
		for k in 2..=6 {
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
		check_structure(&cache);
	}

	#[test]
	fn scan_does_not_flush_protected_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.get(&1); // protected
		for k in 100..200 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
		check_structure(&cache);
	}

	#[test]
	fn victim_falls_back_to_protected_when_probation_empty() {
		let mut cache = LruCache::new(2); // prot_cap = 1
		cache.put(1, 10);
		cache.get(&1); // prob [], prot [1]
		cache.put(2, 20); // prob [2]
		cache.get(&2); // prot [2], demote 1 -> prob [1]
		cache.put(3, 30); // evicts 1
		assert!(cache.get(&1).is_none());
		check_structure(&cache);
	}

	#[test]
	fn randomized_structure_holds() {
		let mut cache = LruCache::new(7);
		let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..20_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = (x % 23) as i64;
			match cache.get(&k) {
				Some(&v) => assert_eq!(v, k * 3),
				None => cache.put(k, k * 3),
			}
		}
		check_structure(&cache);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
