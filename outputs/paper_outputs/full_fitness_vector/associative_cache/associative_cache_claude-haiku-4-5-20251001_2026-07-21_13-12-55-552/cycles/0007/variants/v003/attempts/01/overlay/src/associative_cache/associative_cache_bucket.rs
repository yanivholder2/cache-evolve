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
	// Robin Hood hashing: track probe distance for each item
	// probe_distances[i] = how far item i is from its ideal position (0)
	probe_distances: Vec<u8>,
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
			probe_distances: Vec::with_capacity(settings.bucket_size),
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

		// After get_from_position, the items vector may have been reordered (e.g., by ClassicLRU).
		// To maintain Robin Hood invariants, update probe distances to match current positions.
		// This ensures that probe_distance[i] represents how far item i is from the ideal position 0.
		let items_len = self.items.len();
		if self.probe_distances.len() != items_len {
			self.probe_distances.resize(items_len, 0);
		}
		for i in 0..items_len {
			self.probe_distances[i] = i as u8;
		}

		&self.items[index]
	}

	/// Puts an item at the appropriate position using Robin Hood hashing.
	///
	/// Returns `(&inserted, Option<evicted>)`. Evicts per policy if full.
	/// If the key already exists, updates in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			// Key already exists, update in place
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Robin Hood hashing: insert the item with smart placement
		// We need to track the inserted item's key to find it after insertion
		let inserted_key = item.key;
		self.robin_hood_insert(item);

		// Find and return the inserted item (it may not be at the end due to Robin Hood swapping)
		let pushed_item = self.get_items().iter().find(|i| i.key == inserted_key)
			.expect("bucket invariant: just inserted");
		(pushed_item, evicted_item)
	}

	/// Remove and return the item at the given index, with backward-shift for probe distances.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.get_items().len(),
			"remove_at: index {index} out of bounds (len={})",
			self.get_items().len()
		);

		let removed = self.get_items_mut().remove(index);

		// Backward-shift: for items after the deletion, if they have non-zero probe distance,
		// move them backward and decrement their distance counter.
		if index < self.probe_distances.len() {
			self.probe_distances.remove(index);
		}

		// Decrement probe distances for all items after the removed position
		for i in index..self.probe_distances.len() {
			if self.probe_distances[i] > 0 {
				self.probe_distances[i] -= 1;
			}
		}

		removed
	}
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Robin Hood hashing insertion: insert item with smart placement to reduce probe-distance variance.
	///
	/// Start with probe distance 0 (ideal position) and scan forward.
	/// If we find an item with higher probe distance, swap them.
	/// This ensures that less-displaced items are closer to the front.
	fn robin_hood_insert(&mut self, mut item: Item<K, V>) {
		let mut probe_distance: u8 = 0;

		for position in 0..=self.items.len() {
			if position == self.items.len() {
				// Reached the end, append
				self.items.push(item);
				self.probe_distances.push(probe_distance);
				break;
			}

			let existing_distance = self.probe_distances[position];

			if existing_distance < probe_distance {
				// Existing item is less displaced than new item, keep searching
				probe_distance += 1;
			} else {
				// Existing item is equally or more displaced, swap
				let old_item = self.items[position].clone();
				let old_distance = self.probe_distances[position];

				self.items[position] = item;
				self.probe_distances[position] = probe_distance;

				// Continue with the displaced item
				item = old_item;
				probe_distance = old_distance + 1;
			}
		}
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
