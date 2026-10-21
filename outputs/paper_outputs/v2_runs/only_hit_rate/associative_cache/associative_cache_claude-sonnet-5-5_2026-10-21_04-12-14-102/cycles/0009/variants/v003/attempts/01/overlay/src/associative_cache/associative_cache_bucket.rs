use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Lean positional segmented LRU. Layout (index 0 .. len):
/// protected MRU .. protected LRU | probation MRU .. probation LRU.
/// Probation is the last `max_size/8` positions (absolute count), so demotion
/// of the protected LRU into probation happens implicitly when an entry is
/// moved to the front.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	probation: usize,
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
			probation: (settings.bucket_size / 8).max(1),
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		let evicted = if self.items.len() >= self.max_size { self.items.pop() } else { None };
		let idx = (self.items.len() + 1).saturating_sub(self.probation);
		self.items.insert(idx, item);
		(&self.items[idx], evicted)
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			None
		} else {
			Some(self.items.len() - 1)
		}
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if position > 0 {
			let item = self.items.remove(position);
			self.items.insert(0, item);
		}
		&self.items[0]
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
