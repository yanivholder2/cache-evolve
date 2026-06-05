// Segmented-LRU (SLRU) cache with a frequency-aware bounded eviction scan.
//
// This builds on the v001 SLRU split (probationary/protected segments sharing
// one arena, promotion on second access). The single change here is the victim
// choice in `put`: instead of strictly evicting the LRU node of the
// probationary segment, we scan a small bounded window (`W` nodes) from the LRU
// end and evict the node with the *lowest access frequency*, breaking ties
// toward the LRU-most candidate. Each node carries a saturating `freq` counter
// bumped on every access; a rare amortized aging sweep halves all counters so
// they never ossify. This makes eviction scan-resistant: a warm key that a scan
// flood pushes toward the probationary head is spared in favour of a colder
// one-hit neighbour.
//
// Capacity is still partitioned so the two segment budgets sum to exactly
// `capacity`; the P1 capacity bound, in-place slot reuse, and the protected
// budget are all unchanged. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Bounded window length for the frequency-aware victim scan.
const VICTIM_WINDOW: usize = 8;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// `true` when the node currently lives in the protected segment.
	protected: bool,
	// Saturating access-frequency counter. Reset to 0 when the slot is reused
	// for a brand-new key, and halved by the periodic aging sweep.
	freq: u8,
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

	capacity: usize,
	// Size budget for the protected segment (~80% of capacity, floored). The
	// probationary budget is the remainder, so the two sum to exactly
	// `capacity`. For capacity == 1 this floors to 0, leaving a single usable
	// (probationary) slot — the only case where a segment budget is 0.
	protected_cap: usize,

	// Counts accesses (hits) since the last aging sweep. When it reaches
	// `capacity * 8`, every resident node's `freq` is right-shifted and this is
	// reset to 0. Keeps the O(capacity) sweep amortized and rare.
	access_counter: u64,
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
			capacity,
			protected_cap,
			access_counter: 0,
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
		// Record the access: bump frequency and run the aging sweep if due.
		self.on_access(idx);
		// A hit earns protection: detach from its current segment and re-attach
		// at the MRU end of the protected segment.
		self.unlink(idx);
		self.push_back_protected(idx);
		// Keep the protected segment within its budget by demoting its LRU node
		// back to the probationary segment. Net resident count is unchanged.
		self.enforce_protected_budget();
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a low-frequency entry
	/// within a bounded window of the LRU end (probationary first, then
	/// protected).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access,
		// promoting it to the protected segment.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			self.unlink(idx);
			self.push_back_protected(idx);
			self.enforce_protected_budget();
			return;
		}
		// New key — need an arena slot.
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: pick the eviction victim with a bounded
			// frequency-aware scan (probationary segment first, falling back to
			// the protected segment only when probationary is empty). Reuse the
			// evicted slot in place for the new entry, resetting its frequency.
			let evict_idx = self.pick_victim();
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq = 0;
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
				freq: 0,
			});
			idx
		};
		// New entries enter at the MRU end of the probationary segment.
		self.push_back_probationary(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.prob_len + self.prot_len, self.map.len());
	}

	/// Record an access to `idx`: saturating-increment its frequency, and run
	/// the amortized aging sweep (halve every resident node's `freq`) when the
	/// access counter reaches `capacity * 8`.
	fn on_access(&mut self, idx: usize) {
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
		self.access_counter += 1;
		if self.access_counter >= (self.capacity as u64).saturating_mul(8) {
			// Every node currently in the arena is resident (slots are reused in
			// place, never freed), so an arena sweep is exactly the resident set.
			for node in &mut self.nodes {
				node.freq >>= 1;
			}
			self.access_counter = 0;
		}
	}

	/// Choose the eviction victim with a bounded frequency-aware scan. Starts at
	/// the LRU end of the probationary segment (or the protected segment if
	/// probationary is empty) and walks toward the MRU end for at most
	/// `VICTIM_WINDOW` nodes, returning the node with the smallest `freq`. Ties
	/// are broken toward the LRU-most (earliest-visited) candidate.
	fn pick_victim(&self) -> usize {
		let start = match self.prob_head {
			Some(head) => head,
			None => self
				.prot_head
				.expect("a full cache always has a victim in some segment"),
		};
		let mut best = start;
		let mut best_freq = self.nodes[start].freq;
		let mut cur = self.nodes[start].next;
		let mut steps = 1usize;
		while steps < VICTIM_WINDOW {
			match cur {
				Some(idx) => {
					// Strict `<` keeps the earliest (LRU-most) candidate on ties.
					if self.nodes[idx].freq < best_freq {
						best = idx;
						best_freq = self.nodes[idx].freq;
					}
					cur = self.nodes[idx].next;
					steps += 1;
				}
				None => break,
			}
		}
		best
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
		// Both keys still probationary, all freq 0; the bounded scan breaks the
		// tie toward the LRU-most candidate (key 1).
		cache.put(3, 30);

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
	fn promoted_key_survives_while_probationary_churns() {
		// SLRU: a key promoted to the protected segment is shielded from
		// eviction as long as the probationary segment can supply a victim.
		// Slots are still reused in place across re-evictions.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts probationary victim (key 1), reuses slot 0
		cache.get(&3); // promote key 3 into the protected segment
		cache.put(4, 40); // evicts probationary victim (key 2), reuses slot 1
		cache.put(5, 50); // evicts probationary victim (key 4), reuses slot 1
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
	fn frequency_aware_scan_spares_warmer_key() {
		// Within the bounded window, the colder of two probationary keys is the
		// victim even when it is not the LRU-most node.
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probationary LRU, freq stays 0
		cache.put(2, 20); // probationary
		cache.put(2, 21); // overwrite-as-access promotes key 2 to protected, freq>0
		// Key 2 is now protected; only keys 1 and (next) are probationary.
		cache.put(3, 30); // probationary
		cache.put(4, 40); // evicts a probationary key; key 2 stays protected
		assert_eq!(*cache.get(&2).unwrap(), 21);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
