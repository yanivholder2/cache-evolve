use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::collections::{HashMap, VecDeque};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

/// Bounded ghost list of recently missed keys, stored exactly (no hashing, no
/// aliasing, no counters). Used only by the admission decision on a miss.
///
/// Recency is kept by a FIFO of `(key, seq)` entries with lazy deletion: `live`
/// maps each resident ghost key to the sequence number of its current entry, so
/// a removed or re-recorded key leaves a stale queue entry that is skipped (and
/// dropped) when it reaches the front or when the queue is compacted.
#[derive(Debug)]
struct GhostList<K>
where
	K: Hash + Eq + Copy + Debug,
{
	live: HashMap<K, u64>,
	order: VecDeque<(K, u64)>,
	capacity: usize,
	next_seq: u64,
}

impl<K> GhostList<K>
where
	K: Hash + Eq + Copy + Debug,
{
	fn new(capacity: usize) -> Self {
		Self {
			live: HashMap::with_capacity(capacity),
			order: VecDeque::with_capacity(capacity),
			capacity,
			next_seq: 0,
		}
	}

	/// Second-miss admission. If `key` is ghost-listed (its previous miss is still
	/// in the window) it is removed from the list and admitted. Otherwise it is
	/// recorded in the list and rejected.
	fn admit(&mut self, key: K) -> bool {
		if self.live.remove(&key).is_some() {
			return true;
		}
		self.record(key);
		false
	}

	fn record(&mut self, key: K) {
		// Keep at most `capacity` live ghost keys: drop the oldest live entry.
		while self.live.len() >= self.capacity {
			let (old, seq) = self
				.order
				.pop_front()
				.expect("ghost list invariant: every live key has a queued entry");
			if self.live.get(&old) == Some(&seq) {
				self.live.remove(&old);
			}
		}
		let seq = self.next_seq;
		self.next_seq += 1;
		self.live.insert(key, seq);
		self.order.push_back((key, seq));
		// Compact stale queue entries so the queue stays O(capacity).
		if self.order.len() > 2 * self.capacity {
			let live = &self.live;
			self.order.retain(|(k, s)| live.get(k) == Some(s));
		}
	}
}

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	ghost: GhostList<K>,
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
		// Ghost window widened to four times the cache capacity (H03): a key's first
		// miss stays recorded for about four times as many subsequent distinct misses.
		let ghost = GhostList::new(4 * settings.capacity());
		let mut buckets = Vec::with_capacity(settings.num_of_buckets);
		for _ in 0..settings.num_of_buckets {
			buckets.push(AssociativeCacheBucket::new(&settings));
		}
		Some(Self { buckets, ghost })
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache.
	///
	/// Admission: an insert that would displace a resident item is admitted only
	/// on the key's second miss within the ghost window; a first miss is recorded
	/// in the ghost list and dropped. Inserts into a bucket with free slots, and
	/// updates of resident keys, are never filtered.
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);
		let bucket_index = get_index(&key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];
		let displaces = bucket.get_position(&key).is_none() && bucket.is_full();
		if displaces && !self.ghost.admit(key) {
			debug!("AssociativeCache.put: key={key:?} bucket={bucket_index} rejected (ghost-listed)");
			return;
		}
		bucket.put(Item::new(key, value));
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
		// Third key displaces a resident item, so its first miss is only ghost-listed.
		cache.put(same_bucket_keys[2], 30);
		assert!(cache.get_and_update_item(&same_bucket_keys[2]).is_none());
		// Second miss within the window is admitted and evicts first (FIFO).
		cache.put(same_bucket_keys[2], 30);

		assert!(cache.get_and_update_item(&same_bucket_keys[0]).is_none());
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[1]).unwrap(), 20);
		assert_eq!(*cache.get_and_update_item(&same_bucket_keys[2]).unwrap(), 30);
	}
}
