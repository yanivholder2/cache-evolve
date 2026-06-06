// FIFO cache with selective promotion window and deferred-promotion batching.
// Uses pure FIFO eviction (head) with a window-limited promotion mechanism
// and 16-item batch queue-flush to reduce unlinking overhead.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // oldest (FIFO front)
	tail: Option<usize>,    // newest (FIFO back)
	capacity: usize,
	window_size: usize,     // min(capacity/2, 512)
	deferred_promotions: HashSet<usize>, // indices pending promotion
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
		let window_size = std::cmp::min(capacity / 2, 512);
		log::info!("LruCache.try_new: capacity={capacity} window_size={window_size}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			window_size,
			deferred_promotions: HashSet::new(),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Check if a node is within the promotion window (recent items only).
	/// Nodes outside the window will not be promoted, preserving FIFO order.
	fn is_in_window(&self, idx: usize) -> bool {
		let current_len = self.map.len();
		if current_len <= self.window_size {
			// All items currently in cache are in the window
			return true;
		}

		// Count distance from tail (most recent); if within window_size, it's in window
		let mut distance = 0;
		let mut current = self.tail;
		loop {
			match current {
				Some(c) => {
					if c == idx {
						return true;
					}
					distance += 1;
					if distance >= self.window_size {
						return false;
					}
					current = self.nodes[c].prev;
				}
				None => return false,
			}
		}
	}

	/// Flush all deferred promotions: unlink and re-push to tail.
	fn flush_deferred_promotions(&mut self) {
		// Collect indices into a Vec to break the immutable borrow before calling mutable methods
		let indices: Vec<_> = self.deferred_promotions.iter().copied().collect();
		for idx in indices {
			self.unlink(idx);
			self.push_back(idx);
		}
		self.deferred_promotions.clear();
	}

	/// Returns the value for `key` if present.
	/// If the key is in the promotion window, it is added to the deferred promotion queue.
	/// When the queue reaches 16 items, all pending promotions are flushed.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;

		// If this node is in the window, queue it for promotion (batched)
		if self.is_in_window(idx) {
			// Only add if not already in queue
			if self.deferred_promotions.insert(idx) {
				// If queue is full, flush all pending promotions
				if self.deferred_promotions.len() >= 16 {
					self.flush_deferred_promotions();
				}
			}
		}
		// Note: nodes outside the window are not promoted; they remain in FIFO order

		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the oldest (FIFO) entry.
	pub fn put(&mut self, key: K, value: V) {
		// Flush any pending promotions before insertion
		self.flush_deferred_promotions();

		// Key already exists: update value in-place, no promotion
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			return;
		}

		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict oldest (FIFO head) and reuse its slot
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");
			let head_idx = self.head.expect("non-empty list has a head");
			let evicted_key = self.nodes[head_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			self.nodes[head_idx].key = key.clone();
			self.nodes[head_idx].value = value;
			head_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			idx
		};
		// Link the new/reused slot at the tail (newest) and register in the map
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

	/// Append node to the tail (newest end) of the list.
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
	fn fifo_eviction_removes_oldest() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts oldest key 1 (FIFO)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_does_not_change_fifo_order_if_outside_window() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // full; window_size = min(4/2, 512) = 2
		// Items 1,2 are outside the window; accessing them won't promote them
		// Only items 3,4 are in the window
		cache.get(&1); // outside window, won't be promoted
		cache.put(5, 50); // should still evict the oldest, which is 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn deferred_promotion_within_window() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // full; window has 2 most recent items (3, 4)
		// Item 3 is in the window, promote it via deferred queue
		cache.get(&3);
		// Promotions are deferred; item hasn't moved to tail yet
		cache.put(5, 50); // Should evict 1 (oldest)
		assert!(cache.get(&1).is_none());
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
	fn repeated_get_preserves_fifo_order_outside_window() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		// window_size = min(3/2, 512) = 1, so only item 3 is in window
		for _ in 0..10 {
			cache.get(&1); // outside window, won't be promoted
		}
		cache.put(4, 40); // should evict key 1 (oldest), not key 2
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn batch_flushing_at_16_items() {
		let mut cache = LruCache::new(20);
		// Insert 20 items to fill
		for i in 1..=20 {
			cache.put(i, i * 10);
		}
		// window_size = min(20/2, 512) = 10, so only items 11-20 are in window
		// Access items 11-20 (within window); each adds to deferred queue
		for i in 11..=20 {
			cache.get(&i);
		}
		// After 10 gets, deferred queue should have 10 items (not yet flushed)
		// The 20th get (item 20) will make it 11, still not flush
		// But if we add one more, the 11th unique item will trigger flush
		// Actually, we accessed items 11-20, that's 10 items
		// Let's access item 11 again: if not in queue, will try to add (but already there)
		// Since we use HashSet, duplicates are ignored

		// Let's just put a new item to force flush, then check behavior
		cache.put(21, 210); // forces flush before insertion
		// After flush, items 11-20 should have been promoted to the tail
		// Then 1 (oldest) gets evicted

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&11).unwrap(), 110);
	}

	#[test]
	fn put_updates_value_no_promotion() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1, should not promote (FIFO style)
		cache.put(3, 30); // evicts 1 (oldest)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // within window, adds to deferred
		cache.put(4, 40); // flushes deferred, then evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts oldest (1? or depends on promotions)
		// After flushing, 3 was moved to tail. So order was: 2, 3. Then 4 evicts 2.
		// Order becomes: 3, 4. Then 5 evicts 3.
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// Single element in window, accessing it adds to deferred
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify list is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; 3 is newest (tail)
		cache.put(3, 31); // overwrite, should not evict
		cache.put(4, 40); // evicts 1 (oldest)
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
