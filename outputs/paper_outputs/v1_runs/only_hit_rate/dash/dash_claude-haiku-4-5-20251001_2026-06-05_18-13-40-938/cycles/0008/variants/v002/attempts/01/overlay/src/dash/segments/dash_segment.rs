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
/// Its `get` realises a **simplified two-tier** design: stash and hot form a tier,
/// and items are promoted directly from stash to hot on a stash hit.
/// Demotion cascades down: `hot -> stash -> out`, with a one-more-chance step
/// between tiers (evicted from hot get a chance in stash).
///
/// This simplifies the baseline's three-tier architecture (stash + target + probing)
/// by collapsing the two-tier hot-side (target and probing) into a single unified
/// "hot" tier, eliminating promotion gatekeeping overhead.
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
// Two-tier promotion: items are promoted directly from stash to hot on a hit.
// Demotion from hot back to stash with one-more-chance.

/// Moves `item` into its hot bucket. Any evicted item flows to stash (demotion).
fn promote_to_hot<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
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

/// One-more-chance demotion from hot to stash.
fn demote_from_hot_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug!(
		"demote_from_hot_bucket: key={:?} hot -> stash",
		item.key
	);
	core.put_at(Slot::Stash, insert_pos, item);
}

/// `insert_pos = Front`: promotions land at LRU end of hot tier; the item is managed by
/// the eviction policy's ordering from that point forward.
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
		debug!("promotion_get: key={key:?} found in stash -> hot={hot_idx} insert_pos={insert_pos:?}");
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_hot(core, data, insert_pos);

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
}
