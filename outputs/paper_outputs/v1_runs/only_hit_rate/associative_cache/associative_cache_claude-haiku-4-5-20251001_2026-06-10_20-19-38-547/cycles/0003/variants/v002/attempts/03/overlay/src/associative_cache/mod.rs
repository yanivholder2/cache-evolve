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
	stash: Vec<Item<K, V>>,
	stash_capacity: usize,
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
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}

		// Stash capacity is 20% of total capacity (total_capacity / 5)
		let stash_capacity = (settings.capacity() / 5).max(1);
		info!("AssociativeCache.try_new: stash_capacity={stash_capacity}");

		Some(Self {
			buckets,
			stash: Vec::with_capacity(stash_capacity),
			stash_capacity,
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
		let bucket = self.get_mut_bucket(&key);
		let (_inserted, evicted) = bucket.put(Item::new(key, value));

		// If an item was evicted from the bucket, add it to the stash
		if let Some(evicted_item) = evicted {
			self.add_to_stash(evicted_item);
		}
	}

	/// Returns the value of key if exists wrapped in Some, and None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());

		// Check the bucket first - return immediately if hit
		// This inline pattern avoids holding a bucket reference across the stash check
		if let Some(item) = self.buckets[bucket_index].get(key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true"
			);
			return Some(&item.value);
		}

		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=false"
		);

		// Primary bucket miss - check the stash
		self.get_from_stash(key)
	}

	/// Add an item to the stash using FIFO eviction.
	/// When at capacity, remove the oldest item (index 0).
	fn add_to_stash(&mut self, item: Item<K, V>) {
		if self.stash.len() >= self.stash_capacity {
			// Stash is at capacity - evict the oldest item (FIFO, index 0)
			self.stash.remove(0);
			debug!("Stash evicted item at index 0 (FIFO)");
		}
		self.stash.push(item);
		debug!("Added item to stash, stash size: {}", self.stash.len());
	}

	/// Look up a key in the stash, returning the value if found.
	fn get_from_stash(&self, key: &K) -> Option<&V> {
		if let Some(item) = self.stash.iter().find(|item| item.key == *key) {
			debug!("Stash hit for key={key:?}");
			Some(&item.value)
		} else {
			debug!("Stash miss for key={key:?}");
			None
		}
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
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO) to bucket, but should go to stash

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_some()); // Now in stash!
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn stash_holds_evicted_items() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 4);

		// Fill bucket
		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);

		// Force evictions
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0] to stash
		cache.put(same_bucket_keys[3], 40); // evicts same_bucket_keys[1] to stash

		// Verify evicted items are in stash
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
	}
}
