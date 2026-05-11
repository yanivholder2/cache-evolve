mod combined_segment;
mod promotion_cold_segment;
mod promotion_hot_segment;
mod promotion_logic;

pub use combined_segment::{CombinedSegment, EvictionTieBreaker};
pub use promotion_cold_segment::PromotionColdSegment;
pub use promotion_hot_segment::PromotionHotSegment;

pub(crate) use promotion_logic::promotion_get;

use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::get_index;
use super::dash_bucket::DashBucket;
use super::dash_settings::DashSettings;
use log::debug;
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
	pub fn new(settings: &DashSettings) -> Self {
		assert!(settings.num_of_normal_buckets >= 2, "probing requires num_of_normal_buckets >= 2");
		assert!(settings.num_of_stash_buckets > 0, "num_of_stash_buckets must be > 0");
		let buckets: Vec<_> = (0..settings.num_of_normal_buckets)
			.map(|_| DashBucket::new(settings.bucket_size, settings.eviction_policy))
			.collect();
		let stash_buckets: Vec<_> = (0..settings.num_of_stash_buckets)
			.map(|_| DashBucket::new(settings.bucket_size, settings.eviction_policy))
			.collect();
		SegmentCore {
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
			buckets,
			stash_buckets,
			eviction_policy: settings.eviction_policy,
		}
	}

	/// Returns `(landed_at, Option<evicted>)`. `landed_at` is the `InsertPosition` end of
	/// the resolved bucket where the new item now lives, so callers can address it as
	/// index `0` (Front) or `bucket.size() - 1` (Back) without a `get_position` rescan.
	pub fn put_at(&mut self, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> (InsertPosition, Option<Item<K, V>>) {
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
		let (landed_at, evicted) = bucket.put_at(item, pos);
		if let Some(evicted) = &evicted {
			debug!("SegmentCore.put_at: slot={slot:?} pos={pos:?} evicted_key={:?}", evicted.key);
		}
		(landed_at, evicted)
	}
}

pub trait Segment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn core(&self) -> &SegmentCore<K, V>;
	fn core_mut(&mut self) -> &mut SegmentCore<K, V>;

	fn get(&mut self, key: &K) -> Option<&Item<K, V>>;

	/// Default routes new items to stash.
	fn put(&mut self, item: Item<K, V>) {
		debug!("Segment.put: key={:?}", item.key);
		self.core_mut().put_at(Slot::Stash, InsertPosition::Back, item);
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
}
