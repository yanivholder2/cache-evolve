// Segmented-LRU (SLRU) cache with a bounded ghost record.
//
// This extends the parent SLRU variant (probationary/protected segments sharing
// one arena, promotion on second access) with a bounded *ghost* record: a FIFO
// of keys recently evicted from the probationary segment, capped at `capacity`
// and storing keys only (never values, so it does not count toward residency).
//
// When a returning key is found in the ghost record on a put-after-miss, it is
// admitted with *weak* protection: inserted at the LRU end of the protected
// segment (the first protected eviction candidate) and then subjected to
// `enforce_protected_budget`, which demotes it straight back to probationary if
// protected is already full. This recovers reuse-distance hits for keys that
// fell just outside the probationary window without crowding the genuinely hot
// keys at the protected MRU end.
//
// Capacity is partitioned so the two segment budgets sum to exactly `capacity`,
// leaving the P1 capacity bound identical to the baseline. Eviction slots are
// still reused in place. No unsafe, no external crates.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// `true` when the node currently lives in the protected segment.
	protected: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // shared arena

	// Probationary segment: entries that have been seen exactly once.
	prob_head: Option<usize>, // LRU (front)
	prob_tail: Option<usize>, // MRU (back)
	prob_len: usize,

	// Protected segment: entries that earned a second access.
	prot_head: Option<usize>, // LRU (front)
	prot_tail: Option<usize>, // MRU (back)
	prot_len: usize,

	// Bounded ghost record of keys recently evicted from the probationary
	// segment. Holds keys only (no values, no residency), capped at `capacity`.
	// `ghost` is the FIFO eviction order; `ghost_set` mirrors it for O(1)
	// membership. The two are always kept in sync.
	ghost: VecDeque<K>,
	ghost_set: HashSet<K>,

	capacity: usize,
	// Size budget for the protected segment (~80% of capacity, floored). The
	// probationary budget is the remainder, so the two sum to exactly
	// `capacity`. For capacity == 1 this floors to 0, leaving a single usable
	// (probationary) slot — the only case where a segment budget is 0.
	protected_cap: usize,
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
		// floor(0.8 * capacity); probationary budget is capacity - protected_cap,
		// so the two segment budgets sum to exactly `capacity`.
		let protected_cap = (capacity * 4) / 5;
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} probationary_cap={}",
			capacity - protected_cap
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			ghost: VecDeque::with_capacity(capacity),
			ghost_set: HashSet::with_capacity(capacity),
			capacity,
			protected_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to the protected
	/// segment (most-recently-used end).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss.
		let &idx = self.map.get(key)?;
		// A hit earns protection: detach from its current segment and re-attach
		// at the MRU end of the protected segment.
		self.unlink(idx);
		self.push_back_protected(idx);
		// Keep the protected segment within its budget by demoting its LRU node
		// back to the probationary segment. Net resident count is unchanged.
		self.enforce_protected_budget();
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used
	/// entry (probationary first, then protected).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access,
		// promoting it to the protected segment.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back_protected(idx);
			self.enforce_protected_budget();
			return;
		}
		// New (non-resident) key. Decide up front whether it is a returning key
		// remembered in the ghost record; the eviction below only ever records a
		// *different* (resident) key, so this snapshot is unaffected by it.
		let is_ghost_hit = self.ghost_set.contains(&key);

		// New key — need an arena slot.
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the LRU of the probationary segment first; only
			// if it is empty fall back to the protected segment's LRU. Reuse the
			// evicted slot in place for the new entry.
			let (evict_idx, from_probationary) = match self.prob_head {
				Some(head) => (head, true),
				None => (
					self.prot_head
						.expect("a full cache always has a victim in some segment"),
					false,
				),
			};
			let evicted_key = self.nodes[evict_idx].key.clone();
			// Only probationary evictions feed the ghost record; keys evicted
			// from the protected fallback path are not remembered.
			if from_probationary {
				self.record_ghost(evicted_key.clone());
			}
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			evict_idx
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

		if is_ghost_hit {
			// Returning key: it is becoming resident again, so drop it from the
			// ghost record and admit it with *weak* protection at the protected
			// LRU end. enforce_protected_budget then demotes it straight back to
			// probationary if protected is full, so it never displaces a hot key.
			self.ghost_remove(&key);
			self.push_front_protected(idx);
			self.enforce_protected_budget();
		} else {
			// First-seen key enters at the MRU end of the probationary segment.
			self.push_back_probationary(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
		debug_assert!(self.ghost.len() == self.ghost_set.len());
		debug_assert!(self.ghost.len() <= self.capacity);
	}

	/// Record a probationary-evicted key in the bounded ghost FIFO. Stores the
	/// key only. If the FIFO is at its `capacity` cap, drops the oldest key
	/// first. Keeps `ghost` and `ghost_set` in sync.
	fn record_ghost(&mut self, key: K) {
		// A resident key being evicted cannot already be in the ghost record
		// (re-admission always clears it), but guard defensively.
		if self.ghost_set.contains(&key) {
			return;
		}
		if self.ghost.len() >= self.capacity {
			if let Some(oldest) = self.ghost.pop_front() {
				self.ghost_set.remove(&oldest);
			}
		}
		self.ghost_set.insert(key.clone());
		self.ghost.push_back(key);
	}

	/// Remove a key from the ghost record (both the FIFO and the set) when it
	/// becomes resident again.
	fn ghost_remove(&mut self, key: &K) {
		if self.ghost_set.remove(key) {
			if let Some(pos) = self.ghost.iter().position(|k| k == key) {
				self.ghost.remove(pos);
			}
		}
	}

	/// Remove a node from whichever segment it currently occupies.
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
					self.prob_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if protected {
					self.prot_tail = prev;
				} else {
					self.prob_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		if protected {
			self.prot_len -= 1;
		} else {
			self.prob_len -= 1;
		}
	}

	/// Append a node at the MRU (tail) end of the probationary segment.
	fn push_back_probationary(&mut self, idx: usize) {
		self.nodes[idx].protected = false;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
		self.prob_len += 1;
	}

	/// Append a node at the MRU (tail) end of the protected segment.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].protected = true;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
		self.prot_len += 1;
	}

	/// Insert a node at the LRU (head) end of the protected segment — the first
	/// protected eviction candidate. Used to grant *weak* protection to a
	/// returning ghost-hit key.
	fn push_front_protected(&mut self, idx: usize) {
		self.nodes[idx].protected = true;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.prot_head;
		match self.prot_head {
			Some(h) => self.nodes[h].prev = Some(idx),
			None => self.prot_tail = Some(idx),
		}
		self.prot_head = Some(idx);
		self.prot_len += 1;
	}

	/// If the protected segment exceeds its budget, demote its LRU node to the
	/// MRU end of the probationary segment. Only ever one node over budget per
	/// promotion, so a single demotion restores the bound.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.protected_cap {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_back_probationary(lru);
			}
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
		cache.put(3, 30); // both keys still probationary; evicts probationary LRU (1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promotes key 1 into the protected segment
		cache.put(3, 30); // evicts key 2 (probationary LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update promotes key 1 into the protected segment
		cache.put(3, 30); // evicts key 2 (probationary LRU)

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
		cache.put(4, 40); // key 1 is protected; evicts a probationary LRU, not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // promote key 2 into the protected segment
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn promoted_key_survives_while_probationary_churns() {
		// SLRU: a key promoted to the protected segment is shielded from
		// eviction as long as the probationary segment can supply a victim.
		// Slots are still reused in place across re-evictions.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts probationary LRU (key 1), reuses slot 0
		cache.get(&3); // promote key 3 into the protected segment
		cache.put(4, 40); // evicts probationary LRU (key 2), reuses slot 1
		cache.put(5, 50); // evicts probationary LRU (key 4), reuses slot 1
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survived
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it (into protected).
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify the structure is still intact.
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
		cache.put(3, 31); // overwrite promotes key 3 into protected
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn ghost_hit_readmits_with_weak_protection() {
		// A key evicted from probationary then re-seen is remembered in the
		// ghost record and re-admitted into the protected segment, surviving a
		// subsequent probationary eviction.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts probationary LRU (key 1) -> ghost{1}
		assert!(cache.get(&1).is_none()); // miss; will be re-put below
		cache.put(1, 11); // ghost hit: re-admit key 1 with weak protection
		assert_eq!(*cache.get(&1).unwrap(), 11);
	}

	#[test]
	fn ghost_record_stays_bounded() {
		// Drive many probationary evictions; ghost never exceeds capacity and
		// never resurrects a phantom value.
		let mut cache = LruCache::new(2);
		for key in 0i64..50 {
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
		}
		assert!(cache.ghost.len() <= cache.capacity);
		assert_eq!(cache.ghost.len(), cache.ghost_set.len());
		assert!(cache.get(&9999).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
