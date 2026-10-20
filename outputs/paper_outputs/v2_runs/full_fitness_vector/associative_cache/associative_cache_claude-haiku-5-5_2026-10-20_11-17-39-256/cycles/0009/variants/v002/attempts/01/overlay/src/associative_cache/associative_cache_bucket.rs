use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Per-slot access frequency saturates here (a 2-bit-style counter).
const FREQ_SATURATION: usize = 3;
/// One halving epoch elapses every this many insertions. Each bucket keeps its own
/// clock (its insertion count), so the epoch is per bucket and halving is lazy:
/// a bucket's counters are only brought up to date when the bucket is next touched.
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
	/// Count of insertions into this bucket; the current epoch is `insertions / AGING_PERIOD`.
	insertions: usize,
	/// Number of halving epochs already applied to this bucket's frequency counters.
	epoch: usize,
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
			epoch: 0,
		}
	}

	/// Lazily halves this bucket's frequency counters by the number of epochs that
	/// elapsed since the bucket was last brought up to date. A no-op unless an epoch
	/// boundary has been crossed, so the common path is a single comparison.
	fn catch_up_epoch(&mut self) {
		let target = self.insertions / AGING_PERIOD;
		if target > self.epoch {
			let elapsed = target - self.epoch;
			// Counters are small and saturate at FREQ_SATURATION, so clamping the shift
			// to the word width only guards against overflow; the result is zero either way.
			let shift = elapsed.min(usize::BITS as usize - 1);
			for slot in self.items.iter_mut() {
				slot.lfu_counter >>= shift;
			}
			self.epoch = target;
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

	/// Insert a new key (or update an existing one). Slots start with frequency 0.
	/// Any pending epoch halving is applied first, so victim selection sees the
	/// same counters the eager schedule would have produced.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		self.catch_up_epoch();
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		self.insertions += 1;
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection: the slot with the smallest frequency counter. Ties go to
	/// the lowest index (`min_by_key` returns the first minimum), which is the
	/// baseline's front-of-bucket choice order.
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(index, _)| index)
	}

	/// Promotion is unchanged (delegated to the policy); the hit additionally bumps
	/// the slot's saturating frequency counter.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.catch_up_epoch();
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
