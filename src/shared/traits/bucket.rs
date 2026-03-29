use crate::eviction_policy::EvictionPolicy;
use crate::shared::item::Item;
use log::{debug, trace};
use std::{fmt::Debug, hash::Hash, time::Instant};

pub trait Bucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	// ------------ struct expected fields ----------------------------------------------

	/// Returns the items vector which contains the items in the bucket.
	fn get_items(&self) -> &Vec<Item<K, V>>;

	/// Returns the items vector which contains the items in the bucket.
	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>>;

	/// Returns the maximum size of the bucket.
	/// The maximum size should be lower than or equal to usize::MAX.
	fn get_max_size(&self) -> usize {
		usize::MAX
	}

	/// Returns the eviction policy of the bucket.
	fn get_eviction_policy(&self) -> &EvictionPolicy;

	// ----------------------------------------------------------------------------------

	/// Puts an item into the bucket.
	///
	/// Returns a tuple containing a reference to the pushed item and an optional evicted item.
	/// If no item is evicted, the second element of the tuple will be None.
	/// As a side effect, if the item already exists makes updates according to the eviction policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);

			(pushed_item, None)
		} else {
			// If the key does not exist, add the item to the bucket
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			let pushed_item = self.put_according_to_policy(item);

			(pushed_item, evicted_item)
		}
	}

	fn put_according_to_policy(&mut self, item: Item<K, V>) -> &Item<K, V> {
		match self.get_eviction_policy() {
			EvictionPolicy::Fifo
			| EvictionPolicy::Lifo
			| EvictionPolicy::ClassicLRU
			| EvictionPolicy::TimestampLRU
			| EvictionPolicy::Swap
			| EvictionPolicy::Lfu => {
				self.get_items_mut().push(item);
				self.get_items().last().expect("bucket invariant: just pushed")
			}
		}
	}

	/// Removes the key-value pair with the given key from the bucket.
	/// If the bucket is empty or the key is not found, this function does nothing.
	fn remove(&mut self, key: &K) {
		self.get_items_mut().retain(|item| item.key != *key);
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key)?;
		Some(self.get_from_position(position))
	}

	/// Returns the position of the item with the given key, or `None` if the key is not found.
	fn get_position(&self, key: &K) -> Option<usize> {
		self.get_items().iter().position(|d| d.key == *key)
	}

	/// Returns a reference to the item located in `position`.
	///
	/// As a side effect makes updates according to the eviction policy.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		match self.get_eviction_policy() {
			EvictionPolicy::Fifo | EvictionPolicy::Lifo => &self.get_items()[position],
			EvictionPolicy::ClassicLRU => {
				let item = self.get_items_mut().remove(position);
				self.get_items_mut().push(item);
				self.get_items().last().expect("bucket invariant: just pushed")
			}
			EvictionPolicy::TimestampLRU => {
				self.get_items_mut()[position].timestamp = Instant::now();
				&self.get_items()[position]
			}
			EvictionPolicy::Swap => {
				if position > 0 {
					self.get_items_mut().swap(position, position - 1);
					&self.get_items()[position - 1]
				} else {
					&self.get_items()[position]
				}
			}
			EvictionPolicy::Lfu => {
				self.get_items_mut()[position].lfu_counter += 1;
				&self.get_items()[position]
			}
		}
	}

	/// Evicts an item from the bucket according to the eviction policy and return it.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.get_items().is_empty() {
			return None;
		}

		let evicted = match self.get_eviction_policy() {
			EvictionPolicy::ClassicLRU | EvictionPolicy::Fifo | EvictionPolicy::Swap => {
				// O(n) due to Vec shift — acceptable for simulator; VecDeque would complicate the trait
				Some(self.get_items_mut().remove(0))
			}
			EvictionPolicy::TimestampLRU => {
				let (min_timestamp_index, _) = self
					.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.timestamp)
					.expect("bucket invariant: non-empty after is_empty check");
				Some(self.get_items_mut().remove(min_timestamp_index))
			}
			EvictionPolicy::Lifo => self.get_items_mut().pop(),
			EvictionPolicy::Lfu => {
				let (min_lfu_counter_index, _) = self
					.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.lfu_counter)
					.expect("bucket invariant: non-empty after is_empty check");
				Some(self.get_items_mut().remove(min_lfu_counter_index))
			}
		};
		if let Some(ref item) = evicted {
			debug!("Bucket.evict_item: policy={:?} evicted_key={:?}", self.get_eviction_policy(), item.key);
		}
		evicted
	}

	/// Returns whether the bucket is full.
	fn is_full(&self) -> bool {
		self.size() == self.get_max_size()
	}

	/// Returns the number of items in the bucket.
	fn size(&self) -> usize {
		self.get_items().len()
	}
}


#[cfg(test)]
mod tests {
	use super::*;

	/// Minimal Bucket impl for testing trait default methods in isolation.
	struct TestBucket {
		items: Vec<Item<i64, i64>>,
		max_size: usize,
		eviction_policy: EvictionPolicy,
	}

	impl TestBucket {
		fn new(max_size: usize, policy: EvictionPolicy) -> Self {
			Self { items: Vec::new(), max_size, eviction_policy: policy }
		}
	}

	impl Bucket<i64, i64> for TestBucket {
		fn get_items(&self) -> &Vec<Item<i64, i64>> {
			&self.items
		}
		fn get_items_mut(&mut self) -> &mut Vec<Item<i64, i64>> {
			&mut self.items
		}
		fn get_max_size(&self) -> usize {
			self.max_size
		}
		fn get_eviction_policy(&self) -> &EvictionPolicy {
			&self.eviction_policy
		}
	}

