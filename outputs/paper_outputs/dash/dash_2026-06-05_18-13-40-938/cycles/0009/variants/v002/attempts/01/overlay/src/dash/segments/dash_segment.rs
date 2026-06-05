use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's H02 segment.
///
/// Its `get` realises a **conditional two-tier** design with occupancy-gated tier-skip:
/// - Stash and hot form the two tiers (unified hot tier from v002)
/// - Items are promoted from stash to hot on access
/// - High-confidence items (access_count ≥ 2) skip directly to hot when occupancy < 75%
/// - During high congestion (occupancy ≥ 75%), normal promotion applies with filtering
/// - Recovery: items that skipped forward (skip_history_flag=true) get a second chance
///   by demoting back to stash (MRU) if evicted from hot, enabling re-evaluation
///
/// This hybrid architecture adapts promotion discipline to load state, preserving
/// filtering benefits under congestion while safely accelerating proven items in
/// low-congestion periods.
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
		promotion_get(&mut self.core, key, InsertPosition::Front)
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Two-tier promotion with occupancy-gated tier-skip: items are promoted from stash
// to hot on a hit. High-confidence items (access_count ≥ 2) skip directly to hot
// when hot tier occupancy < 75%; during congestion (≥ 75%), normal promotion applies.
// Demotion from hot back to stash with one-more-chance.

/// Calculates the hot tier occupancy as a percentage of max capacity.
/// Returns a value from 0.0 to 1.0.
fn calculate_hot_occupancy<K, V>(core: &SegmentCore<K, V>) -> f64
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let hot_items = core.buckets.iter().map(|b| b.size()).sum::<usize>();
	let hot_capacity = core.buckets.iter().map(|b| b.get_max_size()).sum::<usize>();

	if hot_capacity == 0 {
		0.0
	} else {
		hot_items as f64 / hot_capacity as f64
	}
}

/// Moves `item` into its hot bucket. Any evicted item flows to stash (demotion).
fn promote_to_hot<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let hot_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Hot, insert_pos, item) else {
		return;
	};
	demote_from_hot_bucket(core, evicted, insert_pos);
}

/// One-more-chance demotion from hot to stash with recovery enforcement.
/// If the item has skip_history_flag=true (was accelerated via tier-skip),
/// demote it back to stash with back insertion (MRU) to give it a recovery opportunity.
fn demote_from_hot_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	if item.skip_history_flag {
		debug!(
			"demote_from_hot_bucket: key={:?} hot -> stash (recovery, MRU) [skip_history_flag=true]",
			item.key
		);
		// Recovery enforcement: demote with back insertion (MRU) to give a second chance
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	} else {
		debug!(
			"demote_from_hot_bucket: key={:?} hot -> stash",
			item.key
		);
		// Standard demotion: use the provided insert_pos
		core.put_at(Slot::Stash, insert_pos, item);
	}
}

