use crate::settings::EvictionPolicy;
use crate::shared::item::Item;
use std::{fmt::Debug, hash::Hash, time::Instant};

pub trait Bucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	// ------------ struct expected fields ----------------------------------------------

	/// Returns the items vector which contains the items in the bucket.
	fn get_items(&self) -> &Vec<Item<K, V>>;

	/// Returns the items vector which contains the items in the bucket.
	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>>;

	/// Returns the maximum size of the bucket.
	/// The maximum size should be lower than or equal to usize::MAX.
	fn get_max_size(&self) -> usize {
		usize::MAX
	}

	/// Returns the eviction policy of the bucket.
	fn get_eviction_policy(&self) -> &EvictionPolicy;

	// ----------------------------------------------------------------------------------

	/// Puts an item into the bucket.
	///
	/// Returns a tuple containing a reference to the pushed item and an optional evicted item.
	/// If no item is evicted, the second element of the tuple will be None.
	/// As a side effect, if the item already exists makes updates according to the eviction policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Check if the key already exists in the bucket
		// TODO: the key should not exist in the bucket. Consider returning an error
		if let Some(position) = self.get_position(&item.key) {
			// If the key exists, update item position inside the bucket and return it
			let pushed_item = self.get_from_position(position);

			(pushed_item, None)
		} else {
			// If the key does not exist, add the item to the bucket
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			let pushed_item = self.put_according_to_policy(item);

			(pushed_item, evicted_item)
		}
	}

	fn put_according_to_policy(&mut self, item: Item<K, V>) -> &Item<K, V> {
		match self.get_eviction_policy() {
			EvictionPolicy::Fifo
			| EvictionPolicy::Lifo
			| EvictionPolicy::ClassicLRU
			| EvictionPolicy::TimestampLRU
			| EvictionPolicy::Lfu => {
				self.get_items_mut().push(item);
				&self.get_items().last().unwrap()
			}
		}
	}

	/// Removes the key-value pair with the given key from the bucket.
	/// If the bucket is empty or the key is not found, this function does nothing.
	fn remove(&mut self, key: &K) {
		self.get_items_mut().retain(|item| item.key != *key);
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key)?;
		Some(self.get_from_position(position))
	}

	/// Returns the position of the item with the given key, or `None` if the key is not found.
	fn get_position(&self, key: &K) -> Option<usize> {
		self.get_items().iter().position(|d| d.key == *key)
	}

	/// Returns a reference to the item located in `position`.
	///
	/// As a side effect makes updates according to the eviction policy.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		match self.get_eviction_policy() {
			EvictionPolicy::Fifo | EvictionPolicy::Lifo => &self.get_items()[position],
			EvictionPolicy::ClassicLRU | EvictionPolicy::TimestampLRU => self.get_and_update_lru_item(position),
			EvictionPolicy::Lfu => {
				self.get_items_mut()[position].lfu_counter += 1;
				&self.get_items()[position]
			}
		}
	}

	/// Returns a reference to the item in position `position`, or `None` if the item is not found.
	/// As a side effect makes updates to support the LRU eviction policy.
	fn get_and_update_lru_item(&mut self, position: usize) -> &Item<K, V> {
		match self.get_eviction_policy() {
			EvictionPolicy::ClassicLRU => {
				let item = self.get_items_mut().remove(position);
				self.get_items_mut().push(item);
				&self.get_items().last().unwrap()
			}
			EvictionPolicy::TimestampLRU => {
				self.get_items_mut()[position].timestamp = Instant::now();
				&self.get_items()[position]
			}
			_ => panic!("This function should only be called with LRU eviction policies"),
		}
	}

	/// Evicts an item from the bucket according to the eviction policy and return it.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.get_items().is_empty() {
			return None;
		}

		match self.get_eviction_policy() {
			EvictionPolicy::ClassicLRU => {
				// TODO: this is in O(n). there could be a more performant way to do that
				Some(self.get_items_mut().remove(0))
			}
			EvictionPolicy::TimestampLRU => {
				let (min_timestamp_index, _) = self
					.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.timestamp.elapsed())
					.unwrap();
				Some(self.get_items_mut().remove(min_timestamp_index))
			}
			EvictionPolicy::Fifo => {
				// TODO: this is in O(n). there could be a more performant way to do that
				Some(self.get_items_mut().remove(0))
			}
			EvictionPolicy::Lifo => self.get_items_mut().pop(),
			EvictionPolicy::Lfu => {
				let (min_lfu_counter_index, _) = self
					.get_items()
					.iter()
					.enumerate()
					.min_by_key(|(_, item)| item.lfu_counter)
					.unwrap();
				Some(self.get_items_mut().remove(min_lfu_counter_index))
			}
		}
	}

	/// Returns whether the bucket is full.
	fn is_full(&self) -> bool {
		self.size() == self.get_max_size()
	}

	/// Returns the number of items in the bucket.
	fn size(&self) -> usize {
		self.get_items().len()
	}
}

// TODO: fix this to use blanket implementation
/*
// This is a simple implementation of the Display trait for the Bucket trait.
// This means that all types that implement the Bucket trait will also implement the Display trait.
impl<T, K, V> Display for T
where
	T: Bucket<K, V>,
	K: Hash + Eq + Copy,
	V: Eq + Copy + Debug,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in self.get_items() {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
*/
