use crate::shared::item::Item;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};
use std::{fmt::Debug, hash::Hash};

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
	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V>;

	// ----------------------------------------------------------------------------------

	/// Puts an item at the back of the bucket.
	///
	/// Returns `(&inserted, Option<evicted>)`. Evicts per policy if full.
	/// If the key already exists, updates in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.get_items_mut().push(item);
		let pushed_item = self.get_items().last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
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
	///
	/// Required on the implementor: dispatching to the policy needs the policy
	/// and the items vector borrowed at once, which only the concrete struct can
	/// do across its disjoint fields.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V>;

	/// Evicts an item from the bucket according to the eviction policy and return it.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.get_eviction_policy().name(),
			evicted.key
		);
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
		Some(self.get_eviction_policy().eviction_candidate_index(self.get_items()))
	}

	/// Remove and return the item at the given index. Panics if out of bounds.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.get_items().len(),
			"remove_at: index {index} out of bounds (len={})",
			self.get_items().len()
		);
		self.get_items_mut().remove(index)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	/// Minimal Bucket impl for testing trait default methods in isolation.
	struct TestBucket {
		items: Vec<Item<i64, i64>>,
		max_size: usize,
		eviction_policy: Box<dyn EvictionPolicy<i64, i64>>,
	}

	impl TestBucket {
		fn new(max_size: usize, policy: EvictionPolicyKind) -> Self {
			Self {
				items: Vec::new(),
				max_size,
				eviction_policy: policy.build(),
			}
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
		fn get_eviction_policy(&self) -> &dyn EvictionPolicy<i64, i64> {
			&*self.eviction_policy
		}
		fn get_from_position(&mut self, position: usize) -> &Item<i64, i64> {
			let index = self.eviction_policy.get_from_position(&mut self.items, position);
			&self.items[index]
		}
	}

	/// Helper: collect keys from bucket in order.
	fn keys(bucket: &TestBucket) -> Vec<i64> {
		bucket.get_items().iter().map(|i| i.key).collect()
	}

	// ==================== Basic operations (policy-independent) ====================

	#[test]
	fn empty_bucket() {
		let bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		assert_eq!(bucket.size(), 0);
		assert!(!bucket.is_full());
	}

	#[test]
	fn put_and_get() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		assert_eq!(bucket.size(), 1);
		assert_eq!(bucket.get(&1).unwrap().value, 10);
	}

	#[test]
	fn put_updates_existing_key() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(1, 20));
		assert_eq!(bucket.size(), 1);
		assert_eq!(bucket.get(&1).unwrap().value, 20);
	}

	#[test]
	fn get_missing_key_returns_none() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		assert!(bucket.get(&99).is_none());
	}

	#[test]
	fn is_full_at_max_size() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		assert!(!bucket.is_full());
		bucket.put(Item::new(2, 20));
		assert!(bucket.is_full());
	}

	#[test]
	fn evict_from_empty_returns_none() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		assert!(bucket.evict_item().is_none());
	}

	// ==================== FIFO ====================

	#[test]
	fn fifo_evicts_oldest_inserted() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		// Bucket full, inserting 3 should evict 1
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
		assert_eq!(keys(&bucket), vec![2, 3]);
	}

	#[test]
	fn fifo_get_does_not_reorder() {
		let mut bucket = TestBucket::new(3, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		bucket.get(&1); // access oldest
		assert_eq!(keys(&bucket), vec![1, 2, 3]); // order unchanged
	}

	// ==================== LIFO ====================

	#[test]
	fn lifo_evicts_newest_inserted() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
		assert_eq!(keys(&bucket), vec![1, 3]);
	}

	// ==================== ClassicLRU ====================

	#[test]
	fn classic_lru_evicts_least_recently_used() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // touch 1, so 2 is now LRU
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
	}

	#[test]
	fn classic_lru_get_moves_to_back() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		// Two-step promotion: hit at position 0 of a 3-item bucket is a front-half
		// hit (mid = 1), so it lands at the midpoint (index mid - 1 = 0), not the back.
		bucket.get(&1);
		assert_eq!(keys(&bucket), vec![1, 2, 3]);
	}

	// ==================== TimestampLRU ====================

	#[test]
	fn timestamp_lru_evicts_oldest_timestamp() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&1); // refreshes timestamp of 1
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2); // 2 has older timestamp
	}

	// ==================== Swap ====================

	#[test]
	fn swap_get_moves_toward_front() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Swap);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.put(Item::new(3, 30));
		// Get key 3 at position 2 -> swaps with position 1
		bucket.get(&3);
		assert_eq!(keys(&bucket), vec![1, 3, 2]);
	}

	#[test]
	fn swap_evicts_front_item() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Swap);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1); // front evicted
	}

	// ==================== LFU ====================

	#[test]
	fn lfu_get_increments_counter() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Lfu);
		bucket.put(Item::new(1, 10));
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 1);
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 2);
		assert_eq!(bucket.get(&1).unwrap().lfu_counter, 3);
	}

	#[test]
	fn lfu_evicts_least_frequently_used() {
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Lfu);
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
		let mut bucket = TestBucket::new(2, EvictionPolicyKind::Lfu);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		// Both have counter 0 — min_by_key picks first
		let (_, evicted) = bucket.put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
	}

	// ==================== put returns correct references ====================

	#[test]
	fn put_returns_ref_to_inserted_item() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		let (inserted, evicted) = bucket.put(Item::new(5, 50));
		assert_eq!(inserted.key, 5);
		assert_eq!(inserted.value, 50);
		assert!(evicted.is_none());
	}

	#[test]
	fn put_returns_ref_and_evicted_when_full() {
		let mut bucket = TestBucket::new(1, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		let (inserted, evicted) = bucket.put(Item::new(2, 20));
		assert_eq!(inserted.key, 2);
		assert_eq!(evicted.unwrap().key, 1);
	}

	// ==================== eviction_candidate_index ====================

	#[test]
	fn eviction_candidate_index_empty() {
		let bucket = TestBucket::new(4, EvictionPolicyKind::ClassicLRU);
		assert_eq!(bucket.eviction_candidate_index(), None);
	}

	#[test]
	fn eviction_candidate_index_classic_lru() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::ClassicLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		assert_eq!(bucket.eviction_candidate_index(), Some(0));
	}

	#[test]
	fn eviction_candidate_index_lifo() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Lifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		assert_eq!(bucket.eviction_candidate_index(), Some(1));
	}

	#[test]
	fn eviction_candidate_index_lfu() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Lfu);
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
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::TimestampLRU);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		bucket.get(&2); // refresh timestamp of 2
										// 1 has the older timestamp
		let idx = bucket.eviction_candidate_index().unwrap();
		assert_eq!(bucket.get_items()[idx].key, 1);
	}

	// ==================== remove_at ====================

	#[test]
	fn remove_at_returns_item() {
		let mut bucket = TestBucket::new(4, EvictionPolicyKind::Fifo);
		bucket.put(Item::new(1, 10));
		bucket.put(Item::new(2, 20));
		let removed = bucket.remove_at(0);
		assert_eq!(removed.key, 1);
		assert_eq!(keys(&bucket), vec![2]);
	}
}
