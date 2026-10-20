//! H03 (cycle 0004): ghost-guided insertion position on v001's SLRU.
//! H03 (cycle 0005): `Item` shrinks to 16 bytes (`{ key, value }`). The SLRU
//! path below is unchanged; it was always purely positional. The non-SLRU
//! fallback keeps per-item policy state (LFU counters) in `meta`, a `Vec<u32>`
//! parallel to `items`.
//! H01 (cycle 0006): protected cap raised from 3/4 to 7/8 of the slots, with a
//! probation floor of 1 slot (2 slots for buckets of 16 or more).
//!
//! One physical bucket still merges a pair of logical buckets
//! (`2 * bucket_size` slots when paired, `bucket_size` for the trailing
//! unpaired bucket), backed by one contiguous `Vec` split at `len0`:
//!
//! - `items[0..len0]` is **probation** (index 0 = LRU, `len0 - 1` = MRU).
//! - `items[len0..]` is **protected** (index `len0` = LRU, last = MRU).
//!
//! - get, protected hit: rotate to protected MRU.
//! - get, probation hit: promote to protected MRU. If protected then exceeds
//!   `protected_cap`, its LRU item is demoted to probation's MRU end by moving
//!   the boundary (`len0 += 1`). A get never drops an item.
//! - put (new key): a per-bucket ring of 16-bit fingerprints remembers
//!   recently evicted probation keys.
//!   - **ghost hit**: clear the ghost entry, evict `items[0]` if full, insert
//!     at probation MRU (index `len0`). Never inserted into protected.
//!   - **ghost miss (cold key)**: insert at probation middle `len0 / 2`. If
//!     full, `items[0]` (probation LRU) is evicted and the new item is placed
//!     at the middle with one short rotate; `len0` stays unchanged.
//!   - all-protected fallback (`len0 == 0`): v001 behaviour.
//!   Every eviction of a probation item records its fingerprint in the ring.
//!
//! SLRU applies only when the configured policy is `ClassicLRU`. Any other
//! policy uses a single-segment fallback through the boxed policy, with
//! `len0 == items.len()` and `meta.len() == items.len()`.

