use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::{fp_from_mixed, mixed_hash, AssociativeCacheBucket};
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// Associative cache. In ClassicLRU (GCLOCK) mode with more than one bucket,
/// each key has two candidate buckets: the primary `b1 = get_index(key)` and a
/// deterministic alternate `b2` (bits 16..32 of the bucket file's mixed hash).
/// A key lives in at most one of them; a newcomer goes to whichever candidate
/// offers the colder victim. Every other policy keeps single-bucket placement.
#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	two_choice: bool,
}

/// Alternate bucket index for two-choice placement. Pure function of the key.
#[inline]
fn alt_bucket(mixed: u64, b1: usize, n_buckets: usize) -> usize {
	let b2 = (((mixed >> 16) & 0xFFFF) as usize) % n_buckets;
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
		info!("AssociativeCache.try_new: settings={settings:?}");
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		let two_choice = buckets.len() > 1 && buckets[0].is_clock_mode();
		Some(Self { buckets, two_choice })
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		let n = self.buckets.len();
		let b1 = get_index(&key, n);
		if !self.two_choice {
			debug!("AssociativeCache.put: key={key:?} bucket={b1}");
			self.buckets[b1].put(Item::new(key, value));
			return;
		}

		let mixed = mixed_hash(&key);
		let b2 = alt_bucket(mixed, b1, n);

		// Update in place if resident in either candidate (at most one).
		if self.buckets[b1].get_position(&key).is_some() {
			self.buckets[b1].put(Item::new(key, value));
			return;
		}
		if self.buckets[b2].get_position(&key).is_some() {
			self.buckets[b2].put(Item::new(key, value));
			return;
		}

		// Choose target bucket.
		let full1 = self.buckets[b1].is_full();
		let full2 = self.buckets[b2].is_full();
		let target = match (full1, full2) {
			(false, _) => b1,
			(true, false) => b2,
			(true, true) => {
				let s1 = self.buckets[b1].victim_score();
				let s2 = self.buckets[b2].victim_score();
				if s2 < s1 {
					b2
				} else {
					b1
				}
			}
		};

		// Ghost check across both candidates (consume on match; short-circuit
		// so at most one ghost entry is consumed).
		let fp = fp_from_mixed(mixed);
		let counter = if self.buckets[b1].take_ghost(fp) || self.buckets[b2].take_ghost(fp) {
			1
		} else {
			0
		};

		debug!("AssociativeCache.put: key={key:?} b1={b1} b2={b2} target={target} counter={counter}");
		self.buckets[target].put_with_counter(Item::new(key, value), counter);
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let n = self.buckets.len();
		let b1 = get_index(key, n);
		let idx = if !self.two_choice || self.buckets[b1].get_position(key).is_some() {
			b1
		} else {
			alt_bucket(mixed_hash(key), b1, n)
		};
		let data = self.buckets[idx].get(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={idx} hit={}",
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
	fn two_choice_key_resident_in_at_most_one_bucket() {
		let mut cache = small_cache(crate::shared::eviction_policy::EvictionPolicyKind::ClassicLRU);
		for k in 0..200i64 {
			if cache.get_and_update_item(&k).is_none() {
				cache.put(k, k);
			}
		}
		let mut seen = std::collections::HashSet::new();
		for b in &cache.buckets {
			assert!(b.size() <= b.get_max_size());
			for it in b.get_items() {
				assert!(seen.insert(it.key), "duplicate key {}", it.key);
			}
		}
		for k in seen {
			assert_eq!(cache.get_and_update_item(&k).copied(), Some(k));
		}
	}
}
