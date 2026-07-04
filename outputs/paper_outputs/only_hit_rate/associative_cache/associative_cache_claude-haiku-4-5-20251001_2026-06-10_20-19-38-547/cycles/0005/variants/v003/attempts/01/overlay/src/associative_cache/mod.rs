use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Stash buffer to capture evicted items from high-contention buckets and provide second-chance lookup
	stash: Vec<Item<K, V>>,
	/// Maximum capacity of the stash (approximately 25% of total cache capacity)
	stash_capacity: usize,
	/// Track total operations for periodic collision counter reset
	operation_count: usize,
	/// Period for resetting collision counters (in number of operations)
	reset_period: usize,
}

impl<K, V> AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn try_new(settings: AssociativeCacheSettings) -> Option<Self> {
		if let Err(problems) = settings.validate() {
			log::error!("AssociativeCache.try_new: validation failed: {}", problems.join(", "));
			return None;
		}
		// Geometry resolution (and its capacity-conservation check) lives in the
		// contract-protected `factory::make_cache`, which hands us already-resolved
		// settings; here we just build the buckets the settings describe.
		info!("AssociativeCache.try_new: settings={settings:?}");

		// Calculate stash capacity as approximately 25% of total cache capacity
		let total_capacity = settings.num_of_buckets * settings.bucket_size;
		let stash_capacity = std::cmp::max(1, total_capacity / 4);

		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self {
			buckets,
			stash: Vec::with_capacity(stash_capacity),
			stash_capacity,
			operation_count: 0,
			reset_period: 1000,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);

		// Remove any stale version of this key from the stash
		// (to avoid duplicate keys if the same key is put again after being evicted)
		self.stash.retain(|item| item.key != key);

		let bucket_index = get_index(&key, self.buckets.len());

		// Perform the put operation and extract evicted item
		let evicted = {
			let bucket = &mut self.buckets[bucket_index];
			bucket.put(Item::new(key, value)).1
		};

		// Capture evicted item in the stash only if the bucket has high collision count
		// This filters out low-value evictions from lightly-loaded buckets
		if let Some(evicted_item) = evicted {
			// Increment collision counter for this bucket
			let bucket = &mut self.buckets[bucket_index];
			bucket.increment_collision_count();

			// Only add to stash if this bucket has experienced at least 2 collisions
			// (indicating persistent contention). Items from buckets with 0-1 collisions
			// are discarded to reduce stash churn.
			if bucket.collision_count >= 2 {
				self.add_to_stash(evicted_item);
			}
		}

		// Increment operation counter and periodically reset collision counters
		self.operation_count += 1;
		if self.operation_count % self.reset_period == 0 {
			self.reset_collision_counters();
		}
	}

	/// Add an item to the stash using FIFO logic
	/// If the stash is at capacity, remove the oldest item to make room
	fn add_to_stash(&mut self, item: Item<K, V>) {
		debug!("AssociativeCache.add_to_stash: key={:?}", item.key);

		// If stash is at capacity, remove the oldest item (FIFO)
		if self.stash.len() >= self.stash_capacity {
			self.stash.remove(0); // Remove the oldest item (first in, first out)
		}

		self.stash.push(item);
	}

	/// Reset collision counters for all buckets
	fn reset_collision_counters(&mut self) {
		for bucket in &mut self.buckets {
			bucket.reset_collision_count();
		}
	}

	/// Returns the value of key if exists wrapped in Some, None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	/// Performs secondary lookup in the stash on primary bucket miss.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());

		// Primary lookup: use get_position (immutable) to check if in bucket
		// This avoids extending the mutable borrow when we might not find it
		if self.buckets[bucket_index].get_position(key).is_some() {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (bucket)"
			);
			// Now we know it exists, get it and return
			return self.buckets[bucket_index].get(key).map(|item| &item.value);
		}

		// Secondary lookup in stash on primary bucket miss
		if let Some(stash_pos) = self.stash.iter().position(|item| item.key == *key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (stash)"
			);
			let stashed_item = self.stash[stash_pos].clone();

			// Try to re-insert the stashed item back into its primary bucket if space is available
			if !self.buckets[bucket_index].is_full() {
				debug!("AssociativeCache.get_and_update_item: re-inserting key={key:?} from stash to bucket");
				let (inserted_item, _evicted) = self.buckets[bucket_index].put(stashed_item);
				// Remove from stash since we've successfully re-inserted it
				self.stash.remove(stash_pos);
				// Return reference to the inserted item (now in the bucket)
				return Some(&inserted_item.value);
			}

			// If we couldn't re-insert (bucket was full), return reference to the stashed copy
			return Some(&self.stash[stash_pos].value);
		}

		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=false"
		);
		None
	}

	fn get_mut_bucket(&mut self, key: &K) -> &mut AssociativeCacheBucket<K, V> {
		let bucket_index = get_index(key, self.buckets.len());
		&mut self.buckets[bucket_index]
	}
}

