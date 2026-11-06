mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::new_bucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::{get_index, hash, index_from_hash, stash_mask_bits};
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

/// Lazy-rebuild threshold for the stash collision word (cycle 10 H01): after an
/// O(1) per-bit clear, `refresh_stash_mask` runs only when more than this many
/// collide bits are set in that bucket's `stash_collide` word.
pub(crate) const COLLIDE_REFRESH: u32 = 6;

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	pub(crate) buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) segment_size: usize,
	pub(crate) stash_buckets: Vec<Box<dyn Bucket<K, V>>>,
	pub(crate) stash_size: usize,
	/// One presence word per stash bucket (cycle 7 H02; k=2 since cycle 9 H02).
	/// Both bits of `stash_mask_bits(hash(key))` are set for every key resident
	/// in that stash bucket. Conservative: may have false positives, never false
	/// negatives.
	pub(crate) stash_masks: Vec<u64>,
	/// One collision word per stash bucket (cycle 10 H01), laid out like
	/// `stash_masks`. Invariant: if bit b of `stash_collide[i]` is clear, at most
	/// one live item of stash bucket i maps to bit b, and `stash_masks[i]` bit b
	/// is set exactly when that item is present. Collide bits may be stale-set
	/// but are never wrongly clear, so a removal may clear exactly the bits
	/// `m & !collide` in O(1).
	pub(crate) stash_collide: Vec<u64>,
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
			.field("stash_collide", &self.stash_collide)
			.finish()
	}
}

