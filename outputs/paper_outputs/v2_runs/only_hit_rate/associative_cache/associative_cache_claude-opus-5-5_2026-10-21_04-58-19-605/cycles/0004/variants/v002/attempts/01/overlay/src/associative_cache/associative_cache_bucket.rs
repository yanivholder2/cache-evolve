use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Insertion depth for new (missed) keys under ClassicLRU, as a fraction of the
/// bucket length measured from the eviction end (index 0). 1/8 = an eighth of
/// the way up from the eviction end (close to, but not quite, pure LIP).
const INSERT_DEPTH_NUM: usize = 1;
const INSERT_DEPTH_DEN: usize = 8;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// When true, new keys are inserted at INSERT_DEPTH_NUM/INSERT_DEPTH_DEN of
	/// the bucket instead of the MRU tail (LIP/BIP-style), and hits use a
	/// two-step (SLRU-like) promotion. Only enabled for ClassicLRU.
	midpoint_insert: bool,
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
			midpoint_insert: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
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
		if !self.midpoint_insert {
			let index = self.eviction_policy.get_from_position(&mut self.items, position);
			return &self.items[index];
		}

		// Two-step promotion: first hit since insertion -> bucket midpoint;
		// second and later hits -> MRU tail. A hit only moves an item, so the
		// bucket size is unchanged.
		let mut it = self.items.remove(position);
		if it.lfu_counter == 0 {
			it.lfu_counter = 1;
			let mid = self.items.len() / 2;
			self.items.insert(mid, it);
			&self.items[mid]
		} else {
			it.lfu_counter = 1;
			self.items.push(it);
			let last = self.items.len() - 1;
			&self.items[last]
		}
	}

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: baseline behaviour (update value, promote per policy).
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Capacity check runs before the insert, so size never exceeds max_size.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		if !self.midpoint_insert {
			// Baseline miss path: push to the tail.
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		// Depth insertion: computed after the eviction. With a full 16-slot
		// bucket, len = 15 here, so idx = 1. Newcomers start un-hit (probation).
		let idx = self.items.len() * INSERT_DEPTH_NUM / INSERT_DEPTH_DEN;
		item.lfu_counter = 0;
		self.items.insert(idx, item);
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
