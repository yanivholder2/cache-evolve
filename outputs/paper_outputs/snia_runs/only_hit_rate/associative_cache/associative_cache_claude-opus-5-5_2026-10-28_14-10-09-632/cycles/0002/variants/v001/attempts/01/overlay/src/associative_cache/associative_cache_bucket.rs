use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::trace;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Newly admitted items are inserted at `len / INSERT_FRACTION_DIV` (measured
/// from the LRU end, index 0) instead of the MRU tail, subject to a probation
/// floor of `MIN_PROBATION_SLOTS` slots (capped at `len / MAX_PROBATION_FRACTION_DIV`).
const INSERT_FRACTION_DIV: usize = 4;
/// Minimum probation depth, in slots, a new item gets before reaching the
/// eviction end.
const MIN_PROBATION_SLOTS: usize = 64;
/// The probation floor never pushes the insertion point past `len / 2`.
const MAX_PROBATION_FRACTION_DIV: usize = 2;

/// Insertion index (from the LRU end) for a newly admitted item, given the
/// bucket's current length (after any eviction):
/// `max(len / 4, min(len / 2, 64))`.
fn insertion_index(len: usize) -> usize {
	std::cmp::max(
		len / INSERT_FRACTION_DIV,
		std::cmp::min(len / MAX_PROBATION_FRACTION_DIV, MIN_PROBATION_SLOTS),
	)
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
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
		&self.items[index]
	}

	/// LRU-insertion-point put with a probation floor: hits on existing keys
	/// behave as the trait default (policy-driven promotion); new items are
	/// inserted at `insertion_index(len)` (from the LRU end) rather than the
	/// MRU tail.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		let ins = insertion_index(self.items.len());
		let idx = if ins == 0 {
			self.items.push(item);
			self.items.len() - 1
		} else {
			self.items.insert(ins, item);
			ins
		};
		(&self.items[idx], evicted_item)
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
