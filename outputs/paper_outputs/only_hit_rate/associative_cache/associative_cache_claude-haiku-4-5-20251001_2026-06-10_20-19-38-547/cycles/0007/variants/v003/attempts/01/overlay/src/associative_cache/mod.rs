use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use associative_cache_bucket::AssociativeCacheBucket;
use associative_cache_settings::AssociativeCacheSettings;
use log::{debug, info};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

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
	// Phase-aware collision management state
	operation_count: u64,
	phase_window_count: u64,
	phase_start_collision_counts: Vec<usize>,
	collision_distribution: Vec<usize>,
	prior_cv: f64,
	in_transition: bool,
	transition_operation_count: u64,
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

		let num_buckets = settings.num_of_buckets;
		Some(Self {
			buckets,
			operation_count: 0,
			phase_window_count: 0,
			phase_start_collision_counts: vec![0; num_buckets],
			collision_distribution: vec![0; num_buckets],
			prior_cv: 0.0,
			in_transition: false,
			transition_operation_count: 0,
		})
	}

	pub fn new(settings: AssociativeCacheSettings) -> Self {
		Self::try_new(settings).expect("AssociativeCache::new: initialization failed")
	}

	/// Compute collision counts for all buckets (collisions = items_in_bucket - 1)
	fn compute_collision_counts(&self) -> Vec<usize> {
		self.buckets.iter().map(|b| b.get_items().len().saturating_sub(1)).collect()
	}

	/// Compute mean of collision distribution
	fn compute_mean(distribution: &[usize]) -> f64 {
		if distribution.is_empty() {
			return 0.0;
		}
		let sum: u64 = distribution.iter().map(|&x| x as u64).sum();
		sum as f64 / distribution.len() as f64
	}

	/// Compute standard deviation of collision distribution
	fn compute_std_dev(distribution: &[usize], mean: f64) -> f64 {
		if distribution.is_empty() {
			return 0.0;
		}
		let variance: f64 = distribution
			.iter()
			.map(|&x| {
				let diff = x as f64 - mean;
				diff * diff
			})
			.sum::<f64>() / distribution.len() as f64;
		variance.sqrt()
	}

	/// Compute coefficient of variation (std_dev / mean)
	fn compute_cv(mean: f64, std_dev: f64) -> f64 {
		if mean == 0.0 {
			0.0
		} else {
			std_dev / mean
		}
	}

	/// Check for phase transition and update state
	fn check_phase_transition(&mut self) {
		if self.operation_count % 2000 != 0 {
			return;
		}

		let current_collision_counts = self.compute_collision_counts();
		let mean = Self::compute_mean(&current_collision_counts);
		let std_dev = Self::compute_std_dev(&current_collision_counts, mean);
		let current_cv = Self::compute_cv(mean, std_dev);

		debug!(
			"AssociativeCache.check_phase_transition: op_count={} mean={:.3} std_dev={:.3} cv={:.3} prior_cv={:.3}",
			self.operation_count, mean, std_dev, current_cv, self.prior_cv
		);

		// Detect phase transition if CV differs by > 20 percentage points
		let cv_diff = (current_cv - self.prior_cv).abs();
		if cv_diff > 0.20 {
			info!(
				"AssociativeCache: phase transition detected at op_count={}. CV diff={:.3}",
				self.operation_count, cv_diff
			);
			self.in_transition = true;
			self.transition_operation_count = 0;
		}

		// Update prior CV for next comparison
		self.prior_cv = current_cv;
	}

	/// Apply dynamic tier rebalancing during phase transitions.
	///
	/// During transition, selectively boost lfu_counter of hot items (those with higher frequency).
	/// This preserves the eviction policy signals while adapting to collision pattern changes.
	/// Hot items (high lfu_counter) experience increased preservation via counter boost (effective_hot_capacity = 110%).
	/// Cold items (low lfu_counter) are left unchanged, allowing natural eviction (effective_cold_capacity = 90%).
	fn apply_transition_tier_rebalancing(&mut self) {
		if !self.in_transition {
			return;
		}

		// Compute effective capacities dynamically during transition
		let hot_capacity_factor = 1.1; // effective_hot_capacity = 110%

		for bucket in &mut self.buckets {
			let items = bucket.get_items_mut();

			// Identify the median lfu_counter to distinguish hot from cold items
			if !items.is_empty() {
				let mut counters: Vec<usize> = items.iter().map(|item| item.lfu_counter).collect();
				counters.sort();
				let median_idx = counters.len() / 2;
				let median_counter = counters[median_idx];

				// Boost hot items (those with lfu_counter > median) to preserve them during transition
				for item in items {
					if item.lfu_counter > median_counter && item.lfu_counter > 0 {
						// Increase the effective capacity perception for hot items
						let boosted = (item.lfu_counter as f64 * hot_capacity_factor) as usize;
						item.lfu_counter = boosted;
					}
					// Cold items (lfu_counter <= median) are not modified,
					// allowing them to be evicted more readily (effective_cold_capacity = 90%)
				}
			}
		}
	}

	/// Update transition state
	fn update_transition_state(&mut self) {
		if self.in_transition {
			self.transition_operation_count += 1;
			// After 500 operations of transition, revert to normal
			if self.transition_operation_count >= 500 {
				info!(
					"AssociativeCache: exiting phase transition at op_count={}",
					self.operation_count
				);
				self.in_transition = false;
				self.transition_operation_count = 0;
			}
		}
	}

	/// Insert a key-value pair into the cache
	pub fn put(&mut self, key: K, value: V) {
		debug!(
			"AssociativeCache.put: key={key:?} bucket={}",
			get_index(&key, self.buckets.len())
		);
		let bucket = self.get_mut_bucket(&key);
		bucket.put(Item::new(key, value));

		// Update operation tracking
		self.operation_count += 1;
		self.phase_window_count += 1;

		// Check for phase transitions every 2000 operations
		self.check_phase_transition();

		// Apply tier rebalancing during transition
		self.apply_transition_tier_rebalancing();

		// Update transition state
		self.update_transition_state();
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let bucket_index = get_index(key, self.buckets.len());

		// Update operation tracking (for get operations)
		self.operation_count += 1;
		self.phase_window_count += 1;

		// Check for phase transitions every 2000 operations
		self.check_phase_transition();

		// Apply tier rebalancing during transition
		self.apply_transition_tier_rebalancing();

		// Update transition state
		self.update_transition_state();

		// NOW borrow the bucket and get the data
		// We do this AFTER all the phase management operations to avoid borrow conflicts
		let bucket = &mut self.buckets[bucket_index];
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
