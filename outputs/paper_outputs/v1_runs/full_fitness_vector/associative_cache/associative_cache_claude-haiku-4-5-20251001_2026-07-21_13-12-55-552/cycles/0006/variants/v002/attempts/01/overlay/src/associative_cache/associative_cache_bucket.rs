use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::debug;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Cache-line aligned bucket for improved spatial locality during scans.
/// Aligned to 64-byte cache-line boundaries to reduce tail latency in ClassicLRU
/// scanning operations, particularly with larger bucket geometries like 32×64.
///
/// Tracks access recency via `last_access_epoch` and periodically reorganizes
/// the bucket to cluster hot items at the front, improving CPU prefetch effectiveness.
#[repr(align(64))]
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Tracks total number of evictions to trigger reorganization every 16 evictions.
	eviction_count: usize,
	/// Global access epoch counter to track recency of item accesses.
	access_epoch: usize,
	/// Reorganization threshold: reorganize every N evictions.
	reorganization_interval: usize,
	/// Threshold for hot items: items accessed within last N operations come first.
	recency_threshold: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let mut items = Vec::with_capacity(settings.bucket_size);
		// Pre-reserve capacity to avoid reallocations during operation,
		// which helps maintain cache-line alignment benefits
		items.reserve(settings.bucket_size);

		Self {
			items,
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			eviction_count: 0,
			access_epoch: 0,
			reorganization_interval: 16,
			recency_threshold: 32,
		}
	}

	/// Update the access epoch for the item at the given position.
	fn update_access_epoch(&mut self, position: usize) {
		self.access_epoch += 1;
		if position < self.items.len() {
			self.items[position].last_access_epoch = self.access_epoch;
		}
	}

	/// Reorganize the bucket to move hot items (recently accessed) to the front.
	/// Partitions items: those accessed within the last `recency_threshold` operations
	/// come first, followed by older items.
	fn reorganize_bucket(&mut self) {
		if self.items.is_empty() {
			return;
		}

		let threshold_epoch = self.access_epoch.saturating_sub(self.recency_threshold);

		// Partition items in-place: hot items (access_epoch >= threshold) go to front
		let mut hot_end = 0;
		for i in 0..self.items.len() {
			if self.items[i].last_access_epoch >= threshold_epoch {
				self.items.swap(i, hot_end);
				hot_end += 1;
			}
		}
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// Update access epoch for this item when it's accessed
		self.update_access_epoch(index);
		&self.items[index]
	}

	/// Optimized get_position with early-exit on first match.
	/// Manually iterates over items and returns immediately upon locating the key,
	/// without completing a full bucket scan. This reduces p99 latency spike
	/// in ClassicLRU with large buckets by avoiding unnecessary memory accesses.
	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		// Manual loop for early-exit; inlining ensures tight coupling with caller's
		// instruction stream, reducing latency spikes from function call overhead.
		for (idx, item) in self.items.iter().enumerate() {
			if item.key == *key {
				return Some(idx);
			}
		}
		None
	}

	/// Override put to update access epoch for new items.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			// Update existing item value
			self.get_items_mut()[position].value = item.value;
			// get_from_position will handle both policy reordering and access epoch update
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// New item: evict if necessary
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Increment access epoch for the new item
		self.access_epoch += 1;
		let mut new_item = item;
		new_item.last_access_epoch = self.access_epoch;

		self.get_items_mut().push(new_item);
		let pushed_item = self.get_items().last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Override evict_item to trigger reorganization every 16 evictions.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);

		// Increment eviction counter and check if reorganization is due
		self.eviction_count += 1;
		if self.eviction_count % self.reorganization_interval == 0 {
			self.reorganize_bucket();
		}

		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?} eviction_count={}",
			self.get_eviction_policy().name(),
			evicted.key,
			self.eviction_count
		);
		Some(evicted)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
