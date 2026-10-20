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
		Some(Self { buckets })
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
		let bucket_id = get_index(&key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_id];
		let (_inserted, evicted) = bucket.put(Item::new(key, value));

		// If an item was evicted, try to place it in neighboring buckets
		if let Some(victim) = evicted {
			self.try_place_victim(victim, bucket_id);
		}
	}

	/// Try to place a victim item in neighboring buckets.
	/// Neighbors are bucket_id ± 1 (mod num_buckets).
	fn try_place_victim(&mut self, victim: Item<K, V>, bucket_id: usize) {
		let num_buckets = self.buckets.len();

		// Calculate neighbor indices
		let left_neighbor = if bucket_id > 0 { bucket_id - 1 } else { num_buckets - 1 };
		let right_neighbor = (bucket_id + 1) % num_buckets;

		// Try neighbors in order
		for neighbor_id in [left_neighbor, right_neighbor] {
			if self.try_place_in_neighbor(victim.clone(), neighbor_id, bucket_id) {
				return;
			}
		}

		// If no neighbor accepted, discard the victim
		debug!(
			"AssociativeCache.try_place_victim: victim key={:?} not placed, discarded",
			victim.key
		);
	}

	/// Try to place the victim in a specific neighbor bucket.
	/// Returns true if placement succeeded, false otherwise.
	fn try_place_in_neighbor(&mut self, victim: Item<K, V>, neighbor_id: usize, primary_bucket_id: usize) -> bool {
		let neighbor = &mut self.buckets[neighbor_id];

		// If neighbor has free space, place the victim there
		if !neighbor.is_full() {
			neighbor.put(victim.clone());
			debug!(
				"AssociativeCache.try_place_in_neighbor: victim key={:?} placed in neighbor bucket {} (had free space)",
				victim.key, neighbor_id
			);
			return true;
		}

		// Neighbor is full. Try to displace its least-accessed item if beneficial.
		// Identify the least-accessed item in the neighbor
		let neighbor_items = neighbor.get_items();
		if neighbor_items.is_empty() {
			return false;
		}

		// Find the least-accessed item: use lfu_counter as primary, timestamp as secondary
		let least_accessed_idx = neighbor_items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| (item.lfu_counter, item.timestamp))
			.map(|(idx, _)| idx)
			.unwrap();

		let neighbor_victim = neighbor_items[least_accessed_idx].clone();

		// Decide if displacement is beneficial:
		// A displacement is beneficial if the victim has a higher access count
		// (or more recent timestamp on tie) than the neighbor's least-accessed item.
		// This ensures we're replacing a less-hot item with a potentially hotter one.
		let victim_is_hotter = victim.lfu_counter > neighbor_victim.lfu_counter
			|| (victim.lfu_counter == neighbor_victim.lfu_counter && victim.timestamp > neighbor_victim.timestamp);

		if victim_is_hotter {
			// Remove the neighbor's victim and place our victim
			neighbor.remove_at(least_accessed_idx);
			neighbor.put(victim.clone());

			debug!(
				"AssociativeCache.try_place_in_neighbor: victim key={:?} displaced neighbor's key={:?} in bucket {}",
				victim.key, neighbor_victim.key, neighbor_id
			);

			// Now try to place the neighbor's victim in the primary bucket or discard
			// For simplicity, we discard the displaced neighbor item rather than cascading
			debug!(
				"AssociativeCache.try_place_in_neighbor: neighbor's displaced key={:?} discarded",
				neighbor_victim.key
			);
			return true;
		}

		// Victim is not hotter, don't displace
		debug!(
			"AssociativeCache.try_place_in_neighbor: victim key={:?} not hotter than neighbor's least-accessed, not displacing",
			victim.key
		);
		false
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
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
		Some(&data?.value)
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
