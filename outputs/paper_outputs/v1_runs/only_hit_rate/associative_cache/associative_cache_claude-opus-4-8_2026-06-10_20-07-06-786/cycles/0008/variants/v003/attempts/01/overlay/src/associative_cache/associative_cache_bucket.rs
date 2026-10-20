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
	/// H02: number of resident protected items (`lfu_counter >= 1`). The
	/// segmented bucket keeps counter-0 (probationary) items in a contiguous
	/// prefix `[0, k)` and counter-1 (protected) items in a contiguous suffix
	/// `[k, len)`, so `k == len - protected_count`. Maintaining this count lets
	/// `eviction_candidate_index` return in O(1) instead of scanning.
	protected_count: usize,
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
			protected_count: 0,
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

	/// Inserts a *new* key at the probationary front (index 0), evicting one item
	/// first if the bucket is full. Existing keys update in place per policy.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		// Admit to the probationary front. `Item::new` already sets lfu_counter = 0,
		// so `protected_count` is unchanged by a brand-new admission.
		self.items.insert(0, item);
		(&self.items[0], evicted_item)
	}

	/// On a hit: promote a probationary item (counter 0 -> 1) and move it to the
	/// protected back; an already-protected item is just moved to the back (MRU),
	/// keeping its counter at 1.
	///
	/// H03: reserve one probationary slot by capping the protected segment at
	/// `max_size - 1`. A probationary hit promotes only while
	/// `protected_count < max_size - 1`; once the cap is reached, promotion is
	/// *withheld* (counter stays 0, `protected_count` does not grow) and the hit
	/// item is instead given a fresh probation lap by moving it to the most-recent
	/// probationary position (index 0). It is never demoted — already-protected
	/// keys keep their counter — so the proven set is immortal against newcomer
	/// churn while a repeatedly-hit unpromoted key still survives.
	///
	/// Fast path: if the hit item is already at the MRU back (`position == len - 1`)
	/// *and* already protected (`lfu_counter >= 1`), the remove+push would restore a
	/// byte-identical vector and the counter is already promoted — so it is a no-op.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if position == self.items.len() - 1 && self.items[position].lfu_counter >= 1 {
			return &self.items[position];
		}
		if self.items[position].lfu_counter == 0 {
			// Probationary hit.
			if self.protected_count < self.max_size - 1 {
				// Below the cap: promote to protected and move to the MRU back.
				self.items[position].lfu_counter = 1;
				self.protected_count += 1;
				let item = self.items.remove(position);
				self.items.push(item);
				return self.items.last().expect("bucket invariant: just pushed");
			} else {
				// Cap reached: withhold promotion. Keep counter at 0 and give the
				// item a fresh probation lap at the most-recent probationary
				// position (index 0). `protected_count` is unchanged and the
				// prefix/suffix segmentation is preserved.
				let item = self.items.remove(position);
				self.items.insert(0, item);
				return &self.items[0];
			}
		}
		// Already-protected hit not at the back: move to the MRU back, keep counter.
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the candidate and keep `protected_count` in sync: a protected victim
	/// (`lfu_counter >= 1`) — which only happens in the all-protected fallback —
	/// decrements the count; a probationary victim leaves it unchanged. With the
	/// H03 cap, a full bucket always has a reserved probationary slot, so the
	/// victim is always probationary and `protected_count` is left unchanged.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		let idx = self.eviction_candidate_index()?;
		let evicted = self.items.remove(idx);
		if evicted.lfu_counter >= 1 {
			self.protected_count -= 1;
		}
		Some(evicted)
	}

	/// Index of the *oldest* un-promoted probationary item — the highest index
	/// whose `lfu_counter == 0`. By the prefix/suffix invariant the counter-0
	/// items occupy the contiguous prefix `[0, k)` with `k == len - protected_count`,
	/// so that oldest probationary index is exactly `k - 1`. When every resident
	/// item is protected (`protected_count == len`, i.e. `k == 0`) fall back to the
	/// protected LRU end (index 0). With the H03 cap a full bucket can never be
	/// all-protected, so this fallback is unreachable while full and only guards the
	/// not-yet-full edge cases.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			None
		} else if self.protected_count == self.items.len() {
			Some(0)
		} else {
			Some(self.items.len() - self.protected_count - 1)
		}
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
