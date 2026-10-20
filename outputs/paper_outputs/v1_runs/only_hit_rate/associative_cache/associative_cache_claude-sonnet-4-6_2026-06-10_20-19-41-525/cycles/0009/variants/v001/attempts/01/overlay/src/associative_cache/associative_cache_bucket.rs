//! AssociativeCacheBucket with mid-point insertion and position-weighted counter-decay eviction.
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
//! Inherited from cycle/0005/v003:
//! - Counter cap at 4 in `eviction_policy.rs`.
//! - Factor-4 wide-bucket geometry in `geometry.rs`.
//! - Counter-decay pass in warm-lock path.
//!
//! Inherited from cycle/0008/v003:
//! - Decay magnitude of 2 for lower half of bucket items.
//!
//! New in this variant (cycle/0009/H01):
//! - Position-weighted decay: lower half of the bucket (older items) decays by 2,
//!   upper half (newer / recently-accessed items) decays by 1. This preserves
//!   frequency signal for hot items while still converting lukewarm older items
//!   to cold in a single decay event.

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

	/// Evicts an item using cold-first selection with a position-weighted counter-decay fallback.
	///
	/// 1. Scan `self.items` for the first cold item (`lfu_counter == 0`).
	///    If found, remove and return it immediately.
	/// 2. If no cold item exists (warm-lock): apply position-weighted decay —
	///    lower half (older items, index < mid) decay by 2 using `saturating_sub(2)`,
	///    upper half (newer/recently-accessed items, index >= mid) decay by 1 using
	///    `saturating_sub(1)` — then evict the LRU front at index 0.
	///
	/// The position-weighted decay preserves the frequency signal of recently-promoted
	/// hot items (high index, high counter) while still converting older lukewarm items
	/// to cold in a single event, reducing warm-lock recurrence.
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
		// Apply position-weighted counter decay to break the warm-lock:
		// - Lower half (older items closer to eviction front): decay aggressively by 2
		// - Upper half (newer / recently-accessed items): decay conservatively by 1
		let mid = self.items.len() / 2;
		for (i, item) in self.items.iter_mut().enumerate() {
			if i < mid {
				// Lower half: older items closer to eviction front — decay aggressively (rate 2)
				item.lfu_counter = item.lfu_counter.saturating_sub(2);
			} else {
				// Upper half: newer / recently-accessed items — decay conservatively (rate 1)
				item.lfu_counter = item.lfu_counter.saturating_sub(1);
			}
		}

		// Evict the LRU front (index 0); it is now cold (or near-cold) after decay.
		Some(self.remove_at(0))
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

		// Evict if full, using the cold-first + position-weighted decay eviction.
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
