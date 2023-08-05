// TODO: finish module documentation
//! Bucket implementation for the Dash cache.
//!
//!
//! # Examples
//!
//! ```
//! ```

use super::item::Item;
use crate::dash_settings::EvictionPolicy;
use std::{
	fmt::{Display, Formatter},
	hash::Hash,
};

#[derive(Debug)]
pub struct Bucket<K, V>
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

impl<K, V> Bucket<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	pub fn new(max_size: usize, eviction_policy: EvictionPolicy) -> Self {
		Bucket {
			// TODO: consider creating a vector with a fixed size for better performance after initialization
			items: Vec::new(),
			max_size,
			eviction_policy,
		}
	}

	/// Removes the key-value pair with the given key from the bucket.
	/// If the bucket is empty or the key is not found, this function does nothing.
	pub fn remove(&mut self, key: &K) {
		if self.items.is_empty() {
			return;
		}

		// Remove the key-value pair with the given key
		self.items.retain(|item| item.key != *key);
	}

	/// Puts an item into the bucket.
	///
	/// Returns a tuple containing a reference to the pushed item and an optional evicted item.
	/// If no item is evicted, the second element of the tuple will be None.
	pub fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Check if the key already exists in the bucket
		if let Some(position) = self.get_position(&item.key) {
			// If the key exists, update item position inside the bucket and return it
			let pushed_item = self.get_and_update_item_in_position(position);
			return (pushed_item, None);
		} else {
			// If the key does not exist, add the item to the bucket
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			let pushed_item = &self.items[self.items.len() - 1];

			return (pushed_item, evicted_item);
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

	/// Updates the item in the bucket according to the eviction policy.
	///
	/// Returns the updated item and None if the item is not in the bucket.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		let position = self.get_position(key)?;
		Some(self.get_and_update_item_in_position(position))
	}

	/// Updates the position of an item in the bucket according to the eviction policy.
	///
	/// Returns a reference to the updated item.
	pub fn get_and_update_item_in_position(&mut self, position: usize) -> &Item<K, V> {
		match self.eviction_policy {
			EvictionPolicy::Fifo | EvictionPolicy::Lifo => &self.items[position],
			EvictionPolicy::Lru => {
				let item = self.items.remove(position);
				self.items.push(item);
				self.items.last().unwrap()
			}
			EvictionPolicy::Lfu => {
				self.items[position].lfu_counter += 1;
				&self.items[position]
			}
		}
	}

	/// Evicts an item from the bucket according to the eviction policy and return it.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		match self.eviction_policy {
			EvictionPolicy::Fifo | EvictionPolicy::Lru => {
				// TODO: this is in O(n). there could be a more performant way to do that
				Some(self.items.remove(0))
			}
			EvictionPolicy::Lifo => self.items.pop(),
			// TODO: implement better
			EvictionPolicy::Lfu => {
				let mut min_lfu_counter = self.items[0].lfu_counter;
				let mut min_lfu_counter_index = 0;
				for (i, item) in self.items.iter().enumerate() {
					if item.lfu_counter < min_lfu_counter {
						min_lfu_counter = item.lfu_counter;
						min_lfu_counter_index = i;
					}
				}
				Some(self.items.remove(min_lfu_counter_index))
			}
		}
	}

	/// Returns the position of the item with the given key, or `None` if the key is not found.
	pub fn get_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Returns whether the bucket is full.
	pub fn is_full(&self) -> bool {
		self.size() == self.max_size
	}

	/// Returns the number of items in the bucket.
	pub fn size(&self) -> usize {
		self.items.len()
	}
}

impl<K, V> Display for Bucket<K, V>
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

/*
// TODO: implement more tests
#[cfg(test)]
mod tests {
	use super::*;
	use crate::dash_settings::DEFAULT_SETTINGS;

	mod get {
		use super::*;

		#[test]
		fn get_item() {
			// Arrange
			let mut bucket = Bucket::new(DEFAULT_SETTINGS.bucket_size, DEFAULT_SETTINGS.eviction_policy);
			let value = 1;
			let item = Item::new(value, value);
			bucket.put(item);

			// Act
			let item = bucket.get_and_update_item(&value).unwrap();

			// Assert
			assert_eq!(item.value, value);
		}
	}

	mod insert {
		use super::*;

		#[test]
		fn insert_one_item() {
			// Assert
			let mut bucket = Bucket::new(DEFAULT_SETTINGS.bucket_size, DEFAULT_SETTINGS.eviction_policy);
			let value = 1;
			let item = Item::new(value, value);

			// Act
			bucket.put(item);

			// Assert
			assert_eq!(bucket.size(), 1);
			assert_eq!(bucket.get_and_update_item(&value).unwrap().value, value);
		}

		#[test]
		fn insert_multiple_items() {
			// Arrange
			let mut bucket = Bucket::new(DEFAULT_SETTINGS.bucket_size, DEFAULT_SETTINGS.eviction_policy);
			let num_of_bucket_items = 5;

			// Act
			for i in 0..num_of_bucket_items {
				let item = Item::new(i, i);
				bucket.put(item);
			}

			// Assert
			assert_eq!(bucket.size(), num_of_bucket_items);
		}

		#[test]
		fn insert_duplicate_items() {
			// Arrange
			let mut bucket = Bucket::new(DEFAULT_SETTINGS.bucket_size, DEFAULT_SETTINGS.eviction_policy);
			let value = 1;
			let num_of_bucket_items = 5;

			// Act
			let item = Item::new(value, value);
			bucket.put(item);
			let res = bucket.put(item);

			// Assert
			assert_eq!(bucket.size(), 1);
			// TODO: assert res as well
		}
	}
}
*/
