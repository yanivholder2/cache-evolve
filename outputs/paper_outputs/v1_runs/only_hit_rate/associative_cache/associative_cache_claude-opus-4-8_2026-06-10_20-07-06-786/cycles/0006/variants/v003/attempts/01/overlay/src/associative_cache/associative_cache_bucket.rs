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

	// ---- Segmented (SLRU-style) bucket with a two-hit promotion threshold -------
	//
	// The bucket is partitioned over the same `items` vector, using each item's
	// `lfu_counter` as a 0/1/2 promotion tier:
	//   * lfu_counter == 0 -> probationary, never hit
	//   * lfu_counter == 1 -> probationary, hit once (one access is weak evidence
	//                         of reuse in a wide bucket, so it is not yet enough)
	//   * lfu_counter >= 2 -> protected (proven by a *second* access)
	//
	// New keys are admitted to the probationary front (index 0). Each hit bumps
	// the counter (saturating at the cap of 2) and moves the item to the MRU back.
	// Eviction prefers the *oldest* still-probationary item (highest index whose
	// lfu_counter < 2), preserving the FIFO-probation lap; only when every item is
	// protected does it fall back to the protected LRU end (index 0).

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

	/// On a hit: bump the counter saturating up to a cap of 2 (0 -> 1 -> 2, then
	/// stays 2) and move the item to the MRU back. A key becomes "protected" only
	/// once its counter reaches 2 (a *second* hit); a counter of 0 or 1 is still
	/// probationary.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.items[position].lfu_counter < 2 {
			self.items[position].lfu_counter += 1;
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the *oldest* still-probationary item first — the highest index whose
	/// `lfu_counter < 2`. New keys are admitted at index 0 (newest) and age toward
	/// higher indices, so this preserves the FIFO-probation lap: a freshly admitted
	/// key survives a full lap and gets the chance to collect a second hit before
	/// it can become the victim. Fall back to the protected LRU end (index 0) only
	/// when every resident item is protected (`lfu_counter >= 2`).
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for idx in (0..self.items.len()).rev() {
			if self.items[idx].lfu_counter < 2 {
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