/// Stash presence bits (k=2 Bloom test, see [`stash_mask_bits`]) for a
/// precomputed key hash. Bucket indexes are taken from the hash's high bits
/// (multiply-shift reduction, cycle 8 H03), so the presence bits use the low
/// 12 bits to stay independent of the stash index.
#[inline(always)]
pub(crate) fn stash_bits_from_hash(h: usize) -> u64 {
	stash_mask_bits(h as u64)
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
			stash_collide: vec![0u64; settings.num_of_stash_buckets],
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Stash presence bits for `key` (see [`stash_bits_from_hash`]).
	#[inline(always)]
	pub(crate) fn stash_bits(key: &K) -> u64 {
		stash_bits_from_hash(hash(key))
	}

	/// Exact presence mask of stash bucket `s`, recomputed from its residents.
	/// Debug/test only: used to verify the fast path never drops a live bit.
	#[cfg(any(test, debug_assertions))]
	pub(crate) fn exact_stash_mask(&self, s: usize) -> u64 {
		self.stash_buckets[s]
			.get_items()
			.iter()
			.fold(0u64, |m, it| m | Self::stash_bits(&it.key))
	}

	/// Rebuilds both `stash_masks[s]` and `stash_collide[s]` in one pass over
	/// the items currently in stash bucket `s` (`collide |= mask & m; mask |= m`).
	/// Afterwards both words are exact.
	pub(crate) fn refresh_stash_mask(&mut self, s: usize) {
		let (mask, collide) = self.stash_buckets[s]
			.get_items()
			.iter()
			.fold((0u64, 0u64), |(mask, collide), it| {
				let m = Self::stash_bits(&it.key);
				(mask | m, collide | (mask & m))
			});
		self.stash_masks[s] = mask;
		self.stash_collide[s] = collide;
	}

	/// Records a key with stash bits `m` arriving in stash bucket `s`.
	#[inline(always)]
	fn stash_mask_insert(&mut self, s: usize, m: u64) {
		self.stash_collide[s] |= self.stash_masks[s] & m;
		self.stash_masks[s] |= m;
	}

	/// Records a key with stash bits `m` leaving stash bucket `s`: clears only
	/// the bits that key owned alone. Collided bits stay set (over-report only).
	/// Rebuilds lazily once too many collide bits have accumulated.
	#[inline(always)]
	fn stash_mask_remove(&mut self, s: usize, m: u64) {
		self.stash_masks[s] &= !(m & !self.stash_collide[s]);
		#[cfg(debug_assertions)]
		{
			let exact = self.exact_stash_mask(s);
			debug_assert!(
				exact & !self.stash_masks[s] == 0,
				"stash fast clear produced a false negative: stash={s} exact={exact:#x} mask={:#x} collide={:#x}",
				self.stash_masks[s],
				self.stash_collide[s]
			);
		}
		if self.stash_collide[s].count_ones() > COLLIDE_REFRESH {
			self.refresh_stash_mask(s);
		}
	}

	/// Removes the item at `pos` of stash bucket `s`, whose key hash `h` the
	/// caller already holds, and updates the presence mask in O(1).
	#[inline]
	pub(crate) fn stash_remove_at(&mut self, s: usize, pos: usize, h: usize) -> Item<K, V> {
		let data = self.stash_buckets[s].remove_at(pos);
		self.stash_mask_remove(s, stash_bits_from_hash(h));
		data
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
	/// Stash inserts keep `stash_masks` conservative: the arriving key's bits are
	/// recorded (collide/OR); an evicted stash resident's uncollided bits are
	/// then cleared in O(1).
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
			// Record the arrival first (while the evicted key's bits are still
			// in the mask), so any bit it shares with the evicted key is marked
			// collided and survives the evicted key's clear.
			self.stash_mask_insert(idx, stash_bits_from_hash(h));
			if let Some(ref e) = evicted {
				debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
				self.stash_mask_remove(idx, Self::stash_bits(&e.key));
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

	fn mask_passes(mask: u64, key: &i64) -> bool {
		let m = SegmentCore::<i64, i64>::stash_bits(key);
		mask & m == m
	}

	#[test]
	fn stash_mask_starts_zero_and_tracks_inserts() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		assert!(core.stash_masks.iter().all(|&m| m == 0));
		assert!(core.stash_collide.iter().all(|&m| m == 0));
		let key = find_keys_for_bucket(0, s.num_of_stash_buckets, 1)[0];
		core.put_at(Slot::Stash, InsertPosition::Back, Item::new(key, key));
		assert!(mask_passes(core.stash_masks[0], &key));
	}

	#[test]
	fn stash_mask_never_has_false_negatives_under_eviction() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size * 3);
		for &k in &keys {
			core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
			for it in core.stash_buckets[0].get_items() {
				assert!(mask_passes(core.stash_masks[0], &it.key));
			}
		}
	}

	#[test]
	fn stash_mask_refresh_matches_k2_bits_of_residents() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size * 2);
		for &k in &keys {
			core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
		}
		let expected = core.exact_stash_mask(0);
		// Fast path: conservative superset of the exact mask.
		assert_eq!(expected & !core.stash_masks[0], 0);
		// Explicit rebuild: exact.
		core.refresh_stash_mask(0);
		assert_eq!(core.stash_masks[0], expected);
	}

	#[test]
	fn refresh_rebuilds_collide_word() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let keys = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size);
		for &k in &keys {
			core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
		}
		core.refresh_stash_mask(0);
		let (mut mask, mut collide) = (0u64, 0u64);
		for it in core.stash_buckets[0].get_items() {
			let m = SegmentCore::<i64, i64>::stash_bits(&it.key);
			collide |= mask & m;
			mask |= m;
		}
		assert_eq!(core.stash_masks[0], mask);
		assert_eq!(core.stash_collide[0], collide);
	}

	/// Random insert/remove sequences on one stash bucket never produce a false
	/// negative, and removing the sole owner of a bit clears it.
	#[test]
	fn stash_collide_random_insert_remove_no_false_negatives() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let pool = find_keys_for_bucket(0, s.num_of_stash_buckets, s.bucket_size * 6);
		let mut x: u64 = 0xD1B5_4A32_D192_ED03;
		let mut next = || {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			x
		};
		for _ in 0..20_000 {
			let r = next();
			let len = core.stash_buckets[0].size();
			if len > 0 && (r & 3 == 0 || len == s.bucket_size && r & 1 == 0) {
				let pos = (next() as usize) % len;
				let key = core.stash_buckets[0].get_items()[pos].key;
				let removed = core.stash_remove_at(0, pos, hash(&key));
				assert_eq!(removed.key, key);
			} else {
				let k = pool[(next() as usize) % pool.len()];
				if core.stash_buckets[0].get_position(&k).is_none() {
					core.put_at(Slot::Stash, InsertPosition::Back, Item::new(k, k));
				}
			}
			let exact = core.exact_stash_mask(0);
			assert_eq!(exact & !core.stash_masks[0], 0, "false negative");
			for it in core.stash_buckets[0].get_items() {
				assert!(mask_passes(core.stash_masks[0], &it.key));
			}
			if core.stash_buckets[0].size() == 0 {
				assert_eq!(core.stash_masks[0] & !core.stash_collide[0], 0, "uncollided bits of an empty bucket are clear");
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
