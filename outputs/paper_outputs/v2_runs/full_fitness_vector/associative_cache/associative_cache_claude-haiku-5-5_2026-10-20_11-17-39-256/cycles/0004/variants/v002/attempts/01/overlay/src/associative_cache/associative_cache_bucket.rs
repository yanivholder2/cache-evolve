use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Per-slot access frequency saturates here (a 2-bit-style counter).
const FREQ_SATURATION: usize = 3;
/// Every this many insertions into a bucket, all of its frequency counters are halved.
const AGING_PERIOD: usize = 256;
/// Width of the victim-search window: a full-bucket eviction examines only this many slots.
const VICTIM_WINDOW: usize = 8;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Count of insertions into this bucket, driving the periodic aging pass.
	insertions: usize,
	/// Start slot of the current victim-search window. Advanced by the window width
	/// (mod bucket size) after each eviction so successive windows cover the bucket.
	cursor: u8,
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
			insertions: 0,
			cursor: 0,
		}
	}

	/// Advances the window cursor by the window width, modulo the bucket size.
	fn advance_cursor(&mut self) {
		let len = self.max_size;
		if len == 0 {
			return;
		}
		let width = VICTIM_WINDOW.min(len);
		self.cursor = ((self.cursor as usize + width) % len) as u8;
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

	/// Insert a new key (or update an existing one). Slots start with frequency 0,
	/// and the bucket ages all frequency counters every `AGING_PERIOD` insertions.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() {
			let evicted = self.evict_item();
			self.advance_cursor();
			evicted
		} else {
			None
		};
		self.items.push(item);
		self.insertions += 1;
		if self.insertions % AGING_PERIOD == 0 {
			for slot in self.items.iter_mut() {
				slot.lfu_counter >>= 1;
			}
		}
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection over a bounded window: only the `VICTIM_WINDOW` slots starting
	/// at the cursor (wrapping mod bucket size) are examined. The minimum-counter slot
	/// wins; ties go to the first slot in window order (strict `<` keeps the earliest).
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		let width = VICTIM_WINDOW.min(len);
		let start = self.cursor as usize % len;
		let mut best_index = start;
		let mut best_counter = self.items[start].lfu_counter;
		for k in 1..width {
			let index = (start + k) % len;
			let counter = self.items[index].lfu_counter;
			if counter < best_counter {
				best_counter = counter;
				best_index = index;
			}
		}
		Some(best_index)
	}

	/// Promotion is unchanged (delegated to the policy); the hit additionally bumps
	/// the slot's saturating frequency counter.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let item = &mut self.items[index];
		item.lfu_counter = (item.lfu_counter + 1).min(FREQ_SATURATION);
		&self.items[index]
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
