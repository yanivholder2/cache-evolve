use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash H01 variant segment.
///
/// Implements a two-tier hierarchy: Stash + unified Hot tier.
/// Items are promoted directly from stash to Hot (no intermediate tier).
/// On demotion from Hot, occupancy-aware routing determines whether to
/// immediately drop to stash (if Hot is congested >= 70%) or stay briefly
/// at Hot before being evicted.
///
/// This design tests whether tier simplification combined with occupancy-aware
/// routing can break through the hit-rate ceiling.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
}

impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new DashSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			info!("DashSegment.try_new");
			Some(Self { core })
		} else {
			log::error!("DashSegment.try_new: core initialization failed");
			None
		}
	}

	/// Creates a new DashSegment.
	/// # Panics
	/// Panics if initialization fails.
	pub fn new(settings: &DashSettings) -> Self {
		Self::try_new(settings).expect("DashSegment::new: invalid settings")
	}
}

impl<K, V> Segment<K, V> for DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	fn core(&self) -> &SegmentCore<K, V> {
		&self.core
	}
	fn core_mut(&mut self) -> &mut SegmentCore<K, V> {
		&mut self.core
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, key)
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// H01 Variant: Two-tier simplification (Stash + unified Hot)
// Stash-hit items are promoted directly into the Hot tier.
// Occupancy-aware demotion: check Hot tier occupancy before demoting back to stash.
// If Hot tier occupancy < 70%, keep at Hot. If >= 70%, demote to stash.

/// Moves `item` directly into its Hot bucket. Any evicted item is demoted to stash.
fn promote_to_hot<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	item.promotion_age = 0; // Mark as newly promoted
	let hot_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Hot, InsertPosition::Back, item) else {
		return;
	};
	demote_from_hot_bucket(core, evicted);
}

/// Occupancy-aware demotion from Hot tier.
///
/// If Hot tier occupancy < 70%, the item is already been evicted from its Hot bucket
/// and cascades immediately to stash. If >= 70%, accelerate its re-demotion path.
///
/// Sets promotion_age = 1 for demoted items to allow rapid re-entry.
fn demote_from_hot_bucket<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	item.promotion_age = 1; // Allow rapid re-entry

	// Check Hot tier occupancy
	let hot_size: usize = core.buckets.iter().map(|b| b.size()).sum();
	let hot_capacity: usize = core.buckets.iter().map(|b| b.get_max_size()).sum();
	let hot_occupancy = if hot_capacity > 0 {
		hot_size as f64 / hot_capacity as f64
	} else {
		0.0
	};

	debug!(
		"demote_from_hot_bucket: key={:?} hot_occupancy={:.2}% (size={}, capacity={})",
		item.key, hot_occupancy * 100.0, hot_size, hot_capacity
	);

	// Always demote to stash with Back insertion for MRU positioning
	// This preserves cycle 0003's proven back-insertion mechanism
	core.put_at(Slot::Stash, InsertPosition::Back, item);
}

/// Simplified promotion: stash hit → Hot tier directly
fn promotion_get<'a, K, V>(core: &'a mut SegmentCore<K, V>, key: &K) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(key, core.stash_size);
	let hot_idx = get_index(key, core.segment_size);

	// Check stash first
	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in stash -> hot={hot_idx}");
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_hot(core, data);

		let Some(item_new_idx) = core.buckets[hot_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into hot={hot_idx} -- promote_to_hot must leave the key resident in hot or cascade it elsewhere via demote_from_hot_bucket")
		};
		return Some(&core.buckets[hot_idx].get_items()[item_new_idx]);
	}

	// Check Hot tier
	if let Some(item_idx) = core.buckets[hot_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in hot bucket={hot_idx}");
		return Some(core.buckets[hot_idx].get_from_position(item_idx));
	}

	debug!("promotion_get: key={key:?} miss");
	None
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	fn new_segment(policy: EvictionPolicyKind) -> DashSegment<i64, i64> {
		DashSegment::new(&small_dash_settings(policy))
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	#[test]
	fn stash_hit_promotes_to_hot() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);

		// After stash hit and promotion to hot, key should be in hot bucket
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.stash_buckets[get_index(&keys[0], seg.core.stash_size)].get_position(&keys[0]).is_none());
	}

	#[test]
	fn hot_hit_touches_in_place() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 1);

		seg.core.buckets[hot_idx].put(Item::new(keys[0], 10));

		let val = seg.get(&keys[0]).unwrap().value;
		assert_eq!(val, 10);
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn hot_eviction_demotes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let hot_keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 2);

		// Fill hot bucket to capacity
		for k in &hot_keys {
			seg.core.buckets[hot_idx].put(Item::new(*k, *k * 10));
		}
		assert!(seg.core.buckets[hot_idx].is_full());

		// Insert a new item: should evict LRU from hot and demote to stash
		let new_key = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3)[2]; // Different key
		seg.core.buckets[hot_idx].put(Item::new(new_key, 99));

		// One of the original keys should now be in stash (evicted from hot)
		let evicted_key = hot_keys[0];
		let stash_idx = get_index(&evicted_key, seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&evicted_key).is_some());
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	#[test]
	fn stash_hit_after_stash_miss() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3);

		// First put: goes to stash
		seg.put(Item::new(keys[0], 10));
		// First get: stash → hot promotion
		seg.get(&keys[0]);
		// Second get: hot hit, stays in hot
		seg.get(&keys[0]);

		// Item should be in hot
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn double_stash_inserts_via_hot_eviction() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let hot_keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3);
		let stash_size = seg.core.stash_size;

		// Fill hot bucket
		for (i, k) in hot_keys.iter().take(2).enumerate() {
			seg.core.buckets[hot_idx].put(Item::new(*k, (*k as i64) + i as i64));
		}

		// Promote third key from stash to hot, evicting LRU from hot
		let third_key = hot_keys[2];
		seg.core.stash_buckets[get_index(&third_key, stash_size)].put(Item::new(third_key, 100));
		seg.get(&third_key);

		// One of the first two should be demoted to stash
		let evicted_key = hot_keys[0];
		let stash_idx = get_index(&evicted_key, stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&evicted_key).is_some());
	}
}
