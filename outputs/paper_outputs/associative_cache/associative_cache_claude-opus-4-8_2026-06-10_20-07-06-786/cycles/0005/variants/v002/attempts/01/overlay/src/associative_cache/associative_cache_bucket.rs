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
	/// Bounded FIFO ring of recently-evicted keys (keys only — no values, no
	/// timestamps). A returning key found here skips probation and is admitted
	/// straight into the protected segment.
	ghost_keys: VecDeque<K>,
	/// Fixed capacity bound for `ghost_keys`, tied to `bucket_size` so the
	/// metadata footprint stays tiny.
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
			ghost_keys: VecDeque::with_capacity(settings.bucket_size),
			ghost_capacity: settings.bucket_size,
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

	// ---- Segmented (SLRU-style) bucket overrides with ghost re-entry -----------
	//
	// The bucket is partitioned into two regions over the same `items` vector,
	// using each item's `lfu_counter` as a promotion flag:
	//   * lfu_counter == 0 -> probationary (new keys live here, near the front)
	//   * lfu_counter >= 1 -> protected (keys proven by a second access, at the back)
	//
	// Never-before-seen keys are admitted to the probationary front (index 0). A
	// hit on a probationary item promotes it to protected and moves it to the back
	// (MRU). A hit on a protected item just moves it to the back. Eviction prefers
	// the *oldest* probationary item, only touching a protected key once every item
	// is protected.
	//
	// Ghost re-entry (H02): each eviction records the victim's key in a small,
	// bounded FIFO ring (`ghost_keys`). A key that returns while still remembered
	// there is treated as already-proven — admitted directly to the protected MRU
	// back with `lfu_counter == 1` — so a recurring hot key no longer pays full
	// probation every lap.

	/// Inserts a *new* key, evicting one item first if the bucket is full and
	/// recording the victim in the ghost ring. A returning key remembered in the
	/// ghost ring is admitted as already-proven (protected MRU back,
	/// `lfu_counter == 1`); a never-seen key is admitted probationary (index 0).
	/// Existing keys update in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Remember the victim so a future return can skip probation. Strict FIFO
		// bound keeps the ghost metadata tiny (keys only).
		if let Some(ref evicted) = evicted_item {
			if self.ghost_capacity > 0 {
				self.ghost_keys.push_back(evicted.key);
				while self.ghost_keys.len() > self.ghost_capacity {
					self.ghost_keys.pop_front();
				}
			}
		}
		// Re-admission: a key remembered in the ghost ring is already-proven.
		if let Some(ghost_pos) = self.ghost_keys.iter().position(|k| *k == item.key) {
			self.ghost_keys.remove(ghost_pos);
			let mut proven = item;
			proven.lfu_counter = 1;
			self.items.push(proven);
			let last = self.items.len() - 1;
			return (&self.items[last], evicted_item);
		}
		// Admit to the probationary front. `Item::new` already sets lfu_counter = 0.
		self.items.insert(0, item);
		(&self.items[0], evicted_item)
	}

	/// On a hit: promote a probationary item (counter 0 -> 1) and move it to the
	/// protected back; an already-protected item is just moved to the back (MRU),
	/// keeping its counter at 1.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.items[position].lfu_counter == 0 {
			self.items[position].lfu_counter = 1;
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the *oldest* un-promoted probationary item first. New keys are
	/// admitted at index 0 (newest) and age toward higher indices, so the oldest
	/// probationary entry is the highest index whose `lfu_counter == 0`. This
	/// makes probation a FIFO queue: a freshly admitted key survives a full lap
	/// through the bucket — earning a chance at promotion — before it can become
	/// the victim. Fall back to the protected LRU end (index 0) only when every
	/// resident item is protected.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for idx in (0..self.items.len()).rev() {
			if self.items[idx].lfu_counter == 0 {
				return Some(idx);
			}
		}
		Some(0)
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
