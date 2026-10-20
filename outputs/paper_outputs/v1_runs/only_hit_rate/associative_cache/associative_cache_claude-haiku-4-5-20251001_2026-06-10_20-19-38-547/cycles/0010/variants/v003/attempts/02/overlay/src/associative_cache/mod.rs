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
	hot_stash: Vec<Item<K, V>>,
	cold_stash: Vec<Item<K, V>>,
	hot_stash_capacity: usize,
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

		let total_capacity = settings.capacity();
		let hot_stash_capacity = (total_capacity as f64 * 0.20).ceil() as usize;
		let cold_stash_capacity = (total_capacity as f64 * 0.05).ceil() as usize;

		Some(Self {
			buckets,
			hot_stash: Vec::with_capacity(hot_stash_capacity),
			cold_stash: Vec::with_capacity(cold_stash_capacity),
			hot_stash_capacity,
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
		let item = Item::new(key, value);
		let (_inserted, evicted) = bucket.put(item);

		// If an item was evicted from the bucket, move it to cold stash
		if let Some(mut evicted_item) = evicted {
			evicted_item.cold_access_count = 0;
			evicted_item.timestamp = Instant::now();

			if self.cold_stash.len() < self.cold_stash_capacity {
				self.cold_stash.push(evicted_item);
			} else if self.cold_stash.len() > 0 {
				// Cold stash is full, evict FIFO (oldest item at front)
				self.cold_stash.remove(0);
				self.cold_stash.push(evicted_item);
			}
		}
	}

	/// Returns the value of key if exists wrapped in Some and None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		// First, try to get from bucket
		let bucket_index = get_index(key, self.buckets.len());
		{
			let bucket = &mut self.buckets[bucket_index];
			if let Some(item) = bucket.get(key) {
				debug!(
					"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit=true"
				);
				return Some(&item.value);
			}
		}

		// Try to get from hot stash (use indexed loop to avoid borrow checker issues)
		for i in 0..self.hot_stash.len() {
			if self.hot_stash[i].key == *key {
				debug!(
					"AssociativeCache.get_and_update_item: key={key:?} hot_hit=true"
				);
				return Some(&self.hot_stash[i].value);
			}
		}

		// Try to get from cold stash
		let cold_hit_index: Option<usize> = {
			let mut idx: Option<usize> = None;
			for i in 0..self.cold_stash.len() {
				if self.cold_stash[i].key == *key {
					idx = Some(i);
					break;
				}
			}
			idx
		};

		if let Some(i) = cold_hit_index {
			// Increment cold access count
			self.cold_stash[i].cold_access_count += 1;

			// Check if we should promote this item to hot stash
			if self.hot_stash.len() < self.hot_stash_capacity {
				// Hot stash has capacity, promote this item
				let mut promoted_item = self.cold_stash.remove(i);
				promoted_item.timestamp = Instant::now();
				debug!(
					"AssociativeCache.get_and_update_item: key={key:?} cold_hit=true promoted=true"
				);
				self.hot_stash.push(promoted_item);
				return Some(&self.hot_stash.last().unwrap().value);
			} else if self.hot_stash.len() > 0 {
				// Hot stash is full; scan cold stash for best candidate to promote
				// Find item in cold with highest promotion_score (only if cold_access_count > 0)
				let mut best_cold_index: Option<usize> = None;
				let mut best_promotion_score = -1.0;

				for cold_idx in 0..self.cold_stash.len() {
					let cold_item = &self.cold_stash[cold_idx];
					if cold_item.cold_access_count > 0 {
						let time_since_eviction_ms = cold_item.timestamp.elapsed().as_millis() as f64;
						let score = cold_item.cold_access_count as f64 / (1.0 + time_since_eviction_ms / 1000.0);
						if score > best_promotion_score {
							best_promotion_score = score;
							best_cold_index = Some(cold_idx);
						}
					}
				}

				if let Some(best_idx) = best_cold_index {
					let promoted_key = self.cold_stash[best_idx].key;
					let is_requested_key = promoted_key == *key;

					// Find item in hot stash with lowest timestamp (LRU)
					let mut lru_index = 0;
					for idx in 0..self.hot_stash.len() {
						if self.hot_stash[idx].timestamp < self.hot_stash[lru_index].timestamp {
							lru_index = idx;
						}
					}

					let mut evicted_from_hot = self.hot_stash.remove(lru_index);
					let mut promoted_item = self.cold_stash.remove(best_idx);
					promoted_item.timestamp = Instant::now();
					promoted_item.cold_access_count = 0;

					debug!(
						"AssociativeCache.get_and_update_item: key={key:?} cold_hit=true promoted=true best_score={best_promotion_score}"
					);

					self.hot_stash.push(promoted_item);
					// Evicted item from hot goes to cold if there's space
					if self.cold_stash.len() < self.cold_stash_capacity {
						evicted_from_hot.cold_access_count = 0;
						self.cold_stash.push(evicted_from_hot);
					}

					// Return the value for the requested key
					if is_requested_key {
						return Some(&self.hot_stash.last().unwrap().value);
					} else {
						// Item is still in cold, find and return it
						for item in &self.cold_stash {
							if item.key == *key {
								return Some(&item.value);
							}
						}
					}
				}
			}

			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} cold_hit=true promoted=false"
			);
			return Some(&self.cold_stash[i].value);
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
}
