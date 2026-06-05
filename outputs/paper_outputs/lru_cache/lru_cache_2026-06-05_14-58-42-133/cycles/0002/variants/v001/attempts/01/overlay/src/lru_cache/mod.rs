// O(1) Segmented LRU (SLRU) cache using HashMap + arena-based doubly-linked lists,
// augmented with a value-less "ghost" directory (A1out-style) over the probationary
// segment.
//
// The arena (`nodes: Vec<Node>`) and `map: HashMap<K, usize>` are unchanged from
// the baseline LRU. The single intrusive list is split into TWO segments:
//   * a *probationary* segment (`head`/`tail`), where freshly inserted keys land;
//   * a *protected* segment (`prot_head`/`prot_tail`), holding keys that have been
//     hit at least once.
//
// On a hit a probationary node is promoted to the protected MRU tail. The protected
// segment is capped at `prot_cap = (capacity * 4) / 5`; overflow demotes the
// protected-LRU node back to the probationary MRU tail. Eviction always targets the
// probationary-LRU (the probationary list head), falling back to the protected-LRU
// only when the probationary segment is empty. Slots are reused in place exactly as
// the baseline does.
//
// Ghost directory: keys evicted from the *probationary* segment (one-hit-wonders or
// never-hit entries) are remembered — without value or arena slot — in a bounded
// FIFO (`ghost` + `ghost_order`, capped at `ghost_cap = capacity`). When such a key
// returns, it is readmitted *directly into the protected segment* rather than landing
// in probationary, capturing reuse-distance signal the plain SLRU discards. The ghost
// directory never holds a key that is currently resident in `map`. No unsafe, no
// external crates.

