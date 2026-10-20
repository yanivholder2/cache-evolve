use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::{get_index, hash};
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// Doorkeeper size in 64-bit words. Must be a power of two so the bit index is a mask.
const DOORKEEPER_WORDS: usize = 64;
/// Doorkeeper size in bits (4096 bits = 512 bytes, held inline on the cache).
const DOORKEEPER_BITS: usize = DOORKEEPER_WORDS * 64;

/// Bit index for a key in the doorkeeper. Uses the high half of the key hash so it
/// is decorrelated from the low bits that select the bucket.
fn doorkeeper_bit<K>(key: &K) -> usize
where
	K: Hash + Debug,
{
	(hash(key) >> 32) & (DOORKEEPER_BITS - 1)
}

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Admission doorkeeper: a key entering a full bucket is admitted only on its
	/// second sighting. Inline fixed-size bitset, no heap allocation.
	doorkeeper: [u64; DOORKEEPER_WORDS],
	/// Next doorkeeper word to clear. Advances one word per put, so a full sweep
	/// of the bitset completes over `DOORKEEPER_WORDS` puts without a bulk clear.
	clear_cursor: usize,
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
		Some(Self {
			buckets,
			doorkeeper: [0; DOORKEEPER_WORDS],
			clear_cursor: 0,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache.
	///
	/// When the target bucket is full and the key is not resident, the insert only
	/// proceeds if the key's doorkeeper bit was already set (second sighting). A
	/// first sighting sets the bit and skips the victim scan entirely.
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);
		// Clear one doorkeeper word per put, advancing the cursor.
		self.doorkeeper[self.clear_cursor] = 0;
		self.clear_cursor = (self.clear_cursor + 1) & (DOORKEEPER_WORDS - 1);

		let bit = doorkeeper_bit(&key);
		let bucket_index = get_index(&key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		if bucket.is_full() && bucket.get_position(&key).is_none() {
			let word = bit / 64;
			let mask = 1u64 << (bit % 64);
			if self.doorkeeper[word] & mask == 0 {
				self.doorkeeper[word] |= mask;
				return;
			}
		}
		bucket.put(Item::new(key, value));
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
	fn doorkeeper_defers_first_sighting_into_full_bucket() {
		// bucket_size=2: fill a bucket, then a first-sighting key must not evict.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // first sighting: rejected, no victim scan

		assert!(cache.get_and_update_item(&same_bucket_keys[2]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
	}
}
