use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Saturation limit of the per-slot frequency counter (2 bits).
const FREQ_MAX: usize = 3;
/// Every this-many evictions in a bucket, all counters decay by one.
const AGE_PERIOD: u8 = 8;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Eviction count modulo AGE_PERIOD, used to trigger counter aging.
	evictions: u8,
	/// Frequency+LRU hybrid is active only for the ClassicLRU policy.
	freq_enabled: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let freq_enabled = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			evictions: 0,
			freq_enabled,
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
		if self.freq_enabled {
			let c = &mut self.items[index].lfu_counter;
			if *c < FREQ_MAX {
				*c += 1;
			}
		}
		&self.items[index]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if !self.freq_enabled {
			return Some(self.eviction_policy.eviction_candidate_index(&self.items));
		}
		// Scan from the LRU end (index 0); first strictly-lower counter wins.
		let mut best = 0;
		let mut best_c = self.items[0].lfu_counter;
		for (i, it) in self.items.iter().enumerate().skip(1) {
			if it.lfu_counter < best_c {
				best = i;
				best_c = it.lfu_counter;
			}
		}
		Some(best)
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let all_saturated = self.freq_enabled && self.items[idx].lfu_counter >= FREQ_MAX;
		let evicted = self.items.remove(idx);
		if self.freq_enabled {
			self.evictions += 1;
			if self.evictions >= AGE_PERIOD || all_saturated {
				self.evictions = 0;
				for it in self.items.iter_mut() {
					it.lfu_counter = it.lfu_counter.saturating_sub(1);
				}
			}
		}
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
