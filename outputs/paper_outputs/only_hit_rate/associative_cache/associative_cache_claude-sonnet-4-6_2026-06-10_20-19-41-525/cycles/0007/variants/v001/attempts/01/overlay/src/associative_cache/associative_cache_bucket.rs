//! AssociativeCacheBucket with mid-point insertion and counter-decay eviction.
//!
//! **v001 (cycle/0004) hypothesis**: adding a counter-decay pass to the
//! cold-first eviction path prevents warm-lock in steady state, improving
//! hit-rate over cycle/0003/v001.
//!
//! Inherited from cycle/0003/v001:
//! - Cold-first eviction: on eviction, the first item with `lfu_counter == 0`
//!   is preferred, protecting warm items that have received at least one GET hit.
//! - Mid-point insertion: new items are inserted at position `len / 2` instead
//!   of being appended at the back.
//!
//! Inherited from cycle/0004/v001 (cycle/0005/v003):
//! - Counter-decay pass: when no cold candidate exists (all `lfu_counter >= 1`,
//!   i.e., warm-lock), decrement every item's counter by 1 (no underflow since
//!   all counters are >= 1), then evict the LRU front (index 0).  This fires
//!   only when the cold pool is exhausted, creating a self-correcting mechanism
//!   as the cache warms up.
//!
//! **New in this variant (cycle/0007/v001 hypothesis)**:
//! - Post-decay cold scan: after the bulk-decay pass, instead of blindly
//!   removing index 0, scan for the first genuinely-cold item (counter == 0)
//!   and evict that.  Falls back to index 0 if no item reached zero after decay
//!   (i.e., all had counter >= 2 before the decay pass).  This aligns the
//!   warm-lock resolution with the same cold-first principle used on the normal
//!   eviction path.

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

	/// Evicts an item using cold-first selection with a counter-decay fallback.
	///
	/// 1. Scan `self.items` for the first cold item (`lfu_counter == 0`).
	///    If found, remove and return it immediately.
	/// 2. If no cold item exists (warm-lock): decrement every item's
	///    `lfu_counter` by 1 (safe — all are >= 1 at this point), then scan
	///    again for the first item that is now genuinely cold (`lfu_counter == 0`).
	///    If a cold item is found after decay, evict it; otherwise fall back to
	///    index 0 (which happens only when all items had `lfu_counter >= 2` before
	///    the decay pass, so all are still warm after it).
	///
	/// Returns `None` only when the bucket is empty (precondition: called only
	/// when `is_full()` is true, so in practice always returns `Some`).
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		// Step 1: find the first cold item (lfu_counter == 0).
		let cold_idx = self.items.iter().position(|item| item.lfu_counter == 0);

		if let Some(i) = cold_idx {
			return Some(self.remove_at(i));
		}

		// Step 2: warm-lock — all items have lfu_counter >= 1.
		// Apply one-shot counter decay to break the warm-lock.
		for item in self.items.iter_mut() {
			item.lfu_counter -= 1;
		}

		// After decay, evict the first item that is now genuinely cold (counter == 0).
		// Falls back to index 0 if no item reached counter == 0 (all had counter >= 2
		// before decay and are still warm after the single decrement).
		let cold_idx = self.items
			.iter()
			.position(|item| item.lfu_counter == 0)
			.unwrap_or(0);
		Some(self.remove_at(cold_idx))
	}

	/// Puts an item into the bucket using mid-point insertion.
	///
	/// - Existing-key update: unchanged — update value in place, call
	///   `get_from_position` per policy, return.
	/// - New key, bucket full: evict via `evict_item` first, then insert at midpoint.
	/// - New key, bucket not full: insert at midpoint directly.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing-key update path: unchanged from the trait default.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Evict if full, using the cold-first + decay eviction.
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
