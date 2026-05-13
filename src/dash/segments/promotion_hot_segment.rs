use super::{promotion_get, Segment, SegmentCore};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use log::info;
use std::fmt::Debug;
use std::hash::Hash;

#[derive(Debug)]
pub struct PromotionHotSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	core: SegmentCore<K, V>,
}

impl<K, V> PromotionHotSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Creates a new PromotionHotSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		let core = SegmentCore::try_new(settings);
		if core.is_none() {
			log::error!("PromotionHotSegment.try_new: core initialization failed");
			return None;
		}
		info!("PromotionHotSegment.try_new");
		Some(Self {
			core: core.unwrap(),
		})
	}

	/// Creates a new PromotionHotSegment.
	/// # Panics
	/// Panics if initialization fails.
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("PromotionHotSegment::new: invalid settings")
	}
}

impl<K, V> Segment<K, V> for PromotionHotSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn core(&self) -> &SegmentCore<K, V> {
		&self.core
	}
	fn core_mut(&mut self) -> &mut SegmentCore<K, V> {
		&mut self.core
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, key, InsertPosition::Back)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicy;
	use crate::shared::traits::bucket::Bucket;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	fn hot_segment(policy: EvictionPolicy) -> PromotionHotSegment<i64, i64> {
		PromotionHotSegment::new(&small_dash_settings(policy))
	}

	#[test]
	fn stash_hit_inserts_at_back_of_target() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Hot insert places at Back (MRU), so insertion order matches array order.
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn stash_hit_at_mru_means_one_access_promotes_to_probing() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// keys[1] lands at MRU on stash-hit insertion, so a single subsequent access
		// is enough to promote — this is the defining contrast with the cold variant.
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
	}

	#[test]
	fn probing_hit_touches_in_place() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.core.buckets[probing_idx].put(Item::new(keys[0], 10));
		let val = seg.get(&keys[0]).unwrap().value;
		assert_eq!(val, 10);
		assert!(seg.core.buckets[probing_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn probing_eviction_demotes_to_target() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Sole resident is already at MRU; one get promotes and evicts probing's LRU.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
		seg.get(&target_keys[0]);

		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	/// Regression for the last-bucket probing wraparound (commit 4b0b763): keys whose target
	/// is `segment_size - 1` must probe at bucket 0, not at `segment_size` (out of bounds).
	#[test]
	fn last_bucket_probing_wraps_to_zero() {
		let mut seg = hot_segment(EvictionPolicy::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = seg_size - 1;
		let probing_idx = (target_idx + 1) % seg_size;
		assert_eq!(probing_idx, 0, "sanity: wraparound math holds");
		let keys = find_keys_for_bucket(target_idx, seg_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		// Hot variant: keys[1] lands at MRU on stash-hit; one more get promotes to probing.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}
}
