use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// A stashed item with access tracking for LRU eviction.
#[derive(Debug, Clone)]
struct StashedItem<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	item: Item<K, V>,
	// Track when this item was last accessed (for LRU eviction within the stash)
	last_access_time: Instant,
}

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Stash to hold evicted items, sized at 25% of total capacity
	stash: Vec<StashedItem<K, V>>,
	/// Maximum size of the stash (25% of total cache capacity)
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

		// Calculate stash capacity as 25% of total cache capacity
		let total_capacity = settings.capacity();
		let stash_capacity = total_capacity / 4;

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

		// If an item was evicted from the primary bucket, capture it to the stash
		if let Some(evicted_item) = evicted {
			self.add_to_stash(evicted_item);
		}
	}

	/// Add an evicted item to the stash, applying LRU eviction if the stash is full.
	fn add_to_stash(&mut self, item: Item<K, V>) {
		let stashed_item = StashedItem {
			item,
			last_access_time: Instant::now(),
		};

		if self.stash.len() < self.stash_capacity {
			// Stash has room, just add it
			self.stash.push(stashed_item);
			debug!("AssociativeCache.add_to_stash: stash_size={}, capacity={}", self.stash.len(), self.stash_capacity);
		} else if self.stash_capacity > 0 {
			// Stash is full, remove the least-recently-accessed item and add the new one
			if let Some(lru_index) = self.find_lru_in_stash() {
				let _evicted = self.stash.remove(lru_index);
				debug!("AssociativeCache.add_to_stash: evicted_key_from_stash={:?}, stash_size={}", _evicted.item.key, self.stash.len());
			}
			self.stash.push(stashed_item);
		}
		// If stash_capacity is 0, we discard evicted items (no stash)
	}

	/// Find the index of the least-recently-accessed item in the stash.
	fn find_lru_in_stash(&self) -> Option<usize> {
		if self.stash.is_empty() {
			return None;
		}

		let (index, _) = self.stash
			.iter()
			.enumerate()
			.min_by_key(|(_, stashed)| stashed.last_access_time)
			.expect("stash is non-empty");

		Some(index)
	}

	/// Returns the value of key if exists wrapped in Some and None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	/// Also checks the stash as a secondary lookup on primary bucket miss.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());

		// Try stash first - inline the lookup and update to avoid nested borrows
		if let Some(position) = self.stash.iter().position(|s| s.item.key == *key) {
			// Update the access time for LRU purposes
			self.stash[position].last_access_time = Instant::now();
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (from_stash)"
			);
			return Some(&self.stash[position].item.value);
		}

		// Try primary bucket after the stash lookup
		let bucket = &mut self.buckets[bucket_index];
		if let Some(item) = bucket.get(key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true"
			);
			return Some(&item.value);
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
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO)

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn stash_captures_evicted_item() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first to stash

		// Item should be in stash, accessible via get
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert!(cache.stash.len() > 0);
	}

	#[test]
	fn stash_lru_eviction() {
		// Cache with capacity 8 (4 buckets * 2 slots), stash capacity 2 (8 / 4)
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let stash_capacity = cache.stash_capacity;
		assert_eq!(stash_capacity, 2); // 8 / 4 = 2

		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 5);

		// Fill primary bucket
		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);

		// Evict to stash (FIFO evicts first)
		cache.put(same_bucket_keys[2], 30); // key[0] -> stash
		cache.put(same_bucket_keys[3], 40); // key[1] -> stash, stash full at 2

		// Access key[0] to make it more recently used
		let _ = cache.get_and_update_item(&same_bucket_keys[0]);

		// Add another, should evict key[1] from stash (it's the LRU now)
		cache.put(same_bucket_keys[4], 50); // key[1] evicted from stash, key[0] stays

		// Verify key[0] is still in stash, key[1] is not
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert!(cache.get_and_update_item(&same_bucket_keys[1]).is_none());
	}
}
