use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::collections::VecDeque;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

pub mod associative_cache_bucket;
pub mod associative_cache_jni;
pub mod associative_cache_settings;

const CHARACTERIZATION_PHASE_THRESHOLD: u32 = 512;
const RECENT_WINDOW_SIZE: usize = 50;
const EARLY_EXIT_OP_THRESHOLD: u32 = 256;
const STABILITY_COUNT_THRESHOLD: u32 = 32;

#[derive(Debug)]
pub struct AssociativeCache<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<AssociativeCacheBucket<K, V>>,
	// Characterization phase fields
	characterization_phase: bool,
	hot_phase_count: u32,
	insertion_bias: f64,
	// Track recent hits during characterization
	recent_item_insertions: VecDeque<Instant>,
	recent_hits_count: u32,
	total_gets_count: u32,
	// Track consecutive operations without hit-rate change for early exit
	stability_count: u32,
	// Track previous hit count to detect stability
	previous_recent_hits_count: u32,
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
		Some(Self {
			buckets,
			characterization_phase: true,
			hot_phase_count: 0,
			insertion_bias: 0.5, // neutral default
			recent_item_insertions: VecDeque::with_capacity(RECENT_WINDOW_SIZE),
			recent_hits_count: 0,
			total_gets_count: 0,
			stability_count: 0,
			previous_recent_hits_count: 0,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);

		// Track insertion count during characterization phase
		if self.characterization_phase {
			self.hot_phase_count += 1;
			self.recent_item_insertions.push_back(Instant::now());
			if self.recent_item_insertions.len() > RECENT_WINDOW_SIZE {
				self.recent_item_insertions.pop_front();
			}

			// Early exit if patterns have stabilized
			let early_exit_condition = self.hot_phase_count >= EARLY_EXIT_OP_THRESHOLD
				&& (self.stability_count >= STABILITY_COUNT_THRESHOLD
					|| self.hot_phase_count >= CHARACTERIZATION_PHASE_THRESHOLD);

			// Transition from characterization to normal phase
			if early_exit_condition {
				self.characterization_phase = false;
				// Compute insertion bias based on collected data
				if self.total_gets_count > 0 {
					self.insertion_bias = self.recent_hits_count as f64 / self.total_gets_count as f64;
				} else {
					self.insertion_bias = 0.5; // No data, use neutral bias
				}
				info!(
					"AssociativeCache: Characterization phase complete at {} operations. insertion_bias={:.2} stability_count={}",
					self.hot_phase_count, self.insertion_bias, self.stability_count
				);
				// Update all buckets with the computed insertion bias
				for bucket in &mut self.buckets {
					bucket.set_insertion_bias(self.insertion_bias);
				}
			}
		}

		let bucket = self.get_mut_bucket(&key);
		bucket.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());
		let bucket = &mut self.buckets[bucket_index];

		// Track accesses during characterization phase
		if self.characterization_phase {
			self.total_gets_count += 1;

			// Check if the accessed item is in the recent window
			if let Some(position) = bucket.get_position(key) {
				if let Some(item_timestamp) = bucket.get_item_timestamp(position) {
					let now = Instant::now();
					let age = now.saturating_duration_since(item_timestamp);

					// Optimize timestamp comparison: only check items in the most recent 50% of insertions
					let check_bound = if self.recent_item_insertions.len() > RECENT_WINDOW_SIZE / 2 {
						// Use the 50% midpoint as the cutoff to reduce expensive comparisons
						self.recent_item_insertions.get(RECENT_WINDOW_SIZE / 2)
					} else {
						// If we haven't filled half the window yet, use the oldest point (original behavior)
						self.recent_item_insertions.front()
					};

					if let Some(bound_timestamp) = check_bound {
						let bound_age = now.saturating_duration_since(*bound_timestamp);
						if age <= bound_age {
							self.recent_hits_count += 1;
						}
					}

					// Track stability: if hit count hasn't changed, increment stability counter
					if self.recent_hits_count == self.previous_recent_hits_count {
						self.stability_count += 1;
					} else {
						// Hit pattern changed, reset stability counter
						self.stability_count = 0;
						self.previous_recent_hits_count = self.recent_hits_count;
					}
				}
			} else {
				// Miss detected, check stability
				if self.recent_hits_count == self.previous_recent_hits_count {
					self.stability_count += 1;
				} else {
					self.stability_count = 0;
					self.previous_recent_hits_count = self.recent_hits_count;
				}
			}
		}

		let data = bucket.get(key);
		debug!(
			"AssociativeCache.get_and_update_item: key={key:?} bucket={bucket_index} hit={}",
			data.is_some()
		);
		Some(&data?.value)
	}

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
}
