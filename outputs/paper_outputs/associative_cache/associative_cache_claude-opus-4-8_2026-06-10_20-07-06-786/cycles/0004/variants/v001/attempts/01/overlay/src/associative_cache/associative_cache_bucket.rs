use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Number of probationary hits a key must accumulate before it is promoted to
/// the protected segment (moved to the MRU back). Tunable: raising it demands
/// stronger proof of reuse before a key earns protection.
const PROMOTION_THRESHOLD: usize = 2;

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

	// ---- Segmented (SLRU-style) bucket overrides --------------------------------
	//
	// The bucket is partitioned over the same `items` vector using each item's
	// `lfu_counter` as a small saturating reuse count (0, 1, 2). Promotion to the
	// protected segment now requires TWO probationary hits, not one:
	//   * lfu_counter == 0 -> probationary, never hit (new keys live here)
	//   * lfu_counter == 1 -> probationary, hit once but unproven (stays in place)
	//   * lfu_counter >= PROMOTION_THRESHOLD (2) -> protected, at the back (MRU)
	//
	// New keys are admitted to the probationary front (index 0). A hit increments
	// the counter saturating at the threshold; the item is only moved to the
	// protected back once its post-increment count reaches the threshold. A hit
	// that takes a key from 0 -> 1 leaves it in place so it keeps its FIFO age.
	// Eviction prefers the oldest counter==0 item, then the oldest counter==1
	// item, only touching the protected LRU end (index 0) once every resident
	// item is protected.

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

	/// On a hit: bump the reuse counter, saturating at `PROMOTION_THRESHOLD`.
	/// Only when the post-increment count reaches the threshold is the item
	/// promoted (or, if already protected, refreshed) by moving it to the
	/// protected back (MRU). A hit that takes a key from 0 -> 1 leaves it in
	/// place, preserving its FIFO age so it is not yet protected.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.items[position].lfu_counter < PROMOTION_THRESHOLD {
			self.items[position].lfu_counter += 1;
		}
		if self.items[position].lfu_counter >= PROMOTION_THRESHOLD {
			// Promote (or refresh an already-protected key) to the MRU back.
			let item = self.items.remove(position);
			self.items.push(item);
			self.items.last().expect("bucket invariant: just pushed")
		} else {
			// 0 -> 1: unproven, keep its FIFO position.
			&self.items[position]
		}
	}

	/// Evict the *oldest* unproven item, across two probationary sub-levels.
	/// New keys are admitted at index 0 (newest) and age toward higher indices,
	/// so the oldest entry at a given level is its highest index. Prefer the
	/// oldest counter==0 entry (never reused); failing that the oldest counter==1
	/// entry (reused once but unproven); only when every resident item is
	/// protected (counter >= PROMOTION_THRESHOLD) fall back to the protected LRU
	/// end (index 0). This preserves the property that a freshly admitted key
	/// survives a full lap before it can be evicted, while preferring an
	/// unproven key over a twice-hit protected one.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for idx in (0..self.items.len()).rev() {
			if self.items[idx].lfu_counter == 0 {
				return Some(idx);
			}
		}
		for idx in (0..self.items.len()).rev() {
			if self.items[idx].lfu_counter == 1 {
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
