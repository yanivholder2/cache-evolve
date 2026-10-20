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

/// Upper bound on cuckoo-style displacement steps per overflowing put.
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

	/// Returns the index of the bucket currently holding `key` (its primary or
	/// its alternate), or `None` if the key is not resident.
	fn locate(&self, key: &K) -> Option<(usize, usize)> {
		let n = self.buckets.len();
		let p = get_index(key, n);
		if let Some(pos) = self.buckets[p].get_position(key) {
			return Some((p, pos));
		}
		let a = alt_index(key, n);
		if a != p {
			if let Some(pos) = self.buckets[a].get_position(key) {
				return Some((a, pos));
			}
		}
		None
	}

	/// Insert a key-value pair into the cache.
	///
	/// New items always land in their primary bucket. An item evicted from a
	/// bucket is offered to its other legal bucket (cuckoo-style) and survives
	/// there if that bucket has room or holds an older eviction candidate; the
	/// chain is bounded by `MAX_KICKS`.
	pub fn put(&mut self, key: K, value: V) {
		let n = self.buckets.len();
		let p = get_index(&key, n);
		debug!("AssociativeCache.put: key={key:?} bucket={p}");

		// Existing key (in primary or alternate): update in place, no movement.
		if let Some((idx, _)) = self.locate(&key) {
			self.buckets[idx].put(Item::new(key, value));
			return;
		}

		let (_, evicted) = self.buckets[p].put(Item::new(key, value));
		let mut displaced = evicted;
		let mut from = p;

		for _ in 0..MAX_KICKS {
			let Some(e) = displaced.take() else {
				return;
			};
			// Legal buckets are derived from the key, never from where it sat.
			let ep = get_index(&e.key, n);
			let target = if from == ep { alt_index(&e.key, n) } else { ep };
			if target == from {
				debug!("AssociativeCache.put: drop key={:?} (no alternate)", e.key);
				return;
			}
			let bucket = &mut self.buckets[target];
			if !bucket.is_full() {
				debug!("AssociativeCache.put: relocate key={:?} {from}->{target}", e.key);
				bucket.put(e);
				return;
			}
			let Some(cand) = bucket.eviction_candidate_index() else {
				return;
			};
			if e.timestamp > bucket.get_items()[cand].timestamp {
				debug!("AssociativeCache.put: kick into {target} key={:?}", e.key);
				let (_, next) = bucket.put(e);
				displaced = next;
				from = target;
			} else {
				debug!("AssociativeCache.put: evict key={:?}", e.key);
				return;
			}
		}
		if let Some(e) = displaced {
			debug!("AssociativeCache.put: evict key={:?} (max kicks)", e.key);
		}
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// Probes the primary bucket, then the alternate. As a side effect makes
	/// updates according to the eviction policy of the bucket that holds it.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let found = self.locate(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={:?} hit={}",
			found.map(|(b, _)| b),
			found.is_some()
		);
		let (idx, pos) = found?;
		Some(&self.buckets[idx].get_from_position(pos).value)
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
		// alternate bucket instead of leaving the cache.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let n = cache.buckets.len();
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, n, 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // displaces first into its alternate

		let alt = alt_index(&same_bucket_keys[0], n);
		assert_ne!(alt, target_bucket_idx);
		assert!(cache.buckets[alt].get_position(&same_bucket_keys[0]).is_some());
		assert!(cache.buckets[target_bucket_idx].get_position(&same_bucket_keys[0]).is_none());

		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[0]).unwrap(), 10);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);

		let total: usize = cache.buckets.iter().map(|b| b.size()).sum();
		assert_eq!(total, 3);
	}

	#[test]
	fn key_resides_in_at_most_one_legal_bucket() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU);
		let n = cache.buckets.len();
		for k in 0..300i64 {
			let key = k % 37;
			if cache.get_and_update_item(&key).is_none() {
				cache.put(key, key);
			}
			for b in &cache.buckets {
				assert!(b.size() <= b.get_max_size());
			}
		}
		for (idx, b) in cache.buckets.iter().enumerate() {
			for item in b.get_items() {
				assert!(idx == get_index(&item.key, n) || idx == alt_index(&item.key, n));
				let copies = cache
					.buckets
					.iter()
					.filter(|bb| bb.get_position(&item.key).is_some())
					.count();
				assert_eq!(copies, 1);
			}
		}
	}
}
