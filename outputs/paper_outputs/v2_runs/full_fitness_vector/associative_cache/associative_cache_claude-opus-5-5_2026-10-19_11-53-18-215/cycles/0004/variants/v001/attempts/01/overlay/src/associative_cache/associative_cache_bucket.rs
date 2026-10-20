//! Segmented-LRU (SLRU) bucket on the merged-pair layout, with 24-byte items.
//!
//! One physical bucket merges a pair of logical buckets
//! (`2 * bucket_size` slots when paired, `bucket_size` for the trailing
//! unpaired bucket), backed by one contiguous `Vec` split at `len0`. The two
//! segments are SLRU segments shared by every key of the physical bucket:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - get, protected hit: rotate to protected MRU (no-op if already MRU).
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key): if full, evict `items[0]` (probation LRU, or protected LRU
//!   when probation is empty), then place the new item at probation MRU.
//!
//! H01 (cycle 0004) movement fixes:
//! - Lookup scans MRU-first (`rposition`).
//! - A protected hit already at MRU does not rotate.
//! - A full-bucket miss with a non-empty probation overwrites `items[0]` and
//!   rotates only the probation segment, leaving protected untouched.
//! - `put_indexed` returns the final index, so `Bucket::put` need not re-scan.
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
use std::mem;

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

	/// MRU-first lookup: hot keys sit at the protected tail.
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
			// Protected hit: move to protected MRU (already there => no-op).
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

	/// Lookup. Scans the bucket once (MRU-first) and applies the hit rule.
	pub fn pair_get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let pos = self.find(key)?;
		let j = self.hit(pos);
		Some(&self.items[j])
	}

	/// Insert. Updates in place (with the hit rule) if the key is resident.
	/// Returns the item dropped from the bucket, if any.
	#[inline]
	pub fn pair_put(&mut self, item: Item<K, V>) -> Option<Item<K, V>> {
		self.put_indexed(item).1
	}

	/// Body of `pair_put`. Returns `(final index of the key, dropped item)`.
	#[inline]
	fn put_indexed(&mut self, item: Item<K, V>) -> (usize, Option<Item<K, V>>) {
		if let Some(p) = self.find(&item.key) {
			self.items[p].value = item.value;
			let idx = self.hit(p);
			return (idx, None);
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
			return (self.items.len() - 1, evicted);
		}

		if self.items.len() >= self.max_size && !self.items.is_empty() {
			if self.len0 > 0 {
				// Probation-only eviction: overwrite probation LRU and rotate the
				// new item to probation MRU. Protected is untouched; len0 unchanged.
				let old = mem::replace(&mut self.items[0], item);
				self.items[0..self.len0].rotate_left(1);
				return (self.len0 - 1, Some(old));
			}
			// All-protected fallback: evict protected LRU, insert at probation MRU.
			let old = self.remove_at(0);
			let idx = self.len0;
			self.items.insert(idx, item);
			self.len0 += 1;
			return (idx, Some(old));
		}

		let idx = self.len0;
		self.items.insert(idx, item);
		self.len0 += 1;
		(idx, None)
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
		let (idx, dropped) = self.put_indexed(item);
		(&self.items[idx], dropped)
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

	/// One paired bucket: 4 slots, protected cap 3.
	fn bucket4(policy: EvictionPolicyKind) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size: 2,
				eviction_policy: policy,
			},
			true,
		)
	}

	fn keys(b: &AssociativeCacheBucket<i64, i64>) -> Vec<i64> {
		b.items.iter().map(|i| i.key).collect()
	}

	#[test]
	fn full_miss_with_probation_evicts_items0_and_keeps_protected() {
		let mut b = bucket4(EvictionPolicyKind::ClassicLRU);
		for k in 1..=4 {
			assert!(b.pair_put(Item::new(k, k * 10)).is_none());
		}
		assert_eq!(b.len0, 4);
		// Promote 1: [2,3,4 | 1].
		assert_eq!(b.pair_get(&1).unwrap().key, 1);
		assert_eq!(keys(&b), vec![2, 3, 4, 1]);
		assert_eq!(b.len0, 3);
		// Full miss: probation LRU (2) goes; 5 lands at probation MRU.
		let dropped = b.pair_put(Item::new(5, 50));
		assert_eq!(dropped.unwrap().key, 2);
		assert_eq!(keys(&b), vec![3, 4, 5, 1]);
		assert_eq!(b.len0, 3);
		assert_eq!(b.items.len(), 4);
	}

	#[test]
	fn full_miss_all_protected_evicts_protected_lru() {
		let mut b = bucket4(EvictionPolicyKind::ClassicLRU);
		// Force an all-protected full bucket (not reachable via the public API
		// with a 3/4 protected cap, but the fallback must still be correct).
		b.items = (1..=4).map(|k| Item::new(k, k * 10)).collect();
		b.len0 = 0;
		let dropped = b.pair_put(Item::new(9, 90));
		assert_eq!(dropped.unwrap().key, 1);
		assert_eq!(keys(&b), vec![9, 2, 3, 4]);
		assert_eq!(b.len0, 1);
		assert_eq!(b.items.len(), 4);
	}

	#[test]
	fn protected_mru_hit_is_stable() {
		let mut b = bucket4(EvictionPolicyKind::ClassicLRU);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_get(&1); // promote: [2 | 1]
		assert_eq!(keys(&b), vec![2, 1]);
		assert_eq!(b.len0, 1);
		for _ in 0..3 {
			assert_eq!(b.pair_get(&1).unwrap().value, 10);
			assert_eq!(keys(&b), vec![2, 1]);
			assert_eq!(b.len0, 1);
		}
		assert_eq!(b.get_from_position(1).key, 1);
		assert_eq!(keys(&b), vec![2, 1]);
	}

	#[test]
	fn bucket_put_returns_inserted_key_for_every_branch() {
		// SLRU insert (not full).
		let mut b = bucket4(EvictionPolicyKind::ClassicLRU);
		for k in 1..=4 {
			let (it, d) = b.put(Item::new(k, k * 10));
			assert_eq!((it.key, it.value), (k, k * 10));
			assert!(d.is_none());
		}
		// Resident key (probation hit -> promotion).
		let (it, d) = b.put(Item::new(2, 21));
		assert_eq!((it.key, it.value), (2, 21));
		assert!(d.is_none());
		// Resident key (protected MRU hit).
		let (it, _) = b.put(Item::new(2, 22));
		assert_eq!((it.key, it.value), (2, 22));
		// Probation-rotate eviction.
		let (it, d) = b.put(Item::new(7, 70));
		assert_eq!((it.key, it.value), (7, 70));
		assert!(d.is_some());

		// All-protected fallback.
		let mut b = bucket4(EvictionPolicyKind::ClassicLRU);
		b.items = (1..=4).map(|k| Item::new(k, k * 10)).collect();
		b.len0 = 0;
		let (it, d) = b.put(Item::new(8, 80));
		assert_eq!((it.key, it.value), (8, 80));
		assert_eq!(d.unwrap().key, 1);

		// Non-SLRU push (with and without eviction) and resident update.
		let mut b = bucket4(EvictionPolicyKind::Fifo);
		for k in 1..=5 {
			let (it, _) = b.put(Item::new(k, k * 10));
			assert_eq!((it.key, it.value), (k, k * 10));
		}
		assert_eq!(b.len0, b.items.len());
		let (it, d) = b.put(Item::new(3, 33));
		assert_eq!((it.key, it.value), (3, 33));
		assert!(d.is_none());
	}

	#[test]
	fn invariants_hold_under_mixed_workload() {
		for policy in [EvictionPolicyKind::ClassicLRU, EvictionPolicyKind::Lfu] {
			let mut b = bucket4(policy);
			for k in 0..2000i64 {
				let key = (k * 7919) % 13;
				if b.pair_get(&key).is_none() {
					b.pair_put(Item::new(key, key));
				}
				assert!(b.items.len() <= b.max_size);
				assert!(b.len0 <= b.items.len());
				assert!(!b.slru || b.items.len() - b.len0 <= b.protected_cap);
				let mut seen = std::collections::HashSet::new();
				for it in &b.items {
					assert!(seen.insert(it.key));
				}
			}
		}
	}
}
