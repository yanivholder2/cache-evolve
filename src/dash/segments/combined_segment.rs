//! Combined strategy: target and probing buckets are peer storage (no tier). On insertion, when a
//! normal bucket must evict, the evicted item drops straight to stash.

use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::get_index;
use log::{debug, warn};
use std::cmp::Ordering;
use std::fmt::Debug;
use std::hash::Hash;

/// Which bucket to evict from when target's and probing's eviction candidates are
/// equally weak.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionTieBreaker {
	EvictTarget,
	EvictProbing,
}

#[derive(Debug)]
pub struct CombinedSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	core: SegmentCore<K, V>,
	tie_breaker: EvictionTieBreaker,
}

impl<K, V> CombinedSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &DashSettings, tie_breaker: EvictionTieBreaker) -> Self {
		assert!(
			matches!(settings.eviction_policy, EvictionPolicy::TimestampLRU | EvictionPolicy::Lfu),
			"CombinedSegment requires TimestampLRU or Lfu, got {:?}", settings.eviction_policy
		);
		Self {
			core: SegmentCore::new(settings),
			tie_breaker,
		}
	}

	/// Places `item` into a normal bucket (Target or Probing) — peer-placement verb. Any
	/// evicted item drops straight to stash; no tier cascade, since the combined eviction
	/// already compared candidates across both buckets.
	fn place_in_normal_bucket(&mut self, slot: Slot, item: Item<K, V>) {
		debug_assert!(!matches!(slot, Slot::Stash), "place_in_normal_bucket: Slot::Stash is invalid");
		if let Some(evicted) = self.core.put_at(slot, InsertPosition::Back, item) {
			debug!("CombinedSegment.place_in_normal_bucket: slot={slot:?} evicted_key={:?} -> stash", evicted.key);
			if let Some(dropped) = self.core.put_at(Slot::Stash, InsertPosition::Back, evicted) {
				warn!("CombinedSegment.place_in_normal_bucket: stash overflow, dropping key={:?}", dropped.key);
			}
		}
	}

	/// True iff target's eviction candidate is weaker than probing's. On a tie, the
	/// configured `EvictionTieBreaker` decides.
	///
	/// Precondition: both buckets are full. Eviction is only chosen when target+probing
	/// are jointly full, so each bucket has at least one item and `eviction_candidate_index`
	/// must return `Some`. Any `None` here violates that contract.
	fn should_evict_from_target(&self, target_idx: usize, probing_idx: usize) -> bool {
		let target_candidate = self.core.buckets[target_idx].eviction_candidate_index();
		let probing_candidate = self.core.buckets[probing_idx].eviction_candidate_index();

		let (Some(target_pos), Some(probing_pos)) = (target_candidate, probing_candidate) else {
			unreachable!(
				"should_evict_from_target: called with empty bucket (target={target_idx}, probing={probing_idx}) -- both buckets must be full"
			)
		};

		let ordering = match self.core.eviction_policy {
			EvictionPolicy::TimestampLRU => {
				let target_timestamp = self.core.buckets[target_idx].get_items()[target_pos].timestamp;
				let probing_timestamp = self.core.buckets[probing_idx].get_items()[probing_pos].timestamp;
				target_timestamp.cmp(&probing_timestamp)
			}
			EvictionPolicy::Lfu => {
				let target_counter = self.core.buckets[target_idx].get_items()[target_pos].lfu_counter;
				let probing_counter = self.core.buckets[probing_idx].get_items()[probing_pos].lfu_counter;
				target_counter.cmp(&probing_counter)
			}
			other => unreachable!(
				"should_evict_from_target: unsupported policy {other:?} -- combined segments require TimestampLRU or Lfu"
			)
		};
		match ordering {
			Ordering::Less => true,
			Ordering::Greater => false,
			Ordering::Equal => self.tie_breaker == EvictionTieBreaker::EvictTarget,
		}
	}
}

