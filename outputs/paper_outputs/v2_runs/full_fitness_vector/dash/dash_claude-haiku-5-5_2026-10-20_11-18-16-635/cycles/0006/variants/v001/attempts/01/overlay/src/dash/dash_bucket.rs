use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::{
	fmt::{Debug, Display, Formatter},
	hash::Hash,
};

/// Saturation cap for the per-item access counter (`Item::lfu_counter`) that the
/// segment bumps on each target-bucket hit. Victim selection prefers the lowest
/// counter, so the cap keeps the signal coarse: a few hits separate an item from a
/// one-shot arrival without letting old popularity pin it forever.
pub const HIT_COUNTER_CAP: usize = 3;

#[derive(Debug)]
pub struct DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Cached lowest-counter resident as `(counter, slot)`, where `slot` is the
	/// first position holding that counter. `None` means the cache is invalid and
	/// must be rebuilt by a scan before it can be trusted.
	min_cache: Option<(usize, usize)>,
}

impl<K, V> DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(max_size: usize, eviction_policy: Box<dyn EvictionPolicy<K, V>>) -> Self {
		DashBucket {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			min_cache: None,
		}
	}

	/// Full scan for the lowest counter; ties resolve to the first position.
	fn scan_min(&self) -> Option<(usize, usize)> {
		let mut best: Option<(usize, usize)> = None;
		for (idx, item) in self.items.iter().enumerate() {
			let better = match best {
				Some((c, _)) => item.lfu_counter < c,
				None => true,
			};
			if better {
				best = Some((item.lfu_counter, idx));
			}
		}
		best
	}

	/// Keeps the cache consistent after an item with `counter` was inserted at
	/// `front` (index 0) or at the back. Must be called after the insert.
	fn note_insert(&mut self, counter: usize, at_front: bool, was_empty: bool) {
		if was_empty {
			self.min_cache = Some((counter, 0));
			return;
		}
		let Some((c, s)) = self.min_cache else {
			return;
		};
		let at = if at_front { 0 } else { self.items.len() - 1 };
		// A front insert shifts every existing resident one slot to the right.
		let shifted = if at_front { s + 1 } else { s };
		self.min_cache = if counter < c || (counter == c && at < shifted) {
			Some((counter, at))
		} else {
			Some((c, shifted))
		};
	}
}

pub fn new_bucket<K, V>(
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
) -> Box<dyn Bucket<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	Box::new(DashBucket::new(max_size, eviction_policy))
}

impl<K, V> Bucket<K, V> for DashBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	/// Hands out raw mutable access, so the cached minimum can no longer be trusted.
	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		self.min_cache = None;
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// The policy may reorder or mutate counters here, so the cache is invalidated.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.min_cache = None;
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Same semantics as the trait default, with cache maintenance on the update path.
	fn put_at(&mut self, item: Item<K, V>, pos: InsertPosition) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() {
			if self.min_cache.is_none() {
				self.min_cache = self.scan_min();
			}
			self.evict_item()
		} else {
			None
		};
		let at_front = pos == InsertPosition::Front;
		let counter = item.lfu_counter;
		let was_empty = self.items.is_empty();
		if at_front {
			self.items.insert(0, item);
		} else {
			self.items.push(item);
		}
		self.note_insert(counter, at_front, was_empty);
		let pushed_item = if at_front { self.items.first() } else { self.items.last() };
		let pushed_item = pushed_item.expect("bucket invariant: just inserted");
		(pushed_item, evicted_item)
	}

	/// Removal keeps the cached slot valid when the removed item is not the cached one.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		let removed = self.items.remove(index);
		self.min_cache = match self.min_cache {
			Some((c, s)) if index < s => Some((c, s - 1)),
			Some((_, s)) if index == s => None,
			other => other,
		};
		removed
	}

	/// Frequency-aware victim selection: evict the resident with the lowest access
	/// counter. Ties go to the policy's own candidate (the parent's existing order),
	/// so with no counter signal this reduces to the policy's eviction choice.
	/// Reads the cached minimum; falls back to a scan when the cache is invalid.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		let policy_idx = self.eviction_policy.eviction_candidate_index(&self.items);
		let (min_counter, min_slot) = self.min_cache.or_else(|| self.scan_min())?;
		if self.items[policy_idx].lfu_counter == min_counter {
			return Some(policy_idx);
		}
		Some(min_slot)
	}
}

impl<K, V> Display for DashBucket<K, V>
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
