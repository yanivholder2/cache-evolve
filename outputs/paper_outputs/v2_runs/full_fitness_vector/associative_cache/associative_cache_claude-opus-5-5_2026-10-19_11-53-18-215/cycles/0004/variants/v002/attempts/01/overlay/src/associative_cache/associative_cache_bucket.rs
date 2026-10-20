//! H02 (cycle 0004): segmented-LRU (SLRU) bucket with an ARC-style adaptive
//! protected cap, on top of v003's merged-pair layout and ghost ring.
//!
//! One physical bucket merges a pair of logical buckets (`2 * bucket_size`
//! slots when paired, `bucket_size` for the trailing unpaired bucket), backed
//! by one contiguous `Vec` split at `len0`:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - get, protected hit: rotate to protected MRU. If the hit landed in the
//!   LRU quarter of protected, grow `protected_cap` by one (up to `max_cap`).
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key): always inserted at probation MRU (index `len0`). If full,
//!   evict `items[0]` first (probation LRU, or protected LRU when probation is
//!   empty).
//!   - If the victim came from probation, its 16-bit fingerprint is recorded
//!     in the bucket's ghost ring (`max_size` entries, FIFO overwrite).
//!   - If the new key's fingerprint is in the ghost ring (checked before the
//!     eviction is recorded), that ghost slot is cleared and `protected_cap`
//!     shrinks by one (down to `min_cap`), demoting protected LRU lazily by
//!     moving the boundary if protected is now over cap.
//!
//! `protected_cap` adapts within `[min_cap, max_cap]` with
//! `min_cap = max(1, max_size / 4)` and `max_cap = max_size - 1`. Buckets with
//! `max_size <= 2` keep a fixed cap and do not use the ghost ring.
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()`.

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
	/// Current (adaptive) maximum number of items in the protected segment.
	protected_cap: usize,
	/// Lower bound for `protected_cap`.
	min_cap: usize,
	/// Upper bound for `protected_cap` (keeps at least one probation slot).
	max_cap: usize,
	/// Whether `protected_cap` adapts (SLRU and `max_size > 2`).
	adaptive: bool,
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
		// Protected starts at 3/4 of the slots, keeping at least one probation slot.
		let protected_cap = (max_size * 3 / 4).min(max_size.saturating_sub(1));
		let slru = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let adaptive = slru && max_size > 2;
		let (min_cap, max_cap) = if adaptive {
			((max_size / 4).max(1), max_size - 1)
		} else {
			(protected_cap, protected_cap)
		};
		Self {
			items: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap,
			min_cap,
			max_cap,
			adaptive,
			max_size,
			slru,
			ghost: vec![0u16; if adaptive { max_size } else { 0 }].into_boxed_slice(),
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
			// Protected hit. A hit in protected's LRU quarter means protected
			// nearly lost a reused item: grow its cap.
			if self.adaptive && pos < self.len0 + (self.items.len() - self.len0) / 4 {
				self.protected_cap = (self.protected_cap + 1).min(self.max_cap);
			}
			// Move to protected MRU.
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

		let use_ghost = self.adaptive;
		// Ghost lookup happens before this put's eviction is recorded, so a new
		// key can never match the fingerprint of the victim it displaces.
		if use_ghost && self.ghost_take(fp(&item.key)) {
			// Probation was too small to hold this key until its reuse: shrink
			// protected, demoting its LRU lazily by moving the boundary.
			self.protected_cap = self.protected_cap.saturating_sub(1).max(self.min_cap);
			while self.items.len() - self.len0 > self.protected_cap {
				self.len0 += 1;
			}
		}

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

		// Every new key (ghost or not) enters at probation MRU.
		self.items.insert(self.len0, item);
		self.len0 += 1;
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

	fn lru_bucket(bucket_size: usize, paired: bool) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			paired,
		)
	}

	fn check_slots(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size);
		assert!(b.len0 <= b.items.len());
		assert!(b.items.len() - b.len0 <= b.protected_cap);
		assert!(b.protected_cap >= b.min_cap && b.protected_cap <= b.max_cap);
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
	}

	#[test]
	fn bounds_are_computed() {
		let b = lru_bucket(2, true);
		assert_eq!((b.max_size, b.protected_cap, b.min_cap, b.max_cap), (4, 3, 1, 3));
		let b = lru_bucket(8, true);
		assert_eq!((b.max_size, b.protected_cap, b.min_cap, b.max_cap), (16, 12, 4, 15));
		let b = lru_bucket(2, false);
		assert!(!b.adaptive);
		assert_eq!(b.min_cap, b.protected_cap);
		assert_eq!(b.max_cap, b.protected_cap);
	}

	#[test]
	fn ghost_hit_lands_in_probation_and_shrinks_protected() {
		let mut b = lru_bucket(2, true);
		for k in 1..=4i64 {
			assert!(b.pair_put(Item::new(k, k)).is_none());
		}
		assert_eq!(b.len0, 4);
		// Key 5 evicts key 1 from probation -> key 1 goes into the ghost ring.
		let ev = b.pair_put(Item::new(5, 5)).expect("full bucket evicts");
		assert_eq!(ev.key, 1);
		check_slots(&b);
		// Re-putting key 1 is a ghost hit: it lands in probation, cap shrinks.
		let ev = b.pair_put(Item::new(1, 1)).expect("full bucket evicts");
		assert_eq!(ev.key, 2);
		check_slots(&b);
		let pos = b.find(&1).expect("re-put key resident");
		assert!(pos < b.len0, "ghost key must be in probation (pos={pos}, len0={})", b.len0);
		assert_eq!(b.protected_cap, 2);
		assert_eq!(b.items.len(), 4);
	}

	#[test]
	fn ghost_hit_demotes_protected_when_over_cap() {
		let mut b = lru_bucket(2, true);
		for k in 1..=4i64 {
			b.pair_put(Item::new(k, k));
		}
		// Promote 2, 3, 4 -> protected full at cap 3, probation = [1].
		for k in 2..=4i64 {
			assert!(b.pair_get(&k).is_some());
		}
		assert_eq!(b.items.len() - b.len0, 3);
		// 5 evicts 1 (probation) -> ghost.
		assert_eq!(b.pair_put(Item::new(5, 5)).unwrap().key, 1);
		// 1 returns: ghost hit, cap 3 -> 2, protected LRU (2) demoted.
		b.pair_put(Item::new(1, 1));
		check_slots(&b);
		assert_eq!(b.protected_cap, 2);
		assert!(b.find(&1).unwrap() < b.len0);
	}

	#[test]
	fn late_protected_hit_grows_cap() {
		let mut b = lru_bucket(8, true); // 16 slots, cap 12, min 4, max 15
		b.protected_cap = 8;
		for k in 0..16i64 {
			b.pair_put(Item::new(k, k));
		}
		for k in 0..8i64 {
			b.pair_get(&k);
		}
		assert_eq!(b.items.len() - b.len0, 8);
		// Protected LRU is key 0: hitting it is in the LRU quarter.
		b.pair_get(&0);
		assert_eq!(b.protected_cap, 9);
		check_slots(&b);
	}

	#[test]
	fn cap_stays_in_bounds_under_random_ops() {
		for (bs, paired) in [(1usize, false), (2, false), (2, true), (3, true), (8, true), (8, false)] {
			let mut b = lru_bucket(bs, paired);
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15 ^ (bs as u64);
			for _ in 0..20000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let range = if x & 1 == 0 { 3 * b.max_size as u64 } else { b.max_size as u64 / 2 + 1 };
				let k = ((x >> 8) % range) as i64;
				if b.pair_get(&k).is_none() {
					b.pair_put(Item::new(k, k));
				}
				check_slots(&b);
			}
		}
	}

	#[test]
	fn ghost_slots_stay_legal_under_churn() {
		let mut b = lru_bucket(2, true);
		for i in 0..5000i64 {
			let k = (i * 7919) % 23;
			if b.pair_get(&k).is_none() {
				b.pair_put(Item::new(k, k));
			}
			check_slots(&b);
		}
	}
}