impl<K, V> Segment<K, V> for CombinedSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn core(&self) -> &SegmentCore<K, V> { &self.core }
	fn core_mut(&mut self) -> &mut SegmentCore<K, V> { &mut self.core }

	/// On a stash hit, peer-places the item into target+probing. Existing target/probing hits touch in
	/// place.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let stash_idx = get_index(key, self.core.stash_size);
		let target_idx = get_index(key, self.core.segment_size);
		let probing_idx = (target_idx + 1) % self.core.segment_size;

		if let Some(pos) = self.core.stash_buckets[stash_idx].get_position(key) {
			debug!("CombinedSegment.get: key={key:?} found in stash");
			let data = self.core.stash_buckets[stash_idx].remove_at(pos);

			// Treat target+probing as one logical bucket: only evict when both are full.
			let target_full = self.core.buckets[target_idx].is_full();
			let probing_full = self.core.buckets[probing_idx].is_full();

			let slot = if !target_full {
				debug!("CombinedSegment.get: key={key:?} insert target={target_idx} (has space)");
				Slot::Target
			} else if !probing_full {
				debug!("CombinedSegment.get: key={key:?} insert probing={probing_idx} (target full, probing has space)");
				Slot::Probing
			} else if self.should_evict_from_target(target_idx, probing_idx) {
				debug!("CombinedSegment.get: key={key:?} evict and insert in target={target_idx}");
				Slot::Target
			} else {
				debug!("CombinedSegment.get: key={key:?} evict and insert in probing={probing_idx}");
				Slot::Probing
			};
			self.place_in_normal_bucket(slot, data);

			let bucket_idx = match slot {
				Slot::Target => target_idx,
				Slot::Probing => probing_idx,
				Slot::Stash => unreachable!("place_in_normal_bucket rejects Slot::Stash"),
			};
			let Some(pos) = self.core.buckets[bucket_idx].get_position(key) else {
				unreachable!("CombinedSegment.get: key={key:?} not found after insert into {slot:?}={bucket_idx} -- place_in_normal_bucket must leave the key resident in the chosen bucket")
			};
			return Some(&self.core.buckets[bucket_idx].get_items()[pos]);
		}

		if let Some(pos) = self.core.buckets[target_idx].get_position(key) {
			debug!("CombinedSegment.get: key={key:?} found in target bucket={target_idx}");
			return Some(self.core.buckets[target_idx].get_from_position(pos));
		}

		if let Some(pos) = self.core.buckets[probing_idx].get_position(key) {
			debug!("CombinedSegment.get: key={key:?} found in probing bucket={probing_idx}");
			return Some(self.core.buckets[probing_idx].get_from_position(pos));
		}

		debug!("CombinedSegment.get: key={key:?} miss");
		None
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicy;
	use crate::shared::traits::bucket::Bucket;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	fn combined_segment(policy: EvictionPolicy) -> CombinedSegment<i64, i64> {
		CombinedSegment::new(&small_dash_settings(policy), EvictionTieBreaker::EvictProbing)
	}

	#[test]
	fn target_not_full_inserts_into_target() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);

		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn target_full_probing_has_space_inserts_into_probing() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		assert!(seg.core.buckets[target_idx].is_full());
		assert!(!seg.core.buckets[probing_idx].is_full());

		seg.put(Item::new(keys[2], 30));
		seg.get(&keys[2]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[2]).is_some(),
			"item should be in probing bucket when target is full and probing has space");
	}

	#[test]
	fn both_full_evicts_weaker_candidate() {
		let mut seg = combined_segment(EvictionPolicy::Lfu);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		// Probing items get boosted counters so they outrank target's items.
		const LFU_BOOST_ROUNDS: usize = 5;
		for _ in 0..LFU_BOOST_ROUNDS {
			for &k in &probing_keys {
				seg.core.buckets[probing_idx].get(&k);
			}
		}

		assert!(seg.core.buckets[target_idx].is_full());
		assert!(seg.core.buckets[probing_idx].is_full());

		seg.put(Item::new(target_keys[2], 30));
		seg.get(&target_keys[2]);

		assert!(
			seg.core.buckets[target_idx].get_position(&target_keys[2]).is_some()
				|| seg.core.buckets[probing_idx].get_position(&target_keys[2]).is_some()
		);
	}

	#[test]
	fn target_hit_touches_in_place() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);

		let val = seg.get(&keys[0]).unwrap().value;
		assert_eq!(val, 10);
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn both_full_timestamp_lru_evicts_older() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		// Target fills first, so its items hold the older timestamps.
		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		// Refresh probing items so they hold the newer timestamps.
		for &k in &probing_keys {
			seg.core.buckets[probing_idx].get(&k);
		}

		assert!(seg.core.buckets[target_idx].is_full());
		assert!(seg.core.buckets[probing_idx].is_full());

		seg.put(Item::new(target_keys[2], 30));
		seg.get(&target_keys[2]);

		assert!(
			seg.core.buckets[target_idx].get_position(&target_keys[2]).is_some()
				|| seg.core.buckets[probing_idx].get_position(&target_keys[2]).is_some()
		);
		for &k in &probing_keys {
			assert!(seg.core.buckets[probing_idx].get_position(&k).is_some(),
				"probing items with newer timestamps should not be evicted");
		}
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		assert!(seg.get(&999).is_none());
	}

	#[test]
	fn tie_breaker_evict_probing_evicts_from_probing() {
		let settings = small_dash_settings(EvictionPolicy::Lfu);
		let mut seg = CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg_size, 2);

		// All items end up at LFU counter 0 → tied weakness comparison.
		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
		for &k in &probing_keys {
			seg.core_mut().buckets[probing_idx].put(Item::new(k, k * 10));
		}

		seg.put(Item::new(target_keys[2], 30));
		seg.get(&target_keys[2]);

		let probing_count: usize = probing_keys.iter()
			.filter(|k| seg.core().buckets[probing_idx].get_position(k).is_some())
			.count();
		assert!(probing_count < probing_keys.len(), "EvictProbing should evict from probing");
	}

	#[test]
	fn tie_breaker_evict_target_evicts_from_target() {
		let settings = small_dash_settings(EvictionPolicy::Lfu);
		let mut seg = CombinedSegment::new(&settings, EvictionTieBreaker::EvictTarget);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg_size, 2);

		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
		for &k in &probing_keys {
			seg.core_mut().buckets[probing_idx].put(Item::new(k, k * 10));
		}

		seg.put(Item::new(target_keys[2], 30));
		seg.get(&target_keys[2]);

		let target_originals: usize = [target_keys[0], target_keys[1]].iter()
			.filter(|k| seg.core().buckets[target_idx].get_position(k).is_some())
			.count();
		assert!(target_originals < 2, "EvictTarget should evict from target");
	}

	/// Eviction only happens when target+probing is jointly full — never when there's
	/// free space anywhere in the joint bucket.
	#[test]
	fn does_not_evict_when_target_has_space() {
		let settings = small_dash_settings(EvictionPolicy::Lfu);
		let mut seg = CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg_size, 2);

		// Probing is full of counter-0 items; target is empty.
		for &k in &probing_keys {
			seg.core_mut().buckets[probing_idx].put(Item::new(k, k * 10));
		}

		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);

		assert!(seg.core().buckets[target_idx].get_position(&target_keys[0]).is_some(),
			"new item should land in target when it has space");
		for &k in &probing_keys {
			assert!(seg.core().buckets[probing_idx].get_position(&k).is_some(),
				"probing items must not be evicted while target has free space");
		}
	}

	/// Regression for the last-bucket probing wraparound (commit 4b0b763): keys whose target
	/// is `segment_size - 1` must probe at bucket 0, not at `segment_size` (out of bounds).
	#[test]
	fn last_bucket_probing_wraps_to_zero() {
		let mut seg = combined_segment(EvictionPolicy::TimestampLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = seg_size - 1;
		let probing_idx = (target_idx + 1) % seg_size;
		assert_eq!(probing_idx, 0, "sanity: wraparound math holds");
		let keys = find_keys_for_bucket(target_idx, seg_size, 3);

		// Fill target via stash hits; third stash-hit overflows into wrapped probing.
		for &k in &keys {
			seg.put(Item::new(k, k * 10));
			seg.get(&k);
		}

		assert!(
			seg.core.buckets[target_idx].get_position(&keys[2]).is_some()
				|| seg.core.buckets[probing_idx].get_position(&keys[2]).is_some(),
			"newest item must land in target or wrapped probing bucket 0"
		);
	}

	/// An evicted item must land in stash, not in its target — combined buckets are
	/// peer storage, so a tier cascade would undo the weakness comparison.
	#[test]
	fn evicted_item_goes_to_stash_not_target() {
		let settings = small_dash_settings(EvictionPolicy::Lfu);
		let mut seg = CombinedSegment::new(&settings, EvictionTieBreaker::EvictProbing);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;

		let keys_for_0 = find_keys_for_bucket(target_idx, seg_size, 4);
		let keys_for_1 = find_keys_for_bucket(probing_idx, seg_size, 1);
		// `misplaced` targets bucket 0 but is planted in bucket 1.
		let misplaced = keys_for_0[2];

		// Bucket 0: strong residents (counter 1 each).
		seg.core_mut().buckets[target_idx].put(Item::new(keys_for_0[0], 10));
		seg.core_mut().buckets[target_idx].put(Item::new(keys_for_0[1], 20));
		seg.core_mut().buckets[target_idx].get(&keys_for_0[0]);
		seg.core_mut().buckets[target_idx].get(&keys_for_0[1]);

		// Bucket 1: `misplaced` at counter 0 (globally weakest) alongside a strong resident.
		seg.core_mut().buckets[probing_idx].put(Item::new(misplaced, 30));
		seg.core_mut().buckets[probing_idx].put(Item::new(keys_for_1[0], 40));
		seg.core_mut().buckets[probing_idx].get(&keys_for_1[0]);

		let trigger = keys_for_0[3];
		seg.put(Item::new(trigger, 100));
		seg.get(&trigger);

		let in_stash = seg.core().stash_buckets.iter()
			.any(|b| b.get_position(&misplaced).is_some());
		let in_target = seg.core().buckets[target_idx].get_position(&misplaced).is_some();
		let in_probing = seg.core().buckets[probing_idx].get_position(&misplaced).is_some();

		assert!(!in_probing, "sanity: misplaced was evicted from bucket 1");
		assert!(!in_target,
			"combined eviction must NOT cascade the evicted item into its target bucket");
		assert!(in_stash, "combined eviction must route the evicted item to stash");
	}
}
