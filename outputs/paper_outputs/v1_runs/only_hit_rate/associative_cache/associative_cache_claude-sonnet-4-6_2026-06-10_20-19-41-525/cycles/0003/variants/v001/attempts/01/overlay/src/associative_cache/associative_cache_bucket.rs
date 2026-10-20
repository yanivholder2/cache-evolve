//! AssociativeCacheBucket with mid-point insertion.
//!
//! **v001 (cycle/0003) hypothesis**: combining cycle/0002/v002's cold-first
//! ClassicLRU eviction with mid-point insertion attacks scan pollution from
//! both sides simultaneously.
//!
//! - Cold-first eviction (inherited from cycle/0002/v002): on eviction, the
//!   first item with `lfu_counter == 0` is preferred, protecting warm items
//!   that have received at least one GET hit.
//! - Mid-point insertion (this file): new items are inserted at position
//!   `len / 2` instead of being appended at the back.  This keeps fresh cold
//!   entries out of the fully-protected MRU tail while still letting them age
//!   past the cold pool so the eviction policy can see them before warm items.
//!
//! Together the two mechanisms should be additive: eviction-side cold-first
//! selection pairs naturally with insertion-side mid-point placement because
//! mid-point insertion never displaces an already-warm MRU item.

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

	/// Puts an item into the bucket using mid-point insertion.
	///
	/// - Existing-key update: unchanged — update value in place, call
	///   `get_from_position` per policy, return.
	/// - New key, bucket full: evict via policy first, then insert at midpoint.
	/// - New key, bucket not full: insert at midpoint directly.
	///
	/// Inserting at the midpoint (`len / 2` after any eviction) keeps new cold
	/// items away from the MRU tail while still placing them ahead of the
	/// coldest front entries, giving cold-first eviction candidates ahead of
	/// them to absorb scan traffic before they age into the warm zone.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing-key update path: unchanged from the trait default.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Evict if full, using the cold-first ClassicLRU policy.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Insert at the midpoint of the post-eviction vector.
		let mid = self.items.len() / 2;
		self.items.insert(mid, item);

		(&self.items[mid], evicted_item)
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
