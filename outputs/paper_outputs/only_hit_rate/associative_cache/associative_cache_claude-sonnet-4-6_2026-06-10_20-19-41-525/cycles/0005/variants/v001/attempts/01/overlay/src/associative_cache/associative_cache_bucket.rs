//! AssociativeCacheBucket with mid-point insertion, counter-decay eviction,
//! and LFU-minimum victim selection after decay.
//!
//! **v001 (cycle/0005) hypothesis**: after the counter-decay pass in the
//! warm-lock branch, scan for the item with the *lowest* post-decay
//! `lfu_counter` instead of blindly evicting the positional LRU front
//! (`remove_at(0)`).  This stacks LFU selectivity onto the decay path from
//! cycle/0004/v001, so that items whose counters were already at 1 (and are
//! now 0 after decay) are evicted in preference to higher-counter hot items
//! that happened to sit at index 0.
//!
//! Inherited from cycle/0004/v001:
//! - Cold-first eviction: on eviction, the first item with `lfu_counter == 0`
//!   is preferred, protecting warm items that have received at least one GET hit.
//! - Mid-point insertion: new items are inserted at position `len / 2` instead
//!   of being appended at the back.
//! - Counter-decay pass: when no cold candidate exists (warm-lock), decrement
//!   every item's counter by 1 before selecting a victim.
//!
//! New in this variant:
//! - After decay, select the victim by minimum `lfu_counter` (ties broken by
//!   lowest index = earliest in recency order = most LRU among equals) rather
//!   than unconditionally evicting index 0.

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

	/// Evicts an item using cold-first selection with a counter-decay + LFU
	/// fallback.
	///
	/// 1. Scan `self.items` for the first cold item (`lfu_counter == 0`).
	///    If found, remove and return it immediately.
	/// 2. If no cold item exists (warm-lock): decrement every item's
	///    `lfu_counter` by 1 (safe — all are >= 1 at this point), then evict
	///    the item with the lowest post-decay `lfu_counter`.  Ties are broken
	///    by lowest index (earliest in recency order = most LRU among equals).
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

		// Evict the item with the lowest post-decay lfu_counter, breaking ties
		// by lowest index (earliest in recency order = most LRU among equals).
		let victim = self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(i, _)| i)
			.unwrap_or(0);
		Some(self.remove_at(victim))
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

		// Evict if full, using the cold-first + decay + LFU-min eviction.
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
