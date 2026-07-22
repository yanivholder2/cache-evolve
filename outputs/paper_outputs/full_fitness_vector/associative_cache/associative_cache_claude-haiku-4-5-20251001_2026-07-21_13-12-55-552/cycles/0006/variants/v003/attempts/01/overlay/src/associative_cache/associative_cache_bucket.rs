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
	// All items in the bucket (for trait compatibility)
	items: Vec<Item<K, V>>,

	// Hot segment: frequently accessed items (60% of bucket_size)
	// Tracks indices of items in the hot segment (in LRU order)
	hot_indices: Vec<usize>,
	max_hot_size: usize,

	// Cold segment: infrequently accessed items (40% of bucket_size)
	// Tracks indices of items in the cold segment (in FIFO order)
	cold_indices: Vec<usize>,
	max_cold_size: usize,

	// Total maximum size
	max_size: usize,

	// Operation counter for frequency decay (every 32 operations)
	operation_count: u32,

	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let max_hot = (settings.bucket_size * 60) / 100;
		let max_cold = settings.bucket_size - max_hot;

		Self {
			items: Vec::with_capacity(settings.bucket_size),
			hot_indices: Vec::with_capacity(max_hot),
			max_hot_size: max_hot,
			cold_indices: Vec::with_capacity(max_cold),
			max_cold_size: max_cold,
			max_size: settings.bucket_size,
			operation_count: 0,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Find which segment and position a key is in
	fn find_item(&self, key: &K) -> Option<(bool, usize)> {
		// Search in hot_indices
		for (hot_pos, &item_idx) in self.hot_indices.iter().enumerate() {
			if self.items[item_idx].key == *key {
				return Some((true, hot_pos));
			}
		}
		// Search in cold_indices
		for (cold_pos, &item_idx) in self.cold_indices.iter().enumerate() {
			if self.items[item_idx].key == *key {
				return Some((false, cold_pos));
			}
		}
		None
	}

	/// Apply frequency decay if needed
	fn maybe_decay_frequencies(&mut self) {
		self.operation_count += 1;
		if self.operation_count % 32 == 0 {
			for item in &mut self.items {
				item.frequency_count = (item.frequency_count as u32 / 2) as u8;
			}
		}
	}

	/// Try to promote an item from cold to hot
	fn try_promote(&mut self, cold_pos: usize) -> bool {
		if self.cold_indices.is_empty() || cold_pos >= self.cold_indices.len() {
			return false;
		}

		let item_idx = self.cold_indices[cold_pos];
		if self.items[item_idx].frequency_count < 1 {
			return false;
		}

		// If hot is not full, just move the item
		if self.hot_indices.len() < self.max_hot_size {
			self.cold_indices.remove(cold_pos);
			self.hot_indices.push(item_idx);
			return true;
		}

		// Hot is full, find and evict the lowest-frequency item in hot
		let min_hot_pos = self.hot_indices
			.iter()
			.enumerate()
			.min_by_key(|(_, &hot_idx)| self.items[hot_idx].frequency_count)
			.map(|(pos, _)| pos);

		if let Some(evict_pos) = min_hot_pos {
			let promoting_idx = self.cold_indices.remove(cold_pos);
			let evicted_idx = self.hot_indices.remove(evict_pos);
			self.hot_indices.push(promoting_idx);
			self.cold_indices.push(evicted_idx);
			return true;
		}

		false
	}

	/// Evict an item following the hot/cold strategy
	fn evict_one_internal(&mut self) -> Option<Item<K, V>> {
		// Prefer to evict from cold using FIFO (first item)
		if !self.cold_indices.is_empty() {
			let item_idx = self.cold_indices.remove(0);
			let evicted = self.items.remove(item_idx);
			// Adjust indices after removal
			Self::adjust_indices_after_remove(&mut self.hot_indices, &mut self.cold_indices, item_idx);
			return Some(evicted);
		}

		// If cold is empty, check if hot occupancy > 80%
		let hot_occupancy_ratio = self.hot_indices.len() as f64 / self.max_hot_size as f64;
		if hot_occupancy_ratio > 0.8 && !self.hot_indices.is_empty() {
			// Evict the first (least recently used) item from hot
			let item_idx = self.hot_indices.remove(0);
			let evicted = self.items.remove(item_idx);
			// Adjust indices after removal
			Self::adjust_indices_after_remove(&mut self.hot_indices, &mut self.cold_indices, item_idx);
			return Some(evicted);
		}

		None
	}

	/// Helper to adjust indices after removing an item
	fn adjust_indices_after_remove(
		hot_indices: &mut Vec<usize>,
		cold_indices: &mut Vec<usize>,
		removed_idx: usize,
	) {
		hot_indices.retain_mut(|idx| {
			if *idx > removed_idx {
				*idx -= 1;
			}
			true
		});
		cold_indices.retain_mut(|idx| {
			if *idx > removed_idx {
				*idx -= 1;
			}
			true
		});
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
		// Check if key already exists
		if let Some((is_hot, pos)) = self.find_item(&item.key) {
			if is_hot {
				// Update existing hot item
				let item_idx = self.hot_indices[pos];
				self.items[item_idx].value = item.value;
				// Apply ClassicLRU: move to back
				if pos < self.hot_indices.len() - 1 {
					self.hot_indices.remove(pos);
					self.hot_indices.push(item_idx);
				}
				self.maybe_decay_frequencies();
				return (&self.items[item_idx], None);
			} else {
				// Update existing cold item
				let item_idx = self.cold_indices[pos];
				self.items[item_idx].value = item.value;
				self.maybe_decay_frequencies();
				return (&self.items[item_idx], None);
			}
		}

		// New key: determine eviction strategy
		let mut evicted_item = None;

		// If bucket is full, evict
		if self.items.len() >= self.max_size {
			evicted_item = self.evict_one_internal();
		}
		// If bucket is not full but cold is full, still evict from cold
		else if self.cold_indices.len() >= self.max_cold_size {
			if !self.cold_indices.is_empty() {
				let item_idx = self.cold_indices.remove(0);
				let evicted = self.items.remove(item_idx);
				Self::adjust_indices_after_remove(&mut self.hot_indices, &mut self.cold_indices, item_idx);
				evicted_item = Some(evicted);
			}
		}

		// Add new item to cold segment
		let new_idx = self.items.len();
		self.items.push(item);
		self.cold_indices.push(new_idx);
		self.maybe_decay_frequencies();

		(&self.items[new_idx], evicted_item)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.maybe_decay_frequencies();

		if let Some((is_hot, pos)) = self.find_item(key) {
			if is_hot {
				// Found in hot: apply ClassicLRU (move to back)
				let item_idx = self.hot_indices[pos];
				if pos < self.hot_indices.len() - 1 {
					self.hot_indices.remove(pos);
					self.hot_indices.push(item_idx);
				}
				return Some(&self.items[item_idx]);
			} else {
				// Found in cold: increment frequency and try to promote
				let item_idx = self.cold_indices[pos];
				self.items[item_idx].frequency_count = self.items[item_idx].frequency_count.saturating_add(1);

				// If frequency >= 1, try to promote to hot
				if self.items[item_idx].frequency_count >= 1 {
					self.try_promote(pos);
				}

				// Find and return the item (might now be in hot)
				if let Some((is_hot_now, new_pos)) = self.find_item(key) {
					let current_idx = if is_hot_now {
						self.hot_indices[new_pos]
					} else {
						self.cold_indices[new_pos]
					};
					return Some(&self.items[current_idx]);
				}
			}
		}

		None
	}

	fn get_position(&self, key: &K) -> Option<usize> {
		// Return position in the combined items vector
		self.items.iter().position(|d| d.key == *key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if position < self.items.len() {
			let key = self.items[position].key;

			// Apply side effects if item is in hot segment
			if let Some((is_hot, pos)) = self.find_item(&key) {
				if is_hot {
					let item_idx = self.hot_indices[pos];
					// Apply ClassicLRU: move to back
					if pos < self.hot_indices.len() - 1 {
						self.hot_indices.remove(pos);
						self.hot_indices.push(item_idx);
					}
				}
			}
		}

		&self.items[position]
	}

	fn size(&self) -> usize {
		self.items.len()
	}

	fn is_full(&self) -> bool {
		self.size() == self.max_size
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		self.evict_one_internal()
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.size() == 0 {
			return None;
		}

		// Prefer cold segment using FIFO (first item in cold_indices)
		if !self.cold_indices.is_empty() {
			return Some(self.cold_indices[0]);
		}

		// If cold is empty, check if hot occupancy > 80%
		let hot_occupancy_ratio = self.hot_indices.len() as f64 / self.max_hot_size as f64;
		if hot_occupancy_ratio > 0.8 && !self.hot_indices.is_empty() {
			// Return the first (least recently used) item in hot
			return Some(self.hot_indices[0]);
		}

		None
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);

		let evicted = self.items.remove(index);
		Self::adjust_indices_after_remove(&mut self.hot_indices, &mut self.cold_indices, index);
		evicted
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