use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use crate::shared::utils::hash;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// 16-bit key fingerprint: upper 16 bits of the key hash, with 0 mapped to 1
/// so that 0 marks an empty ghost slot.
#[inline]
fn fingerprint<K: Hash + Debug>(key: &K) -> u16 {
	let h = hash(key) as u64;
	let fp = (h >> 48) as u16;
	if fp == 0 {
		1
	} else {
		fp
	}
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
	/// Ring of fingerprints of recently evicted probation keys (0 = empty).
	ghosts: Vec<u16>,
	/// Next ring slot to overwrite.
	ghost_head: usize,
	/// Fallback-only per-item policy metadata (e.g. LFU counters), parallel to
	/// `items`: `meta[i]` belongs to `items[i]`. Always empty when `slru`.
	meta: Vec<u32>,
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
		// Protected gets 7/8 of the slots, keeping a probation floor scaled to
		// bucket size: 4 -> 3, 8 -> 7, 16 -> 14, 32 -> 28; 1 slot -> 0.
		let probation_min = if max_size >= 16 { 2 } else { 1 };
		let protected_cap = (max_size * 7 / 8).min(max_size.saturating_sub(probation_min));
		let slru = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let ghost_len = if slru { (max_size.saturating_add(1) / 2).max(1) } else { 0 };
		let meta = if slru { Vec::new() } else { Vec::with_capacity(max_size) };
		Self {
			items: Vec::with_capacity(max_size),
			len0: 0,
			protected_cap,
			max_size,
			slru,
			ghosts: vec![0u16; ghost_len],
			ghost_head: 0,
			meta,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn find(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Fallback only: re-establish `meta.len() == items.len()` in case `items`
	/// was edited externally through `get_items_mut`. New entries get 0.
	#[inline]
	fn sync_meta(&mut self) {
		if self.meta.len() != self.items.len() {
			self.meta.resize(self.items.len(), 0);
		}
	}

	/// Records the fingerprint of an evicted probation key in the ghost ring.
	#[inline]
	fn record_ghost(&mut self, key: &K) {
		if self.ghosts.is_empty() {
			return;
		}
		self.ghosts[self.ghost_head] = fingerprint(key);
		self.ghost_head += 1;
		if self.ghost_head == self.ghosts.len() {
			self.ghost_head = 0;
		}
	}

	/// Returns whether `fp` is in the ghost ring, clearing the entry if so.
	#[inline]
	fn take_ghost(&mut self, fp: u16) -> bool {
		match self.ghosts.iter().position(|&g| g == fp) {
			Some(i) => {
				self.ghosts[i] = 0;
				true
			}
			None => false,
		}
	}

	/// Evicts `items[0]` (probation LRU, or protected LRU when probation is
	/// empty), recording a ghost if it came from probation.
	#[inline]
	fn evict_front(&mut self) -> Item<K, V> {
		let from_probation = self.len0 > 0;
		let old = self.remove_at(0);
		if from_probation {
			self.record_ghost(&old.key);
		}
		old
	}

	/// Applies the hit rule to the item at `pos`. Returns its new index.
	#[inline]
	fn hit(&mut self, pos: usize) -> usize {
		if !self.slru {
			self.sync_meta();
			let idx = self
				.eviction_policy
				.get_from_position_meta(self.items.as_mut_slice(), self.meta.as_mut_slice(), pos);
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
			self.sync_meta();
			let evicted = if self.items.len() >= self.max_size && !self.items.is_empty() {
				let c = self.eviction_policy.eviction_candidate_index_meta(&self.items, &self.meta);
				self.meta.remove(c);
				Some(self.items.remove(c))
			} else {
				None
			};
			self.items.push(item);
			self.meta.push(0);
			self.len0 = self.items.len();
			return evicted;
		}

		let full = self.items.len() >= self.max_size && !self.items.is_empty();
		let ghost_hit = self.take_ghost(fingerprint(&item.key));

		if ghost_hit || self.len0 == 0 {
			// Ghost hit, or all-protected fallback: v001 behaviour (insert at
			// probation MRU after evicting items[0] if full).
			let evicted = if full { Some(self.evict_front()) } else { None };
			self.items.insert(self.len0, item);
			self.len0 += 1;
			return evicted;
		}

		// Cold key with a non-empty probation: insert at probation middle.
		let mid = self.len0 / 2;
		if full {
			// Evict probation LRU and slide the new item to `mid` in one move.
			// Probation keeps `len0` items (len0 - 1 survivors + the new one).
			let old = std::mem::replace(&mut self.items[0], item);
			self.items[0..=mid].rotate_left(1);
			self.record_ghost(&old.key);
			Some(old)
		} else {
			self.items.insert(mid, item);
			self.len0 += 1;
			None
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		let key = item.key;
		let dropped = self.pair_put(item);
		// Re-find by key: the item may sit at probation MRU, probation middle,
		// or (for a resident update) protected MRU.
		let pos = self.find(&key).expect("bucket invariant: just inserted");
		(&self.items[pos], dropped)
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		self.pair_get(key)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.hit(position);
		&self.items[index]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if !self.slru && self.meta.len() == self.items.len() {
			return Some(self.eviction_policy.eviction_candidate_index_meta(&self.items, &self.meta));
		}
		Some(self.eviction_policy.eviction_candidate_index(&self.items))
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
		if !self.slru && index < self.meta.len() {
			self.meta.remove(index);
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

	/// One paired SLRU bucket of 8 slots (protected cap 7, 4 ghost slots).
	fn bucket8() -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 2,
				bucket_size: 4,
				eviction_policy: EvictionPolicyKind::ClassicLRU,
			},
			true,
		)
	}

	/// One unpaired fallback bucket of `size` slots with `policy`.
	fn fallback(size: usize, policy: EvictionPolicyKind) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::with_layout(
			&AssociativeCacheSettings {
				num_of_buckets: 1,
				bucket_size: size,
				eviction_policy: policy,
			},
			false,
		)
	}

	fn check_structure(b: &AssociativeCacheBucket<i64, i64>) {
		assert!(b.items.len() <= b.max_size);
		assert!(b.len0 <= b.items.len());
		assert!(b.items.len() - b.len0 <= b.protected_cap);
		let mut seen = std::collections::HashSet::new();
		for it in &b.items {
			assert!(seen.insert(it.key), "duplicate key {}", it.key);
		}
	}

	#[test]
	fn protected_cap_is_seven_eighths_with_scaled_floor() {
		for (bucket_size, paired, expected) in [
			(1usize, false, 0usize),
			(2, false, 1),
			(2, true, 3),
			(4, true, 7),
			(8, true, 14),
			(16, true, 28),
		] {
			let b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::with_layout(
				&AssociativeCacheSettings {
					num_of_buckets: 2,
					bucket_size,
					eviction_policy: EvictionPolicyKind::ClassicLRU,
				},
				paired,
			);
			assert_eq!(b.protected_cap, expected, "max_size {}", b.max_size);
		}
	}

	#[test]
	fn cold_key_lands_at_probation_middle() {
		let mut b = bucket8();
		for k in 1..=5i64 {
			let len0 = b.len0;
			b.pair_put(Item::new(k, k));
			assert_eq!(b.find(&k), Some(len0 / 2), "cold key {k} not at len0/2");
			check_structure(&b);
		}
		// Full bucket: cold key still lands at the middle, len0 unchanged.
		for k in 6..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		assert_eq!(b.items.len(), 8);
		let len0 = b.len0;
		let evicted = b.pair_put(Item::new(100, 100));
		assert!(evicted.is_some());
		assert_eq!(b.len0, len0);
		assert_eq!(b.find(&100), Some(len0 / 2));
		check_structure(&b);
	}

	#[test]
	fn evicted_then_reput_lands_at_probation_mru() {
		let mut b = bucket8();
		for k in 1..=8i64 {
			b.pair_put(Item::new(k, k));
		}
		let victim = b.items[0].key;
		let evicted = b.pair_put(Item::new(1000, 1000)).expect("full bucket evicts");
		assert_eq!(evicted.key, victim);
		assert!(b.find(&victim).is_none());
		b.pair_put(Item::new(victim, victim));
		assert_eq!(b.find(&victim), Some(b.len0 - 1), "ghost hit must land at probation MRU");
		check_structure(&b);
	}

	#[test]
	fn long_scan_never_evicts_protected() {
		let mut b = bucket8();
		let hot = [1i64, 2, 3];
		for &k in &hot {
			b.pair_put(Item::new(k, k));
		}
		for &k in &hot {
			assert!(b.pair_get(&k).is_some());
		}
		assert_eq!(b.len0, 0);
		for k in 10_000..10_200i64 {
			assert!(b.pair_get(&k).is_none());
			b.pair_put(Item::new(k, k));
			check_structure(&b);
			for &h in &hot {
				let p = b.find(&h).expect("protected item evicted by scan");
				assert!(p >= b.len0, "hot key left protected");
			}
		}
	}

	#[test]
	fn ghost_ring_wraps_and_stays_bounded() {
		let mut b = bucket8();
		for k in 0..1000i64 {
			if b.pair_get(&(k % 37)).is_none() {
				b.pair_put(Item::new(k % 37, k));
			}
			check_structure(&b);
		}
		assert_eq!(b.ghosts.len(), 4);
		assert!(b.ghost_head < b.ghosts.len());
	}

	#[test]
	fn slru_bucket_has_no_meta() {
		let mut b = bucket8();
		for k in 0..100i64 {
			if b.pair_get(&(k % 13)).is_none() {
				b.pair_put(Item::new(k % 13, k));
			}
		}
		assert!(b.meta.is_empty());
	}

	// ==================== Fallback LFU (counters in `meta`) ====================

	#[test]
	fn lfu_get_increments_counter() {
		let mut b = fallback(4, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		for expected in 1..=3u32 {
			assert!(b.pair_get(&1).is_some());
			assert_eq!(b.meta[b.find(&1).unwrap()], expected);
		}
	}

	#[test]
	fn lfu_evicts_least_frequently_used() {
		let mut b = fallback(2, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_get(&1); // counter: 1
		b.pair_get(&1); // counter: 2
		b.pair_get(&2); // counter: 1
		let evicted = b.pair_put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 2);
		// The newcomer starts at 0; the survivor keeps its count.
		assert_eq!(b.meta[b.find(&1).unwrap()], 2);
		assert_eq!(b.meta[b.find(&3).unwrap()], 0);
	}

	#[test]
	fn lfu_evicts_first_on_tie() {
		let mut b = fallback(2, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		let evicted = b.pair_put(Item::new(3, 30));
		assert_eq!(evicted.unwrap().key, 1);
	}

	#[test]
	fn eviction_candidate_index_lfu() {
		let mut b = fallback(4, EvictionPolicyKind::Lfu);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_get(&1);
		b.pair_get(&1);
		b.pair_get(&2);
		assert_eq!(Bucket::eviction_candidate_index(&b), Some(1));
	}

	#[test]
	fn fallback_meta_stays_parallel_under_reordering_policies() {
		for policy in [
			EvictionPolicyKind::TimestampLRU,
			EvictionPolicyKind::Swap,
			EvictionPolicyKind::Fifo,
			EvictionPolicyKind::Lifo,
			EvictionPolicyKind::Lfu,
		] {
			let mut b = fallback(5, policy);
			for k in 0..500i64 {
				let key = (k * 7919) % 23;
				if b.pair_get(&key).is_none() {
					b.pair_put(Item::new(key, key));
				}
				assert_eq!(b.meta.len(), b.items.len(), "{policy:?}: meta out of sync");
				assert_eq!(b.len0, b.items.len());
				assert!(b.items.len() <= b.max_size);
			}
			let removed = b.remove_at(0);
			assert!(b.find(&removed.key).is_none());
			assert_eq!(b.meta.len(), b.items.len());
		}
	}

	#[test]
	fn timestamp_lru_fallback_evicts_least_recently_touched() {
		let mut b = fallback(3, EvictionPolicyKind::TimestampLRU);
		b.pair_put(Item::new(1, 10));
		b.pair_put(Item::new(2, 20));
		b.pair_put(Item::new(3, 30));
		b.pair_get(&1);
		let evicted = b.pair_put(Item::new(4, 40));
		assert_eq!(evicted.unwrap().key, 2);
	}
}
