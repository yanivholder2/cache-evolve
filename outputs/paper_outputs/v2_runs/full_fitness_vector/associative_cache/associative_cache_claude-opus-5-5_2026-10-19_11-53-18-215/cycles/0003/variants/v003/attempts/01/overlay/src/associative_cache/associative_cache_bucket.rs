//! H03: segmented-LRU (SLRU) bucket with a per-bucket ghost list, on top of
//! v002's merged-pair layout.
//!
//! One physical bucket merges a pair of logical buckets (`2 * bucket_size`
//! slots when paired, `bucket_size` for the trailing unpaired bucket), backed
//! by one contiguous `Vec` split at `len0`:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - get, protected hit: rotate to protected MRU.
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key): if full, evict `items[0]` (probation LRU, or protected LRU
//!   when probation is empty).
//!   - If the victim came from probation, its 16-bit fingerprint is recorded
//!     in the bucket's ghost ring (`max_size` entries, FIFO overwrite).
//!   - If the new key's fingerprint is in the ghost ring (checked before the
//!     eviction is recorded), that ghost slot is cleared and the key is
//!     inserted as protected MRU (demoting protected LRU if over cap).
//!     Otherwise it is inserted at probation MRU (index `len0`).
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()`. The ghost list is unused when `protected_cap == 0`.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Ghost fingerprint: high hash bits (the low bits picked the bucket), with
/// the low bit forced on so `0` stays free as the "empty" sentinel.
#[inline]
fn fp<K: Hash + Debug>(key: &K) -> u16 {
	(((hash(key) as u64) >> 32) as u16) | 1
}

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
	/// Ring of fingerprints of keys recently evicted from probation (0 = empty).
	ghost: Box<[u16]>,
	/// Next ghost slot to overwrite.
	ghost_head: usize,
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
			ghost: vec![0u16; max_size].into_boxed_slice(),
			ghost_head: 0,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru {
			let idx = self.eviction_policy.get_from_position(&mut self.items, pos);
			self.len0 = self.items.len();
			return idx;
		}
		if pos >= self.len0 {
			// Protected hit: move to protected MRU.
			self.items[pos..].rotate_left(1);
			return self.items.len() - 1;
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
		self.items.len() - 1
	}

	/// Scans the ghost ring for `f`; on a match clears that slot and returns true.
	#[inline]
	fn ghost_take(&mut self, f: u16) -> bool {
		if let Some(slot) = self.ghost.iter_mut().find(|g| **g == f) {
			*slot = 0;
			true
		} else {
			false
		}
	}

	#[inline]
	fn ghost_record(&mut self, f: u16) {
		if self.ghost.is_empty() {
			return;
		}
		self.ghost[self.ghost_head] = f;
		self.ghost_head = (self.ghost_head + 1) % self.ghost.len();
	}

	/// Lookup. Scans the whole bucket once and applies the hit rule.
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

		let use_ghost = self.protected_cap > 0;
		// Ghost lookup happens before this put's eviction is recorded, so a new
		// key can never match the fingerprint of the victim it displaces.
		let ghost_hit = use_ghost && self.ghost_take(fp(&item.key));

		// Evict first so capacity is never exceeded.
		let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
			// items[0] is probation LRU, or protected LRU when probation is empty.
			let from_probation = self.len0 > 0;
			let victim = self.remove_at(0);
			if use_ghost && from_probation {
				self.ghost_record(fp(&victim.key));
			}
			Some(victim)
		} else {
			None
		};

		if ghost_hit {
			// Re-admitted ghost: straight to protected MRU.
			self.items.push(item);
			if self.items.len() - self.len0 > self.protected_cap {
				// Demote protected LRU (index len0) to probation MRU.
				self.len0 += 1;
			}
		} else {
			self.items.insert(self.len0, item);
			self.len0 += 1;
		}
		evicted
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

	fn four_slot_lru() -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size: 2,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			true,
		)
	}

	fn check_slots(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size);
		assert!(b.len0 <= b.items.len());
		assert!(b.items.len() - b.len0 <= b.protected_cap);
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
	}

	#[test]
	fn ghost_reinsert_lands_in_protected() {
		let mut b = four_slot_lru();
		assert_eq!(b.max_size, 4);
		assert_eq!(b.protected_cap, 3);
		for k in 1..=4i64 {
			assert!(b.pair_put(Item::new(k, k)).is_none());
		}
		assert_eq!(b.len0, 4);
		// Key 5 evicts key 1 from probation -> key 1 goes into the ghost ring.
		let ev = b.pair_put(Item::new(5, 5)).expect("full bucket evicts");
		assert_eq!(ev.key, 1);
		check_slots(&b);
		// Re-putting key 1 is a ghost hit: it lands in the protected segment.
		let ev = b.pair_put(Item::new(1, 1)).expect("full bucket evicts");
		assert_eq!(ev.key, 2);
		check_slots(&b);
		let pos = b.find(&1).expect("re-put key resident");
		assert!(pos >= b.len0, "ghost key must be protected (pos={pos}, len0={})", b.len0);
		assert_eq!(b.items.len(), 4);
	}

	#[test]
	fn ghost_slots_stay_legal_under_churn() {
		let mut b = four_slot_lru();
		for i in 0..5000i64 {
			let k = (i * 7919) % 23;
			if b.pair_get(&k).is_none() {
				b.pair_put(Item::new(k, k));
			}
			check_slots(&b);
		}
	}
}
