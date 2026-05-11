use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::item::Item;
use log::{debug, trace};
use std::{fmt::Debug, hash::Hash, time::Instant};

/// Where to insert a new item within a bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InsertPosition {
	#[default]
	Back,
	Front,
}

pub trait Bucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	// ------------ struct expected fields ----------------------------------------------

	fn get_items(&self) -> &Vec<Item<K, V>>;

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>>;

	fn get_max_size(&self) -> usize {
		usize::MAX
	}

	fn get_eviction_policy(&self) -> &EvictionPolicy;

	// ----------------------------------------------------------------------------------

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.put_at(item, InsertPosition::Back)
	}

	/// If the key already exists, updates in place per policy and ignores `pos`.
	fn put_at(
		&mut self,
		item: Item<K, V>,
		pos: InsertPosition,
	) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put_at: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let pushed_item = match pos {
			InsertPosition::Back => {
				self.get_items_mut().push(item);
				self.get_items().last().expect("bucket invariant: just pushed")
			}
			InsertPosition::Front => {
				self.get_items_mut().insert(0, item);
				self.get_items().first().expect("bucket invariant: just inserted")
			}
		};
		(pushed_item, evicted_item)
	}

	fn remove(&mut self, key: &K) {
		self.get_items_mut().retain(|item| item.key != *key);
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key)?;
		Some(self.get_from_position(position))
	}

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
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!("Bucket.evict_item: policy={:?} evicted_key={:?}", self.get_eviction_policy(), evicted.key);
		Some(evicted)
	}

	/// Returns whether the bucket is full.
	fn is_full(&self) -> bool {
		self.size() == self.get_max_size()
	}

	/// Returns the number of items in the bucket.
	fn size(&self) -> usize {
		self.get_items().len()
	}

	/// Returns the index of the item that would be evicted, without removing it.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.get_items().is_empty() {
			return None;
		}
		Some(match self.get_eviction_policy() {
			EvictionPolicy::ClassicLRU | EvictionPolicy::Fifo | EvictionPolicy::Swap => 0,
			EvictionPolicy::TimestampLRU => {
				self.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.timestamp)
					.expect("bucket invariant: non-empty after is_empty check")
					.0
			}
			EvictionPolicy::Lifo => self.get_items().len() - 1,
			EvictionPolicy::Lfu => {
				self.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.lfu_counter)
					.expect("bucket invariant: non-empty after is_empty check")
					.0
			}
		})
	}

	/// Returns the index of the "hottest" item - the one most protected from eviction.
	/// Symmetric inverse of `eviction_candidate_index`.
	fn promotion_candidate_index(&self) -> Option<usize> {
		if self.get_items().is_empty() {
			return None;
		}
		Some(match self.get_eviction_policy() {
			EvictionPolicy::ClassicLRU | EvictionPolicy::Fifo | EvictionPolicy::Swap => {
				self.get_items().len() - 1
			}
			EvictionPolicy::Lifo => 0,
			EvictionPolicy::TimestampLRU => {
				self.get_items()
					.iter()
					.enumerate()
					.max_by_key(|(_, item)| item.timestamp)
					.expect("bucket invariant: non-empty after is_empty check")
					.0
			}
			EvictionPolicy::Lfu => {
				self.get_items()
					.iter()
					.enumerate()
					.max_by_key(|(_, item)| item.lfu_counter)
					.expect("bucket invariant: non-empty after is_empty check")
					.0
			}
		})
	}

	/// Remove and return the item at the given index. Panics if out of bounds.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.get_items().len(),
			"remove_at: index {index} out of bounds (len={})", self.get_items().len()
		);
		self.get_items_mut().remove(index)
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

	// ==================== put_at ====================

	#[test]
	fn put_at_back_is_same_as_put() {
		let mut a = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		let mut b = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		a.put(Item::new(1, 10));
		b.put_at(Item::new(1, 10), InsertPosition::Back);
		assert_eq!(keys(&a), keys(&b));
	}

	#[test]
	fn put_at_front_inserts_at_position_zero() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put_at(Item::new(3, 30), InsertPosition::Front);
		assert_eq!(keys(&bucket), vec![3, 1, 2]);
	}

	#[test]
	fn put_at_front_evicts_when_full() {
		let mut bucket = TestBucket::new(2, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let (_, evicted) = bucket.put_at(Item::new(3, 30), InsertPosition::Front);
		assert_eq!(evicted.unwrap().key, 1);
		assert_eq!(keys(&bucket), vec![3, 2]);
	}

	#[test]
	fn put_at_front_no_eviction_when_not_full() {
		let mut bucket = TestBucket::new(3, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		let (_, evicted) = bucket.put_at(Item::new(2, 20), InsertPosition::Front);
		assert!(evicted.is_none());
		assert_eq!(keys(&bucket), vec![2, 1]);
	}

	// ==================== eviction_candidate_index ====================

	#[test]
	fn eviction_candidate_index_empty() {
		let bucket = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		assert_eq!(bucket.eviction_candidate_index(), None);
	}

	#[test]
	fn eviction_candidate_index_classic_lru() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		assert_eq!(bucket.eviction_candidate_index(), Some(0));
	}

	#[test]
	fn eviction_candidate_index_lifo() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		assert_eq!(bucket.eviction_candidate_index(), Some(1));
	}

	#[test]
	fn eviction_candidate_index_lfu() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Lfu);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // counter: 1
		bucket.get(&1); // counter: 2
		bucket.get(&2); // counter: 1
		// key 2 has lower counter
		assert_eq!(bucket.eviction_candidate_index(), Some(1));
	}

	#[test]
	fn eviction_candidate_index_timestamp_lru() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&2); // refresh timestamp of 2
		// 1 has the older timestamp
		let idx = bucket.eviction_candidate_index().unwrap();
		assert_eq!(bucket.get_items()[idx].key, 1);
	}

	// ==================== promotion_candidate_index ====================

	#[test]
	fn promotion_candidate_index_empty() {
		let bucket = TestBucket::new(4, EvictionPolicy::ClassicLRU);
		assert_eq!(bucket.promotion_candidate_index(), None);
	}

	#[test]
	fn promotion_candidate_index_classic_lru_fifo_swap_is_last() {
		for policy in [EvictionPolicy::ClassicLRU, EvictionPolicy::Fifo, EvictionPolicy::Swap] {
			let mut bucket = TestBucket::new(4, policy);
			bucket.put(Item::new(1, 10));
			bucket.put(Item::new(2, 20));
			assert_eq!(bucket.promotion_candidate_index(), Some(1), "policy={policy:?}");
		}
	}

	#[test]
	fn promotion_candidate_index_lifo_is_first() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		assert_eq!(bucket.promotion_candidate_index(), Some(0));
	}

	#[test]
	fn promotion_candidate_index_lfu_picks_max_counter() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Lfu);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&2); // counter: 1
		bucket.get(&2); // counter: 2
		bucket.get(&1); // counter: 1
		let idx = bucket.promotion_candidate_index().unwrap();
		assert_eq!(bucket.get_items()[idx].key, 2);
	}

	#[test]
	fn promotion_candidate_index_timestamp_lru_picks_max_timestamp() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // refresh timestamp of 1
		let idx = bucket.promotion_candidate_index().unwrap();
		assert_eq!(bucket.get_items()[idx].key, 1);
	}

	// ==================== remove_at ====================

	#[test]
	fn remove_at_returns_item() {
		let mut bucket = TestBucket::new(4, EvictionPolicy::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let removed = bucket.remove_at(0);
		assert_eq!(removed.key, 1);
		assert_eq!(keys(&bucket), vec![2]);
	}
}
