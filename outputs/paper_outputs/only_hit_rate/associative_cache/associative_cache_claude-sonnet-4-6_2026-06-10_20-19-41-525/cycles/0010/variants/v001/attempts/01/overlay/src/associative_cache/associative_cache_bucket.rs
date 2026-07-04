//! AssociativeCacheBucket with mid-point insertion and value-split counter-decay eviction.
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
//! - Decay magnitude of 2 for lukewarm items (counter <= 2).
//!
//! New in this variant (cycle/0010/H01):
//! - Value-split decay: items with counter <= 2 (lukewarm tier) decay by 2
//!   (collapsing to cold in one pass), while items with counter >= 3 (hot tier)
//!   decay by only 1, preserving the frequency signal and reducing spurious
//!   hot-item evictions after warm-lock recovery.

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

	/// Evicts an item using cold-first selection with a value-split counter-decay fallback.
	///
	/// 1. Scan `self.items` for the first cold item (`lfu_counter == 0`).
	///    If found, remove and return it immediately.
	/// 2. If no cold item exists (warm-lock): apply value-split bulk decay:
	///    - Lukewarm tier (counter <= 2): decay by 2 via `saturating_sub(2)`,
	///      collapsing counter 1 or 2 directly to 0 (cold) in a single pass.
	///    - Hot tier (counter >= 3): decay by 1 via `saturating_sub(1)`,
	///      preserving the frequency signal: counter 3 → 2, counter 4 → 3.
	///    Then evict the LRU front at index 0.
	///
	/// The value-split ensures the cold pool grows quickly from lukewarm items
	/// while hot items (counter 3–4) retain enough signal to avoid spurious
	/// eviction on the very next eviction call after warm-lock recovery.
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
		// Apply value-split counter decay to break the warm-lock:
		// - Lukewarm tier (counter <= 2): decay by 2; counter 1 → 0, counter 2 → 0.
		// - Hot tier (counter >= 3): decay by 1; counter 3 → 2, counter 4 → 3.
		for item in self.items.iter_mut() {
			if item.lfu_counter <= 2 {
				// Lukewarm tier: aggressive decay — collapse to cold in one pass.
				item.lfu_counter = item.lfu_counter.saturating_sub(2);
			} else {
				// Hot tier: conservative decay — preserve frequency signal.
				item.lfu_counter = item.lfu_counter.saturating_sub(1);
			}
		}

		// Evict the LRU front (index 0).
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

		// Evict if full, using the cold-first + value-split decay eviction.
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