use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// false = probationary segment, true = protected segment.
	protected: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	// Probationary segment list.
	head: Option<usize>, // probationary LRU (front)
	tail: Option<usize>, // probationary MRU (back)
	// Protected segment list.
	prot_head: Option<usize>, // protected LRU (front)
	prot_tail: Option<usize>, // protected MRU (back)
	prot_len: usize,          // number of resident protected nodes
	prot_cap: usize,          // protected-segment capacity (~80% of capacity)
	capacity: usize,
	// Ghost directory: value-less keys recently evicted from the probationary
	// segment, ordered FIFO and bounded to `ghost_cap`. Never resident in `map`.
	ghost: HashSet<K>,
	ghost_order: VecDeque<K>,
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
		// ~80% of total capacity is reserved for the protected segment.
		let prot_cap = (capacity * 4) / 5;
		// The ghost directory remembers up to `capacity` recently-evicted keys.
		let ghost_cap = capacity;
		log::info!("LruCache.try_new: capacity={capacity} prot_cap={prot_cap} ghost_cap={ghost_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			prot_cap,
			capacity,
			ghost: HashSet::with_capacity(ghost_cap),
			ghost_order: VecDeque::with_capacity(ghost_cap),
			ghost_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it within the SLRU.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// A hit promotes the node into / within the protected segment.
		self.promote(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used entry.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote (mirror a get-hit).
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.promote(idx);
			return;
		}
		// New key — does it return from the ghost directory? A returning ghost key
		// is readmitted directly into the protected segment.
		let from_ghost = self.remove_ghost(&key);
		// New key — need an arena slot.
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick a victim. Prefer the probationary-LRU (head); fall back
			// to the protected-LRU only when the probationary segment is empty.
			let victim = match self.head {
				Some(h) => h,
				None => self
					.prot_head
					.expect("non-empty cache has a probationary or protected head"),
			};
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			// Detach from whichever segment it currently occupies, accounting for
			// protected occupancy before clearing the flag.
			let victim_protected = self.nodes[victim].protected;
			if victim_protected {
				self.prot_len -= 1;
			}
			self.unlink(victim);
			// Only probationary victims are recorded as ghosts; protected victims
			// already proved their worth and are dropped outright.
			if !victim_protected {
				self.record_ghost(evicted_key);
			}
			// Reuse the slot in place. Segment is fixed up below based on `from_ghost`.
			self.nodes[victim].protected = false;
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			victim
		} else {
			// Arena has room: allocate a new slot.
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
		if from_ghost {
			// Readmit directly into the protected MRU tail, honouring the cap.
			self.nodes[idx].protected = true;
			self.push_back(idx);
			self.prot_len += 1;
			if self.prot_len > self.prot_cap {
				self.demote_one();
			}
		} else {
			// Link the new/reused slot at the probationary tail (MRU).
			self.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Record `key` (just evicted from the probationary segment) into the ghost
	/// directory, evicting the FIFO-oldest ghost when over `ghost_cap`. The key is
	/// guaranteed non-resident in `map` at call time.
	fn record_ghost(&mut self, key: K) {
		if self.ghost_cap == 0 {
			return;
		}
		if self.ghost.insert(key.clone()) {
			self.ghost_order.push_back(key);
			if self.ghost_order.len() > self.ghost_cap {
				if let Some(old) = self.ghost_order.pop_front() {
					self.ghost.remove(&old);
				}
			}
		}
	}

	/// Remove `key` from the ghost directory if present, returning whether it was a
	/// ghost hit.
	fn remove_ghost(&mut self, key: &K) -> bool {
		if self.ghost.remove(key) {
			if let Some(pos) = self.ghost_order.iter().position(|k| k == key) {
				self.ghost_order.remove(pos);
			}
			true
		} else {
			false
		}
	}

	/// Promote `idx` on a hit. A probationary node moves into the protected MRU
	/// tail (possibly demoting the protected-LRU to keep `prot_len <= prot_cap`);
	/// a node already protected is simply moved to the protected MRU tail.
	fn promote(&mut self, idx: usize) {
		if self.nodes[idx].protected {
			// Already protected: move to protected MRU tail.
			self.unlink(idx);
			self.push_back(idx);
		} else {
			// Probationary -> protected.
			self.unlink(idx);
			self.nodes[idx].protected = true;
			self.push_back(idx);
			self.prot_len += 1;
			if self.prot_len > self.prot_cap {
				self.demote_one();
			}
		}
	}

	/// Demote the protected-LRU node back to the probationary MRU tail.
	fn demote_one(&mut self) {
		if let Some(ph) = self.prot_head {
			// `ph` is currently protected, so unlink updates the protected list.
			self.unlink(ph);
			self.nodes[ph].protected = false;
			self.prot_len -= 1;
			self.push_back(ph);
		}
	}

	/// Remove node from its current segment's list. The node's `protected` flag
	/// selects which segment's head/tail endpoints to fix up.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if protected {
					self.prot_head = next;
				} else {
					self.head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if protected {
					self.prot_tail = prev;
				} else {
					self.tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU tail of its current segment (selected by the
	/// node's `protected` flag).
	fn push_back(&mut self, idx: usize) {
		let protected = self.nodes[idx].protected;
		let old_tail = if protected { self.prot_tail } else { self.tail };
		self.nodes[idx].prev = old_tail;
		self.nodes[idx].next = None;
		match old_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => {
				if protected {
					self.prot_head = Some(idx);
				} else {
					self.head = Some(idx);
				}
			}
		}
		if protected {
			self.prot_tail = Some(idx);
		} else {
			self.tail = Some(idx);
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
		cache.put(3, 30); // evicts key 1 (probationary LRU)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it (into protected).
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify lists are still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn ghost_readmission_enters_protected() {
		// capacity 2, prot_cap = 1. Insert 1,2; insert 3 evicts probationary-LRU 1
		// into the ghost directory. Re-inserting 1 should readmit it as protected.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 -> ghost
		assert!(cache.get(&1).is_none());
		cache.put(1, 11); // ghost hit -> protected admission
		assert_eq!(*cache.get(&1).unwrap(), 11);
		// Ghost directory must not retain a now-resident key.
		assert!(!cache.ghost.contains(&1));
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
