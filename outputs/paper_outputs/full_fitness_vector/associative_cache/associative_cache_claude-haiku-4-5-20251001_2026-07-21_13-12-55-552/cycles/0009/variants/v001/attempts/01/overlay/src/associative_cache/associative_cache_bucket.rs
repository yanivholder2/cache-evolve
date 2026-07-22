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
	/// Whether to insert new items at head (true) or tail (false)
	prefer_head_insertion: bool,
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
			prefer_head_insertion: false,
		}
	}

	/// Set the insertion preference for the next put operation
	pub fn set_insertion_preference(&mut self, prefer_head: bool) {
		self.prefer_head_insertion = prefer_head;
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

	/// Override put to support insertion position bias
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			// Update existing key - behavior unchanged
			log::trace!("Bucket.put: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// New item insertion - decide position based on preference
		let evicted_item = if self.is_full() {
			self.evict_item()
		} else {
			None
		};

		// Insert at head (back, protected) or tail (front, eviction) based on preference
		// Head = items.push() (back/end of vector) - protected from eviction, for hot items
		// Tail = items.insert(0, ...) (front/start) - vulnerable to eviction, for cold items
		if self.prefer_head_insertion {
			// Hot workload: insert at head (back/end) for retention
			self.items.push(item);
		} else {
			// Cold workload: insert at tail (front/start) for quick eviction
			self.items.insert(0, item);
		}

		let pushed_item = if self.prefer_head_insertion {
			self.items.last().expect("bucket invariant: just pushed")
		} else {
			&self.items[0]
		};

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
