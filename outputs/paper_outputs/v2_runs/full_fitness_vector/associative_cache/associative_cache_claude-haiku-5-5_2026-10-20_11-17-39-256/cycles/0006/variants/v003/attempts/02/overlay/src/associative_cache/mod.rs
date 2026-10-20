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

/// Number of bits in the doorkeeper admission bitset (held as `DOORKEEPER_WORDS` u64 words).
const DOORKEEPER_BITS: usize = 4096;
const DOORKEEPER_WORDS: usize = DOORKEEPER_BITS / 64;

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	/// Doorkeeper: a fixed-size bitset recording first sightings of keys that would
	/// displace a resident item.
	doorkeeper: Box<[u64]>,
	/// Gated put misses seen since the doorkeeper was last cleared.
	put_misses_since_reset: usize,
	/// Number of gated put misses after which the doorkeeper is cleared (the slot count).
	doorkeeper_reset_interval: usize,
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
			doorkeeper: vec![0u64; DOORKEEPER_WORDS].into_boxed_slice(),
			put_misses_since_reset: 0,
			doorkeeper_reset_interval: settings.capacity().max(1),
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache.
	///
	/// The doorkeeper only gates puts that would displace a resident item: when the
	/// target bucket has a free slot the insert always proceeds, so an empty or
	/// under-filled cache round-trips every put. When the bucket is full and the key
	/// is not resident, a first sighting only sets its doorkeeper bit and is dropped,
	/// so one-shot keys never reach the victim scan. A repeat sighting (bit already
	/// set) proceeds with the baseline insert path. Resident keys are updated in place.
	pub fn put(&mut self, key: K, value: V) {
		let key_hash = hash(&key);
		let bucket_index = key_hash % self.buckets.len();
		debug!("AssociativeCache.put: key={key:?} bucket={bucket_index}");
		let (resident, full) = {
			let bucket = &self.buckets[bucket_index];
			(bucket.get_position(&key).is_some(), bucket.is_full())
		};
		if full && !resident && !self.doorkeeper_admit(key_hash) {
			return;
		}
		self.buckets[bucket_index].put(Item::new(key, value));
	}

	/// Records a gated put miss in the doorkeeper. Returns `true` if the key was already
	/// sighted (its bit was set) and the insert should proceed.
	fn doorkeeper_admit(&mut self, key_hash: usize) -> bool {
		// Bit index comes from the high bits of the hash, independent of the set index.
		let bit = key_hash.rotate_right(32) % DOORKEEPER_BITS;
		let word = bit / 64;
		let mask = 1u64 << (bit % 64);
		let seen = self.doorkeeper[word] & mask != 0;
		self.doorkeeper[word] |= mask;

		self.put_misses_since_reset += 1;
		if self.put_misses_since_reset >= self.doorkeeper_reset_interval {
			self.doorkeeper.iter_mut().for_each(|w| *w = 0);
			self.put_misses_since_reset = 0;
		}
		seen
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
		cache.put(same_bucket_keys[2], 30); // first sighting on a full bucket: doorkeeper drops it
		cache.put(same_bucket_keys[2], 30); // second sighting admitted: evicts first (FIFO)

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}
}
