use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EVICT_WINDOW;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::debug;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

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
		Self {
			items: Vec::with_capacity(settings.bucket_size),
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

	/// ClassicLRU: evict the min-frequency item in the LRU-tail window, aging
	/// (saturating-decrementing) every other item in that window so a once-hot
	/// item cannot pin a slot forever. Other policies keep the default body.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.eviction_policy.name() != "ClassicLRU" {
			let idx = self.eviction_candidate_index()?;
			let evicted = self.remove_at(idx);
			debug!(
				"Bucket.evict_item: policy={} evicted_key={:?}",
				self.eviction_policy.name(),
				evicted.key
			);
			return Some(evicted);
		}

		let idx = self.eviction_candidate_index()?;
		let w = self.items.len().min(EVICT_WINDOW);
		for (i, item) in self.items[..w].iter_mut().enumerate() {
			if i != idx {
				item.lfu_counter = item.lfu_counter.saturating_sub(1);
			}
		}
		let evicted = self.remove_at(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.eviction_policy.name(),
			evicted.key
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
