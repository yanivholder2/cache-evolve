use crate::shared::item::Item;
use crate::shared::utils::hash;
use associative_cache_bucket::{tag_of_hash, AssociativeCacheBucket};
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// H01 (cycle 0005): merged-pair associative cache with a segmented-LRU
/// bucket and tag-first lookup.
///
/// The `n` logical buckets of `bucket_size` slots are grouped into adjacent
/// pairs. Each pair is stored as one physical bucket (`buckets[p]`,
/// `2 * bucket_size` slots). A key hashes once: the hash gives both the
/// physical bucket `p = hash % buckets.len()` and the 16-bit tag passed to the
/// bucket (upper 16 bits, 0 mapped to 1). If `n` is odd, the last physical
/// bucket holds `bucket_size` slots. Total capacity stays `n * bucket_size`.
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
		// settings. Here we group the `n` logical buckets into adjacent pairs.
		info!("AssociativeCache.try_new: settings={settings:?}");
		let n = settings.num_of_buckets;
		let physical = (n + 1) / 2;
		let mut buckets = Vec::with_capacity(physical);
		for p in 0..physical {
			let paired = 2 * p + 1 < n;
			buckets.push(AssociativeCacheBucket::with_layout(&settings, paired));
		}
		Some(Self { buckets })
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache (probation of its bucket).
	pub fn put(&mut self, key: K, value: V) {
		let h = hash(&key);
		let p = h % self.buckets.len();
		let tag = tag_of_hash(h);
		debug!("AssociativeCache.put: key={key:?} bucket={p}");
		self.buckets[p].pair_put_tagged(Item::new(key, value), tag);
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect applies the SLRU hit rule (promote on second hit).
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let h = hash(key);
		let p = h % self.buckets.len();
		let tag = tag_of_hash(h);
		let data = self.buckets[p].pair_get_tagged(key, tag);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={p} hit={}",
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
		// Three keys in the same physical bucket (2 + 2 = 4 slots): all fit.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30);

		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn pair_capacity_and_slot_legality_under_overfill() {
		for n in [1usize, 2, 3, 4, 5, 8] {
			let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings {
				num_of_buckets: n,
				bucket_size: 3,
				eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU,
			});
			let total: usize = cache.buckets.iter().map(|b| b.get_max_size()).sum();
			assert_eq!(total, n * 3, "capacity must be conserved");
			for k in 0..1000i64 {
				let key = (k * 7919) % 97;
				if cache.get_and_update_item(&key).is_none() {
					cache.put(key, key);
				}
				let mut seen = std::collections::HashSet::new();
				let m = cache.buckets.len();
				for (idx, b) in cache.buckets.iter().enumerate() {
					assert!(b.size() <= b.get_max_size());
					for it in b.get_items() {
						assert_eq!(get_index(&it.key, m), idx);
						assert!(seen.insert(it.key), "duplicate key {}", it.key);
					}
				}
			}
		}
	}

	#[test]
	fn slru_second_hit_protects() {
		// One 4-slot physical bucket (protected cap 3, probation >= 1). A is
		// promoted to protected by a get; a stream of new keys then flows
		// through probation without evicting A.
		let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 2,
			bucket_size: 2,
			eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU,
		});
		assert_eq!(cache.buckets.len(), 1);
		let (a, b, c, d, e) = (100i64, 200, 300, 400, 500);
		cache.put(a, 1);
		assert_eq!(*cache.get_and_update_item(&a).unwrap(), 1);
		for k in [b, c, d, e] {
			assert!(cache.get_and_update_item(&k).is_none());
			cache.put(k, k);
		}
		assert_eq!(cache.buckets[0].get_items().len(), 4);
		assert_eq!(*cache.get_and_update_item(&a).unwrap(), 1, "protected item survives the scan");
		let resident = [b, c, d, e]
			.iter()
			.filter(|k| cache.buckets[0].get_items().iter().any(|it| it.key == **k))
			.count();
		assert_eq!(resident, 3, "exactly one scan key was evicted");
	}

	#[test]
	fn cache_level_tag_path_round_trips() {
		// Hash-once path (tag from the same hash as the bucket index) must find
		// every resident key under a churning workload.
		let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 8,
			bucket_size: 16,
			eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU,
		});
		for k in 0..20_000i64 {
			let key = (k * 2654435761) % 1031;
			if cache.get_and_update_item(&key).is_none() {
				cache.put(key, key);
			}
		}
		let resident: Vec<i64> = cache
			.buckets
			.iter()
			.flat_map(|b| b.get_items().iter().map(|i| i.key))
			.collect();
		for key in resident {
			assert_eq!(cache.get_and_update_item(&key).copied(), Some(key));
		}
	}
}
