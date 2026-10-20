mod dash_segment;
mod segment;

pub use dash_segment::DashSegment;
pub use segment::Segment;

use super::dash_bucket::{new_bucket_concrete, tag_of_hash, DashBucket};
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
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

pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Normal buckets stored contiguously by value: a probe indexes straight into the
	/// bucket struct (no `Box` pointer chase) and calls its methods statically (no vtable).
	/// Each bucket header carries an inline per-slot key-tag array so a probe can reject
	/// non-matching slots without touching the bucket's item buffer.
	pub(crate) buckets: Vec<DashBucket<K, V>>,
	pub(crate) segment_size: usize,
	/// Stash buckets, stored contiguously by value like `buckets`.
	pub(crate) stash_buckets: Vec<DashBucket<K, V>>,
	pub(crate) stash_size: usize,
	/// `borrowed[j]` = number of items currently in `stash_buckets[j]` whose primary
	/// stash bucket `s0` is not `j` (i.e. overflow borrowed from the left neighbour
	/// `(j + stash_size - 1) % stash_size`). Kept exact by the stash mutation helpers
	/// ([`put_stash_at_index`](Self::put_stash_at_index),
	/// [`remove_stash_at`](Self::remove_stash_at), [`put_at`](Self::put_at)).
	pub(crate) borrowed: Vec<u8>,
	/// `foreign[i]` = number of items currently in normal bucket `buckets[i]` whose
	/// target index is not `i` (i.e. items placed there as their probing slot, target
	/// `(i + segment_size - 1) % segment_size`). Incremented by `put_at(Slot::Probing)`;
	/// decremented by the segment's demotion path when a foreign evictee leaves bucket `i`
	/// (see [`note_normal_removal`](Self::note_normal_removal)).
	pub(crate) foreign: Vec<u8>,
}

