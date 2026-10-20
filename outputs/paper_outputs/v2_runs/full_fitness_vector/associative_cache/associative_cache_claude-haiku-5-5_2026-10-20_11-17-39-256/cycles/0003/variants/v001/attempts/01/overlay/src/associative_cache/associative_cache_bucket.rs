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
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Count of insertions into this bucket, driving the periodic aging pass.
	insertions: usize,
	/// Cached index of the minimum-counter slot (the first such slot on ties, matching
	/// the scan's tie-break). `None` means the cache is invalid and must be rebuilt by a scan.
	min_index: Option<usize>,
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
			min_index: None,
		}
	}

	/// Full scan for the first slot with the smallest frequency counter.
	fn scan_min_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(index, _)| index)
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
		// Arbitrary mutation through this handle may move or re-count slots: drop the cache.
		self.min_index = None;
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
			// Use the cached minimum if valid; otherwise scan once and use the result.
			let victim = match self.min_index {
				Some(index) => index,
				None => self.scan_min_index().expect("bucket invariant: full bucket is non-empty"),
			};
			// remove_at invalidates the cache: Vec::remove shifts later slots.
			Some(self.remove_at(victim))
		} else {
			None
		};
		self.items.push(item);
		self.insertions += 1;
		if self.insertions % AGING_PERIOD == 0 {
			for slot in self.items.iter_mut() {
				slot.lfu_counter >>= 1;
			}
			self.min_index = None;
		} else if let Some(min) = self.min_index {
			// The new slot is last; it becomes the minimum only if strictly smaller
			// (on ties the earlier index keeps the win).
			let new_index = self.items.len() - 1;
			if self.items[new_index].lfu_counter < self.items[min].lfu_counter {
				self.min_index = Some(new_index);
			}
		}
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection: the slot with the smallest frequency counter, first index on ties.
	/// Uses the cached minimum when valid, otherwise scans.
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.min_index.or_else(|| self.scan_min_index())
	}

	/// Promotion is unchanged (delegated to the policy); the hit additionally bumps
	/// the slot's saturating frequency counter.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let item = &mut self.items[index];
		item.lfu_counter = (item.lfu_counter + 1).min(FREQ_SATURATION);
		// Counters only rise on a hit, so the cached minimum stays valid unless the hit
		// slot was the cached minimum. A policy that moved items (index != position)
		// invalidates every cached position.
		if index != position || self.min_index == Some(index) {
			self.min_index = None;
		}
		&self.items[index]
	}

	/// Removes the slot at `index`. Shifts later slots, so the cache is dropped.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		self.min_index = None;
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
