use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::trace;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Newly admitted items are inserted at `len / INSERT_FRACTION_DIV` (measured
/// from the LRU end, index 0) instead of the MRU tail. LRU-insertion-point
/// scheme: a new item must be hit again before it drifts the remaining
/// ~1/INSERT_FRACTION_DIV of the bucket down to the eviction end.
const INSERT_FRACTION_DIV: usize = 4;

/// Absolute probation floor: a new item gets at least this many slots of
/// probation before reaching the eviction end (subject to the fraction cap).
const MIN_PROBATION_SLOTS: usize = 64;

/// Cap on the probation floor: never insert deeper than `len / 2`.
const MAX_PROBATION_FRACTION_DIV: usize = 2;

/// Bimodal insertion (BIP): every `BIMODAL_PERIOD`-th new item admitted to a
/// bucket is inserted at the MRU tail instead of the probation point, giving a
/// deterministic sample of long-reuse-distance items a full bucket lifetime.
const BIMODAL_PERIOD: u32 = 32;

/// Probation insertion index for a bucket currently holding `len` items:
/// `max(len / 4, min(len / 2, 64))`. Always `<= len`.
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
	/// Per-bucket count of new-item insertions (drives the bimodal choice).
	insert_count: u32,
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
			insert_count: 0,
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

	/// Bimodal LRU-insertion-point put with a probation floor: hits on existing
	/// keys behave as the trait default (policy-driven promotion); new items are
	/// inserted at `insertion_index(len)` (from the LRU end), except every
	/// `BIMODAL_PERIOD`-th new item, which goes to the MRU tail.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.insert_count = self.insert_count.wrapping_add(1);
		let idx = if self.insert_count % BIMODAL_PERIOD == 0 {
			self.items.push(item);
			self.items.len() - 1
		} else {
			let ins = insertion_index(self.items.len());
			if ins == 0 {
				self.items.push(item);
				self.items.len() - 1
			} else {
				self.items.insert(ins, item);
				ins
			}
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
