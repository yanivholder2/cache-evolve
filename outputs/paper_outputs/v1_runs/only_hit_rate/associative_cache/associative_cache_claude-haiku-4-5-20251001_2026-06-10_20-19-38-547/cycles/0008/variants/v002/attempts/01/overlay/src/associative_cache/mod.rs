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
	/// Hot tier stash: holds items that have re-accessed multiple times (approximately 8% of total cache capacity)
	hot_stash: Vec<Item<K, V>>,
	/// Maximum capacity of the hot stash tier
	hot_stash_capacity: usize,
	/// Warm tier stash: holds items that have re-accessed once (approximately 9% of total cache capacity)
	warm_stash: Vec<Item<K, V>>,
	/// Maximum capacity of the warm stash tier
	warm_stash_capacity: usize,
	/// Cold tier stash: holds newly evicted items (approximately 8% of total cache capacity)
	cold_stash: Vec<Item<K, V>>,
	/// Maximum capacity of the cold stash tier
	cold_stash_capacity: usize,
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

		// Calculate stash capacities: partition 25% total stash into cold (8%), warm (9%), hot (8%)
		let total_capacity = settings.num_of_buckets * settings.bucket_size;
		let cold_stash_capacity = std::cmp::max(1, total_capacity * 8 / 100);
		let warm_stash_capacity = std::cmp::max(1, total_capacity * 9 / 100);
		let hot_stash_capacity = std::cmp::max(1, total_capacity * 8 / 100);

		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self {
			buckets,
			hot_stash: Vec::with_capacity(hot_stash_capacity),
			hot_stash_capacity,
			warm_stash: Vec::with_capacity(warm_stash_capacity),
			warm_stash_capacity,
			cold_stash: Vec::with_capacity(cold_stash_capacity),
			cold_stash_capacity,
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

		// Remove any stale version of this key from all stash tiers
		// (to avoid duplicate keys if the same key is put again after being evicted)
		self.hot_stash.retain(|item| item.key != key);
		self.warm_stash.retain(|item| item.key != key);
		self.cold_stash.retain(|item| item.key != key);

		let bucket = self.get_mut_bucket(&key);
		let (_inserted, evicted) = bucket.put(Item::new(key, value));

		// Capture evicted item in the cold stash tier
		if let Some(evicted_item) = evicted {
			self.add_to_cold_stash(evicted_item);
		}
	}

	/// Add an item to the cold stash tier using FIFO logic
	/// If the cold stash is at capacity, remove the oldest item to make room
	fn add_to_cold_stash(&mut self, item: Item<K, V>) {
		debug!("AssociativeCache.add_to_cold_stash: key={:?}", item.key);

		// If cold stash is at capacity, remove the oldest item (FIFO)
		if self.cold_stash.len() >= self.cold_stash_capacity {
			self.cold_stash.remove(0); // Remove the oldest item (first in, first out)
		}

		self.cold_stash.push(item);
	}

	/// Promote an item from cold stash to warm stash
	/// If warm stash is at capacity, evict the oldest item from warm stash using FIFO
	fn promote_cold_to_warm(&mut self, cold_item: Item<K, V>, cold_pos: usize) {
		debug!("AssociativeCache.promote_cold_to_warm: key={:?}", cold_item.key);

		// Remove from cold stash
		self.cold_stash.remove(cold_pos);

		// If warm stash is at capacity, remove the oldest item (FIFO) to make room
		if self.warm_stash.len() >= self.warm_stash_capacity {
			self.warm_stash.remove(0);
		}

		// Add to warm stash
		self.warm_stash.push(cold_item);
	}

	/// Promote an item from warm stash to hot stash
	/// If hot stash is at capacity, evict the oldest item from hot stash using FIFO
	fn promote_warm_to_hot(&mut self, warm_item: Item<K, V>, warm_pos: usize) {
		debug!("AssociativeCache.promote_warm_to_hot: key={:?}", warm_item.key);

		// Remove from warm stash
		self.warm_stash.remove(warm_pos);

		// If hot stash is at capacity, remove the oldest item (FIFO) to make room
		if self.hot_stash.len() >= self.hot_stash_capacity {
			self.hot_stash.remove(0);
		}

		// Add to hot stash
		self.hot_stash.push(warm_item);
	}

	/// Returns the value of key if exists wrapped in Some, None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	/// Performs secondary lookup in the stash (hot, warm, then cold) on primary bucket miss.
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

		// Secondary lookup: check hot stash first
		if let Some(hot_pos) = self.hot_stash.iter().position(|item| item.key == *key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (hot_stash)"
			);
			// Item is already in hot tier, return as-is
			return Some(&self.hot_stash[hot_pos].value);
		}

		// Secondary lookup: check warm stash second
		if let Some(warm_pos) = self.warm_stash.iter().position(|item| item.key == *key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (warm_stash)"
			);
			let warm_item = self.warm_stash[warm_pos].clone();

			// Warm hit: promote to hot tier (this is the second re-access)
			self.promote_warm_to_hot(warm_item, warm_pos);

			// Return reference to the item now in hot stash (it was just promoted)
			return Some(&self.hot_stash.last().unwrap().value);
		}

		// Secondary lookup: check cold stash
		if let Some(cold_pos) = self.cold_stash.iter().position(|item| item.key == *key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true (cold_stash)"
			);
			let cold_item = self.cold_stash[cold_pos].clone();

			// Cold hit: promote to warm tier (first re-access)
			self.promote_cold_to_warm(cold_item, cold_pos);

			// Return reference to the item now in warm stash (it was just promoted)
			return Some(&self.warm_stash.last().unwrap().value);
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
		// With stash enabled, the evicted item is now recoverable from the cold stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO)

		// With stash, the evicted item can be recovered from the stash
		// (This is the key improvement - items are not lost)
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn stash_captures_evicted_item() {
		// Verify that evicted items are captured in the cold stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO)

		// The first key should be in the cold stash now
		assert!(cache.cold_stash.len() > 0);
		assert_eq!(cache.cold_stash[0].key, same_bucket_keys[0]);
		assert_eq!(cache.cold_stash[0].value, 10);
	}

	#[test]
	fn stash_provides_second_chance() {
		// Verify that items evicted from bucket can be recovered from stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0]

		// Direct bucket lookup should fail
		let bucket = &mut cache.buckets[target_bucket_idx];
		assert!(bucket.get(&same_bucket_keys[0]).is_none());

		// But we should still be able to get it from the stash via get_and_update_item
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
	}

	#[test]
	fn promotion_cold_to_warm() {
		// Verify that accessing an item in cold stash promotes it to warm stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0] to cold stash

		// Verify it's in cold stash
		assert_eq!(cache.cold_stash.len(), 1);
		assert_eq!(cache.warm_stash.len(), 0);
		assert_eq!(cache.hot_stash.len(), 0);

		// Access the item - first access should promote cold to warm
		let val = cache.get_and_update_item(&same_bucket_keys[0]);
		assert_eq!(*val.unwrap(), 10);

		// After access, it should be promoted to warm stash
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 1);
		assert_eq!(cache.hot_stash.len(), 0);
	}

	#[test]
	fn promotion_warm_to_hot() {
		// Verify that accessing an item in warm stash promotes it to hot stash
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0] to cold stash

		// First access: cold to warm
		cache.get_and_update_item(&same_bucket_keys[0]);
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 1);
		assert_eq!(cache.hot_stash.len(), 0);

		// Second access: warm to hot
		let val = cache.get_and_update_item(&same_bucket_keys[0]);
		assert_eq!(*val.unwrap(), 10);
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 0);
		assert_eq!(cache.hot_stash.len(), 1);
	}

	#[test]
	fn stash_cold_fifo_eviction() {
		// Verify that cold stash uses FIFO eviction when at capacity
		let mut cache = AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 2,
			bucket_size: 1, // Very small buckets to force evictions
			eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::Fifo,
		});

		// Put keys to same bucket to cause evictions
		let keys = find_keys_for_bucket(0, 2, 4);
		for (i, key) in keys.iter().enumerate() {
			cache.put(*key, (i as i64) * 10);
		}

		// Most should be evicted from the buckets, but cold stash capacity is limited
		// (2 * 1) * 8 / 100 = 0 but max 1, so at most 1 item in cold
		assert!(cache.cold_stash.len() <= 1);
	}

	#[test]
	fn three_tier_recovery_sequence() {
		// Comprehensive test of the three-tier recovery system
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0]

		// Initially: cold has the evicted item
		assert_eq!(cache.cold_stash.len(), 1);
		assert_eq!(cache.warm_stash.len(), 0);
		assert_eq!(cache.hot_stash.len(), 0);

		// First re-access: cold → warm
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 1);
		assert_eq!(cache.hot_stash.len(), 0);

		// Second re-access: warm → hot
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 0);
		assert_eq!(cache.hot_stash.len(), 1);

		// Third access: stays in hot
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(cache.cold_stash.len(), 0);
		assert_eq!(cache.warm_stash.len(), 0);
		assert_eq!(cache.hot_stash.len(), 1);
	}

	#[test]
	fn transient_items_not_promoted() {
		// Items that are evicted once but not re-accessed don't consume hot-tier space
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // evicts same_bucket_keys[0] to cold

		// Don't access same_bucket_keys[0] again
		// Instead, evict it from cold by filling cold stash with other items
		let other_keys = find_keys_for_bucket(1, cache.buckets.len(), 5);
		for (i, key) in other_keys.iter().enumerate() {
			cache.put(*key, (i as i64) * 100);
		}

		// The hot stash should remain empty because transient items
		// (like same_bucket_keys[0]) are dropped from cold without promotion
		assert_eq!(cache.hot_stash.len(), 0);
	}
}
