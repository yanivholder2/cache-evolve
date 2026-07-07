//! SLRU bucket: combines a two-queue (probationary / protected) structure
//! with cold-first eviction and decay-by-2 warm-lock, all confined to the
//! probationary segment.
//!
//! Layout of `items`:
//!   `items[0..prob_count]`   — probationary segment, index 0 = oldest
//!   `items[prob_count..]`    — protected segment, index `prob_count` = oldest
//!
//! Capacities (50/50 split):
//!   `prob_cap  = max_size / 2`
//!   `prot_cap  = max_size - prob_cap`

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
	/// Number of items in the probationary segment (items[0..prob_count]).
	prob_count: usize,
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
			prob_count: 0,
		}
	}

	/// Maximum number of probationary slots.
	fn prob_cap(&self) -> usize {
		self.max_size / 2
	}

	/// Maximum number of protected slots.
	fn prot_cap(&self) -> usize {
		self.max_size - self.prob_cap()
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

	/// SLRU put:
	/// - Existing key: update value, call `get_from_position`, return.
	/// - New key, full: evict first.
	/// - New key: insert at back of probationary (`items[prob_count]`).
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: update value in-place, then apply SLRU promotion.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		// New key: evict if full.
		let evicted_item = if self.is_full() { self.evict_item() } else { None };

		// Insert at back of probationary segment (index == prob_count before insert).
		self.items.insert(self.prob_count, item);
		self.prob_count += 1;

		// The newly inserted item sits at index prob_count - 1.
		let pushed_item = &self.items[self.prob_count - 1];
		(pushed_item, evicted_item)
	}

	/// SLRU access: increment lfu_counter (capped at 4), then either promote a
	/// probationary hit to protected or refresh a protected hit within protected.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		// Increment frequency counter, capped at 4.
		self.items[position].lfu_counter =
			self.items[position].lfu_counter.saturating_add(1).min(4);

		if position < self.prob_count {
			// ── Probationary hit: promote to protected ──────────────────────────
			let item = self.items.remove(position);
			self.prob_count -= 1;

			// If protected is at capacity, demote its oldest item to
			// probationary-front (index 0) to make room.
			let prot_cap = self.prot_cap();
			if self.items.len() - self.prob_count == prot_cap {
				// items[prob_count] is the oldest protected item after the remove.
				let demoted = self.items.remove(self.prob_count);
				self.items.insert(0, demoted);
				self.prob_count += 1;
			}

			// Push promoted item to protected-back (end of items).
			self.items.push(item);
		} else {
			// ── Protected hit: refresh within protected ──────────────────────────
			let item = self.items.remove(position);
			self.items.push(item);
			// prob_count is unchanged.
		}

		// The accessed item was just pushed to the back.
		self.items.last().expect("get_from_position: items cannot be empty")
	}

	/// SLRU eviction policy:
	///
	/// 1. Cold-first scan in probationary: evict the first item with
	///    `lfu_counter == 0`.
	/// 2. Warm-lock: if all probationary items are warm, apply
	///    `saturating_sub(2)` decay across the entire probationary segment,
	///    then evict the oldest probationary item.
	/// 3. Fallback: if probationary is empty, evict the oldest protected item.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		// Step 1: cold-first scan within probationary.
		if let Some(i) = self.items[0..self.prob_count]
			.iter()
			.position(|item| item.lfu_counter == 0)
		{
			let evicted = self.items.remove(i);
			self.prob_count -= 1;
			return Some(evicted);
		}

		// Step 2: warm-lock — decay all probationary items, then evict oldest.
		if self.prob_count > 0 {
			for item in self.items[0..self.prob_count].iter_mut() {
				item.lfu_counter = item.lfu_counter.saturating_sub(2);
			}
			let evicted = self.items.remove(0);
			self.prob_count -= 1;
			return Some(evicted);
		}

		// Step 3: probationary empty — evict oldest protected item.
		// prob_count remains 0.
		Some(self.items.remove(0))
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
