// TODO: finish module documentation
//! Bucket implementation for the Dash cache.
//!
//!
//! # Examples
//!
//! ```
//! ```

use crate::shared::item::Item;
use crate::shared::settings::EvictionPolicy;
use crate::shared::traits::bucket::Bucket;
use std::{
	fmt::{Display, Formatter},
	hash::Hash,
};

#[derive(Debug)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	// TODO: consider using a linked list for O(1) changes
	items: Vec<Item<K, V>>,
	max_size: usize,
	// TODO: make this a reference with a lifetime
	eviction_policy: EvictionPolicy,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	pub fn new(max_size: usize, eviction_policy: EvictionPolicy) -> Self {
		DashBucket {
			// TODO: consider creating a vector with a fixed size for better performance after initialization
			items: Vec::new(),
			max_size,
			eviction_policy,
		}
	}

	/// Returns a reference to the item in position `position`, or `None` if the item is not found.
	/// This will not update the position of the item
	pub fn get_from_position(&mut self, position: usize) -> Option<&Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		Some(&self.items[position])
	}
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
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

	fn get_eviction_policy(&self) -> &EvictionPolicy {
		&self.eviction_policy
	}

	fn get_and_update_lru_item(&mut self, position: usize) -> &Item<K, V> {
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().unwrap()
	}

	fn evict_lru_item(&mut self) -> Option<Item<K, V>> {
		// TODO: this is in O(n). there could be a more performant way to do that
		Some(self.items.remove(0))
	}
}

impl<K, V> Display for DashBucket<K, V>
where
	K: Hash + Eq + Copy + Display,
	V: Eq + Copy + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}

// TODO: implement
/*
#[cfg(test)]
mod tests {
}
*/
