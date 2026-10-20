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

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-slot frequency counters, kept in a separate contiguous array parallel to
	/// `items` (index i of `counters` belongs to index i of `items`). The counter is
	/// not stored in the item itself.
	counters: Vec<usize>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Count of insertions into this bucket, driving the periodic aging pass.
	insertions: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			counters: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			insertions: 0,
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

	/// Insert a new key (or update an existing one). Slots start with frequency 0,
	/// and the bucket ages all frequency counters every `AGING_PERIOD` insertions.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		self.counters.push(0);
		self.insertions += 1;
		if self.insertions % AGING_PERIOD == 0 {
			for counter in self.counters.iter_mut() {
				*counter >>= 1;
			}
		}
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection: the slot with the smallest frequency counter. Ties go to
	/// the lowest index (first-found minimum). The scan returns early at the first
	/// zero-counter slot, which is the first-found minimum.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let mut best: Option<(usize, usize)> = None; // (index, counter)
		for (index, &counter) in self.counters.iter().enumerate() {
			if counter == 0 {
				return Some(index);
			}
			match best {
				// Strict `<` keeps the first-found minimum on ties.
				Some((_, min_counter)) if min_counter <= counter => {}
				_ => best = Some((index, counter)),
			}
		}
		best.map(|(index, _)| index)
	}

	/// Promotion is delegated to the policy. The policy moves at most one item from
	/// `position` to the returned index (remove+push, swap, or in place); the
	/// parallel counter is rotated the same way so it stays attached to its slot.
	/// The hit then bumps the slot's saturating counter, skipping the store at the cap.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if index > position {
			self.counters[position..=index].rotate_left(1);
		} else if index < position {
			self.counters[index..=position].rotate_right(1);
		}
		if self.counters[index] < FREQ_SATURATION {
			self.counters[index] += 1;
		}
		&self.items[index]
	}

	/// Removes the slot at `index` together with its parallel counter.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.counters.remove(index);
		self.items.remove(index)
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
