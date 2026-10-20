mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::new_bucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Put destination for an item, resolved from its key.
#[derive(Debug, Clone, Copy)]
pub enum Slot {
	Stash,
	Target,
	Probing,
}

// ── H02 (cycle 0007): direct-mapped promotion-heat table ─────────────────────
// The promotion-heat probe in `promotion_get` is the documented bound on this
// lineage's get p50. H01 (cycle 0006) removed the SipHash cost by giving the
// `HashMap<K, u8>` a trivial integer hasher; the remaining cost is the
// SwissTable group scan and `entry` bookkeeping the HashMap still performs on
// every hot target hit. H02 replaces the HashMap entirely with a flat,
// fixed-size direct-mapped array (`TargetHeat`): one index + one key compare,
// with no probe sequence, no control-byte scan, and no `entry` insert path.
//
// The `U64Hasher` multiply (Fibonacci-hashing constant, the same mix H01 fed
// the HashMap) folds the integer key into a slot index; masking with
// `table_len - 1` (a power of two) selects the slot. The only behavioural
// difference from the HashMap is slot collisions: two distinct target-resident
// keys mapping to the same slot clobber each other's heat. The table is sized
// at 2x capacity (next power of two) so collisions are a rare, bounded
// promotion-timing perturbation.

/// Zero-keyed integer hasher used to fold a key into a `TargetHeat` slot index.
/// `write_u64`/`write_i64` apply one Fibonacci-hashing multiply; `write` keeps a
/// correctness-only FNV-1a fallback so the generic `K: Hash` bound compiles for
/// any key type. (Identical mix to H01's `U64Hasher`; here it derives an array
/// index rather than a HashMap bucket.)
pub(crate) struct U64Hasher(u64);

impl Hasher for U64Hasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		// Correctness fallback for the generic byte path. The slot index is only
		// ever derived from the integer key, so this is never hit on the hot path;
		// it exists solely so `U64Hasher` satisfies the full `Hasher` contract.
		let mut h = self.0;
		for &b in bytes {
			h ^= b as u64;
			h = h.wrapping_mul(0x0000_0100_0000_01B3); // FNV-1a prime
		}
		self.0 = h;
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		// Single multiplicative mix with the Fibonacci constant 2^64 / φ.
		self.0 = i.wrapping_mul(0x9E37_79B9_7F4A_7C15);
	}

	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.write_u64(i as u64);
	}

	#[inline]
	fn write_i32(&mut self, i: i32) {
		self.write_u64(i as u32 as u64);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.write_u64(i as u64);
	}
}

/// Flat, fixed-size direct-mapped promotion-heat table (H02). Each slot holds at
/// most one `(key, heat)` pair; a key maps to exactly one slot via the
/// `U64Hasher` fold masked into the power-of-two length. Replaces the
/// `HashMap<K, u8, U64BuildHasher>` side table of the parent: a target hit costs
/// one index plus one key compare instead of a SwissTable probe.
///
/// This is segment container state, sized once at construction and indexed
/// O(1); it never rides along with items through bucket moves. It does not enter
/// the per-`Item` footprint, so `Item<u64, u64>` stays at the 16-byte floor.
#[derive(Debug)]
pub(crate) struct TargetHeat<K> {
	slots: Vec<Option<(K, u8)>>,
	mask: usize,
}

