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

	/// Cap on the protected segment as a fraction of bucket capacity:
	/// ~80% (a 4/5 split), rounded down, with a floor of 1 so a tiny bucket
	/// can still hold one protected resident. Kept as a small helper so the
	/// protected/probationary ratio is easy to tune.
	fn protected_cap(&self) -> usize {
		((self.max_size * 4) / 5).max(1)
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

	// ---- Segmented (SLRU-style) bucket with a *bounded* protected segment -------
	//
	// The bucket is partitioned into two regions over the same `items` vector,
	// using each item's `lfu_counter` as a promotion flag:
	//   * lfu_counter == 0 -> probationary (new keys live here, near the front)
	//   * lfu_counter >= 1 -> protected (keys proven by a second access, at the back)
	//
	// New keys are admitted to the probationary front (index 0). A hit on a
	// probationary item promotes it to protected and moves it to the back (MRU).
	// A hit on a protected item just moves it to the back. Eviction prefers the
	// probationary item nearest the front, only touching a protected key (the
	// protected LRU end, index 0) once every item is protected.
	//
	// Unlike the unbounded parent, the protected segment is held to a fixed
	// fraction of the bucket (`protected_cap`). When a promotion would push the
	// protected set past the cap, the stalest protected resident is first
	// demoted back to probation, so a fully-proven working set can never crowd
	// out all probationary slots.

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
	/// Promotion is gated by the protected cap: if the protected segment is
	/// already full, the protected LRU member (the lowest-index resident with
	/// `lfu_counter >= 1`, since protected items migrate to the back on hit) is
	/// demoted back to probation first, holding the protected set at a fixed size.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.items[position].lfu_counter == 0 {
			let protected_count = self.items.iter().filter(|i| i.lfu_counter >= 1).count();
			if protected_count >= self.protected_cap() {
				if let Some(lru_idx) = self.items.iter().position(|i| i.lfu_counter >= 1) {
					self.items[lru_idx].lfu_counter = 0;
				}
			}
			self.items[position].lfu_counter = 1;
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the probationary item nearest the front first; fall back to the
	/// protected LRU end (index 0) only when every resident item is protected.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for (idx, item) in self.items.iter().enumerate() {
			if item.lfu_counter == 0 {
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
