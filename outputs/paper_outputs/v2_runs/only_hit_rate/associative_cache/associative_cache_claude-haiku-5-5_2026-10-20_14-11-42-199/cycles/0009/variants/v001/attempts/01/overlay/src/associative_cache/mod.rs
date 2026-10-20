use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::collections::{HashMap, HashSet, VecDeque};
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
	/// Shared access-frequency counter covering both resident and non-resident keys.
	/// Bumped on every access (hit or miss) and consulted by the admission gate.
	freq: HashMap<K, usize>,
	/// Second-miss ghost list: keys that missed into a full bucket and were refused
	/// on that first miss. A key present here is admitted to the admission comparison
	/// on its next miss. Bounded by `ghost_capacity` (one cache capacity), oldest first.
	ghost: HashSet<K>,
	ghost_order: VecDeque<K>,
	ghost_capacity: usize,
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
		let ghost_capacity = settings.num_of_buckets * settings.bucket_size;
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self {
			buckets,
			freq: HashMap::new(),
			ghost: HashSet::new(),
			ghost_order: VecDeque::new(),
			ghost_capacity,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache.
	///
	/// Ghost gate: when the target bucket is full and the key is not resident, a key
	/// seen for the first time (not in the ghost list) is refused and recorded in the
	/// ghost list. A key already in the ghost list is forgotten there and proceeds.
	///
	/// Admission gate: when the target bucket is full and the key is not already
	/// resident, the incoming key is admitted only if its frequency is strictly
	/// greater than the frequency of the victim the bucket's eviction policy would
	/// displace. Otherwise the insert is bypassed and the victim stays resident.
	pub fn put(&mut self, key: K, value: V) {
		let bucket_index = get_index(&key, self.buckets.len());
		debug!("AssociativeCache.put: key={key:?} bucket={bucket_index}");
		let bucket = &mut self.buckets[bucket_index];
		if bucket.is_full() && bucket.get_position(&key).is_none() {
			if !self.ghost.contains(&key) {
				debug!("AssociativeCache.put: ghost-refuse key={key:?} bucket={bucket_index}");
				Self::ghost_record(&mut self.ghost, &mut self.ghost_order, self.ghost_capacity, key);
				return;
			}
			Self::ghost_forget(&mut self.ghost, &mut self.ghost_order, &key);
			let victim_key = bucket
				.eviction_candidate_index()
				.map(|idx| bucket.get_items()[idx].key);
			if let Some(victim_key) = victim_key {
				// The incoming key's own access counts, even if no preceding get bumped it.
				let incoming = self.freq.get(&key).copied().unwrap_or(0).max(1);
				let victim = self.freq.get(&victim_key).copied().unwrap_or(0);
				if incoming <= victim {
					debug!("AssociativeCache.put: bypass key={key:?} incoming={incoming} victim={victim}");
					return;
				}
			}
		} else {
			// Non-full insert bypasses the ghost check; keep resident and ghost disjoint.
			Self::ghost_forget(&mut self.ghost, &mut self.ghost_order, &key);
		}
		bucket.put(Item::new(key, value));
	}

	/// Record a refused key in the ghost list, evicting the oldest entry when full.
	fn ghost_record(ghost: &mut HashSet<K>, order: &mut VecDeque<K>, capacity: usize, key: K) {
		if capacity == 0 {
			return;
		}
		if order.len() >= capacity {
			if let Some(oldest) = order.pop_front() {
				ghost.remove(&oldest);
			}
		}
		if ghost.insert(key) {
			order.push_back(key);
		}
	}

	/// Remove a key from the ghost list if present.
	fn ghost_forget(ghost: &mut HashSet<K>, order: &mut VecDeque<K>, key: &K) {
		if ghost.remove(key) {
			if let Some(pos) = order.iter().position(|k| k == key) {
				order.remove(pos);
			}
		}
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		*self.freq.entry(*key).or_insert(0) += 1;
		let bucket_index = get_index(key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		let data = bucket.get(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit={}",
			data.is_some()
		);
		Some(&data?.value)
	}

	#[allow(dead_code)]
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
		// bucket_size=2, find 3 keys that hash to same bucket → first should be evicted.
		// The ghost gate refuses a key's first miss into a full bucket, so the third key
		// is put twice: the first put is refused and recorded, the second is admitted.
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::Fifo);
		let target_bucket_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_bucket_idx, cache.buckets.len(), 3);

		cache.put(same_bucket_keys[0], 10);
		cache.put(same_bucket_keys[1], 20);
		cache.put(same_bucket_keys[2], 30); // refused by ghost gate (first miss)
		cache.put(same_bucket_keys[2], 30); // second miss: admitted, evicts first (FIFO)

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}
}
