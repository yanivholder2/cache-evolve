use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

/// Cache-line aligned bucket for improved spatial locality during scans.
/// Aligned to 64-byte cache-line boundaries to reduce tail latency in ClassicLRU
/// scanning operations, particularly with larger bucket geometries like 32×64.
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
	insertion_bias: f64,
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
			insertion_bias: 0.5, // neutral default during characterization
		}
	}

	/// Set the insertion bias for positioning new items
	pub fn set_insertion_bias(&mut self, bias: f64) {
		self.insertion_bias = bias;
	}
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Compute insertion position based on insertion_bias
	/// High bias (temporal reuse): insert at back to survive longer
	/// Low bias (temporal spreading): insert in middle to be evicted sooner
	fn compute_insertion_position(&self) -> usize {
		let len = self.items.len();

		if len == 0 {
			return 0;
		}

		// Bias is between 0.0 and 1.0
		// 1.0: insert at back (position = len)
		// 0.0: insert at front (position = 0)
		// 0.5: insert in middle (position = len / 2)
		let position = (self.insertion_bias * len as f64) as usize;
		position.min(len) // Ensure we don't exceed bounds
	}

	/// Get the timestamp of an item at a given position
	pub fn get_item_timestamp(&self, position: usize) -> Option<Instant> {
		self.items.get(position).map(|item| item.timestamp)
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

	/// Custom put implementation with insertion-position bias.
	/// Overrides the default trait implementation to insert items at positions
	/// determined by insertion_bias rather than always at the back.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Check if key already exists
		if let Some(position) = self.get_position(&item.key) {
			// Update existing key
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Handle eviction if bucket is full
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Determine insertion position based on insertion_bias
		let insertion_position = self.compute_insertion_position();

		// Insert at the computed position
		if insertion_position >= self.items.len() {
			// Insert at back
			self.items.push(item);
		} else {
			// Insert at computed position
			self.items.insert(insertion_position, item);
		}

		let pushed_item = &self.items[insertion_position];
		(pushed_item, evicted_item)
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
