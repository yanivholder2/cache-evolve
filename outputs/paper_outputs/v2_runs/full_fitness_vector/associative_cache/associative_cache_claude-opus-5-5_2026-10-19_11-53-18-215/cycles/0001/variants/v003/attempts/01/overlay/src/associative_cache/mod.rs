use crate::shared::item::Item;
use crate::shared::utils::hash;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// H03: paired-bucket associative cache.
///
/// The `n` logical buckets of `bucket_size` slots are grouped into adjacent
/// pairs `(2p, 2p + 1)`. Each pair is stored as one physical bucket
/// (`buckets[p]`, `2 * bucket_size` slots). A key hashes to its physical
/// bucket `p = hash % buckets.len()`, and a hash bit picks its home half. The
/// other half of the pair is its partner. If `n` is odd, the last physical
/// bucket has a single half and behaves like a baseline bucket. Total capacity
/// stays `n * bucket_size`.
///
/// Grouping the pair into one physical bucket means every resident key sits
/// in the bucket its key hashes to, whether it is in its home half or its
/// partner half. The partner probe is a scan of the adjacent range in the same
/// `Vec`, with no second hash.
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
			buckets.push(AssociativeCacheBucket::with_layout(&settings, paired, physical));
		}
		Some(Self { buckets })
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache: home half first, demoting the
	/// evicted native item into the partner half when that is worthwhile.
	pub fn put(&mut self, key: K, value: V) {
		let h = hash(&key);
		let p = h % self.buckets.len();
		debug!("AssociativeCache.put: key={key:?} bucket={p}");
		self.buckets[p].pair_put(Item::new(key, value), h);
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy. A hit
	/// in the partner half promotes the item back to its home half.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let h = hash(key);
		let p = h % self.buckets.len();
		let data = self.buckets[p].pair_get(key, h);
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
		// Three keys in the same physical pair (2 + 2 slots). If all three share
		// a home half, the first is evicted from home but survives as a guest in
		// the empty partner half. So no key is lost.
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
	fn partner_hit_promotes_home() {
		// One pair, half size 1: two keys with the same home half. Inserting the
		// second demotes the first into the partner. A get on the first swaps them.
		let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(AssociativeCacheSettings {
			num_of_buckets: 2,
			bucket_size: 1,
			eviction_policy: crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU,
		});
		let half = |k: &i64| cache_half(k, 1);
		let keys: Vec<i64> = (0..200).filter(|k| half(k) == 0).take(2).collect();
		cache.put(keys[0], 1);
		cache.put(keys[1], 2);
		assert_eq!(cache.buckets[0].get_items().len(), 2);
		assert_eq!(cache.buckets[0].get_items()[1].key, keys[0], "first key demoted to partner");
		assert_eq!(*cache.get_and_update_item(&keys[0]).unwrap(), 1);
		assert_eq!(cache.buckets[0].get_items()[0].key, keys[0], "promoted home");
		assert_eq!(cache.buckets[0].get_items()[1].key, keys[1], "swapped into partner");
	}

	fn cache_half(k: &i64, divisor: usize) -> usize {
		(hash(k) / divisor) & 1
	}
}
