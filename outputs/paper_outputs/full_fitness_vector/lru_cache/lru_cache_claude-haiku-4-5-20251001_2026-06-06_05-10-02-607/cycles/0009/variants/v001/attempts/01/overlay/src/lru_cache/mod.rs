// Adaptive LRU cache with sampled saturation: uses simple bounded-window eviction when underutilized,
// switches to frequency buckets only when cache saturation is high (≥75%) or hit rate drops below threshold.
// Saturation is computed asynchronously at configurable intervals to reduce per-operation overhead.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	freq: u32,  // Access frequency counter (used when bucketing is enabled)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	capacity: usize,

	// Frequency bucket fields (used only when bucketing_enabled = true)
	freq_buckets: Vec<Option<usize>>, // freq_buckets[freq] = head of frequency bucket
	min_freq: usize,        // Minimum non-empty frequency level
	eviction_count: u32,    // Number of evictions since last decay
	decay_period: u32,      // Decay frequency every N evictions

	// Adaptive bucketing control
	saturation_threshold: f64,    // Default 0.75 (75%)
	hit_rate_threshold: f64,      // Default 0.65 (65%)
	recent_hits: u64,             // Hits in recent window
	recent_accesses: u64,         // Total accesses in recent window
	bucketing_enabled: bool,      // Whether to use frequency buckets

	// Saturation sampling: compute saturation asynchronously
	saturation_sample_count: u32,    // Increments on each operation
	saturation_sample_threshold: u32, // Compute saturation every N operations (default 64)
	saturation_state: bool,          // Cached saturation state: true = high, false = low

	// For simple bounded-window eviction (when bucketing disabled)
	head: Option<usize>,    // LRU (front) for simple mode
	tail: Option<usize>,    // MRU (back) for simple mode
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
			capacity,
			freq_buckets: vec![None],
			min_freq: 0,
			eviction_count: 0,
			decay_period: capacity as u32,
			saturation_threshold: 0.75,
			hit_rate_threshold: 0.65,
			recent_hits: 0,
			recent_accesses: 0,
			bucketing_enabled: false,
			saturation_sample_count: 0,
			saturation_sample_threshold: 64,
			saturation_state: false, // Start assuming low saturation
			head: None,
			tail: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Compute actual saturation level (division operation)
	fn compute_saturation(&self) -> bool {
		let saturation = self.nodes.len() as f64 / self.capacity as f64;
		saturation >= self.saturation_threshold
	}

	/// Check if we should enable/disable bucketing based on cached saturation and hit rate
	fn should_use_bucketing(&self) -> bool {
		let hit_rate = if self.recent_accesses > 0 {
			self.recent_hits as f64 / self.recent_accesses as f64
		} else {
			1.0 // Assume good hit rate if no accesses yet
		};

		self.saturation_state || hit_rate < self.hit_rate_threshold
	}

	/// Update saturation sampling counter; recompute saturation if threshold reached
	fn update_saturation_sample(&mut self) {
		self.saturation_sample_count += 1;
		if self.saturation_sample_count >= self.saturation_sample_threshold {
			self.saturation_state = self.compute_saturation();
			self.saturation_sample_count = 0;
		}
	}

	/// Update hit rate tracking
	fn update_hit_rate(&mut self, is_hit: bool) {
		self.recent_accesses += 1;
		if is_hit {
			self.recent_hits += 1;
		}

		// Reset counters periodically (every capacity accesses)
		if self.recent_accesses > self.capacity as u64 * 2 {
			self.recent_hits = 0;
			self.recent_accesses = 0;
		}
	}

	/// Transition from simple mode to bucketing mode: rebuild frequency buckets
	fn enable_bucketing(&mut self) {
		if self.bucketing_enabled {
			return;
		}
		log::debug!("LruCache: enabling bucketing mode (saturation={:.2}%)",
			self.nodes.len() as f64 / self.capacity as f64 * 100.0);

		self.bucketing_enabled = true;

		// Clear old bucket structure
		self.freq_buckets.clear();
		self.freq_buckets.push(None);
		self.min_freq = 0;

		// Reset frequency for all nodes and rebuild buckets
		// Start with freq=1 for all nodes, organizing them into bucket 1
		for idx in 0..self.nodes.len() {
			self.nodes[idx].freq = 1;
		}

		// Build frequency buckets from current nodes
		for idx in 0..self.nodes.len() {
			self.link_to_bucket(idx, 1);
		}

		// Clear simple mode pointers
		self.head = None;
		self.tail = None;
	}

	/// Transition from bucketing mode to simple mode
	fn disable_bucketing(&mut self) {
		if !self.bucketing_enabled {
			return;
		}
		log::debug!("LruCache: disabling bucketing mode");

		self.bucketing_enabled = false;

		// Rebuild simple LRU list from current nodes
		// Keep the relative order but clear frequencies
		self.freq_buckets.clear();
		self.freq_buckets.push(None);

		// Reset head/tail pointers
		self.head = None;
		self.tail = None;

		// Rebuild the simple doubly-linked list from nodes
		// We'll reconstruct a simple LRU order based on the existing pointers
		// For simplicity, clear all pointers and rebuild from scratch
		for idx in 0..self.nodes.len() {
			self.nodes[idx].prev = None;
			self.nodes[idx].next = None;
			self.nodes[idx].freq = 1; // Reset frequency
		}

		// Rebuild simple LRU list: put all nodes at tail in order
		for idx in 0..self.nodes.len() {
			self.push_back_simple(idx);
		}
	}

	/// Returns the value for `key` if present, promoting it to higher frequency or MRU.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Update saturation sample counter on every operation
		self.update_saturation_sample();

		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;

		self.update_hit_rate(true);

		if self.bucketing_enabled {
			self.update_node_frequency(idx);
		} else {
			self.unlink_simple(idx);
			self.push_back_simple(idx);
		}

		// Check if we should switch modes
		let should_bucket = self.should_use_bucketing();
		if should_bucket && !self.bucketing_enabled {
			self.enable_bucketing();
		} else if !should_bucket && self.bucketing_enabled {
			self.disable_bucketing();
		}

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts using adaptive strategy.
	pub fn put(&mut self, key: K, value: V) {
		// Update saturation sample counter on every operation
		self.update_saturation_sample();

		self.update_hit_rate(false); // put typically requires an eviction, so it's a miss

		// Key already exists: update value in-place and promote
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			if self.bucketing_enabled {
				self.update_node_frequency(idx);
			} else {
				self.unlink_simple(idx);
				self.push_back_simple(idx);
			}
			return;
		}

		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: find and evict a candidate
			let evict_idx = if self.bucketing_enabled {
				self.find_evictable_node_bucketed()
			} else {
				self.find_evictable_node_simple()
			};

			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);

			if self.bucketing_enabled {
				// Unlink from frequency bucket
				let old_freq = self.nodes[evict_idx].freq as usize;
				self.unlink_from_bucket(evict_idx, old_freq);

				// Update min_freq if we just emptied the min_freq bucket
				if old_freq == self.min_freq && self.freq_buckets[old_freq].is_none() {
					for freq in old_freq + 1..self.freq_buckets.len() {
						if self.freq_buckets[freq].is_some() {
							self.min_freq = freq;
							break;
						}
					}
				}
			} else {
				self.unlink_simple(evict_idx);
			}

			// Reuse the slot for the new entry
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq = 1;  // New entries start with freq=1

			if self.bucketing_enabled {
				// Link into frequency bucket 1
				self.link_to_bucket(evict_idx, 1);

				// Increment eviction counter and possibly decay frequencies
				self.eviction_count += 1;
				if self.eviction_count >= self.decay_period {
					self.decay_frequencies();
					self.eviction_count = 0;
				}
			} else {
				self.push_back_simple(evict_idx);
			}

			evict_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				freq: 1,  // New entries start with freq=1
			});

			if self.bucketing_enabled {
				self.link_to_bucket(idx, 1);
			} else {
				self.push_back_simple(idx);
			}
			idx
		};

		// Register in the map
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);

		// Check if we should switch modes
		let should_bucket = self.should_use_bucketing();
		if should_bucket && !self.bucketing_enabled {
			self.enable_bucketing();
		} else if !should_bucket && self.bucketing_enabled {
			self.disable_bucketing();
		}
	}

	// ===== Bucketed mode operations =====

	/// Find a node to evict using frequency buckets (O(1)).
	fn find_evictable_node_bucketed(&self) -> usize {
		for freq in self.min_freq..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[freq] {
				return idx;
			}
		}
		// Fallback
		for freq in 0..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[freq] {
				return idx;
			}
		}
		panic!("No evictable node found");
	}

	/// Find a node to evict using bounded-window scan (simple mode).
	fn find_evictable_node_simple(&self) -> usize {
		// Use bounded-window eviction: scan at most min(capacity/4, 256) nodes
		let scan_limit = std::cmp::min(self.capacity / 4, 256);
		let mut idx = self.head;
		let mut count = 0;

		while let Some(current) = idx {
			if count >= scan_limit {
				// Reached scan limit, evict this node
				return current;
			}
			idx = self.nodes[current].next;
			count += 1;
		}

		// If we scanned all nodes, evict the head (LRU)
		self.head.expect("should have at least one node when at capacity")
	}

	/// Increment frequency and move node to the next frequency bucket.
	fn update_node_frequency(&mut self, idx: usize) {
		let old_freq = self.nodes[idx].freq as usize;
		let new_freq = old_freq + 1;

		// Unlink from old frequency bucket
		self.unlink_from_bucket(idx, old_freq);

		// Increment frequency
		self.nodes[idx].freq += 1;

		// Link into new frequency bucket
		self.link_to_bucket(idx, new_freq);

		// Update min_freq if old bucket is now empty and was the minimum
		if old_freq == self.min_freq && self.freq_buckets[old_freq].is_none() {
			for freq in old_freq + 1..self.freq_buckets.len() {
				if self.freq_buckets[freq].is_some() {
					self.min_freq = freq;
					return;
				}
			}
		}
	}

	/// Remove node from its frequency bucket.
	fn unlink_from_bucket(&mut self, idx: usize, freq: usize) {
		if freq >= self.freq_buckets.len() {
			return;
		}

		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;

		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				self.freq_buckets[freq] = next;
			}
		}

		if let Some(n) = next {
			self.nodes[n].prev = prev;
		}

		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Link node into the specified frequency bucket (at the head).
	fn link_to_bucket(&mut self, idx: usize, freq: usize) {
		while self.freq_buckets.len() <= freq {
			self.freq_buckets.push(None);
		}

		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.freq_buckets[freq];

		if let Some(next) = self.freq_buckets[freq] {
			self.nodes[next].prev = Some(idx);
		}

		self.freq_buckets[freq] = Some(idx);

		if freq < self.min_freq {
			self.min_freq = freq;
		}
	}

	/// Decay all frequencies by dividing by 2.
	fn decay_frequencies(&mut self) {
		let mut nodes_to_reinsert = Vec::new();

		for freq in 0..self.freq_buckets.len() {
			let mut idx = self.freq_buckets[freq].take();
			while let Some(current) = idx {
				let next = self.nodes[current].next;
				self.nodes[current].freq /= 2;
				let new_freq = self.nodes[current].freq as usize;
				self.nodes[current].prev = None;
				self.nodes[current].next = None;
				nodes_to_reinsert.push((current, new_freq));
				idx = next;
			}
		}

		self.freq_buckets.clear();
		self.freq_buckets.push(None);
		self.min_freq = usize::MAX;

		for (idx, freq) in nodes_to_reinsert {
			self.link_to_bucket(idx, freq);
		}

		if self.min_freq == usize::MAX {
			self.min_freq = 0;
		}
	}

	// ===== Simple mode operations =====

	/// Remove node from simple LRU list.
	fn unlink_simple(&mut self, idx: usize) {
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

	/// Append node to the tail (MRU end) of simple LRU list.
	fn push_back_simple(&mut self, idx: usize) {
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
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
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
		cache.get(&1);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.put(3, 30);

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
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
		cache.put(4, 40);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2);
		cache.put(4, 40);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&3);
		cache.put(4, 40);
		cache.put(5, 50);
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
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
		cache.put(3, 31);
		cache.put(4, 40);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
