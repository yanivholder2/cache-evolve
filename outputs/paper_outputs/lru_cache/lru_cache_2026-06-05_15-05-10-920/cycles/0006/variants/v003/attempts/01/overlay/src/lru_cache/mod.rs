// LFU (Least Frequently Used) cache with frequency decay.
// Nodes track frequency and are evicted based on minimum frequency with LRU tiebreaker.
// Frequency decay is applied periodically to prevent stale hot items from dominating.

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
	freq: u8,  // Frequency counter for LFU
	access_time: u64,  // For tiebreaking when frequencies are equal
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // Least Frequently Used (front), LRU among tied frequencies
	tail: Option<usize>,    // Most Recently Used (back)
	capacity: usize,
	access_counter: u64,    // Monotonic timestamp for access ordering
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
			capacity,
			access_counter: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Check if frequency reset threshold has been reached and perform decay if needed.
	/// Threshold: capacity × 16 accesses.
	fn check_and_perform_frequency_reset(&mut self) {
		let threshold = self.capacity as u64 * 16;
		if self.access_counter >= threshold {
			// Apply selective decay: only decay items with frequency >= 4
			for node in &mut self.nodes {
				if node.freq >= 4 {
					node.freq = (node.freq as u16 * 3 / 4) as u8;
				}
			}
			self.access_counter = 0;
			log::debug!("LruCache: frequency decay applied (threshold: {})", threshold);
		}
	}

	/// Returns the value for `key` if present, updating frequency and promoting to MRU.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;

		// Increment frequency and update access time
		if self.nodes[idx].freq < 255 {
			self.nodes[idx].freq += 1;
		}
		self.access_counter = self.access_counter.saturating_add(1);
		self.nodes[idx].access_time = self.access_counter;

		// Check if frequency reset is needed
		self.check_and_perform_frequency_reset();

		// Promote to MRU (tail) by unlinking and re-attaching
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-frequently-used entry.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;

			// Increment frequency and update access time
			if self.nodes[idx].freq < 255 {
				self.nodes[idx].freq += 1;
			}
			self.access_counter = self.access_counter.saturating_add(1);
			self.nodes[idx].access_time = self.access_counter;

			// Check if frequency reset is needed
			self.check_and_perform_frequency_reset();

			// Promote to MRU
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: find and evict LFU (minimum frequency, with LRU tiebreaker)
			let evict_idx = self.find_lfu();
			debug_assert_eq!(evict_idx, self.head.unwrap_or(0), "LFU should be at head");
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq = 1;
			self.access_counter = self.access_counter.saturating_add(1);
			self.nodes[evict_idx].access_time = self.access_counter;
			evict_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.access_counter = self.access_counter.saturating_add(1);
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 1,
				access_time: self.access_counter,
			});
			idx
		};

		// Check if frequency reset is needed
		self.check_and_perform_frequency_reset();

		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Find the index of the LFU (Least Frequently Used) node.
	/// If multiple nodes share the minimum frequency, return the LRU one (earliest access_time).
	fn find_lfu(&self) -> usize {
		let mut lfu_idx = self.head.expect("must have at least one node");
		let mut min_freq = self.nodes[lfu_idx].freq;
		let mut min_time = self.nodes[lfu_idx].access_time;

		for (i, node) in self.nodes.iter().enumerate() {
			if node.freq < min_freq || (node.freq == min_freq && node.access_time < min_time) {
				lfu_idx = i;
				min_freq = node.freq;
				min_time = node.access_time;
			}
		}
		lfu_idx
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
	fn eviction_removes_lfu() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (freq 1, LRU)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn frequency_increases_with_access() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);

		// Access key 1 multiple times to increase its frequency
		cache.get(&1);
		cache.get(&1);
		cache.get(&1);

		// Now evict: key 2 has freq 1, key 1 has freq 4, key 3 has freq 1
		// Should evict key 2 (freq 1, most recent LRU among freq-1 items)
		cache.put(4, 40);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none()); // should be evicted
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_mru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 most recent
		cache.put(3, 30); // evicts key 2 (LFU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_mru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU
		cache.put(3, 30); // evicts key 2 (LFU)

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
		cache.put(4, 40); // evicts 1 (all have freq 1, 1 is LRU)
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
		// Key 1 has high frequency, keys 2 and 3 have freq 1
		cache.put(4, 40); // should evict key 2 (freq 1, LRU among freq-1)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // increases freq of 2

		// Key 1 and 3 have freq 1, key 2 has freq 2
		cache.put(4, 40); // evicts key 1 or 3 (both freq 1, will pick LRU)

		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, freq 1
		cache.put(2, 20); // slot 1, freq 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3, freq 1
		cache.get(&3); // promote slot 0 (key 3) freq 2
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4, freq 1
		cache.put(5, 50); // evicts 4 (slot 1) -> slot 1 reused AGAIN for key 5, freq 1
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
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
		cache.put(3, 31); // overwrite MRU; increases freq of 3
		cache.put(4, 40); // evicts 1 (LFU, freq 1)
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
