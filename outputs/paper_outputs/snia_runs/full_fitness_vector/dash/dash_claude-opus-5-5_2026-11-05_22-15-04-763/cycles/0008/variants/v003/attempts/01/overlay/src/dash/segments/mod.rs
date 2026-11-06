mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::new_bucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::{get_index, hash, index_from_hash};
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

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	pub(crate) buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) segment_size: usize,
	pub(crate) stash_buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) stash_size: usize,
	/// One presence word per stash bucket (cycle 7 H02). Bit `hash(key) & 63`
	/// is set for every key resident in that stash bucket. Conservative: may
	/// have false positives, never false negatives. Maintained by `put_at`
	/// (OR on insert, recompute on eviction) and `refresh_stash_mask` (called
	/// from every stash removal site).
	pub(crate) stash_masks: Vec<u64>,
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
			.finish()
	}
}

/// Stash presence bit for a precomputed key hash. Bucket indexes are taken
/// from the hash's high bits (multiply-shift reduction, cycle 8 H03), so the
/// presence bit uses the low 6 bits to stay independent of the stash index.
#[inline(always)]
pub(crate) fn stash_bit_from_hash(h: usize) -> u64 {
	1u64 << ((h as u64) & 63)
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
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			stash_masks: vec![0u64; settings.num_of_stash_buckets],
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Stash presence bit for `key` (see [`stash_bit_from_hash`]).
	#[inline(always)]
	pub(crate) fn stash_bit(key: &K) -> u64 {
		stash_bit_from_hash(hash(key))
	}

	/// Recomputes `stash_masks[s]` from the items currently in stash bucket `s`.
	/// Must be called after any item leaves a stash bucket. Each re-hash is now
	/// the cheap fmix64 hasher.
	pub(crate) fn refresh_stash_mask(&mut self, s: usize) {
		let mask = self.stash_buckets[s]
			.get_items()
			.iter()
			.fold(0u64, |m, it| m | Self::stash_bit(&it.key));
		self.stash_masks[s] = mask;
	}

	/// Returns `true` if `key` is present in any slot (stash, target, or probing).
	pub fn contains_key(&self, key: &K) -> bool {
		let h = hash(key);
		let stash_idx = index_from_hash(h, self.stash_size);
		let target_idx = index_from_hash(h, self.segment_size);
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
	/// Stash inserts keep `stash_masks` conservative: the arriving key's bit is
	/// OR-ed in; if the insert evicted a stash resident the mask is recomputed.
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		let h = hash(&item.key);
		let idx = match slot {
			Slot::Stash => index_from_hash(h, self.stash_size),
			Slot::Target => index_from_hash(h, self.segment_size),
			Slot::Probing => (index_from_hash(h, self.segment_size) + 1) % self.segment_size,
		};
		self.put_at_idx(slot, idx, h, pos, item)
	}

	/// [`put_at`](Self::put_at) with the bucket index and key hash precomputed
	/// by the caller. `idx` must be the index `put_at` would derive for `slot`.
	#[inline]
	pub(crate) fn put_at_idx(
		&mut self,
		slot: Slot,
		idx: usize,
		h: usize,
		pos: InsertPosition,
		item: Item<K, V>,
	) -> Option<Item<K, V>> {
		if let Slot::Stash = slot {
			let (_, evicted) = self.stash_buckets[idx].put_at(item, pos);
			if let Some(ref e) = evicted {
				debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
				self.refresh_stash_mask(idx);
			} else {
				self.stash_masks[idx] |= stash_bit_from_hash(h);
			}
			return evicted;
		}
		let (_, evicted) = self.buckets[idx].put_at(item, pos);
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

	// ── stash_masks ──────────────────────────────────────────────────────────

	#[test]
	fn stash_mask_starts_zero_and_tracks_inserts() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert!(core.stash_masks.iter().all(|&m| m == 0));
		let key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(key, key));
		assert_ne!(core.stash_masks[0] & SegmentCore::<i64, i64>::stash_bit(&key), 0);
	}

	#[test]
	fn stash_mask_never_has_false_negatives_under_eviction() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size * 3);
		for &k in &keys {
			core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
			for it in core.stash_buckets[0].get_items() {
				assert_ne!(core.stash_masks[0] & SegmentCore::<i64, i64>::stash_bit(&it.key), 0);
			}
		}
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
