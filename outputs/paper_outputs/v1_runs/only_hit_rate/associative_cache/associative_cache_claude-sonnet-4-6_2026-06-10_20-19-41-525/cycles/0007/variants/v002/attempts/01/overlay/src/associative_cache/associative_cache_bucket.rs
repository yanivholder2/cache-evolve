//! AssociativeCacheBucket with mid-point insertion, counter-decay eviction,
//! and an ARC-style ghost buffer for re-admission of recently evicted keys.
//!
//! **v002 (cycle/0007) hypothesis**: adding a ghost buffer to the bucket
//! re-admits recently evicted keys with an elevated counter (lfu_counter = 2)
//! and MRU back-insertion, converting regret evictions into protected warm
//! entries on their next access.  This builds on cycle/0005/v003's foundation:
//!
//! Inherited from cycle/0005/v003:
//! - Cold-first eviction: on eviction, the first item with `lfu_counter == 0`
//!   is preferred, protecting warm items that have received at least one GET hit.
//! - Mid-point insertion: new items are inserted at position `len / 2` instead
//!   of being appended at the back.
//! - Counter-decay pass: when no cold candidate exists (all `lfu_counter >= 1`,
//!   i.e., warm-lock), decrement every item's counter by 1, then evict the LRU
//!   front (index 0).
//! - Counter cap at 4 (enforced in eviction_policy.rs).
//!
//! New in this variant:
//! - Ghost buffer: a bounded `VecDeque<K>` recording recently evicted keys.
//!   On a new-key put, if the key appears in the ghost buffer it is re-admitted
//!   at lfu_counter = 2 (warm) and MRU back position, bypassing mid-point
//!   insertion.  The ghost buffer is bounded to `bucket_size` entries.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::VecDeque;
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
	/// Ghost buffer: keys of recently evicted items, bounded to ghost_capacity.
	ghost: VecDeque<K>,
	ghost_capacity: usize,
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
			ghost: VecDeque::new(),
			ghost_capacity: settings.bucket_size,  // ghost list is same size as the bucket
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

	/// Evicts an item using cold-first selection with a counter-decay fallback,
	/// and records the evicted key in the ghost buffer.
	///
	/// 1. Scan `self.items` for the first cold item (`lfu_counter == 0`).
	///    If found, remove it, record its key in the ghost buffer, and return.
	/// 2. If no cold item exists (warm-lock): decrement every item's
	///    `lfu_counter` by 1 (safe — all are >= 1 at this point), then evict
	///    the LRU front at index 0, recording its key in the ghost buffer.
	///
	/// The ghost buffer is bounded to `ghost_capacity` entries; the oldest
	/// ghost entry is dropped when the buffer is full.
	///
	/// Returns `None` only when the bucket is empty.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		// Step 1: find the first cold item (lfu_counter == 0).
		let cold_idx = self.items.iter().position(|item| item.lfu_counter == 0);

		if let Some(i) = cold_idx {
			let evicted = self.remove_at(i);
			// Record evicted key in ghost buffer (bounded).
			if self.ghost.len() >= self.ghost_capacity {
				self.ghost.pop_front();  // drop oldest ghost entry to stay bounded
			}
			self.ghost.push_back(evicted.key);
			return Some(evicted);
		}

		// Step 2: warm-lock — all items have lfu_counter >= 1.
		// Apply one-shot counter decay to break the warm-lock.
		for item in self.items.iter_mut() {
			item.lfu_counter -= 1;
		}

		// Evict the LRU front (index 0); it is now cold (or near-cold) after decay.
		let evicted = self.remove_at(0);
		// Record evicted key in ghost buffer (bounded).
		if self.ghost.len() >= self.ghost_capacity {
			self.ghost.pop_front();  // drop oldest ghost entry to stay bounded
		}
		self.ghost.push_back(evicted.key);
		Some(evicted)
	}

	/// Puts an item into the bucket.
	///
	/// - Existing-key update: update value in place, call `get_from_position`
	///   per policy, return.
	/// - Ghost-hit re-admission: if the key appears in the ghost buffer, remove
	///   it from the ghost, set `lfu_counter = 2` (warm), evict if full, and
	///   push the item at the MRU back position.
	/// - New key, non-ghost: evict if full (cold-first + decay), insert at
	///   midpoint of the post-eviction vector.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing-key update path: unchanged.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// Ghost-hit re-admission path.
		if let Some(ghost_pos) = self.ghost.iter().position(|&k| k == item.key) {
			// Ghost hit: this key was evicted recently and is being re-requested.
			// Re-admit it at the MRU back with an elevated counter so it immediately
			// enters the warm-protected pool and is not cold-first-evicted right away.
			self.ghost.remove(ghost_pos);
			let mut re_item = item;
			re_item.lfu_counter = 2;  // warm, survives two decay events before becoming cold
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(re_item);
			return (&self.items[self.items.len() - 1], evicted_item);
		}

		// Non-ghost path: mid-point insertion + cold-first eviction + bulk decay.
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
