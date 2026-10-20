use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_two_indices;
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

	/// Returns the index of the candidate bucket (of the two) holding `key`, if any.
	fn locate(&self, key: &K, i1: usize, i2: usize) -> Option<usize> {
		if self.buckets[i1].get_position(key).is_some() {
			return Some(i1);
		}
		if i2 != i1 && self.buckets[i2].get_position(key).is_some() {
			return Some(i2);
		}
		None
	}

	/// Chooses the target bucket for a new (non-resident) key among two candidates.
	///
	/// (a) Prefer a non-full bucket; if both are non-full, the smaller one (tie → i1).
	/// (b) If both are full, the one whose eviction candidate is staler (older
	///     timestamp; tie → i1).
	fn choose_target(&self, i1: usize, i2: usize) -> usize {
		if i1 == i2 {
			return i1;
		}
		let b1 = &self.buckets[i1];
		let b2 = &self.buckets[i2];
		match (b1.is_full(), b2.is_full()) {
			(false, true) => i1,
			(true, false) => i2,
			(false, false) => {
				if b2.size() < b1.size() {
					i2
				} else {
					i1
				}
			}
			(true, true) => {
				let t1 = b1.eviction_candidate_index().map(|i| b1.get_items()[i].timestamp);
				let t2 = b2.eviction_candidate_index().map(|i| b2.get_items()[i].timestamp);
				match (t1, t2) {
					(Some(t1), Some(t2)) if t2 < t1 => i2,
					_ => i1,
				}
			}
		}
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		let (i1, i2) = get_two_indices(&key, self.buckets.len());
		// Existing key: update in place in whichever bucket holds it (never duplicate).
		let target = match self.locate(&key, i1, i2) {
			Some(idx) => idx,
			None => self.choose_target(i1, i2),
		};
		debug!("AssociativeCache.put: key={key:?} candidates=({i1},{i2}) bucket={target}");
		self.buckets[target].put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let (i1, i2) = get_two_indices(key, self.buckets.len());
		let found = self.locate(key, i1, i2);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} candidates=({i1},{i2}) hit={}",
			found.is_some()
		);
		let bucket_index = found?;
		let bucket = &mut self.buckets[bucket_index];
		let position = bucket.get_position(key)?;
		Some(&bucket.get_from_position(position).value)
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
		// With two-choice placement a key may land in either candidate bucket, so
		// assert only the structural properties: occupancy is bounded, no key is
		// duplicated, and the most recently put key always hits.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU);
		for k in 0..50i64 {
			if cache.get_and_update_item(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert_eq!(*cache.get_and_update_item(&k).unwrap(), k * 10);
			let total: usize = cache.buckets.iter().map(|b| b.size()).sum();
			assert!(total <= 8);
			for b in &cache.buckets {
				assert!(b.size() <= b.get_max_size());
			}
		}
		let mut seen = std::collections::HashSet::new();
		for b in &cache.buckets {
			for item in b.get_items() {
				assert!(seen.insert(item.key), "key {} resident in two buckets", item.key);
			}
		}
	}
}
