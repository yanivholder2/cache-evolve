//! H02 (cycle 0003): segmented-LRU (SLRU) bucket on top of v003's merged-pair
//! layout, with cheaper data movement. The SLRU semantics are unchanged from
//! cycle 0002 / v002.
//!
//! One physical bucket still merges a pair of logical buckets
//! (`2 * bucket_size` slots when paired, `bucket_size` for the trailing
//! unpaired bucket), backed by one contiguous `Vec` split at `len0`. The two
//! segments are SLRU segments shared by every key of the physical bucket:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - lookup scans from the MRU end (tail), so hot protected keys are found
//!   after a few compares.
//! - get, protected hit: rotate to protected MRU (no-op if already MRU).
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key) on a full bucket with non-empty probation: one rotate over
//!   probation only evicts `items[0]` and places the new item at probation
//!   MRU. With empty probation: evict protected LRU (`items[0]`) and insert at
//!   probation MRU (index `len0`).
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()`.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
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
	/// Number of items in probation (`items[0..len0]`). In the fallback
	/// (non-SLRU) mode this always equals `items.len()`.
	len0: usize,
	/// Maximum number of items in the protected segment.
	protected_cap: usize,
	/// Total slots in this physical bucket.
	max_size: usize,
	/// Whether SLRU is active (policy is ClassicLRU).
	slru: bool,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// A single (unpaired) bucket of `settings.bucket_size` slots.
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self::with_layout(settings, false)
	}

	/// A paired (`2 * bucket_size` slots) or unpaired (`bucket_size`) bucket.
	pub fn with_layout(settings: &AssociativeCacheSettings, paired: bool) -> Self {
		let max_size = if paired { 2 * settings.bucket_size } else { settings.bucket_size };
		// Protected gets 3/4 of the slots, keeping at least one probation slot.
		let protected_cap = (max_size * 3 / 4).min(max_size.saturating_sub(1));
		Self {
			items: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap,
			max_size,
			slru: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// MRU-first lookup: protected MRU sits at the tail. Keys are unique, so the
	/// result is the same index a forward scan would find.
	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.items.iter().rposition(|d| d.key == *key)
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru {
			let idx = self.eviction_policy.get_from_position(&mut self.items, pos);
			self.len0 = self.items.len();
			return idx;
		}
		let last = self.items.len() - 1;
		if pos >= self.len0 {
			// Protected hit: move to protected MRU (already there if pos == last).
			if pos != last {
				self.items[pos..].rotate_left(1);
			}
			return last;
		}
		if self.protected_cap == 0 {
			// No protected segment (1-slot bucket): plain LRU in probation.
			self.items[pos..self.len0].rotate_left(1);
			return self.len0 - 1;
		}
		// Probation hit: promote to protected MRU in one rotate.
		self.items[pos..].rotate_left(1);
		self.len0 -= 1;
		if self.items.len() - self.len0 > self.protected_cap {
			// Demote protected LRU (now at index len0) to probation MRU.
			self.len0 += 1;
		}
		last
	}

	/// Lookup. Scans the bucket from the MRU end and applies the hit rule.
	pub fn pair_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let pos = self.find(key)?;
		let j = self.hit(pos);
		Some(&self.items[j])
	}

	/// Insert. Updates in place (with the hit rule) if the key is resident.
	/// Returns the item dropped from the bucket, if any.
	pub fn pair_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		if let Some(p) = self.find(&item.key) {
			self.items[p].value = item.value;
			self.hit(p);
			return None;
		}

		if !self.slru {
			let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
				let c = self.eviction_policy.eviction_candidate_index(&self.items);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.len0 = self.items.len();
			return evicted;
		}

		if self.items.len() >= self.max_size && !self.items.is_empty() {
			if self.len0 > 0 {
				// Evict probation LRU and place the new item at probation MRU in
				// one move spanning only probation. len0 is unchanged.
				let old = std::mem::replace(&mut self.items[0], item);
				self.items[0..self.len0].rotate_left(1);
				return Some(old);
			}
			// All protected: evict protected LRU, insert at probation MRU.
			let evicted = self.remove_at(0);
			self.items.insert(self.len0, item);
			self.len0 += 1;
			return Some(evicted);
		}
		self.items.insert(self.len0, item);
		self.len0 += 1;
		None
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		let key = item.key;
		let dropped = self.pair_put(item);
		let pos = self.get_position(&key).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get(key)
	}

	fn get_position(&self, key: &K) -> Option<usize> {
		self.find(key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.hit(position);
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		if index < self.len0 {
			self.len0 -= 1;
		}
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for (i, item) in self.items.iter().enumerate() {
			let seg = if i < self.len0 { "prob" } else { "prot" };
			writeln!(f, "    [{seg}] {}", item)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn keys(b: &AssociativeCacheBucket<i64, i64>) -> Vec<i64> {
		b.items.iter().map(|i| i.key).collect()
	}

	fn paired_lru_bucket() -> AssociativeCacheBucket<i64, i64> {
		// 4 slots, protected cap 3.
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size: 2,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			true,
		)
	}

	#[test]
	fn full_miss_with_probation_evicts_items0_and_keeps_protected() {
		let mut b = paired_lru_bucket();
		b.pair_put(Item::new(1, 1));
		assert!(b.pair_get(&1).is_some()); // promote 1 to protected
		for k in [2, 3, 4] {
			assert!(b.pair_put(Item::new(k, k)).is_none());
		}
		assert_eq!(keys(&b), vec![2, 3, 4, 1]);
		assert_eq!(b.len0, 3);
		let prev0 = b.items[0].key;
		let protected_before: Vec<i64> = b.items[b.len0..].iter().map(|i| i.key).collect();

		let evicted = b.pair_put(Item::new(5, 5)).expect("full bucket evicts");
		assert_eq!(evicted.key, prev0);
		assert_eq!(b.len0, 3);
		assert_eq!(keys(&b), vec![3, 4, 5, 1]);
		let protected_after: Vec<i64> = b.items[b.len0..].iter().map(|i| i.key).collect();
		assert_eq!(protected_before, protected_after);
	}

	#[test]
	fn full_miss_all_protected_evicts_protected_lru() {
		// protected_cap < max_size keeps probation non-empty in normal use, so
		// the all-protected fallback is exercised by setting state directly.
		let mut b = paired_lru_bucket();
		for k in [1, 2, 3, 4] {
			b.pair_put(Item::new(k, k));
		}
		b.len0 = 0; // force all-protected state
		let evicted = b.pair_put(Item::new(9, 9)).unwrap();
		assert_eq!(evicted.key, 1);
		assert_eq!(b.len0, 1);
		assert_eq!(keys(&b), vec![9, 2, 3, 4]);
	}

	#[test]
	fn protected_mru_hit_is_stable() {
		let mut b = paired_lru_bucket();
		b.pair_put(Item::new(1, 1));
		b.pair_put(Item::new(2, 2));
		b.pair_get(&1);
		b.pair_get(&2);
		let before = keys(&b);
		assert_eq!(b.pair_get(&2).unwrap().key, 2);
		assert_eq!(keys(&b), before);
	}
}
