use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::{ghost_fp, mixed_hash, AssociativeCacheBucket};
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

/// Two-choice alternate bucket: bits 16..32 of the FNV-1a+mix hash (disjoint
/// from the ghost fingerprint's top 16 bits and from the low bits that drive
/// `b1`), reduced mod `n_buckets`; bumped to `b1 + 1` if it collides with `b1`.
/// A pure function of the key, so lookups are deterministic.
#[inline]
fn alt_bucket<K: Hash>(key: &K, b1: usize, n_buckets: usize) -> usize {
	let b2 = (((mixed_hash(key) >> 16) & 0xffff) as usize) % n_buckets;
	if b2 == b1 {
		(b1 + 1) % n_buckets
	} else {
		b2
	}
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

	/// Whether two-choice placement is active (ClassicLRU / GCLOCK mode with
	/// more than one bucket).
	#[inline]
	fn two_choice(&self) -> bool {
		self.buckets.len() > 1 && self.buckets[0].clock_mode()
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		let n = self.buckets.len();
		let b1 = get_index(&key, n);
		debug!("AssociativeCache.put: key={key:?} bucket={b1}");
		if !self.two_choice() {
			self.buckets[b1].put(Item::new(key, value));
			return;
		}
		let b2 = alt_bucket(&key, b1, n);

		// Update in place if resident in either candidate (a key lives in at
		// most one bucket).
		if self.buckets[b1].get_position(&key).is_some() {
			self.buckets[b1].put(Item::new(key, value));
			return;
		}
		if self.buckets[b2].get_position(&key).is_some() {
			self.buckets[b2].put(Item::new(key, value));
			return;
		}

		// Choose the target bucket.
		let full1 = self.buckets[b1].is_full();
		let full2 = self.buckets[b2].is_full();
		let target = if !full1 {
			b1
		} else if !full2 {
			b2
		} else if self.buckets[b2].victim_score() < self.buckets[b1].victim_score() {
			b2
		} else {
			b1
		};

		// Ghost check across both candidate rings, before the eviction.
		let fp = ghost_fp(&key);
		let counter = if self.buckets[b1].take_ghost(fp) || self.buckets[b2].take_ghost(fp) {
			1
		} else {
			0
		};
		debug!("AssociativeCache.put: two-choice b1={b1} b2={b2} target={target} counter={counter}");
		self.buckets[target].put_with_counter(Item::new(key, value), counter);
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let n = self.buckets.len();
		let b1 = get_index(key, n);
		if !self.two_choice() {
			let data = self.buckets[b1].get(key);
			debug!(
				"AssociativeCache.get_and_update_item: key={key:?} bucket={b1} hit={}",
				data.is_some()
			);
			return Some(&data?.value);
		}
		if let Some(pos) = self.buckets[b1].get_position(key) {
			debug!("AssociativeCache.get_and_update_item: key={key:?} bucket={b1} hit=true");
			return Some(&self.buckets[b1].get_from_position(pos).value);
		}
		let b2 = alt_bucket(key, b1, n);
		let pos = self.buckets[b2].get_position(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={b2} (alt) hit={}",
			pos.is_some()
		);
		Some(&self.buckets[b2].get_from_position(pos?).value)
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
		cache.put(same_bucket_keys[2], 30); // evicts first (FIFO)

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}

	#[test]
	fn two_choice_key_never_in_both_buckets() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU);
		for k in 0..200i64 {
			if cache.get_and_update_item(&k).is_none() {
				cache.put(k, k);
			}
		}
		let mut seen = std::collections::HashSet::new();
		for b in &cache.buckets {
			assert!(b.size() <= b.get_max_size());
			for i in b.get_items() {
				assert!(seen.insert(i.key), "key {} resident twice", i.key);
			}
		}
		for k in seen {
			assert_eq!(*cache.get_and_update_item(&k).unwrap(), k);
		}
	}
}
