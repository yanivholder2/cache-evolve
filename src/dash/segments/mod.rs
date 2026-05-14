mod combined_segment;
mod promotion_cold_segment;
mod promotion_hot_segment;
mod promotion_logic;

pub use combined_segment::{CombinedSegment, EvictionTieBreaker};
pub use promotion_cold_segment::PromotionColdSegment;
pub use promotion_hot_segment::PromotionHotSegment;

pub(crate) use promotion_logic::promotion_get;

use super::dash_bucket::DashBucket;
use super::dash_settings::DashSettings;
use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::get_index;
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

#[derive(Debug)]
pub struct SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub(crate) buckets: Vec<DashBucket<K, V>>,
	pub(crate) segment_size: usize,
	pub(crate) stash_buckets: Vec<DashBucket<K, V>>,
	pub(crate) stash_size: usize,
	/// Eviction policy is segment-wide (every bucket uses the same one). Stored here so
	/// segment-level logic can read it without reaching into a specific bucket.
	pub(crate) eviction_policy: EvictionPolicy,
}

impl<K, V> SegmentCore<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
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
			.map(|_| DashBucket::new(settings.bucket_size, settings.eviction_policy))
			.collect();
		let stash_buckets: Vec<_> = (0..settings.num_of_stash_buckets)
			.map(|_| DashBucket::new(settings.bucket_size, settings.eviction_policy))
			.collect();
		Some(SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			eviction_policy: settings.eviction_policy,
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

/// A Segment owns one [`SegmentCore`] (the shared normal + stash buckets) and
/// decides *where* items go on `get` (promotion, peer-placement, touch-in-place)
/// while deferring per-bucket ordering to [`EvictionPolicy`].
///
/// # Contract
///
/// **Mutation scope of `get`:** implementations may freely move items between
/// slots (stash → target, target → probing) during a `get`. Callers must not
/// assume the item's slot is unchanged after a hit.
///
/// **`put` precondition:** the key must not already be resident anywhere in the
/// segment. The debug assertion in the default `put` enforces this. A caller that
/// re-inserts a resident key violates the uniqueness invariant (ADR 0004) and
/// will produce a phantom duplicate that can never be evicted through the normal
/// path.
///
///
/// **Stash overflow:** when the stash bucket is full, the weakest stash resident
/// is evicted to make room for the arriving item. Do not retry; stash overflow
/// is the defined end of the line.
pub trait Segment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Shared reference to the segment's normal and stash buckets.
	fn core(&self) -> &SegmentCore<K, V>;

	/// Exclusive reference to the segment's normal and stash buckets.
	fn core_mut(&mut self) -> &mut SegmentCore<K, V>;

	/// Look up `key` and return a reference to its [`Item`] if present.
	///
	/// **Postcondition:** returns `Some(&item)` where `item.key == *key` if the
	/// key is resident in any slot (stash, target, or probing); `None` on a miss.
	/// Implementations must search all three slots.
	///
	/// **Mutation:** implementations may reorder items within a bucket (the common
	/// case) or move them between slots (e.g., stash → target on promotion).
	/// The returned reference reflects the item's location after any such move.
	/// Depends on the uniqueness invariant (ADR 0004) holding.
	fn get<'a>(&'a mut self, key: &K) -> Option<&'a Item<K, V>>;

	/// Insert a new `item` into the segment.
	///
	/// **Precondition:** `item.key` must not already be resident anywhere in the
	/// segment. Violated in debug builds via a `debug_assert` that panics; in
	/// release builds, a duplicate insertion silently creates a phantom item
	/// that can never be retrieved or evicted.
	///
	/// **Default behaviour:** routes the item to its stash bucket
	/// (`Slot::Stash`, `InsertPosition::Back`). This is the recommended default
	/// for new Segment implementations. Override only if the Variant requires
	/// fundamentally different initial placement (rare).
	fn put(&mut self, item: Item<K, V>) {
		let core = self.core_mut();
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("Segment.put: key={:?}", item.key);
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicy;
	use crate::test_utils::small_dash_settings;

	fn assert_put_goes_to_stash(mut seg: impl Segment<i64, i64>) {
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	#[test]
	fn all_segments_put_goes_to_stash() {
		let lru = small_dash_settings(EvictionPolicy::ClassicLRU);
		let ts = small_dash_settings(EvictionPolicy::TimestampLRU);
		assert_put_goes_to_stash(PromotionColdSegment::new(&lru));
		assert_put_goes_to_stash(PromotionHotSegment::new(&lru));
		assert_put_goes_to_stash(CombinedSegment::new(&ts, EvictionTieBreaker::EvictProbing));
	}

	#[test]
	fn try_new_rejects_single_normal_bucket() {
		let mut s = small_dash_settings(EvictionPolicy::ClassicLRU);
		s.num_of_normal_buckets = 1;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}

	#[test]
	fn try_new_rejects_zero_stash_buckets() {
		let mut s = small_dash_settings(EvictionPolicy::ClassicLRU);
		s.num_of_stash_buckets = 0;
		assert!(s.validate().is_err());
		assert!(SegmentCore::<i64, i64>::try_new(&s).is_none());
	}
}
