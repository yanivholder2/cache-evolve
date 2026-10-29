//! Associative (set-associative) cache.
//!
//! Variant notes (H03, one hash per op):
//! - Before this change the cache picked a bucket with
//!   `shared::utils::get_index`, i.e. `std::collections::hash_map::DefaultHasher`
//!   (SipHash-1-3) `% num_of_buckets`. Each ClassicLRU bucket then hashed the
//!   key a second time with its multiplicative `TagHasher` for the 1-byte tag.
//!   So every get/put paid for a full SipHash on top of the tag hash, and the
//!   SipHash was the main cost this change removes.
//! - Now each op computes `h = tag_hash(key)` once. The bucket index comes
//!   from bits 24..=55 of `h` (a mask for power-of-two bucket counts, `%`
//!   otherwise; see `utils::bucket_index_from_hash`), and the tag comes from
//!   bits 56..=63. `get_index` uses the same mapping, so slot legality is
//!   defined by one function.
//!
//! Variant notes (cycle 10 H01): ClassicLRU buckets also skip promotion on a
//! hit within the last `len/8 + 1` slots (see `LAZY_PROMOTE_SHIFT` in
//! `associative_cache_bucket`).

use crate::shared::item::Item;
use crate::shared::utils::bucket_index_from_hash;
use associative_cache_bucket::{tag_from_hash, tag_hash, AssociativeCacheBucket};
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
		let h = tag_hash(&key);
		let bucket_index = bucket_index_from_hash(h, self.buckets.len());
		debug!("AssociativeCache.put: key={key:?} bucket={bucket_index}");
		self.buckets[bucket_index].put_hashed(Item::new(key, value), tag_from_hash(h));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let h = tag_hash(key);
		let bucket_index = bucket_index_from_hash(h, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		let data = bucket.get_hashed(key, tag_from_hash(h));
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
	use crate::shared::traits::bucket::Bucket;
	use crate::shared::utils::get_index;
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
		// Use keys known to land in distinct-enough buckets: one per bucket.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let keys: Vec<i64> = (0..4).map(|b| find_keys_for_bucket(b, 4, 1)[0]).collect();
		for &k in &keys {
			cache.put(k, k * 10);
		}
		for &k in &keys {
			assert_eq!(*cache.get_and_update_item(&k).unwrap(), k * 10);
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

	/// Items land in the bucket `get_index` names (slot legality), for both a
	/// power-of-two and a non-power-of-two bucket count.
	#[test]
	fn placement_matches_get_index() {
		for &n in &[4usize, 7, 128] {
			let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings {
				num_of_buckets: n,
				bucket_size: 3,
				eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU,
			});
			for k in 0..5_000i64 {
				let key = (k * 37) % 1_500;
				if cache.get_and_update_item(&key).is_none() {
					cache.put(key, key);
				}
			}
			for (idx, b) in cache.buckets.iter().enumerate() {
				for it in b.get_items() {
					assert_eq!(get_index(&it.key, n), idx);
				}
			}
		}
	}

	/// `tag_hash`-based selection spreads sequential and strided keys (stride 8
	/// and 4096, like MSR block addresses) evenly across 64 and 1024 buckets.
	#[test]
	fn tag_hash_selection_spreads_keys() {
		const N: i64 = 1 << 20;
		for &stride in &[1i64, 8, 4096] {
			for &buckets in &[64usize, 1024] {
				let mut occ = vec![0u64; buckets];
				for i in 0..N {
					let key = i * stride;
					occ[bucket_index_from_hash(tag_hash(&key), buckets)] += 1;
				}
				let max = *occ.iter().max().unwrap();
				let min = *occ.iter().min().unwrap();
				assert!(min > 0, "stride={stride} buckets={buckets}: empty bucket");
				let ratio = max as f64 / min as f64;
				assert!(ratio < 1.5, "stride={stride} buckets={buckets}: max/min={ratio} (max={max} min={min})");
			}
		}
	}
}
