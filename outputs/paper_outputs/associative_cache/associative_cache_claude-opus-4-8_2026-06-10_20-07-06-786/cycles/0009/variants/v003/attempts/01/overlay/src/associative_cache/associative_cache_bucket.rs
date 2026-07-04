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
	///
	/// H03: the dominant steady-state admission — a full bucket whose victim is
	/// the *oldest probationary* item (the boundary slot, when
	/// `protected_count < len`) — is folded into a single in-place
	/// rotate-and-replace instead of `evict_item` (an `items.remove(idx)`) plus
	/// `items.insert(0, item)` (a full-length right shift). The post-state is
	/// byte-identical: new probationary item at index 0, the former prefix
	/// `[0, idx)` shifted to `[1, idx]`, the oldest-probationary victim removed,
	/// the protected suffix untouched, and `protected_count` unchanged (both the
	/// dropped victim and the new item are probationary). The all-protected
	/// fallback (`protected_count == len`, victim at index 0) keeps the original
	/// remove+insert path.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		if self.is_full() {
			let len = self.items.len();
			if self.protected_count < len {
				// Boundary eviction fast path: the oldest probationary item sits at
				// `idx = len - protected_count - 1`. Rotating the prefix `[0..=idx]`
				// right by one moves that victim to index 0 while shifting the former
				// prefix `[0, idx)` up to `[1, idx]`; the protected suffix `(idx, len)`
				// is untouched and the vector length never changes. A single
				// `mem::replace` then overwrites the victim slot with the new
				// probationary item and hands back the evicted item.
				let idx = len - self.protected_count - 1;
				self.items[..=idx].rotate_right(1);
				let evicted = std::mem::replace(&mut self.items[0], item);
				return (&self.items[0], Some(evicted));
			}
			// All-protected fallback: victim is the protected LRU end (index 0).
			let evicted_item = self.evict_item();
			self.items.insert(0, item);
			return (&self.items[0], evicted_item);
		}
		// Not full: admit to the probationary front. `Item::new` already sets
		// lfu_counter = 0, so `protected_count` is unchanged by a fresh admission.
		self.items.insert(0, item);
		(&self.items[0], None)
	}

	/// On a hit: promote a probationary item (counter 0 -> 1) and move it to the
	/// protected back; an already-protected item is just moved to the back (MRU),
	/// keeping its counter at 1.
	///
	/// Fast path (H03): if the hit item is already at the MRU back
	/// (`position == len - 1`) *and* already protected (`lfu_counter >= 1`), the
	/// remove+push below would restore a byte-identical vector and the counter is
	/// already promoted — so the whole operation is a no-op. Return the item in
	/// place, skipping the O(bucket_size) memmove. This is provably hit-rate- and
	/// state-neutral.
	///
	/// H02: a promotion (counter 0 -> 1) is the only place `protected_count` grows
	/// on a hit; the MRU-protected fast path and protected-on-protected hits leave
	/// it unchanged.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if position == self.items.len() - 1 && self.items[position].lfu_counter >= 1 {
			return &self.items[position];
		}
		if self.items[position].lfu_counter == 0 {
			self.items[position].lfu_counter = 1;
			self.protected_count += 1;
		}
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Evict the candidate and keep `protected_count` in sync: a protected victim
	/// (`lfu_counter >= 1`) — which only happens in the all-protected fallback —
	/// decrements the count; a probationary victim leaves it unchanged.
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
	/// protected LRU end (index 0). This is the O(1) arithmetic equivalent of the
	/// parent's O(bucket_size) reverse scan (H02).
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
