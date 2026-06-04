use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

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
	stash: Vec<Item<K, V>>,       // recently evicted items
	stash_max_size: usize,        // capacity cap for stash
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
			stash: Vec::with_capacity((settings.bucket_size / 4).max(1)),
			stash_max_size: (settings.bucket_size / 4).max(1),
		}
	}

	/// Push an item onto the stash, dropping the oldest entry if the stash is at capacity.
	fn stash_push(&mut self, item: Item<K, V>) {
		if self.stash.len() >= self.stash_max_size {
			self.stash.remove(0); // drop oldest stash entry (FIFO overflow)
		}
		self.stash.push(item);
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Case 1: update existing key in primary
		if let Some(position) = self.get_position(&item.key) {
			self.get_items_mut()[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		// Case 2: key is in stash — promote to primary
		if let Some(stash_pos) = self.stash.iter().position(|i| i.key == item.key) {
			let mut stash_item = self.stash.remove(stash_pos);
			stash_item.value = item.value;
			if self.is_full() {
				if let Some(evicted) = self.evict_item() {
					self.stash_push(evicted);
				}
			}
			self.get_items_mut().push(stash_item);
			let pushed = self.get_items().last().expect("bucket invariant: just pushed");
			return (pushed, None);
		}
		// Cases 3 and 4: new key
		if self.is_full() {
			if let Some(evicted) = self.evict_item() {
				self.stash_push(evicted);
			}
		}
		self.get_items_mut().push(item);
		let pushed = self.get_items().last().expect("bucket invariant: just pushed");
		(pushed, None)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		// Primary hit
		if let Some(position) = self.get_position(key) {
			return Some(self.get_from_position(position));
		}
		// Stash hit — promote to primary
		if let Some(stash_pos) = self.stash.iter().position(|i| &i.key == key) {
			let stash_item = self.stash.remove(stash_pos);
			if self.is_full() {
				if let Some(evicted) = self.evict_item() {
					self.stash_push(evicted);
				}
			}
			self.get_items_mut().push(stash_item);
			return self.get_items().last();
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
