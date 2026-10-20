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

	/// Maximum number of protected (lfu_counter >= 1) items allowed at once.
	/// Capped at half the bucket (at least 1) so at least `max_size - cap`
	/// slots stay probationary, giving freshly admitted keys a longer FIFO
	/// probation lap before they become an eviction candidate.
	fn protected_cap(&self) -> usize {
		(self.max_size / 2).max(1)
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

	// ---- Segmented (SLRU-style) bucket overrides --------------------------------
	//
	// The bucket is partitioned into two regions over the same `items` vector,
	// using each item's `lfu_counter` as a promotion flag:
	//   * lfu_counter == 0 -> probationary (new keys live here, near the front)
	//   * lfu_counter >= 1 -> protected (keys proven by a second access, at the back)
	//
	// New keys are admitted to the probationary front (index 0). A hit on a
	// probationary item promotes it to protected and moves it to the back (MRU).
	// A hit on a protected item just moves it to the back. Eviction prefers the
	// *oldest* probationary item (highest index among lfu_counter == 0), only
	// touching a protected key (the protected LRU end, index 0) once every item
	// is protected.
	//
	// The protected segment is capped at `protected_cap()` (half the bucket).
	// When a promotion would overflow the cap, the oldest protected item is
	// demoted back into probation instead of a proven key being discarded —
	// occupancy is conserved (one demotion plus one promotion, no eviction).

	/// Inserts a *new* key at the probationary front (index 0), evicting one item
	/// first if the bucket is full. Existing keys update in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Admit to the probationary front. `Item::new` already sets lfu_counter = 0.
		self.items.insert(0, item);
		(&self.items[0], evicted_item)
	}

	/// On a hit: promote a probationary item (counter 0 -> 1) and move it to the
	/// protected back; an already-protected item is just moved to the back (MRU),
	/// keeping its counter at 1.
	///
	/// When promoting would push the protected count past `protected_cap()`, the
	/// oldest protected item (lowest index with `lfu_counter >= 1`) is demoted
	/// first: its counter is reset to 0 and it is moved to the probationary front
	/// (index 0), re-entering the FIFO probation queue as the newest probationary
	/// entry. The hit item is then promoted normally — total occupancy is
	/// unchanged and the item returned is still the one at the requested key.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let is_promotion = self.items[position].lfu_counter == 0;
		let mut position = position;
		if is_promotion {
			let protected_count = self.items.iter().filter(|i| i.lfu_counter >= 1).count();
			if protected_count >= self.protected_cap() {
				// The hit item is probationary (counter 0), so it can never be the
				// protected item we demote; indices are therefore distinct.
				if let Some(demote_idx) = self.items.iter().position(|i| i.lfu_counter >= 1) {
					self.items[demote_idx].lfu_counter = 0;
					let demoted = self.items.remove(demote_idx);
					self.items.insert(0, demoted);
					// Reconcile the hit item's index after remove(demote_idx) +
					// insert(0): a demotion below `position` nets zero shift, one
					// above it shifts the hit item up by one.
					if demote_idx > position {
						position += 1;
					}
				}
			}
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
