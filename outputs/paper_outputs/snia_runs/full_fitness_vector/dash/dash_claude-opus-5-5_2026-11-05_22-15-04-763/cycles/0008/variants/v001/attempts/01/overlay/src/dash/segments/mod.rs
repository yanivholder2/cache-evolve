mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::new_bucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Put destination for an item, resolved from its key.
#[derive(Debug, Clone, Copy)]
pub enum Slot {
	Stash,
	Target,
	Probing,
}

/// Width of one stash presence counter, in bits.
///
/// Cycle 8 H01 asks for 16 counters x 4 bits, with a fall back to 8 counters
/// x 8 bits when a stash bucket can hold 15 or more items. A 4-bit counter
/// saturates at 15, and the evaluated geometry uses `bucket_size = 64` (the
/// default is 16), so this variant uses the **8 x 8-bit** layout. One counter
/// holds up to 255, which covers any stash bucket of `bucket_size <= 255`.
pub(crate) const STASH_COUNTER_BITS: u32 = 8;
const STASH_COUNTER_MASK: u64 = (1u64 << STASH_COUNTER_BITS) - 1;
/// Largest stash bucket size for which counters cannot overflow.
const STASH_COUNTER_MAX: usize = STASH_COUNTER_MASK as usize;

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	pub(crate) buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) segment_size: usize,
	pub(crate) stash_buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) stash_size: usize,
	/// One presence word per stash bucket (cycle 7 H02), now a counting filter
	/// (cycle 8 H01). Each u64 holds 8 counters of 8 bits. The counter at shift
	/// `stash_shift_from_hash(hash(key))` counts how many residents of that stash
	/// bucket map to it. Conservative: false positives are possible, false
	/// negatives are not. `put_at` (insert / eviction) and
	/// `stash_count_dec` (promotion out of stash) keep it current in O(1), with
	/// no rescan of the bucket.
	pub(crate) stash_masks: Vec<u64>,
	/// `false` only when `bucket_size > 255`, where an 8-bit counter could
	/// overflow. In that case the masks are pinned to all-ones (every lookup
	/// scans) and the counter updates are skipped.
	pub(crate) stash_counting: bool,
}

// Manual Debug: `Box<dyn Bucket>` is not `Debug` (the `Bucket` trait has no
// Debug supertrait), so render each bucket through its items instead of deriving.
impl<K, V> Debug for SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("SegmentCore")
			.field("buckets", &self.buckets.iter().map(|b| b.get_items()).collect::<Vec<_>>())
			.field("segment_size", &self.segment_size)
			.field("stash_buckets", &self.stash_buckets.iter().map(|b| b.get_items()).collect::<Vec<_>>())
			.field("stash_size", &self.stash_size)
			.field("stash_masks", &self.stash_masks)
			.field("stash_counting", &self.stash_counting)
			.finish()
	}
}

/// Bit shift of the stash presence counter for a precomputed key hash:
/// `(h >> 61) * 8`. It uses the top 3 bits of the same hash that `get_index`
/// reduces modulo the bucket counts (the low bits), so the counter choice is
/// independent of the bucket index.
#[inline]
pub(crate) fn stash_shift_from_hash(h: usize) -> u32 {
	(((h as u64) >> (64 - 3)) as u32) * STASH_COUNTER_BITS
}

/// `true` if the counter at `shift` in `mask` is non-zero, i.e. the key may be
/// in the stash bucket.
#[inline]
pub(crate) fn stash_may_contain(mask: u64, shift: u32) -> bool {
	(mask >> shift) & STASH_COUNTER_MASK != 0
}