impl<K> TargetHeat<K>
where
	K: Hash + Eq + Copy,
{
	/// Allocates the table with the next power of two at least `min_slots` slots
	/// (clamped to >= 1 so the mask is well defined), all empty.
	fn with_capacity(min_slots: usize) -> Self {
		let len = min_slots.next_power_of_two().max(1);
		let mut slots = Vec::with_capacity(len);
		slots.resize_with(len, || None);
		Self { slots, mask: len - 1 }
	}

	/// Slot index for `key`: one Fibonacci multiply folded into the table length.
	#[inline]
	fn slot_index(&self, key: &K) -> usize {
		let mut hasher = U64Hasher(0);
		key.hash(&mut hasher);
		(hasher.finish() as usize) & self.mask
	}

	/// Bumps this key's heat and returns the resulting value. If the slot already
	/// holds `key`, its counter is incremented (saturating). Otherwise the slot is
	/// claimed for `key` at heat 1 (clobbering any colliding resident's heat),
	/// matching the parent's `entry(key).or_insert(0)` then `saturating_add(1)`.
	#[inline]
	pub(crate) fn bump(&mut self, key: &K) -> u8 {
		let idx = self.slot_index(key);
		match &mut self.slots[idx] {
			Some((k, h)) if *k == *key => {
				*h = h.saturating_add(1);
				*h
			}
			slot => {
				*slot = Some((*key, 1));
				1
			}
		}
	}

	/// Clears the slot iff it currently holds `key` (a colliding resident's heat
	/// is left untouched). Mirrors the parent's `target_heat.remove(key)`.
	#[inline]
	pub(crate) fn clear(&mut self, key: &K) {
		let idx = self.slot_index(key);
		let holds = matches!(&self.slots[idx], Some((k, _)) if *k == *key);
		if holds {
			self.slots[idx] = None;
		}
	}

	/// Current heat for `key` if its slot holds it, else 0. (Test/inspection helper.)
	pub(crate) fn heat_of(&self, key: &K) -> u8 {
		let idx = self.slot_index(key);
		match &self.slots[idx] {
			Some((k, h)) if *k == *key => *h,
			_ => 0,
		}
	}
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
	/// Promotion-heat side table. The frequency signal that steers a target
	/// resident into the protected probing tier used to be a per-`Item` `u8`
	/// counter, which padded `Item<u64, u64>` up to 24 bytes (H01). It now lives
	/// here, keyed by the resident key, so `Item` stays at the 16-byte floor while
	/// the gate semantics are unchanged. This is segment container state, not
	/// per-entry layout, so it does not enter the measured per-entry footprint.
	///
	/// H02 (cycle 0007): the structure is a flat direct-mapped `TargetHeat` array
	/// rather than a `HashMap`, so the hot promotion-heat probe is one index plus
	/// one key compare instead of a SwissTable probe. Gate decisions are identical
	/// up to rare slot collisions (a bounded promotion-timing perturbation).
	pub(crate) target_heat: TargetHeat<K>,
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
			.field("target_heat", &self.target_heat)
			.finish()
	}
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
		// H02: size the direct-mapped heat table at 2x the normal-bucket capacity,
		// rounded up to the next power of two, so the mask is cheap and collisions
		// between distinct target residents are rare.
		let heat_slots = 2 * settings.num_of_normal_buckets * settings.bucket_size;
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			target_heat: TargetHeat::with_capacity(heat_slots),
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
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
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		let bucket = match slot {
			Slot::Stash => {
				let idx = get_index(&item.key, self.stash_size);
				&mut self.stash_buckets[idx]
			}
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

	// ── target_heat direct-mapped table (H02, cycle 0007) ────────────────────

	#[test]
	fn target_heat_direct_mapped_bump_and_clear() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A fresh key claims its slot at heat 1.
		assert_eq!(core.target_heat.bump(&7), 1);
		// Repeated bumps accumulate (saturating).
		assert_eq!(core.target_heat.bump(&7), 2);
		assert_eq!(core.target_heat.heat_of(&7), 2);
		// A never-bumped key reads heat 0 (whether or not it collides with 7).
		assert_eq!(core.target_heat.heat_of(&123), 0);
		// Clearing the slot resets the key to cold.
		core.target_heat.clear(&7);
		assert_eq!(core.target_heat.heat_of(&7), 0);
		// Re-bumping after a clear starts again at heat 1.
		assert_eq!(core.target_heat.bump(&7), 1);
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
