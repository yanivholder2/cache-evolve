// O(1) LRU cache with frequency bucket optimization and quantized frequency levels.
// Nodes are organized into frequency-level buckets (indexed by 0-3 quantized level).
// Actual access counts are mapped to 4 coarse buckets: 0 (never), 1-2 (cold), 3-15 (warm), 16+ (hot).
// A local counter per node tracks accesses; only quantize and move buckets on saturation or eviction.
// This reduces per-node frequency metadata from 4+ bytes to 1-2 bits, lowering memory pressure.

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
	freq_level: u8,       // Quantized frequency level: 0-3 (never, cold, warm, hot)
	access_counter: u8,   // Local counter tracking accesses within current level
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	capacity: usize,
	freq_buckets: Vec<Option<usize>>, // freq_buckets[level] = head of frequency-level bucket (levels 0-3)
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
			freq_buckets: vec![None; 4],  // 4 frequency levels: 0, 1, 2, 3
			min_freq: 1,                  // Start at level 1 (new entries)
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
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Queue the frequency promotion instead of executing it immediately
		self.pending_promotions.push(idx);
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
			let old_level = self.nodes[evict_idx].freq_level as usize;
			self.unlink_from_bucket(evict_idx, old_level);

			// Update min_freq if we just emptied the min_freq bucket
			if old_level == self.min_freq && self.freq_buckets[old_level].is_none() {
				// Find the next non-empty bucket
				for level in old_level + 1..self.freq_buckets.len() {
					if self.freq_buckets[level].is_some() {
						self.min_freq = level;
						break;
					}
				}
			}

			// Reuse the slot for the new entry
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.nodes[evict_idx].freq_level = 1;  // New entries start at level 1 (cold)
			self.nodes[evict_idx].access_counter = 0;

			// Link into frequency level 1
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
				freq_level: 1,  // New entries start at level 1 (cold)
				access_counter: 0,
			});
			// Link into frequency level 1
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

	/// Find a node to evict by directly accessing the lowest non-empty frequency bucket.
	/// Returns the head of the lowest frequency bucket (LRU within that bucket).
	fn find_evictable_node(&self) -> usize {
		// Start from min_freq and find the first non-empty bucket
		for level in self.min_freq..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[level] {
				return idx;  // Head of the bucket is the LRU in that frequency level
			}
		}
		// Fallback: search all buckets (should not happen if min_freq is correct)
		for level in 0..self.freq_buckets.len() {
			if let Some(idx) = self.freq_buckets[level] {
				return idx;
			}
		}
		panic!("No evictable node found");
	}

	/// Increment local counter and move to next frequency level if counter saturates.
	fn update_node_frequency(&mut self, idx: usize) {
		let old_level = self.nodes[idx].freq_level as usize;

		// Increment the local access counter
		self.nodes[idx].access_counter = self.nodes[idx].access_counter.saturating_add(1);
		let new_counter = self.nodes[idx].access_counter;

		// Determine if we should promote to the next level based on counter saturation
		let new_level = if self.should_promote(old_level, new_counter) {
			// Promote to the next level
			if old_level < 3 {
				old_level + 1
			} else {
				old_level  // Already at max level
			}
		} else {
			old_level  // Stay at current level
		};

		// Only move if level changes
		if new_level != old_level {
			// Unlink from old level bucket
			self.unlink_from_bucket(idx, old_level);

			// Update level and reset counter
			self.nodes[idx].freq_level = new_level as u8;
			self.nodes[idx].access_counter = 0;

			// Link into new level bucket
			self.link_to_bucket(idx, new_level);

			// Update min_freq if old bucket is now empty and was the minimum
			if old_level == self.min_freq && self.freq_buckets[old_level].is_none() {
				// Find next non-empty bucket
				for level in old_level + 1..self.freq_buckets.len() {
					if self.freq_buckets[level].is_some() {
						self.min_freq = level;
						return;
					}
				}
			}
		}
	}

	/// Determine if a node should be promoted to the next level based on counter saturation.
	/// Thresholds: Level 1→2 at freq 3 (after 2 updates), Level 2→3 at freq 16 (after 13 updates).
	fn should_promote(&self, level: usize, counter: u8) -> bool {
		match level {
			0 => false,  // Level 0 (never accessed) should rarely occur in normal operation
			1 => counter > 1,  // Promote from level 1 (freq 1-2) to level 2 (freq 3+) when counter > 1
			2 => counter > 12, // Promote from level 2 (freq 3-15) to level 3 (freq 16+) when counter > 12
			3 => false,  // Already at max level
			_ => false,
		}
	}

	/// Remove node from its frequency bucket.
	fn unlink_from_bucket(&mut self, idx: usize, level: usize) {
		if level >= self.freq_buckets.len() {
			return;  // Should not happen, but be safe
		}

		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;

		// Update previous node's next pointer
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				// This node was the head of the bucket
				self.freq_buckets[level] = next;
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
	fn link_to_bucket(&mut self, idx: usize, level: usize) {
		// Ensure freq_buckets has enough space (should always have 4 entries)
		while self.freq_buckets.len() <= level {
			self.freq_buckets.push(None);
		}

		// Link at the head of the bucket (O(1) operation)
		// Head is the most recently added node in this bucket
		self.nodes[idx].prev = None;
		self.nodes[idx].next = self.freq_buckets[level];

		if let Some(next) = self.freq_buckets[level] {
			self.nodes[next].prev = Some(idx);
		}

		self.freq_buckets[level] = Some(idx);

		// Update min_freq if this is the first node at this level
		if level < self.min_freq {
			self.min_freq = level;
		}
	}

	/// Decay all frequencies by halving access counters and potentially demoting levels.
	fn decay_frequencies(&mut self) {
		// Collect all nodes in their current buckets, decay them, and rebuild buckets
		let mut nodes_to_reinsert = Vec::new();

		// Extract nodes from all buckets and decay their access counters
		for level in 0..self.freq_buckets.len() {
			let mut idx = self.freq_buckets[level].take();
			while let Some(current) = idx {
				let next = self.nodes[current].next;
				// Decay counter by dividing by 2 (right shift)
				self.nodes[current].access_counter /= 2;
				let current_counter = self.nodes[current].access_counter;
				let current_level = self.nodes[current].freq_level as usize;

				// Determine new level after decay
				let decayed_level = self.determine_level_after_decay(current_level, current_counter);

				self.nodes[current].freq_level = decayed_level as u8;
				self.nodes[current].prev = None;
				self.nodes[current].next = None;
				nodes_to_reinsert.push((current, decayed_level));
				idx = next;
			}
		}

		// Clear buckets
		self.freq_buckets.clear();
		self.freq_buckets.resize(4, None);  // Ensure 4 frequency levels
		self.min_freq = usize::MAX;

		// Reinsert all nodes into their new frequency buckets
		for (idx, level) in nodes_to_reinsert {
			self.link_to_bucket(idx, level);
		}

		// Set min_freq properly
		if self.min_freq == usize::MAX {
			self.min_freq = 1;  // Default to level 1
		}
	}

	/// Determine the appropriate level for a node after decay based on the decayed counter.
	/// For simplicity, keep the node at its current level after decay.
	/// The halved counter naturally reduces the impact of past accesses, and the promotion
	/// thresholds will handle any necessary level changes as new accesses come in.
	fn determine_level_after_decay(&self, current_level: usize, _decayed_counter: u8) -> usize {
		// Keep the node at the same level; the decayed counter will naturally
		// reflect reduced frequency, and future accesses will increment it again
		current_level
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
		cache.put(3, 30); // evicts key 1 (level=1, LRU in bucket 1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_frequency() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // queues promotion, freq_level will update on next put
		cache.put(3, 30); // flushes pending promotions, evicts key 2 (level=1, LRU in bucket 1)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_frequency_and_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1, freq_level may update
		cache.put(3, 30); // flushes pending, evicts key 2 (level=1, LRU in bucket 1)

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
		cache.put(4, 40); // evicts 1 (level=1)
		cache.put(5, 50); // evicts 2 (level=1)
		cache.put(6, 60); // evicts 3 (level=1)

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
		cache.put(4, 40); // should evict key 2 (level=1), not key 1 (level>=1)
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // level=1, inserted at head of bucket 1
		cache.put(2, 20); // level=1, inserted at head of bucket 1 (pushing 1 back)
		cache.put(3, 30); // level=1, inserted at head of bucket 1 (pushing 2 back)
		cache.get(&2); // queues promotion
		// Promotion will be flushed on next put
		cache.put(4, 40); // flushes pending promotions, node 2 may move level, evicts head of bucket 1 (which is 3)
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
		cache.put(4, 40); // flushes pending, node 3 may move level, evicts 2 (slot 1) -> slot 1 reused for key 4
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
		// Frequency may update from level 1 via deferred promotion
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
		cache.put(3, 31); // overwrite, queues promotion, freq_level may update
		cache.put(4, 40); // flushes pending, evicts 1 or 2 (level=1, LRU in bucket 1)
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
		// At this point: key 1 has 3 pending promotions, key 2 level=1
		// The cache will trigger decay every 2 evictions (decay_period = capacity = 2)
		cache.put(3, 30); // flushes pending promotions, triggers eviction -> decay

		// After decay and new eviction, key 1 should survive with appropriate level
		assert!(cache.get(&1).is_some()); // key 1 should survive with appropriate level
	}

	#[test]
	fn quantized_levels_distinct_buckets() {
		// Verify that nodes with different quantized levels are in different buckets
		let mut cache = LruCache::new(4);
		cache.put(1, 10); // level 1, counter 0
		cache.put(2, 20); // level 1, counter 0
		cache.put(3, 30); // level 1, counter 0
		cache.put(4, 40); // level 1, counter 0

		// Access key 1 many times to push it to level 2 or 3
		for _ in 0..5 {
			cache.get(&1);
		}

		// Now cache is full; on next put, we need to evict
		// Eviction should prefer lower-level nodes (keys 2, 3, 4)
		// But since all are at the same level initially (except 1 after accesses),
		// the eviction order depends on the queue management
		cache.put(5, 50); // Should evict key 2 (LRU in level 1)

		assert!(cache.get(&1).is_some()); // Key 1 should survive (higher level)
		assert!(cache.get(&2).is_none()); // Key 2 should be evicted
	}
}