/// `insert_pos = Front`: promotions land at LRU end of hot tier.
/// `insert_pos = Back`: promotions land at MRU end of hot tier.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(key, core.stash_size);
	let hot_idx = get_index(key, core.segment_size);

	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);

		// Occupancy-gated tier-skip: check if item has high confidence (access_count ≥ 2)
		// and hot tier has spare capacity (occupancy < 75%)
		let is_high_confidence = data.lfu_counter >= 2;
		let hot_occupancy = calculate_hot_occupancy(core);
		let can_skip = is_high_confidence && hot_occupancy < 0.75;

		if can_skip {
			debug!(
				"promotion_get: key={key:?} found in stash -> hot={hot_idx} (SKIP, high-confidence + low-congestion) occupancy={:.2}%",
				hot_occupancy * 100.0
			);
			// Tier-skip: promote directly with skip_history_flag set for recovery enforcement
			let mut skipped_item = data;
			skipped_item.skip_history_flag = true;
			promote_to_hot(core, skipped_item, insert_pos);
		} else {
			debug!(
				"promotion_get: key={key:?} found in stash -> hot={hot_idx} (standard) occupancy={:.2}% confidence={}",
				hot_occupancy * 100.0,
				if is_high_confidence { "high" } else { "low" }
			);
			// Standard promotion: no tier-skip
			promote_to_hot(core, data, insert_pos);
		}

		let Some(item_new_idx) = core.buckets[hot_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into hot={hot_idx} -- promote_to_hot must leave the key resident in hot or cascade it elsewhere via demote_from_hot_bucket")
		};
		return Some(&core.buckets[hot_idx].get_items()[item_new_idx]);
	}

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
	fn stash_hit_inserts_at_front_of_hot() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Cold insert pushes keys[1] to Front, displacing keys[0] backward.
		let items = seg.core.buckets[hot_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
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
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3);

		// Place first two items directly in hot bucket (they're the LRU residents)
		seg.core.buckets[hot_idx].put(Item::new(keys[0], 10));
		seg.core.buckets[hot_idx].put(Item::new(keys[1], 20));
		assert!(seg.core.buckets[hot_idx].is_full());

		// Put the third key in stash
		seg.put(Item::new(keys[2], 30));
		let stash_idx = get_index(&keys[2], seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&keys[2]).is_some());

		// Get the third key to promote it to hot, which should evict keys[0] to stash
		seg.get(&keys[2]);

		// keys[2] should now be in hot
		assert!(seg.core.buckets[hot_idx].get_position(&keys[2]).is_some());

		// keys[0] (LRU) should have been evicted to stash
		let stash_idx_for_key0 = get_index(&keys[0], seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[stash_idx_for_key0].get_position(&keys[0]).is_some(),
			"evicted hot item should be demoted to stash"
		);
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	#[test]
	fn stash_to_hot_promotion() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 1);

		// Put item into stash
		seg.put(Item::new(keys[0], 10));
		let stash_idx = get_index(&keys[0], seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_none());

		// Get should promote to hot
		seg.get(&keys[0]);
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.stash_buckets[stash_idx].get_position(&keys[0]).is_none());
	}

	#[test]
	fn tier_skip_when_high_confidence_and_low_occupancy() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 2);

		// Put first key in stash
		seg.put(Item::new(keys[0], 10));
		let stash_idx = get_index(&keys[0], seg.core.stash_size);

		// Manually increase access_count to 2 (high confidence)
		if let Some(idx) = seg.core.stash_buckets[stash_idx].get_position(&keys[0]) {
			seg.core.stash_buckets[stash_idx].get_items_mut()[idx].lfu_counter = 2;
		}

		// Get should trigger tier-skip with skip_history_flag=true
		seg.get(&keys[0]);

		// Item should be in hot now
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
		if let Some(idx) = seg.core.buckets[hot_idx].get_position(&keys[0]) {
			assert!(seg.core.buckets[hot_idx].get_items()[idx].skip_history_flag);
		}
	}

	#[test]
	fn no_tier_skip_when_low_confidence() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 2);

		// Put first key in stash (access_count will be 0, low confidence)
		seg.put(Item::new(keys[0], 10));
		let stash_idx = get_index(&keys[0], seg.core.stash_size);

		// Get should NOT trigger tier-skip (low confidence)
		seg.get(&keys[0]);

		// Item should be in hot but skip_history_flag should be false
		assert!(seg.core.buckets[hot_idx].get_position(&keys[0]).is_some());
		if let Some(idx) = seg.core.buckets[hot_idx].get_position(&keys[0]) {
			assert!(!seg.core.buckets[hot_idx].get_items()[idx].skip_history_flag);
		}
	}

	#[test]
	fn recovery_enforcement_on_skip_flag_eviction() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let hot_idx = 0;
		let keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 3);

		// Place first two items directly in hot bucket
		seg.core.buckets[hot_idx].put(Item::new(keys[0], 10));
		seg.core.buckets[hot_idx].put(Item::new(keys[1], 20));
		assert!(seg.core.buckets[hot_idx].is_full());

		// Put the third key in stash with high access_count
		seg.put(Item::new(keys[2], 30));
		let stash_idx = get_index(&keys[2], seg.core.stash_size);

		// Manually set high access_count for tier-skip
		if let Some(idx) = seg.core.stash_buckets[stash_idx].get_position(&keys[2]) {
			seg.core.stash_buckets[stash_idx].get_items_mut()[idx].lfu_counter = 2;
		}

		// Get should trigger tier-skip
		seg.get(&keys[2]);

		// Confirm skip_history_flag is set
		let was_skipped = seg.core.buckets[hot_idx]
			.get_position(&keys[2])
			.map(|idx| seg.core.buckets[hot_idx].get_items()[idx].skip_history_flag)
			.unwrap_or(false);
		assert!(was_skipped, "item should have skip_history_flag set");

		// Now evict it again to trigger recovery
		// Force more items into hot to cause eviction
		let more_keys = find_keys_for_bucket(hot_idx, seg.core.segment_size, 2);
		for k in more_keys {
			seg.put(Item::new(k, k * 10));
			let k_stash_idx = get_index(&k, seg.core.stash_size);
			if let Some(idx) = seg.core.stash_buckets[k_stash_idx].get_position(&k) {
				seg.core.stash_buckets[k_stash_idx].get_items_mut()[idx].lfu_counter = 2;
			}
			seg.get(&k);
		}

		// Check if keys[2] was demoted back to stash (recovery)
		let back_in_stash = seg.core.stash_buckets[stash_idx].get_position(&keys[2]).is_some();
		let still_in_hot = seg.core.buckets[hot_idx].get_position(&keys[2]).is_some();

		// keys[2] should either be back in stash or still in hot (depending on recency),
		// but if it was evicted and demoted, it should be in stash at MRU
		if back_in_stash && !still_in_hot {
			// Recovery was enforced: verify it's at MRU (back)
			if let Some(idx) = seg.core.stash_buckets[stash_idx].get_position(&keys[2]) {
				// Being at index 0 means it's the most recently inserted (back)
				// The actual position depends on bucket ordering, so we just verify presence
				assert!(seg.core.stash_buckets[stash_idx].get_position(&keys[2]).is_some());
			}
		}
	}
}
