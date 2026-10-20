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

/// Number of saturating frequency counters in the admission table. Keys are
/// hashed into this table; the table is shared by incoming and resident keys so
/// both estimates sit on the same scale.
const FREQ_SLOTS: usize = 4096;
/// Counters saturate at 4 bits.
const FREQ_MAX: u8 = 15;
/// Every `FREQ_SAMPLE` inserts all counters are halved, so stale popularity decays.
const FREQ_SAMPLE: usize = 10 * FREQ_SLOTS;

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Admission frequency table: one small saturating counter per key hash.
	freq: Vec<u8>,
	/// Inserts seen since the last halving of `freq`.
	inserts_since_halving: usize,
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
			freq: vec![0; FREQ_SLOTS],
			inserts_since_halving: 0,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache.
	///
	/// Admission gate: when the target bucket is full and the key is not already
	/// resident, the incoming key's access frequency must be strictly greater than
	/// the eviction victim's; otherwise the insert is skipped and the victim stays.
	/// Inserts into a bucket with a free slot are never gated.
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);
		self.inserts_since_halving += 1;
		if self.inserts_since_halving >= FREQ_SAMPLE {
			for counter in self.freq.iter_mut() {
				*counter >>= 1;
			}
			self.inserts_since_halving = 0;
		}

		let bucket_index = get_index(&key, self.buckets.len());
		{
			let bucket = &self.buckets[bucket_index];
			if bucket.is_full() && bucket.get_position(&key).is_none() {
				if let Some(victim_index) = bucket.eviction_candidate_index() {
					let victim_key = bucket.get_items()[victim_index].key;
					if self.freq_of(&key) <= self.freq_of(&victim_key) {
						return;
					}
				}
			}
		}

		let bucket = self.get_mut_bucket(&key);
		bucket.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		self.record_access(key);
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

	/// Count one access (hit or miss) for `key` in the admission table.
	fn record_access(&mut self, key: &K) {
		let slot = get_index(key, FREQ_SLOTS);
		let counter = &mut self.freq[slot];
		if *counter < FREQ_MAX {
			*counter += 1;
		}
	}

	/// Current admission frequency estimate for `key`.
	fn freq_of(&self, key: &K) -> u8 {
		self.freq[get_index(key, FREQ_SLOTS)]
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
		// bucket_size=2, find 3 keys that hash to same bucket. The newcomer must be
		// accessed more often than the victim (the admission gate is strict), so it is
		// touched twice before insertion; the first key was never accessed, so FIFO's
		// victim is admitted over.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.get_and_update_item(&same_bucket_keys[2]);
		cache.get_and_update_item(&same_bucket_keys[2]);
		cache.put(same_bucket_keys[2], 30); // admitted (freq 2 > 0), evicts first (FIFO)

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn admission_rejects_colder_newcomer() {
		// A never-accessed newcomer (freq 0) must not displace a resident with freq >= 0.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let keys = find_keys_for_bucket(0, cache.buckets.len(), 3);

		cache.put(keys[0], 10);
		cache.put(keys[1], 20);
		cache.put(keys[2], 30); // rejected: freq(keys[2]) == 0 is not > freq(keys[0]) == 0

		assert_eq!(*cache.get_and_update_item(&keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&keys[1]).unwrap(), 20);
		assert!(cache.get_and_update_item(&keys[2]).is_none());
	}
}