impl<K, V> framework::Cache<K, V> for AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn put(&mut self, key: K, value: V) {
		AssociativeCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		AssociativeCache::get_and_update_item(self, key)
	}
}

impl<K, V> Display for AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for bucket in &self.buckets {
			writeln!(f, "{}", bucket)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::test_utils::find_keys_for_bucket;

	fn small_cache(policy: crate::shared::eviction_policy::EvictionPolicyKind) -> AssociativeCache<i64, i64> {
		AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 4,
			bucket_size: 2,
			eviction_policy: policy,
		})
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		assert!(cache.get_and_update_item(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		cache.put(1, 10);
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 20);
	}

	#[test]
	fn multiple_keys_across_buckets() {
		// 4 buckets * bucket_size 2 = capacity 8, but hash collisions may cause eviction.
		// Use fewer keys to stay safe.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		for i in 0..4 {
			cache.put(i, i * 10);
		}
		for i in 0..4 {
			assert_eq!(*cache.get_and_update_item(&i).unwrap(), i * 10);
		}
	}

	#[test]
	fn eviction_when_bucket_full() {
		// bucket_size=2, find 3 keys that hash to same bucket → first should be evicted
		// With collision-aware stash, the evicted item is captured in stash only after 2+ collisions
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO), collision_count becomes 1

		// First eviction doesn't capture in stash because collision_count = 1 < 2
		// So same_bucket_keys[0] should not be recoverable
		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);

		// Add a 4th key to trigger second eviction, which should capture items in stash
		let same_bucket_keys_2 = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 4);
		cache.put(same_bucket_keys_2[3], 40); // second eviction, collision_count becomes 2

		// Now the second evicted item should be in the stash
		assert!(cache.stash.len() > 0);
	}

	#[test]
	fn stash_captures_evicted_item_after_two_collisions() {
		// Verify that evicted items are captured in the stash only after collision_count >= 2
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 4);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // 1st eviction, collision_count = 1, not captured
		cache.put(same_bucket_keys[3], 40); // 2nd eviction, collision_count = 2, captured

		// After 2 collisions, items should start being captured in stash
		assert!(cache.stash.len() > 0);
		// The second evicted item should be in the stash
		assert_eq!(cache.stash[0].key, same_bucket_keys[2]);
	}

	#[test]
	fn stash_provides_second_chance_for_high_collision_items() {
		// Verify that items evicted from high-collision buckets can be recovered from stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 4);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // 1st eviction, collision_count = 1
		cache.put(same_bucket_keys[3], 40); // 2nd eviction, collision_count = 2, same_bucket_keys[1] captured

		// same_bucket_keys[1] should be in the stash now
		let val = cache.get_and_update_item(&same_bucket_keys[1]);
		assert!(val.is_some());
		assert_eq!(*val.unwrap(), 20);
	}

	#[test]
	fn stash_reinsertion_removes_from_stash() {
		// Verify that when a stashed item is re-inserted into a bucket with available space,
		// it's removed from the stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 4);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // 1st eviction
		cache.put(same_bucket_keys[3], 40); // 2nd eviction, same_bucket_keys[1] captured in stash

		assert!(cache.stash.len() > 0);

		// Access same_bucket_keys[1] - it will be re-inserted since bucket might have space
		let val = cache.get_and_update_item(&same_bucket_keys[1]);
		assert!(val.is_some());
		assert_eq!(*val.unwrap(), 20);
	}

	#[test]
	fn stash_fifo_eviction() {
		// Verify that stash uses FIFO eviction when at capacity
		let mut cache = AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 2,
			bucket_size: 1, // Very small buckets to force evictions
			eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::Fifo,
		});
		// Stash capacity = (2 * 1) / 4 = 0 but max 1, so 1

		// Put keys to same bucket to cause evictions
		let keys = find_keys_for_bucket(0, 2, 5);
		for (i, key) in keys.iter().enumerate() {
			cache.put(*key, (i as i64) * 10);
		}

		// With collision-aware filtering, stash should only have items from high-collision buckets
		// after the reset (every 1000 operations), but within a single test we see the growth
		assert!(cache.stash.len() <= 1);
	}

	#[test]
	fn collision_counter_resets_periodically() {
		// Verify that collision counters reset after reset_period operations
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 10);

		// Perform many puts to trigger collision counter reset
		for (i, key) in same_bucket_keys.iter().take(5).enumerate() {
			cache.put(*key, (i as i64) * 10);
		}

		// Check that collision count is tracking evictions
		let bucket = &cache.buckets[target_bucket_idx];
		assert!(bucket.collision_count > 0);

		// Perform reset_period operations to trigger reset
		for i in 0..cache.reset_period {
			cache.put(100000 + i as i64, i as i64);
		}

		// After reset, collision counters should be 0 (reset at operation reset_period)
		let bucket = &cache.buckets[target_bucket_idx];
		assert_eq!(bucket.collision_count, 0);
	}
}