impl<K, V> SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new SegmentCore with fallible bucket initialization.
	/// Returns `None` if the settings are invalid.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Err(problems) = settings.validate() {
			log::error!("SegmentCore.try_new: validation failed: {}", problems.join(", "));
			return None;
		}
		info!("SegmentCore.try_new: settings={settings:?}");
		let buckets: Vec<_> = (0..settings.num_of_normal_buckets)
			.map(|_| new_bucket(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		let stash_buckets: Vec<_> = (0..settings.num_of_stash_buckets)
			.map(|_| new_bucket(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		let stash_counting = settings.bucket_size <= STASH_COUNTER_MAX;
		let initial_mask = if stash_counting { 0u64 } else { u64::MAX };
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			stash_masks: vec![initial_mask; settings.num_of_stash_buckets],
			stash_counting,
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Shift of the stash presence counter for `key` (see [`stash_shift_from_hash`]).
	#[inline]
	pub(crate) fn stash_nibble(key: &K) -> u32 {
		stash_shift_from_hash(hash(key))
	}

	/// O(1) increment of the counter at `shift` in stash bucket `s`.
	#[inline]
	pub(crate) fn stash_count_inc(&mut self, s: usize, shift: u32) {
		if !self.stash_counting {
			return;
		}
		debug_assert!(
			(self.stash_masks[s] >> shift) & STASH_COUNTER_MASK < STASH_COUNTER_MASK,
			"stash counter overflow: stash={s} shift={shift}"
		);
		self.stash_masks[s] += 1u64 << shift;
	}

	/// O(1) decrement of the counter at `shift` in stash bucket `s`.
	#[inline]
	pub(crate) fn stash_count_dec(&mut self, s: usize, shift: u32) {
		if !self.stash_counting {
			return;
		}
		debug_assert!(
			(self.stash_masks[s] >> shift) & STASH_COUNTER_MASK != 0,
			"stash counter underflow: stash={s} shift={shift}"
		);
		self.stash_masks[s] -= 1u64 << shift;
	}

	/// Debug-only cross-check: recomputes the counters of stash bucket `s` from
	/// its residents and reports whether they match `stash_masks[s]`. It is
	/// only called from `debug_assert!`s and tests.
	#[allow(dead_code)]
	pub(crate) fn stash_mask_consistent(&self, s: usize) -> bool {
		if !self.stash_counting {
			return self.stash_masks[s] == u64::MAX;
		}
		let recomputed = self.stash_buckets[s]
			.get_items()
			.iter()
			.fold(0u64, |m, it| m + (1u64 << Self::stash_nibble(&it.key)));
		recomputed == self.stash_masks[s]
	}

	/// Returns `true` if `key` is present in any slot (stash, target, or probing).
	pub fn contains_key(&self, key: &K) -> bool {
		let stash_idx = get_index(key, self.stash_size);
		let target_idx = get_index(key, self.segment_size);
		let probing_idx = (target_idx + 1) % self.segment_size;
		self.stash_buckets[stash_idx].get_position(key).is_some()
			|| self.buckets[target_idx].get_position(key).is_some()
			|| self.buckets[probing_idx].get_position(key).is_some()
	}

	/// Total items across all normal and stash buckets.
	pub fn total_items(&self) -> usize {
		self.buckets.iter().map(|b| b.size()).sum::<usize>()
			+ self.stash_buckets.iter().map(|b| b.size()).sum::<usize>()
	}

	/// Maximum items the segment can hold: sum of max_size across all normal and stash buckets.
	pub fn max_capacity(&self) -> usize {
		self.buckets.iter().map(|b| b.get_max_size()).sum::<usize>()
			+ self.stash_buckets.iter().map(|b| b.get_max_size()).sum::<usize>()
	}

	/// Returns `true` if every item resides in a bucket consistent with its key's derived slot.
	///
	/// A normal bucket at index `i` is legal for key `k` iff `i` is the target index or the
	/// probing index of `k`. A stash bucket at index `j` is legal iff `j` is the stash index of `k`.
	pub fn check_slot_legality(&self) -> bool {
		for (i, bucket) in self.buckets.iter().enumerate() {
			for item in bucket.get_items() {
				let target = get_index(&item.key, self.segment_size);
				let probing = (target + 1) % self.segment_size;
				if i != target && i != probing {
					return false;
				}
			}
		}
		for (j, bucket) in self.stash_buckets.iter().enumerate() {
			for item in bucket.get_items() {
				if j != get_index(&item.key, self.stash_size) {
					return false;
				}
			}
		}
		true
	}

	/// Returns `true` if no individual bucket holds more items than its `max_size`.
	pub fn check_capacity_bound(&self) -> bool {
		self.buckets.iter().all(|b| b.size() <= b.get_max_size())
			&& self.stash_buckets.iter().all(|b| b.size() <= b.get_max_size())
	}

	/// Push `item` directly into the normal bucket at `bucket_idx`, bypassing slot derivation
	/// and capacity limits. Intended only for invariant-suite negative fixtures.
	pub fn force_push_normal_bucket(&mut self, bucket_idx: usize, item: Item<K, V>) {
		self.buckets[bucket_idx].get_items_mut().push(item);
	}

	/// Inserts `item` at `pos` in the bucket identified by `slot` (derived from `item.key`).
	/// Returns the evicted item, if any.
	///
	/// Stash inserts keep `stash_masks` exact in O(1). An evicted stash resident
	/// has its counter decremented, then the arriving key's counter is
	/// incremented. Decrementing first keeps every counter at or below the bucket
	/// size.
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		if let Slot::Stash = slot {
			let h = hash(&item.key);
			let idx = h % self.stash_size;
			let (_, evicted) = self.stash_buckets[idx].put_at(item, pos);
			if let Some(ref e) = evicted {
				debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
				self.stash_count_dec(idx, Self::stash_nibble(&e.key));
			}
			self.stash_count_inc(idx, stash_shift_from_hash(h));
			debug_assert!(
				self.stash_mask_consistent(idx),
				"SegmentCore.put_at: stash counters out of sync at stash={idx}"
			);
			return evicted;
		}
		let bucket = match slot {
			Slot::Stash => unreachable!("stash handled above"),
			Slot::Target => {
				let idx = get_index(&item.key, self.segment_size);
				&mut self.buckets[idx]
			}
			Slot::Probing => {
				let target_idx = get_index(&item.key, self.segment_size);
				let probing_idx = (target_idx + 1) % self.segment_size;
				&mut self.buckets[probing_idx]
			}
		};
		let (_, evicted) = bucket.put_at(item, pos);
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
		}
		evicted
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::shared::traits::bucket::InsertPosition;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	#[test]
	fn try_new_rejects_single_normal_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_normal_buckets = 1;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	#[test]
	fn try_new_rejects_zero_stash_buckets() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 0;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	// ── stash_masks (counting) ───────────────────────────────────────────────

	#[test]
	fn stash_mask_starts_zero_and_tracks_inserts() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert!(core.stash_masks.iter().all(|&m| m == 0));
		let key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(key, key));
		assert!(stash_may_contain(core.stash_masks[0], SegmentCore::<i64, i64>::stash_nibble(&key)));
		assert_eq!(core.stash_masks[0], 1u64 << SegmentCore::<i64, i64>::stash_nibble(&key));
	}

	#[test]
	fn stash_mask_counters_exact_under_eviction() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size * 3);
		for &k in &keys {
			core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
			for it in core.stash_buckets[0].get_items() {
				assert!(stash_may_contain(
					core.stash_masks[0],
					SegmentCore::<i64, i64>::stash_nibble(&it.key)
				));
			}
			assert!(core.stash_mask_consistent(0));
		}
	}

	#[test]
	fn stash_counting_disabled_for_huge_buckets() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.bucket_size = STASH_COUNTER_MAX + 1;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert!(!core.stash_counting);
		assert!(core.stash_masks.iter().all(|&m| m == u64::MAX));
		let key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(key, key));
		assert_eq!(core.stash_masks[0], u64::MAX);
	}

	// ── total_items ──────────────────────────────────────────────────────────

	#[test]
	fn total_items_empty() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.total_items(), 0);
	}

	#[test]
	fn total_items_counts_normal_and_stash() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let normal_key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(normal_key, normal_key));
		assert_eq!(core.total_items(), 1);
		let stash_key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(stash_key, stash_key));
		assert_eq!(core.total_items(), 2);
		// Verify the stash bucket actually received the item, not just the count.
		assert!(core.stash_buckets[0].get_position(&stash_key).is_some());
	}

	// ── max_capacity ─────────────────────────────────────────────────────────

	#[test]
	fn max_capacity_matches_settings() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		let expected = (s.num_of_normal_buckets + s.num_of_stash_buckets) * s.bucket_size;
		assert_eq!(core.max_capacity(), expected);
	}

	// ── check_slot_legality ──────────────────────────────────────────────────

	#[test]
	fn check_slot_legality_empty_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		assert!(SegmentCore::<i64, i64>::new(&s).check_slot_legality());
	}

	#[test]
	fn check_slot_legality_target_slot_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_probing_slot_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A key at target bucket 0 placed in its probing bucket 1 must be legal.
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_probing_wraps_around() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A key at target bucket 3 (last) has probing index 0 via modulo wrap.
		// Placing it in bucket 0 via Slot::Probing must be legal.
		let key = find_keys_for_bucket(s.num_of_normal_buckets - 1, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_detects_wrong_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Key hashes to bucket 2; forcing it into bucket 0 violates legality.
		let bad_key = find_keys_for_bucket(2, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(0, Item::new(bad_key, bad_key));
		assert!(!core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_detects_wrong_stash_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Find a key that hashes to stash 1, then force it into stash 0.
		let key = (0i64..200)
			.find(|k| get_index(k, s.num_of_stash_buckets) == 1)
			.expect("no key hashes to stash 1");
		core.stash_buckets[0].get_items_mut().push(Item::new(key, key));
		assert!(!core.check_slot_legality());
	}

	// ── check_capacity_bound ─────────────────────────────────────────────────

	#[test]
	fn check_capacity_bound_empty_is_within_bound() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		assert!(SegmentCore::<i64, i64>::new(&s).check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_at_limit_is_within_bound() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_normal_buckets, s.bucket_size);
		for k in keys {
			core.force_push_normal_bucket(0, Item::new(k, k));
		}
		assert!(core.check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_detects_normal_overflow() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Push bucket_size + 1 items to exceed capacity.
		let keys = find_keys_for_bucket(0, s.num_of_normal_buckets, s.bucket_size + 1);
		for k in keys {
			core.force_push_normal_bucket(0, Item::new(k, k));
		}
		assert!(!core.check_capacity_bound());
	}

	#[test]
	fn check_capacity_bound_detects_stash_overflow() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Overfill stash bucket 0 beyond its max_size.
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size + 1);
		for k in keys {
			core.stash_buckets[0].get_items_mut().push(Item::new(k, k));
		}
		assert!(!core.check_capacity_bound());
	}

	// ── force_push_normal_bucket ─────────────────────────────────────────────

	#[test]
	fn force_push_normal_bucket_item_appears_in_target_bucket() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// Use a key whose target is bucket 0, not bucket 3, to confirm the push
		// bypasses slot derivation and lands in the requested bucket regardless.
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.force_push_normal_bucket(3, Item::new(key, key));
		assert!(core.buckets[3].get_position(&key).is_some());
		assert_eq!(core.total_items(), 1);
	}
}
