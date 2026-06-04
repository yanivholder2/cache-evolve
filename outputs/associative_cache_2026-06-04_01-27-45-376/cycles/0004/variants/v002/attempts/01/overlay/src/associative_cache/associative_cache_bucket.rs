use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::classic_lru_eviction_candidate_with_cursor;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::cell::Cell;
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
	/// CLOCK-style rotation cursor across untouched eviction candidates.
	///
	/// Only consulted when the policy is `ClassicLRU`: it shifts the starting
	/// point of the v003 untouched-first scan so that, when multiple slots
	/// have `lfu_counter == 0`, eviction pressure rotates across them instead
	/// of always targeting the leftmost one.
	///
	/// Interior mutability is required because the `Bucket` trait's default
	/// `eviction_candidate_index` borrows `&self`, so the override below
	/// advances the cursor through a `Cell`.
	victim_cursor: Cell<u8>,
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
			victim_cursor: Cell::new(0),
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

	/// Override of the trait default to apply CLOCK-style rotation to
	/// ClassicLRU's untouched-first eviction filter.
	///
	/// For ClassicLRU, the scan starts at a per-bucket rotating cursor
	/// (`self.victim_cursor`) and wraps around, so the leftmost-bias of v003
	/// is spread across untouched slots. For every other policy, this falls
	/// back to delegating to the policy's own
	/// `EvictionPolicy::eviction_candidate_index` exactly as the trait default
	/// would.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if self.eviction_policy.name() == "ClassicLRU" {
			let cursor = self.victim_cursor.get();
			let idx = classic_lru_eviction_candidate_with_cursor(&self.items, cursor);
			// Advance the cursor by 1 modulo `max_size` (saturating to at
			// least 1 to keep `%` defined even in pathological settings).
			let modulus = self.max_size.max(1) as u8;
			// `modulus` is in [1, 255] for any sensible bucket size; the
			// `wrapping_add(1) % modulus` step is well-defined.
			let next = if modulus == 0 { 0 } else { cursor.wrapping_add(1) % modulus };
			self.victim_cursor.set(next);
			Some(idx)
		} else {
			Some(self.eviction_policy.eviction_candidate_index(&self.items))
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
