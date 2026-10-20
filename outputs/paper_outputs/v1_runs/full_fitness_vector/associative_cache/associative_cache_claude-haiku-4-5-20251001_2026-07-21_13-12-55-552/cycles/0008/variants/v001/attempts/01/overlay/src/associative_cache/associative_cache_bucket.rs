use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Cache-line aligned bucket for improved spatial locality during scans.
/// Aligned to 64-byte cache-line boundaries to reduce tail latency in ClassicLRU
/// scanning operations, particularly with larger bucket geometries like 64×64.
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
