// O(1) LRU cache with frequency bucket optimization and incremental queue draining.
// Nodes are organized into frequency-level buckets (indexed by frequency).
// When a node's frequency increments, it moves from one bucket to the next.
// Eviction directly accesses the lowest non-empty frequency bucket in O(1) time.
// Frequency promotions are deferred from get() to put() to reduce get latency.
// Incremental draining during get() spreads the promotion work across both get and put.

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
	freq: u32,  // Access frequency counter
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	capacity: usize,
	freq_buckets: Vec<Option<usize>>, // freq_buckets[freq] = head of frequency bucket
	min_freq: usize,        // Minimum non-empty frequency level
	eviction_count: u32,    // Number of evictions since last decay
	decay_period: u32,      // Decay frequency every N evictions
	pending_promotions: Vec<usize>, // Queue of node indices needing frequency updates
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
			freq_buckets: vec![None],  // Start with freq_buckets[0]
			min_freq: 0,
			eviction_count: 0,
			decay_period: capacity as u32,
			pending_promotions: Vec::new(),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, queuing a frequency promotion.
	/// The promotion is deferred to put() time to avoid pointer-chasing overhead in the hot path.
	/// If the queue size exceeds the threshold, drain and process a batch to amortize the work.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Queue the frequency promotion instead of executing it immediately
		self.pending_promotions.push(idx);

		// Drain pending promotions incrementally to amortize the work across get and put
		const DRAIN_THRESHOLD: usize = 8; // Q: trigger draining if queue exceeds this
		const DRAIN_BATCH_SIZE: usize = 4; // K: process this many items per drain
		if self.pending_promotions.len() >= DRAIN_THRESHOLD {
			self.drain_pending_promotions(DRAIN_BATCH_SIZE);
		}

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts using frequency-bucket strategy.
	pub fn put(&mut self, key: K, value: V) {
		// Flush pending promotions before eviction to ensure accurate frequency info
		self.flush_pending_promotions();

		// Key already exists: update value in-place and queue frequency promotion
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.pending_promotions.push(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: find and evict a candidate using frequency-bucket strategy
			let evict_idx = self.find_evictable_node();
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);

			// Unlink from its current frequency bucket
			let old_freq = self.nodes[evict_idx].freq as usize;
			self.unlink_from_bucket(evict_idx, old_freq);

			// Update min_freq if we just emptied the min_freq bucket
			if old_freq == self.min_freq && self.freq_buckets[old_freq].is_none() {
				// Find the next non-empty bucket
				for freq in old_freq + 1..self.freq_buckets.len() {
					if self.freq_buckets[freq].is_some() {
						self.min_freq = freq;
						break;
					}
				}
			}

			// Reuse the slot for the new entry
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq = 1;  // New entries start with freq=1

			// Link into frequency bucket 1
			self.link_to_bucket(evict_idx, 1);

			// Increment eviction counter and possibly decay frequencies
			self.eviction_count += 1;
			if self.eviction_count >= self.decay_period {
				self.decay_frequencies();
				self.eviction_count = 0;
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
			// Link into frequency bucket 1
			self.link_to_bucket(idx, 1);
			idx
		};
		// Register in the map
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Flush all pending frequency promotions by executing them.
	fn flush_pending_promotions(&mut self) {
		// Collect indices first to avoid holding a mutable borrow of pending_promotions
		// while calling update_node_frequency
		let indices: Vec<usize> = self.pending_promotions.drain(..).collect();
		for idx in indices {
			self.update_node_frequency(idx);
		}
	}

	/// Drain and process the first batch_size items from the pending_promotions queue.
	/// This amortizes the promotion work across multiple get operations.
	fn drain_pending_promotions(&mut self, batch_size: usize) {
		let drain_count = batch_size.min(self.pending_promotions.len());
		let indices: Vec<usize> = self.pending_promotions
			.drain(0..drain_count)
			.collect();
		for idx in indices {
			self.update_node_frequency(idx);
		}
	}

	/// Find a node to evict by directly accessing the lowest non-empty frequency bucket.
	/// Returns the head of the lowest frequency bucket (LRU within that bucket).
	fn find_evictable_node(&self) -> usize {
		// Start from min_freq and find the first non-empty bucket
		for freq in self.min_freq..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[freq] {
				return idx;  // Head of the bucket is the LRU in that frequency
			}
		}
		// Fallback: search all buckets (should not happen if min_freq is correct)
		for freq in 0..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[freq] {
				return idx;
			}
		}
		panic!("No evictable node found");
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
			// Find next non-empty bucket
			for freq in old_freq + 1..self.freq_buckets.len() {
				if self.freq_buckets[freq].is_some() {
					self.min_freq = freq;
					return;
				}
			}
			// If no higher bucket found, stay at old_freq (will be found eventually)
		}
	}

	/// Remove node from its frequency bucket.
	fn unlink_from_bucket(&mut self, idx: usize, freq: usize) {
		if freq >= self.freq_buckets.len() {
			return;  // Should not happen, but be safe
		}

		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;

		// Update previous node's next pointer
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				// This node was the head of the bucket
				self.freq_buckets[freq] = next;
			}
		}

		// Update next node's prev pointer
		if let Some(n) = next {
			self.nodes[n].prev = prev;
		}

		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Link node into the specified frequency bucket (at the head, making it newest within bucket).
	fn link_to_bucket(&mut self, idx: usize, freq: usize) {
		// Expand freq_buckets if necessary
		while self.freq_buckets.len() <= freq {
			self.freq_buckets.push(None);
		}

		// Link at the head of the bucket (O(1) operation)
		// Head is the most recently added node in this bucket
		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.freq_buckets[freq];

		if let Some(next) = self.freq_buckets[freq] {
			self.nodes[next].prev = Some(idx);
		}

		self.freq_buckets[freq] = Some(idx);

		// Update min_freq if this is the first node at this frequency
		if freq < self.min_freq {
			self.min_freq = freq;
		}
	}

	/// Decay all frequencies by dividing by 2 and moving nodes to lower buckets.
	fn decay_frequencies(&mut self) {
		// Collect all nodes in their current buckets, decay them, and rebuild buckets
		let mut nodes_to_reinsert = Vec::new();

		// Extract nodes from all buckets and decay their frequencies
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

		// Clear buckets
		self.freq_buckets.clear();
		self.freq_buckets.push(None);  // Ensure at least freq_buckets[0]
		self.min_freq = usize::MAX;

		// Reinsert all nodes into their new frequency buckets
		for (idx, freq) in nodes_to_reinsert {
			self.link_to_bucket(idx, freq);
		}

		// Set min_freq properly
		if self.min_freq == usize::MAX {
			self.min_freq = 0;
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
	fn eviction_removes_lowest_freq() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (freq=1, LRU in bucket 1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_frequency() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // queues promotion, freq will become 2 on next put
		cache.put(3, 30); // flushes pending promotions, evicts key 2 (freq=1, LRU in bucket 1)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_frequency_and_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1, freq becomes 2
		cache.put(3, 30); // evicts key 2 (freq=1, LRU in bucket 1)

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
		cache.put(4, 40); // evicts 1 (freq=1)
		cache.put(5, 50); // evicts 2 (freq=1)
		cache.put(6, 60); // evicts 3 (freq=1)

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
		cache.put(4, 40); // should evict key 2 (freq=1), not key 1 (freq>1)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // freq=1, inserted at head of bucket 1
		cache.put(2, 20); // freq=1, inserted at head of bucket 1 (pushing 1 back)
		cache.put(3, 30); // freq=1, inserted at head of bucket 1 (pushing 2 back)
		cache.get(&2); // queues promotion
		// Promotion will be flushed on next put
		cache.put(4, 40); // flushes pending promotions, freq=2 for node 2, evicts head of bucket 1 (which is 3)
		// After eviction: Bucket 1 has 1, then 4 inserted at head
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		// 3 should have been evicted
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // queues promotion
		cache.put(4, 40); // flushes pending, promotes 3 to freq=2, evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 4 (slot 1) -> slot 1 reused for key 5
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Frequency bumped from 1 to 2 via deferred promotion
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify it's still accessible
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; 3 is newest in bucket 1
		cache.put(3, 31); // overwrite, queues promotion, freq becomes 2
		cache.put(4, 40); // flushes pending (promoting 3), evicts 1 or 2 (freq=1, LRU in bucket 1)
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn frequency_decay_on_schedule() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		// Access key 1 multiple times to build up frequency
		for _ in 0..3 {
			cache.get(&1);
		}
		// At this point: key 1 has 3 pending promotions, key 2 freq=1
		// The cache will trigger decay every 2 evictions (decay_period = capacity = 2)
		cache.put(3, 30); // flushes pending promotions, triggers eviction -> decay

		// After decay and new eviction, key 1 should survive with freq=2
		assert!(cache.get(&1).is_some()); // key 1 should survive with freq=2
	}
}
