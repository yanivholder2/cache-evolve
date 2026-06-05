// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// H03: per-node saturating frequency counter + small-window victim selection.
// At eviction time the first WINDOW nodes from the LRU end are scanned and the
// one with the lowest frequency counter is chosen as the victim instead of
// always evicting the strict LRU head.  This prevents frequently-accessed
// entries from being displaced during cold-start bursts.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// Saturating access-frequency counter; capped at 3. Reset to 0 on
	/// construction and whenever the slot is reused for a new key.
	freq: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
}

/// Number of LRU-end candidates examined during victim selection.
const WINDOW: usize = 8;

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
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		// Increment saturating frequency counter (cap 3)
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-frequently-used
	/// entry within a window of WINDOW candidates from the LRU end.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			// Increment saturating frequency counter (cap 3)
			self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1).min(3);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: scan up to WINDOW nodes from the LRU end and evict
			// the one with the lowest frequency counter.
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");
			let mut victim = self.head.expect("non-empty list has a head");
			let mut cursor = victim;
			for _ in 1..WINDOW {
				match self.nodes[cursor].next {
					Some(next_idx) => {
						if self.nodes[next_idx].freq < self.nodes[victim].freq {
							victim = next_idx;
						}
						cursor = next_idx;
					}
					None => break,
				}
			}
			// victim is the lowest-frequency node among the first WINDOW LRU-end entries
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].freq = 0;
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 0,
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the tail (MRU end) of the list.
	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = None;
		match self.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
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
	fn eviction_removes_lru_when_all_freqs_equal() {
		// With equal frequencies the head (LRU) is still selected as victim
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (both have freq 0, head wins)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 most recent; also bumps freq to 1
		cache.put(3, 30); // key 2 is at LRU end with freq 0; key 1 has freq 1 but is MRU
						  // window scan: only key 2 is at the LRU end (key 1 is tail), evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU, bumps freq to 1
		cache.put(3, 30); // key 2 is LRU with freq 0; evicts key 2

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
		cache.put(4, 40); // all freq 0; head (key 1) is evicted
		cache.put(5, 50); // all freq 0; head (key 2) is evicted
		cache.put(6, 60); // all freq 0; head (key 3) is evicted

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
	fn repeated_get_preserves_position_as_mru() {
		// Key 1 gets freq 3 (saturated) and is MRU; key 2 is LRU with freq 0.
		// Inserting key 4 must evict key 2 (lowest freq at LRU end), not key 1.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1); // saturates freq at 3; keeps promoting 1 to MRU
		}
		// Order at this point: 2 (LRU, freq 0), 3 (freq 0), 1 (MRU, freq 3)
		cache.put(4, 40); // window scans 2 and 3 (and 1 if WINDOW large enough);
						  // key 2 has freq 0 and is at head — evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU; bumps freq to 1
						 // Order: 1 (LRU, freq 0), 3 (freq 0), 2 (MRU, freq 1)
		cache.put(4, 40); // window: 1 has freq 0, 3 has freq 0 — head (1) wins ties; evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, freq 0
		cache.put(2, 20); // slot 1, freq 0
		cache.put(3, 30); // evicts 1 (head, freq 0) -> slot 0 reused for key 3, freq 0
		cache.get(&3);    // promote slot 0 (key 3) to MRU; freq -> 1
		cache.put(4, 40); // key 2 is head (freq 0), key 3 is tail (freq 1); evicts key 2 (slot 1)
		cache.put(5, 50); // key 3 is head (freq 1), key 4 is tail (freq 0);
						  // window: key 3 freq=1, key 4 freq=0 — evicts key 4 (lowest freq)
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it
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
		cache.put(3, 30); // full; 3 is MRU (tail), freq 0
		cache.put(3, 31); // overwrite MRU; freq bumped to 1
						  // Eviction should take 1 (LRU, freq 0) — lowest freq in window
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

	#[test]
	fn freq_protected_entry_survives_cold_burst() {
		// Hot key 1 is accessed repeatedly; cold keys 2..=9 flood the cache.
		// With capacity 4 and WINDOW 8, key 1 (freq=3) should survive all evictions
		// as long as there are lower-freq candidates in the window.
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		for _ in 0..5 {
			cache.get(&1); // freq saturates at 3
		}
		// Fill remaining 3 slots
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		// Now evict with cold keys; key 1 has freq=3, others have freq=0
		cache.put(5, 50); // evicts lowest-freq in window; key 1 protected
		cache.put(6, 60);
		cache.put(7, 70);
		// Key 1 should still be resident
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}
}
