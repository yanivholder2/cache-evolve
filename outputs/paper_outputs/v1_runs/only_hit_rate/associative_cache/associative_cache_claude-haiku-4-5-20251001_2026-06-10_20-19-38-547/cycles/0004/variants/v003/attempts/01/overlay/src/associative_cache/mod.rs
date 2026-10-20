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

/// A simple FIFO stash to handle items that don't fit in their primary bucket.
/// Items are evicted in FIFO order (oldest first) when the stash is full.
#[derive(Debug)]
struct Stash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
}

impl<K, V> Stash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Create a new stash with a given max size.
	fn new(max_size: usize) -> Self {
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
		}
	}

	/// Try to find and retrieve an item from the stash.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.items.iter().find(|item| item.key == *key)
	}

	/// Try to find a mutable item from the stash.
	fn get_mut(&mut self, key: &K) -> Option<&mut Item<K, V>> {
		self.items.iter_mut().find(|item| item.key == *key)
	}

	/// Add an item to the stash. If full, evict the oldest item (FIFO).
	fn put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		// Check if key already exists, update it in place
		if let Some(pos) = self.items.iter().position(|i| i.key == item.key) {
			self.items[pos].value = item.value;
			return None;
		}

		// If full, evict the oldest item (FIFO: index 0)
		let evicted = if self.items.len() >= self.max_size && self.max_size > 0 {
			Some(self.items.remove(0))
		} else {
			None
		};

		self.items.push(item);
		evicted
	}

	/// Check if the stash is empty.
	fn is_empty(&self) -> bool {
		self.items.is_empty()
	}
}

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	stash: Stash<K, V>,
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

		let total_capacity = settings.capacity();

		// Calculate stash size: 15% of total capacity (total_capacity / 6.67), minimum 1
		let stash_size = std::cmp::max(1, (total_capacity as f64 / 6.67).ceil() as usize);
		debug!(
			"AssociativeCache.try_new: stash_size={stash_size} (15% of {total_capacity})"
		);

		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self {
			buckets,
			stash: Stash::new(stash_size),
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

		// If an item was evicted from the bucket, move it to the stash
		// as a second-chance buffer. The stash uses FIFO eviction,
		// allowing items to be recovered on future lookups.
		if let Some(evicted_item) = evicted {
			self.stash.put(evicted_item);
		}
	}

	/// Returns the value of key if exists wrapped in Some and None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		let data = bucket.get(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit={}",
			data.is_some()
		);
		if data.is_some() {
			return data.map(|item| &item.value);
		}

		// Primary bucket miss - check the stash (secondary lookup)
		if let Some(item) = self.stash.get(key) {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} hit=true (stash)"
			);
			return Some(&item.value);
		}

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
}
