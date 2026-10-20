use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::{alt_index, get_index};
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// Maximum number of cuckoo-style displacement steps after a primary-bucket
/// eviction. After this many kicks, whatever item is still displaced is dropped.
const MAX_KICKS: usize = 2;

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

	/// Insert a key-value pair into the cache.
	///
	/// New keys always go into their primary bucket. If that evicts an item, the
	/// evicted item is moved (cuckoo-style) into its *other* legal bucket — free
	/// slot wins outright; a full bucket accepts it only if it is more recent
	/// than that bucket's eviction candidate, which is then displaced in turn.
	/// The chain is bounded by `MAX_KICKS`; at most one item leaves the cache.
	pub fn put(&mut self, key: K, value: V) {
		let n = self.buckets.len();
		let p = get_index(&key, n);
		let a = alt_index(&key, n);
		debug!("AssociativeCache.put: key={key:?} primary={p} alt={a}");

		// Existing key in its alternate bucket: update in place, no movement.
		if a != p {
			if let Some(pos) = self.buckets[a].get_position(&key) {
				self.buckets[a].get_items_mut()[pos].value = value;
				self.buckets[a].get_from_position(pos);
				return;
			}
		}

		// Insert (or update in place, if present) in the primary bucket.
		let (_, evicted) = self.buckets[p].put(Item::new(key, value));
		let Some(mut displaced) = evicted else {
			return;
		};

		let mut from = p;
		for _ in 0..MAX_KICKS {
			// Legal buckets are always derived from the key, never from position.
			let dp = get_index(&displaced.key, n);
			let da = alt_index(&displaced.key, n);
			let to = if from == dp { da } else { dp };
			if to == from {
				// Single-bucket cache: no alternate to move into.
				break;
			}
			let target = &mut self.buckets[to];
			if !target.is_full() {
				debug!(
					"AssociativeCache.put: relocated key={:?} {from}->{to} (free slot)",
					displaced.key
				);
				let _ = target.put(displaced);
				return;
			}
			let Some(cand) = target.eviction_candidate_index() else {
				break;
			};
			if displaced.timestamp > target.get_items()[cand].timestamp {
				let victim = target.evict_item().expect("full bucket has an eviction candidate");
				debug!(
					"AssociativeCache.put: relocated key={:?} {from}->{to}, displacing key={:?}",
					displaced.key, victim.key
				);
				let _ = target.put(displaced);
				displaced = victim;
				from = to;
			} else {
				break;
			}
		}
		debug!("AssociativeCache.put: evicted key={:?}", displaced.key);
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// Probes the primary bucket, then the alternate. As a side effect makes
	/// updates according to the eviction policy of the bucket holding the key.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let n = self.buckets.len();
		let p = get_index(key, n);
		if let Some(pos) = self.buckets[p].get_position(key) {
			debug!("AssociativeCache.get_and_update_item: key={key:?} bucket={p} hit=primary");
			return Some(&self.buckets[p].get_from_position(pos).value);
		}
		let a = alt_index(key, n);
		if a != p {
			if let Some(pos) = self.buckets[a].get_position(key) {
				debug!("AssociativeCache.get_and_update_item: key={key:?} bucket={a} hit=alt");
				return Some(&self.buckets[a].get_from_position(pos).value);
			}
		}
		debug!("AssociativeCache.get_and_update_item: key={key:?} miss");
		None
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
		// bucket_size=2, 3 keys share primary bucket 0. The third put evicts the
		// first (FIFO) from bucket 0, but it is relocated into its (empty)
		// alternate bucket instead of being dropped.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let n = cache.buckets.len();
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, n, 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // displaces first into its alternate bucket

		let alt = alt_index(&same_bucket_keys[0], n);
		assert_ne!(alt, target_bucket_idx);
		assert!(cache.buckets[target_bucket_idx].get_position(&same_bucket_keys[0]).is_none());
		assert!(cache.buckets[alt].get_position(&same_bucket_keys[0]).is_some());

		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
		for bucket in &cache.buckets {
			assert!(bucket.size() <= bucket.get_max_size());
		}
	}

	#[test]
	fn keys_live_only_in_legal_buckets_and_once() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU);
		let n = cache.buckets.len();
		for k in 0..200i64 {
			if cache.get_and_update_item(&(k % 37)).is_none() {
				cache.put(k % 37, k);
			}
		}
		let mut seen = std::collections::HashSet::new();
		for (idx, bucket) in cache.buckets.iter().enumerate() {
			assert!(bucket.size() <= bucket.get_max_size());
			for item in bucket.get_items() {
				assert!(idx == get_index(&item.key, n) || idx == alt_index(&item.key, n));
				assert!(seen.insert(item.key), "duplicate key {}", item.key);
			}
		}
	}
}
