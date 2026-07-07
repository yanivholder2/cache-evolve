//! AssociativeCacheBucket with back-append insertion and counter-decay eviction.
//!
//! **v001 (cycle/0008) hypothesis**: replacing mid-point insertion with O(1)
//! back-append in the incumbent cold-first stack reduces per-op-time without
//! regressing hit-rate, because cold-first eviction and the warm-lock fallback
//! at `remove_at(0)` are both position-independent for newly inserted cold items.
//!
//! Inherited from cycle/0004/v001:
//! - Cold-first eviction: on eviction, the first item with `lfu_counter == 0`
//!   is preferred, protecting warm items that have received at least one GET hit.
//! - Counter-decay pass: when no cold candidate exists (warm-lock), decrement
//!   every item's counter by 1, then evict the LRU front (index 0).
//!
//! Changed in this variant:
//! - Back-append insertion: new items are pushed to the back of the Vec with
//!   `Vec::push` (O(1) amortised) instead of inserted at midpoint with
//!   `Vec::insert(mid, item)` (O(n/2) due to element shifting).
//!
//! Correctness reasoning: new items have `lfu_counter == 0` regardless of
//! insertion position. The cold-first scan finds the *first* cold item from
//! index 0 forward; since older un-hit items cluster near the front (positional
//! LRU from `get_from_position`), those front-cold items are evicted before the
//! newly appended back-cold item. The warm-lock fallback evicts `items[0]`, the
//! oldest item; new items at the back are maximally distant from index 0 and are
//! equally (or more) protected from warm-lock eviction than mid-point items.

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
	///    `lfu_counter` by 1 (safe — all are >= 1 at this point), then evict
	///    the LRU front at index 0.
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

		// Evict the LRU front (index 0); it is now cold (or near-cold) after decay.
		Some(self.remove_at(0))
	}

	/// Puts an item into the bucket using back-append insertion.
	///
	/// - Existing-key update: unchanged — update value in place, call
	///   `get_from_position` per policy, return.
	/// - New key, bucket full: evict via `evict_item` first, then push to back.
	/// - New key, bucket not full: push to back directly.
	///
	/// Back-append is O(1) amortised vs O(n/2) for mid-point insertion.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing-key update path: unchanged from the trait default.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Evict if full, using the cold-first + decay eviction.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Back-append: O(1) amortised, replaces O(n/2) mid-point insert.
		self.items.push(item);
		let back = self.items.len() - 1;

		(&self.items[back], evicted_item)
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
