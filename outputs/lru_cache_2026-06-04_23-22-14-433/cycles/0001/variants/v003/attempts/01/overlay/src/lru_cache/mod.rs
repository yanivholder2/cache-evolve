// O(1) LRU cache using HashMap + arena-based doubly-linked list.
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.
//
// Variant: frequency-biased eviction. Each node carries a saturating u8 frequency counter
// incremented on every `get`. When the arena is full, the eviction victim is chosen as the
// minimum-frequency node among the first EVICT_SCAN candidates from the LRU head rather
// than always taking the strict LRU head. This prevents one-time-scan items from displacing
// frequently-accessed entries that happen to sit near the LRU end of the list.

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
	/// Saturating access-frequency counter; initialised to 1 on insertion.
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

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	/// Number of LRU-end candidates inspected during eviction to find the minimum-frequency victim.
	const EVICT_SCAN: usize = 8;

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
		// Increment access frequency (saturating so it never wraps)
		self.nodes[idx].freq = self.nodes[idx].freq.saturating_add(1);
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-frequently-used entry
	/// among the first EVICT_SCAN candidates from the LRU head.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: scan up to EVICT_SCAN nodes from the LRU head and evict the one
			// with the lowest frequency. Ties are broken in favour of the head (oldest).
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");
			let mut victim = self.head.expect("non-empty list has a head");
			let mut cursor_opt = self.nodes[victim].next;
			let mut steps = 1usize;
			while steps < Self::EVICT_SCAN {
				let cursor = match cursor_opt {
					Some(c) => c,
					None => break,
				};
				if self.nodes[cursor].freq < self.nodes[victim].freq {
					victim = cursor;
				}
				cursor_opt = self.nodes[cursor].next;
				steps += 1;
			}
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].freq = 1;
			victim
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 1,
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
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (head, same freq as key 2)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 more frequent (freq=2) and MRU
		cache.put(3, 30); // key 2 has lower freq (1) so it is evicted

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU (freq unchanged)
		cache.put(3, 30); // evicts key 2 (now LRU, same freq)

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
		cache.put(4, 40); // evicts 1 (all freq=1, head wins tie)
		cache.put(5, 50); // evicts 2 (all freq=1, head wins tie)
		cache.put(6, 60); // evicts 3 (all freq=1, head wins tie)

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
			cache.get(&1); // key 1 accumulates high freq; stays in cache
		}
		cache.put(4, 40); // should evict key 2 (lowest freq=1, head of remaining pair)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU; freq(2)=2
		             // Order: 1 (LRU, freq=1), 3 (freq=1), 2 (MRU, freq=2)
		cache.put(4, 40); // scan: head=1(freq=1), next=3(freq=1) — tie, head wins; evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn freq_biased_eviction_saves_hot_item() {
		// Verify that the frequency scan protects a frequently-accessed item
		// sitting near the LRU end from being evicted in favour of a cold item.
		//
		// Setup (capacity = 3):
		//   put 1, put 2, put 3  -> list: 1(freq=1) -> 2(freq=1) -> 3(freq=1)
		//   get 1 x 5            -> 1 promoted to MRU, freq=6; list: 2->3->1
		//   put 4                -> 2 and 3 both have freq=1, scan picks head=2
		//                           list: 3(freq=1) -> 1(freq=6) -> 4(freq=1)
		//   get 3 x 5            -> 3 promoted to MRU, freq=6; list: 1->4->3
		//   put 5                -> scan: 1(freq=6), 4(freq=1) — 4 < 6, victim=4
		//                           key 1 survives; key 4 evicted
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..5 {
			cache.get(&1);
		}
		cache.put(4, 40); // evicts 2 (head, freq=1)
		for _ in 0..5 {
			cache.get(&3);
		}
		cache.put(5, 50); // scan: 1(freq=6), 4(freq=1) — evicts 4
		assert_eq!(*cache.get(&1).unwrap(), 10); // hot item survives
		assert!(cache.get(&4).is_none());         // cold item evicted
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	/// With the frequency-biased eviction policy, a slot can be reused in a different
	/// order than strict LRU when one resident has higher frequency than another.
	///
	/// Sequence (capacity = 2):
	///   put 1 (slot 0, freq=1), put 2 (slot 1, freq=1) — full
	///   put 3 -> scan: 1(freq=1), 2(freq=1) tie, head=1 evicted; slot 0 -> key 3 (freq=1)
	///             list: 2(freq=1) -> 3(freq=1)
	///   get 3 -> freq(3)=2; list: 2(freq=1) -> 3(freq=2)
	///   put 4 -> scan: 2(freq=1), 3(freq=2); 1 < 2 so victim=2; slot 1 -> key 4 (freq=1)
	///             list: 3(freq=2) -> 4(freq=1)
	///   put 5 -> scan: 3(freq=2), 4(freq=1); 1 < 2 so victim=4; slot 1 -> key 5 (freq=1)
	///             list: 3(freq=2) -> 5(freq=1)
	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 (tie at freq=1, head wins) -> slot 0 reused for key 3
		cache.get(&3);    // promote slot 0 (key 3) to MRU; freq(3)=2
		cache.put(4, 40); // evicts 2 (freq=1 < freq(3)=2) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (freq=1 < freq(3)=2) -> slot 1 reused AGAIN for key 5
		assert_eq!(*cache.get(&3).unwrap(), 30); // key 3 survived (high freq)
		assert!(cache.get(&4).is_none());         // key 4 evicted (low freq)
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
		cache.put(3, 30); // full; 3 is MRU (tail)
		cache.put(3, 31); // overwrite MRU
		                  // All entries still have freq=1; eviction takes 1 (LRU head)
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
