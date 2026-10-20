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

	// ---- Segmented (SLRU-style) bucket overrides --------------------------------
	//
	// The bucket is partitioned into two regions over the same `items` vector,
	// using each item's `lfu_counter` as a promotion flag *and* a frequency tally:
	//   * lfu_counter == 0 -> probationary (new keys live here, near the front)
	//   * lfu_counter >= 1 -> protected (keys proven by a second access, at the back)
	//
	// New keys are admitted to the probationary front (index 0). A hit on a
	// probationary item promotes it to protected and moves it to the back (MRU).
	// A hit on a protected item also moves it to the back and *increments* its
	// frequency tally. Eviction prefers the *oldest* probationary item (highest
	// index among lfu_counter == 0); only when every item is protected does it
	// fall back to evicting the least-frequently-used protected key (ties broken
	// toward the LRU end, lowest index).

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

	/// On a hit: increment the item's frequency tally (saturating) and move it to
	/// the protected back (MRU). The first hit takes a probationary item from
	/// counter 0 -> 1 — the promotion event; each subsequent hit keeps
	/// incrementing, banking true reuse frequency into the counter while leaving
	/// recency ordering of the vector unchanged from the parent (every hit still
	/// moves the item to the MRU back).
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.items[position].lfu_counter = self.items[position].lfu_counter.saturating_add(1);
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the *oldest* un-promoted probationary item first. New keys are
	/// admitted at index 0 (newest) and age toward higher indices, so the oldest
	/// probationary entry is the highest index whose `lfu_counter == 0`. This
	/// makes probation a FIFO queue: a freshly admitted key survives a full lap
	/// through the bucket — earning a chance at promotion — before it can become
	/// the victim.
	///
	/// When every resident item is protected (all `lfu_counter >= 1`), fall back
	/// to evicting the *least-frequently-used* protected key — the minimum
	/// `lfu_counter` — breaking ties toward the oldest position (lowest index,
	/// the protected LRU end). This banks the dormant frequency tally into a live
	/// eviction signal exactly where the parent was blind, so the coldest
	/// protected key dies rather than merely the least-recently-touched one.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for idx in (0..self.items.len()).rev() {
			if self.items[idx].lfu_counter == 0 {
				return Some(idx);
			}
		}
		// Fully protected: choose the least-frequently-used item, ties to the
		// lowest index (LRU end). Iterating ascending and using strict `<` keeps
		// the first (lowest-index) minimum.
		let mut victim = 0usize;
		let mut min_count = self.items[0].lfu_counter;
		for idx in 1..self.items.len() {
			if self.items[idx].lfu_counter < min_count {
				min_count = self.items[idx].lfu_counter;
				victim = idx;
			}
		}
		Some(victim)
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