	/// Helper: collect keys from bucket in order.
	fn keys(bucket: &TestBucket) -> Vec<i64> {
		bucket.get_items().iter().map(|i| i.key).collect()
	}

	// ==================== Basic operations (policy-independent) ====================

	#[test]
	fn empty_bucket() {
		let bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		assert_eq!(bucket.size(), 0);
		assert!(!bucket.is_full());
	}

	#[test]
	fn put_and_get() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		assert_eq!(bucket.size(), 1);
		assert_eq!(bucket.get(&1).unwrap().value, 10);
	}

	#[test]
	fn put_updates_existing_key() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(1, 20));
		assert_eq!(bucket.size(), 1);
		assert_eq!(bucket.get(&1).unwrap().value, 20);
	}

	#[test]
	fn get_missing_key_returns_none() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		assert!(bucket.get(&99).is_none());
	}

	#[test]
	fn remove_existing_key() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.remove(&1);
		assert_eq!(bucket.size(), 1);
		assert!(bucket.get(&1).is_none());
		assert_eq!(bucket.get(&2).unwrap().value, 20);
	}

	#[test]
	fn remove_missing_key_is_noop() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.remove(&99);
		assert_eq!(bucket.size(), 1);
	}

	#[test]
	fn is_full_at_max_size() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		assert!(!bucket.is_full());
		bucket.put(Item::new(2, 20));
		assert!(bucket.is_full());
	}

	#[test]
	fn evict_from_empty_returns_none() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		assert!(bucket.evict_item().is_none());
	}

	// ==================== FIFO ====================

	#[test]
	fn fifo_evicts_oldest_inserted() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		// Bucket full, inserting 3 should evict 1
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
		assert_eq!(keys(&bucket), vec![2, 3]);
	}

	#[test]
	fn fifo_get_does_not_reorder() {
		let mut bucket = TestBucket::new(3, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		bucket.get(&1); // access oldest
		assert_eq!(keys(&bucket), vec![1, 2, 3]); // order unchanged
	}

	// ==================== LIFO ====================

	#[test]
	fn lifo_evicts_newest_inserted() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
		assert_eq!(keys(&bucket), vec![1, 3]);
	}

	#[test]
	fn lifo_get_does_not_reorder() {
		let mut bucket = TestBucket::new(3, EvictionPolicy::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&2);
		assert_eq!(keys(&bucket), vec![1, 2]); // order unchanged
	}

	// ==================== ClassicLRU ====================

	#[test]
	fn classic_lru_evicts_least_recently_used() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // touch 1, so 2 is now LRU
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
	}

	#[test]
	fn classic_lru_get_moves_to_back() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		bucket.get(&1); // move 1 to back (most recent)
		assert_eq!(keys(&bucket), vec![2, 3, 1]);
	}

	#[test]
	fn classic_lru_eviction_chain() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		// Access 1 -> order: [2, 1], evict 2
		bucket.get(&1);
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
		// Now: [1, 3]. Access 3 -> order: [1, 3], evict 1
		bucket.get(&3);
		let (_, evicted) = bucket.put(Item::new(4, 40));
		assert_eq!(evicted.unwrap().key, 1);
	}

	// ==================== TimestampLRU ====================

	#[test]
	fn timestamp_lru_evicts_oldest_timestamp() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // refreshes timestamp of 1
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2); // 2 has older timestamp
	}

	#[test]
	fn timestamp_lru_get_does_not_reorder_vec() {
		let mut bucket = TestBucket::new(3, EvictionPolicy::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1);
		assert_eq!(keys(&bucket), vec![1, 2]); // vec order unchanged
	}

	// ==================== Swap ====================

	#[test]
	fn swap_get_moves_toward_front() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Swap);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		// Get key 3 at position 2 -> swaps with position 1
		bucket.get(&3);
		assert_eq!(keys(&bucket), vec![1, 3, 2]);
		// Get key 3 again at position 1 -> swaps with position 0
		bucket.get(&3);
		assert_eq!(keys(&bucket), vec![3, 1, 2]);
		// Get key 3 at position 0 -> no swap (already at front)
		bucket.get(&3);
		assert_eq!(keys(&bucket), vec![3, 1, 2]);
	}

	#[test]
	fn swap_evicts_front_item() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Swap);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1); // front evicted
	}

	// ==================== LFU ====================

	#[test]
	fn lfu_get_increments_counter() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Lfu);
		bucket.put(Item::new(1, 10));
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 1);
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 2);
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 3);
	}

	#[test]
	fn lfu_evicts_least_frequently_used() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Lfu);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // counter: 1
		bucket.get(&1); // counter: 2
		bucket.get(&2); // counter: 1
		// 2 has lower count, should be evicted
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
	}

	#[test]
	fn lfu_evicts_first_on_tie() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::Lfu);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		// Both have counter 0 — min_by_key picks first
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
	}

	// ==================== put returns correct references ====================

	#[test]
	fn put_returns_ref_to_inserted_item() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		let (inserted, evicted) = bucket.put(Item::new(5, 50));
		assert_eq!(inserted.key, 5);
		assert_eq!(inserted.value, 50);
		assert!(evicted.is_none());
	}

	#[test]
	fn put_returns_ref_and_evicted_when_full() {
		let mut bucket = TestBucket::new(1, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		let (inserted, evicted) = bucket.put(Item::new(2, 20));
		assert_eq!(inserted.key, 2);
		assert_eq!(evicted.unwrap().key, 1);
	}
}