// Manual Debug: render each bucket through its items (compact, policy-free output).
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
			.field("borrowed", &self.borrowed)
			.field("foreign", &self.foreign)
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
		debug_assert!(
			settings.bucket_size < u8::MAX as usize,
			"SegmentCore.try_new: bucket_size={} exceeds the u8 borrowed/foreign counters",
			settings.bucket_size
		);
		info!("SegmentCore.try_new: settings={settings:?}");
		let buckets: Vec<_> = (0..settings.num_of_normal_buckets)
			.map(|_| new_bucket_concrete(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		let stash_buckets: Vec<_> = (0..settings.num_of_stash_buckets)
			.map(|_| new_bucket_concrete(settings.bucket_size, settings.eviction_policy.build()))
			.collect();
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			borrowed: vec![0u8; settings.num_of_stash_buckets],
			foreign: vec![0u8; settings.num_of_normal_buckets],
		})
	}

	/// Creates a new SegmentCore.
	/// # Panics
	/// Panics if initialization fails (e.g., invalid settings).
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("SegmentCore::new: invalid settings")
	}

	/// Neighbour stash bucket of the primary stash bucket `s0`: `(s0 + 1) % stash_size`.
	/// Equals `s0` only when `stash_size == 1`.
	#[inline]
	pub fn stash_neighbor(&self, s0: usize) -> usize {
		(s0 + 1) % self.stash_size
	}

	/// The two candidate stash buckets for `key`: `(s0, s1)` with
	/// `s0 = get_index(key, stash_size)` and `s1 = (s0 + 1) % stash_size`.
	#[inline]
	pub fn stash_indices(&self, key: &K) -> (usize, usize) {
		let s0 = get_index(key, self.stash_size);
		(s0, self.stash_neighbor(s0))
	}

	/// Returns `true` if `key` is present in any slot (stash s0/s1, target, or probing).
	pub fn contains_key(&self, key: &K) -> bool {
		let (s0, s1) = self.stash_indices(key);
		let target_idx = get_index(key, self.segment_size);
		let probing_idx = (target_idx + 1) % self.segment_size;
		self.stash_buckets[s0].get_position(key).is_some()
			|| (s1 != s0 && self.stash_buckets[s1].get_position(key).is_some())
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
	/// probing index of `k`. A stash bucket at index `j` is legal iff `j` is one of the two
	/// stash candidates `(s0, s1)` of `k` (two-choice stash placement).
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
				let (s0, s1) = self.stash_indices(&item.key);
				if j != s0 && j != s1 {
					return false;
				}
			}
		}
		true
	}

	/// Returns `true` if every bucket's inline tag array mirrors its items (or tags are
	/// disabled for that bucket). Diagnostic helper for the tag filter.
	pub fn check_tags_consistent(&self) -> bool {
		self.buckets.iter().all(|b| b.tags_consistent()) && self.stash_buckets.iter().all(|b| b.tags_consistent())
	}

	/// Returns `true` if every `borrowed[j]` equals the number of items in stash bucket
	/// `j` whose primary stash bucket is not `j`. Recomputed from bucket contents.
	///
	/// Only meaningful when stash buckets are mutated through the `SegmentCore` helpers;
	/// negative fixtures that push into stash buckets directly bypass the bookkeeping.
	pub fn check_borrowed_counts(&self) -> bool {
		self.stash_buckets.iter().enumerate().all(|(j, bucket)| {
			let actual = bucket
				.get_items()
				.iter()
				.filter(|it| get_index(&it.key, self.stash_size) != j)
				.count();
			actual == self.borrowed[j] as usize
		})
	}

	/// Returns `true` if every `foreign[i]` equals the number of items in normal bucket
	/// `i` whose target index is not `i`. Recomputed from bucket contents.
	///
	/// Only meaningful when foreign items enter normal buckets through the `SegmentCore`
	/// helpers; negative fixtures that push into normal buckets directly bypass it.
	pub fn check_foreign_counts(&self) -> bool {
		self.buckets.iter().enumerate().all(|(i, bucket)| {
			let actual = bucket
				.get_items()
				.iter()
				.filter(|it| get_index(&it.key, self.segment_size) != i)
				.count();
			actual == self.foreign[i] as usize
		})
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

	/// Records that an item whose target index is `true_target` just left normal bucket
	/// `source_idx`: decrements `foreign[source_idx]` if it was foreign there. The caller
	/// supplies `true_target` (already computed on its path), so no extra hash is needed.
	#[inline]
	pub fn note_normal_removal(&mut self, source_idx: usize, true_target: usize) {
		if true_target != source_idx {
			debug_assert!(self.foreign[source_idx] > 0, "foreign counter underflow at bucket={source_idx}");
			self.foreign[source_idx] -= 1;
		}
	}

	/// Inserts `item` at `pos` into normal bucket `bucket_idx` (caller guarantees it is the
	/// key's target or probing bucket, and the key is not resident), keeping `foreign`
	/// exact for both the inserted item and any evictee. Returns the evicted item, if any.
	/// Counter-aware fixture helper; the hot paths use [`put_at_h`](Self::put_at_h).
	pub fn put_normal_at_index(
		&mut self,
		bucket_idx: usize,
		pos: InsertPosition,
		item: Item<K, V>,
	) -> Option<Item<K, V>> {
		let h = hash(&item.key);
		let target = h % self.segment_size;
		debug_assert!(bucket_idx == target || bucket_idx == (target + 1) % self.segment_size);
		if target != bucket_idx {
			self.foreign[bucket_idx] += 1;
		}
		let (_, evicted) = self.buckets[bucket_idx].put_at_tagged(item, tag_of_hash(h), pos);
		if let Some(ref e) = evicted {
			let t = get_index(&e.key, self.segment_size);
			self.note_normal_removal(bucket_idx, t);
		}
		evicted
	}

	/// Decrements `borrowed[stash_idx]` if `evicted` (just removed from stash bucket
	/// `stash_idx`) was a borrowed item there.
	#[inline]
	fn note_stash_removal(&mut self, stash_idx: usize, removed: &Item<K, V>) {
		if get_index(&removed.key, self.stash_size) != stash_idx {
			debug_assert!(self.borrowed[stash_idx] > 0, "borrowed counter underflow at stash={stash_idx}");
			self.borrowed[stash_idx] -= 1;
		}
	}

	/// Inserts `item` at `pos` into the stash bucket at `stash_idx` (caller guarantees
	/// `stash_idx` is one of the key's two stash candidates, and the key is not resident).
	/// Returns the evicted item, if any. Keeps `borrowed[stash_idx]` exact.
	pub fn put_stash_at_index(
		&mut self,
		stash_idx: usize,
		pos: InsertPosition,
		item: Item<K, V>,
	) -> Option<Item<K, V>> {
		let h = hash(&item.key);
		self.put_stash_at_index_h(stash_idx, pos, item, h)
	}

	/// [`put_stash_at_index`](Self::put_stash_at_index) with the key hash `h` supplied by
	/// the caller (`h == hash(&item.key)`), so neither the primary index nor the slot
	/// tag needs a new hash.
	#[inline]
	pub fn put_stash_at_index_h(
		&mut self,
		stash_idx: usize,
		pos: InsertPosition,
		item: Item<K, V>,
		h: usize,
	) -> Option<Item<K, V>> {
		debug_assert_eq!(h, hash(&item.key));
		let s0 = h % self.stash_size;
		debug_assert!(stash_idx == s0 || stash_idx == self.stash_neighbor(s0));
		debug_assert!(self.stash_buckets[stash_idx].get_position(&item.key).is_none());
		if stash_idx != s0 {
			self.borrowed[stash_idx] += 1;
		}
		let (_, evicted) = self.stash_buckets[stash_idx].put_at_tagged(item, tag_of_hash(h), pos);
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_stash_at_index: stash={stash_idx} pos={pos:?} evicted_key={:?}", e.key);
			self.note_stash_removal(stash_idx, e);
		}
		evicted
	}

	/// Removes and returns the item at `item_idx` of stash bucket `stash_idx`, keeping
	/// `borrowed[stash_idx]` exact.
	#[inline]
	pub fn remove_stash_at(&mut self, stash_idx: usize, item_idx: usize) -> Item<K, V> {
		let removed = self.stash_buckets[stash_idx].remove_at(item_idx);
		self.note_stash_removal(stash_idx, &removed);
		removed
	}

	/// Inserts `item` at `pos` in the bucket identified by `slot` (derived from `item.key`).
	/// `Slot::Stash` targets the primary stash bucket `s0` (never a borrow; only its
	/// eviction can drop a borrowed item). Returns the evicted item, if any.
	///
	/// `Slot::Probing` increments `foreign[probing_idx]` (the inserted item is foreign by
	/// definition). Evictees from normal buckets are **not** classified here: the caller
	/// must report each via [`note_normal_removal`](Self::note_normal_removal).
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>> {
		let h = hash(&item.key);
		self.put_at_h(slot, pos, item, h)
	}

	/// [`put_at`](Self::put_at) with the key hash `h` supplied by the caller
	/// (`h == hash(&item.key)`): the slot index and the slot tag both derive from it.
	#[inline]
	pub fn put_at_h(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>, h: usize) -> Option<Item<K, V>> {
		debug_assert_eq!(h, hash(&item.key));
		let tag = tag_of_hash(h);
		let (bucket, stash_idx) = match slot {
			Slot::Stash => {
				let idx = h % self.stash_size;
				(&mut self.stash_buckets[idx], Some(idx))
			}
			Slot::Target => {
				let idx = h % self.segment_size;
				(&mut self.buckets[idx], None)
			}
			Slot::Probing => {
				let target_idx = h % self.segment_size;
				let probing_idx = (target_idx + 1) % self.segment_size;
				self.foreign[probing_idx] += 1;
				(&mut self.buckets[probing_idx], None)
			}
		};
		let (_, evicted) = bucket.put_at_tagged(item, tag, pos);
		if let Some(ref e) = evicted {
			debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", e.key);
			if let Some(idx) = stash_idx {
				self.note_stash_removal(idx, e);
			}
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

	// ── borrowed counters ────────────────────────────────────────────────────

	#[test]
	fn borrowed_starts_zero() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.borrowed.len(), s.num_of_stash_buckets);
		assert!(core.borrowed.iter().all(|&b| b == 0));
		assert!(core.check_borrowed_counts());
	}

	#[test]
	fn borrowed_tracks_put_and_remove() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(key, key));
		assert_eq!(core.borrowed[2], 1);
		assert!(core.check_borrowed_counts());
		let native = find_keys_for_bucket(2, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(native, native));
		assert_eq!(core.borrowed[2], 1);
		let idx = core.stash_buckets[2].get_position(&key).unwrap();
		let removed = core.remove_stash_at(2, idx);
		assert_eq!(removed.key, key);
		assert_eq!(core.borrowed[2], 0);
		assert!(core.check_borrowed_counts());
		assert!(core.check_tags_consistent());
	}

	#[test]
	fn borrowed_decrements_on_put_at_stash_eviction() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let borrowed_keys = find_keys_for_bucket(1, 4, s.bucket_size);
		for &k in &borrowed_keys {
			core.put_stash_at_index(2, InsertPosition::Back, Item::new(k, k));
		}
		assert_eq!(core.borrowed[2] as usize, s.bucket_size);
		let native = find_keys_for_bucket(2, 4, 1)[0];
		let evicted = core.put_at(Slot::Stash, InsertPosition::Back, Item::new(native, native));
		assert!(evicted.is_some());
		assert_eq!(core.borrowed[2] as usize, s.bucket_size - 1);
		assert!(core.check_borrowed_counts());
		assert!(core.check_tags_consistent());
	}

	// ── foreign counters ─────────────────────────────────────────────────────

	#[test]
	fn foreign_starts_zero() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let core = SegmentCore::<i64, i64>::new(&s);
		assert_eq!(core.foreign.len(), s.num_of_normal_buckets);
		assert!(core.foreign.iter().all(|&f| f == 0));
		assert!(core.check_foreign_counts());
	}

	#[test]
	fn foreign_tracks_probing_put_and_target_put() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let n = s.num_of_normal_buckets;
		let k = find_keys_for_bucket(0, n, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(k, k));
		assert_eq!(core.foreign[1], 1);
		assert!(core.check_foreign_counts());
		let native = find_keys_for_bucket(1, n, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(native, native));
		assert_eq!(core.foreign[1], 1);
		assert!(core.check_foreign_counts());
	}

	#[test]
	fn foreign_put_normal_at_index_tracks_evictee() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let n = s.num_of_normal_buckets;
		let foreign_keys = find_keys_for_bucket(0, n, s.bucket_size);
		for &k in &foreign_keys {
			core.put_normal_at_index(1, InsertPosition::Back, Item::new(k, k));
		}
		assert_eq!(core.foreign[1] as usize, s.bucket_size);
		let native = find_keys_for_bucket(1, n, 1)[0];
		let evicted = core.put_normal_at_index(1, InsertPosition::Back, Item::new(native, native));
		assert!(evicted.is_some());
		assert_eq!(core.foreign[1] as usize, s.bucket_size - 1);
		assert!(core.check_foreign_counts());
		assert!(core.check_tags_consistent());
	}

	// ── stash_indices ────────────────────────────────────────────────────────

	#[test]
	fn stash_indices_are_adjacent_with_wrap() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let core = SegmentCore::<i64, i64>::new(&s);
		for k in 0i64..50 {
			let (s0, s1) = core.stash_indices(&k);
			assert_eq!(s0, get_index(&k, 4));
			assert_eq!(s1, (s0 + 1) % 4);
		}
	}

	#[test]
	fn contains_key_sees_neighbor_stash_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(1, 4, 1)[0];
		core.put_stash_at_index(2, InsertPosition::Back, Item::new(key, key));
		assert!(core.contains_key(&key));
		assert!(core.check_slot_legality());
		assert!(core.check_borrowed_counts());
		assert!(core.check_foreign_counts());
	}

	// ── concrete bucket storage ──────────────────────────────────────────────

	#[test]
	fn buckets_are_concrete_and_statically_usable() {
		use crate::shared::traits::bucket::Bucket;
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Target, InsertPosition::Back, Item::new(key, 5));
		// Still usable through the trait object view.
		let as_dyn: &dyn Bucket<i64, i64> = &core.buckets[0];
		assert_eq!(as_dyn.get_position(&key), Some(0));
		assert_eq!(core.buckets[0].get_position(&key), Some(0));
		assert_eq!(core.buckets[0].get_from_position(0).value, 5);
	}

	// ── tag filter ───────────────────────────────────────────────────────────

	#[test]
	fn tagged_lookup_matches_plain_lookup() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let n = s.num_of_normal_buckets;
		let keys = find_keys_for_bucket(0, n, s.bucket_size);
		for &k in &keys {
			core.put_at(Slot::Target, InsertPosition::Front, Item::new(k, k));
		}
		assert!(core.check_tags_consistent());
		for k in 0i64..200 {
			let tag = tag_of_hash(hash(&k));
			assert_eq!(core.buckets[0].get_position_tagged(&k, tag), core.buckets[0].get_position(&k));
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
		assert!(core.check_foreign_counts());
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
		assert!(core.check_foreign_counts());
	}

	#[test]
	fn check_slot_legality_probing_slot_is_legal() {
		let s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// A key at target bucket 0 placed in its probing bucket 1 must be legal.
		let key = find_keys_for_bucket(0, s.num_of_normal_buckets, 1)[0];
		core.put_at(Slot::Probing, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
		assert!(core.check_foreign_counts());
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
		assert_eq!(core.foreign[0], 1);
		assert!(core.check_foreign_counts());
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
		// With two-choice placement, stash 1 and 2 are legal for a key whose s0 is 1;
		// stash 3 is not. Use 4 stash buckets so an illegal stash bucket exists.
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		let key = (0i64..200)
			.find(|k| get_index(k, s.num_of_stash_buckets) == 1)
			.expect("no key hashes to stash 1");
		core.stash_buckets[3].get_items_mut().push(Item::new(key, key));
		assert!(!core.check_slot_legality());
	}

	#[test]
	fn check_slot_legality_accepts_neighbor_stash_bucket() {
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_stash_buckets = 4;
		let mut core = SegmentCore::<i64, i64>::new(&s);
		// s0 = 3 wraps to s1 = 0.
		let key = (0i64..200)
			.find(|k| get_index(k, s.num_of_stash_buckets) == 3)
			.expect("no key hashes to stash 3");
		core.put_stash_at_index(0, InsertPosition::Back, Item::new(key, key));
		assert!(core.check_slot_legality());
		assert_eq!(core.borrowed[0], 1);
		assert!(core.check_borrowed_counts());
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
