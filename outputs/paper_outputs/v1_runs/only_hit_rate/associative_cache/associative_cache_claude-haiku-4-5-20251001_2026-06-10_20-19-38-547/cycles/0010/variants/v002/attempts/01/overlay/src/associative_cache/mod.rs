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

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	// Three-tier stash for items evicted from buckets or promoted from lower tiers
	hot_stash: Vec<Item<K, V>>,
	warm_stash: Vec<Item<K, V>>,
	cold_stash: Vec<Item<K, V>>,
	// Capacities for each tier
	hot_stash_capacity: usize,
	warm_stash_capacity: usize,
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
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}

		// Calculate stash capacities as fractions of bucket capacity.
		// Stash tiers provide a tiering layer to reduce hot-tier churn:
		// - Hot stash: 10% of bucket capacity
		// - Warm stash: 10% of bucket capacity
		// - Cold stash: 5% of bucket capacity
		let bucket_capacity = settings.num_of_buckets * settings.bucket_size;
		let hot_stash_capacity = bucket_capacity / 10; // 10% of bucket capacity
		let warm_stash_capacity = bucket_capacity / 10; // 10% of bucket capacity
		let cold_stash_capacity = bucket_capacity / 20; // 5% of bucket capacity

		Some(Self {
			buckets,
			hot_stash: Vec::with_capacity(hot_stash_capacity),
			warm_stash: Vec::with_capacity(warm_stash_capacity),
			cold_stash: Vec::with_capacity(cold_stash_capacity),
			hot_stash_capacity,
			warm_stash_capacity,
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
		let bucket = self.get_mut_bucket(&key);
		let (_inserted, evicted) = bucket.put(Item::new(key, value));

		// If bucket evicted an item, insert it into cold stash
		if let Some(evicted_item) = evicted {
			self.insert_into_cold_stash(evicted_item);
		}
	}

	/// Returns the value of key if exists wrapped in Some, and None otherwise.
	///
	/// As a side effect makes updates according to promotion rules:
	/// - Items accessed in cold tier are promoted to warm
	/// - Items accessed in warm tier are promoted to hot
	/// - Hot tier hits are refreshed with updated timestamps
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		let data = bucket.get(key);
		if data.is_some() {
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true"
			);
			return Some(&data?.value);
		}

		// Miss in bucket; check hot stash
		if let Some(index) = self.find_in_stash(&self.hot_stash, key) {
			let mut item = self.hot_stash.remove(index);
			debug!("AssociativeCache.get_and_update_item: key={key:?} hot_hit=true");
			item.timestamp = Instant::now(); // Update access time
			self.hot_stash.push(item);
			return Some(&self.hot_stash.last().unwrap().value);
		}

		// Miss in hot; check warm stash
		if let Some(index) = self.find_in_stash(&self.warm_stash, key) {
			let item = self.warm_stash.remove(index);
			debug!("AssociativeCache.get_and_update_item: key={key:?} warm_hit=true");
			// Promote from warm to hot
			self.promote_warm_to_hot(item);
			return Some(&self.hot_stash.last().unwrap().value);
		}

		// Miss in warm; check cold stash
		if let Some(index) = self.find_in_stash(&self.cold_stash, key) {
			let item = self.cold_stash.remove(index);
			debug!("AssociativeCache.get_and_update_item: key={key:?} cold_hit=true");
			// Promote from cold to warm
			self.promote_cold_to_warm(item);
			return Some(&self.warm_stash.last().unwrap().value);
		}

		debug!("AssociativeCache.get_and_update_item: key={key:?} hit=false");
		None
	}

	fn get_mut_bucket(&mut self, key: &K) -> &mut AssociativeCacheBucket<K, V> {
		let bucket_index = get_index(key, self.buckets.len());
		&mut self.buckets[bucket_index]
	}

	/// Find a key in a stash and return its index, or None if not found.
	fn find_in_stash(&self, stash: &[Item<K, V>], key: &K) -> Option<usize> {
		stash.iter().position(|item| item.key == *key)
	}

	/// Find the LRU item in a stash by comparing timestamps.
	fn find_lru_index(&self, stash: &[Item<K, V>]) -> Option<usize> {
		if stash.is_empty() {
			return None;
		}
		let (min_idx, _) = stash
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.timestamp)
			.unwrap();
		Some(min_idx)
	}

	/// Insert an item into the cold stash, evicting LRU if full.
	fn insert_into_cold_stash(&mut self, mut item: Item<K, V>) {
		item.timestamp = Instant::now(); // Update timestamp on insertion
		if self.cold_stash.len() < self.cold_stash_capacity {
			self.cold_stash.push(item);
		} else if !self.cold_stash.is_empty() {
			// Cold is full, evict LRU
			if let Some(lru_idx) = self.find_lru_index(&self.cold_stash) {
				self.cold_stash[lru_idx] = item; // Replace LRU with new item
			}
		}
	}

	/// Promote an item from cold to warm tier.
	/// If warm is full, evict LRU warm item to cold.
	fn promote_cold_to_warm(&mut self, mut item: Item<K, V>) {
		item.timestamp = Instant::now(); // Update timestamp on promotion
		if self.warm_stash.len() < self.warm_stash_capacity {
			self.warm_stash.push(item);
		} else if !self.warm_stash.is_empty() {
			// Warm is full, evict LRU warm to cold
			if let Some(lru_idx) = self.find_lru_index(&self.warm_stash) {
				let evicted_warm = self.warm_stash.remove(lru_idx);
				self.warm_stash.push(item);
				// Insert evicted warm item into cold
				self.insert_into_cold_stash(evicted_warm);
			}
		}
	}

	/// Promote an item from warm to hot tier.
	/// If hot is full, evict LRU hot item to warm.
	fn promote_warm_to_hot(&mut self, mut item: Item<K, V>) {
		item.timestamp = Instant::now(); // Update timestamp on promotion
		if self.hot_stash.len() < self.hot_stash_capacity {
			self.hot_stash.push(item);
		} else if !self.hot_stash.is_empty() {
			// Hot is full, evict LRU hot to warm
			if let Some(lru_idx) = self.find_lru_index(&self.hot_stash) {
				let evicted_hot = self.hot_stash.remove(lru_idx);
				self.hot_stash.push(item);
				// Insert evicted hot item into warm
				if self.warm_stash.len() < self.warm_stash_capacity {
					self.warm_stash.push(evicted_hot);
				} else if !self.warm_stash.is_empty() {
					// Warm is also full, evict LRU warm to cold
					if let Some(warm_lru_idx) = self.find_lru_index(&self.warm_stash) {
						let evicted_warm = self.warm_stash.remove(warm_lru_idx);
						self.warm_stash.push(evicted_hot);
						self.insert_into_cold_stash(evicted_warm);
					}
				}
			}
		}
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
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO), moves to cold stash

		// First key should be in cold stash now
		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_some());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}
}
