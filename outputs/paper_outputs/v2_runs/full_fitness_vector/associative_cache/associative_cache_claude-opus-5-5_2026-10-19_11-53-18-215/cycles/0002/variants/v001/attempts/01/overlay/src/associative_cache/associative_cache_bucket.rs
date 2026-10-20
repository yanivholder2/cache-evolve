//! H01 (cycle 0002): merged physical bucket with one plain policy.
//!
//! One physical bucket covers two adjacent logical buckets, so it holds
//! `2 * bucket_size` slots (`bucket_size` for the odd trailing bucket). Total
//! capacity stays the same as the baseline's `n` buckets of `bucket_size`.
//! Items live in one contiguous `Vec`, and the eviction policy orders the whole
//! bucket. There are no halves, no promotion, and no demotion.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Total slots in this physical bucket.
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// A single (unpaired) bucket of `settings.bucket_size` slots.
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self::with_layout(settings, false, 1)
	}

	/// A merged (`2 * bucket_size`) or single (`bucket_size`) bucket. The
	/// divisor argument is kept for signature compatibility and is unused.
	pub fn with_layout(settings: &AssociativeCacheSettings, paired: bool, _divisor: usize) -> Self {
		let max_size = if paired { 2 * settings.bucket_size } else { settings.bucket_size };
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Applies the policy's access side effect to the item at `position`.
	/// Returns the item's new index.
	#[inline]
	fn touch(&mut self, position: usize) -> usize {
		self.eviction_policy.get_from_position_in_slice(&mut self.items[..], position)
	}

	/// Lookup: one linear scan over the whole bucket. `_h` is unused.
	pub fn pair_get(&mut self, key: &K, _h: usize) -> Option<&Item<K, V>> {
		let pos = self.find(key)?;
		let new_pos = self.touch(pos);
		Some(&self.items[new_pos])
	}

	/// Insert: update in place if present; otherwise evict the policy's
	/// candidate when full, then append. Returns the dropped item, if any.
	pub fn pair_put(&mut self, item: Item<K, V>, _h: usize) -> Option<Item<K, V>> {
		if let Some(p) = self.find(&item.key) {
			self.items[p].value = item.value;
			self.touch(p);
			return None;
		}
		let dropped = if !self.items.is_empty() && self.items.len() >= self.max_size {
			let c = self.eviction_policy.eviction_candidate_index(&self.items);
			Some(self.items.remove(c))
		} else {
			None
		};
		self.items.push(item);
		dropped
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
		let key = item.key;
		let h = hash(&key);
		let dropped = self.pair_put(item, h);
		let pos = self.get_position(&key).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get(key, 0)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.touch(position);
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in self.items.iter() {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
