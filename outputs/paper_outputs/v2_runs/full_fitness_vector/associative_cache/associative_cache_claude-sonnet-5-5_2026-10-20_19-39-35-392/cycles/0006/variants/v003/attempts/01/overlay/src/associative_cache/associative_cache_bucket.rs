use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Stateless scan-resistant bucket: mid-bucket insertion, single-step
/// transpose promotion on hit (toward the protected end = back of the vec),
/// eviction always from the LRU end (index 0).
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
		// Single-step transpose toward the protected (back) end.
		let last = self.items.len() - 1;
		if position < last {
			self.items.swap(position, position + 1);
			&self.items[position + 1]
		} else {
			&self.items[position]
		}
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			None
		} else {
			Some(0)
		}
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = if position + 1 < self.items.len() {
				self.items.swap(position, position + 1);
				position + 1
			} else {
				position
			};
			return (&self.items[idx], None);
		}
		let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
			Some(self.items.remove(0))
		} else {
			None
		};
		let mid = self.items.len() / 2;
		self.items.push(item);
		let mut i = self.items.len() - 1;
		while i > mid {
			self.items.swap(i, i - 1);
			i -= 1;
		}
		(&self.items[mid], evicted)
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
