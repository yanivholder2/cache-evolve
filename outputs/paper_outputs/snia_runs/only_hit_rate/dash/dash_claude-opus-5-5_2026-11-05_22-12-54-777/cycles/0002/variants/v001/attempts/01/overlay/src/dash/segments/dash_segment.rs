use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment — cycle 0002 v001 (H01).
///
/// Built on the 24 normal / 8 stash topology of cycle 0001 v002 (see
/// `src/shared/geometry.rs`, inherited unchanged). Changes against that parent:
///
/// 1. **MRU promotion landing.** stash→target and target→probing insert at the
///    MRU end (`InsertPosition::Back`) of the destination bucket. The single
///    `insert_pos` is split into a promote position and a demote position.
/// 2. **Stricter probing trigger.** A target hit promotes to probing only when
///    the item is both at the hottest position *and* has already been hit at
///    least once while resident in its current target bucket. The "hit in
///    target" bit lives in the top bit of `Item::lfu_counter` (no growth of
///    `Item`); it is cleared whenever the item moves between buckets and set
///    on the first target hit.
/// 3. **Softer demotion position.** Items demoted into stash by normal-tier
///    pressure land at the middle of the stash bucket (index `len / 2`), not
///    the LRU end. Fresh puts still enter the stash at its normal insert
///    position (`Back`, via the default `Segment::put`).
///
/// The one-more-chance retry of a foreign probing resident into its true target
/// keeps the parent's position (`InsertPosition::Front`).
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

// ── Positions & per-item flag ──────────────────────────────────────────────

/// Promotions (stash→target, target→probing) land at the MRU end.
const PROMOTE_POS: InsertPosition = InsertPosition::Back;
/// One-more-chance retry of a foreign probing resident into its true target
/// keeps the parent's LRU-end landing.
const RETRY_POS: InsertPosition = InsertPosition::Front;

/// "Hit at least once while resident in the current target bucket." Stored in
/// the top bit of `lfu_counter`, so `Item` does not grow.
const TARGET_HIT_FLAG: usize = 1usize << (usize::BITS - 1);

#[inline]
fn clear_target_hit<K, V>(item: &mut Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	item.lfu_counter &= !TARGET_HIT_FLAG;
}

#[inline]
fn has_target_hit<K, V>(item: &Item<K, V>) -> bool
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	item.lfu_counter & TARGET_HIT_FLAG != 0
}

// ── Promotion logic ────────────────────────────────────────────────────────

/// Moves `item` into its target bucket at the MRU end. Any evicted item flows
/// through the tier cascade.
fn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	clear_target_hit(&mut item);
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Target, PROMOTE_POS, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx);
}

/// Moves `item` into its probing bucket at the MRU end. Any evicted item flows
/// through the tier cascade.
fn promote_to_probing<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	clear_target_hit(&mut item);
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let Some(evicted) = core.put_at(Slot::Probing, PROMOTE_POS, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx);
}

/// Demotes `item` into its stash bucket at the middle of the recency order
/// (index `len / 2`, computed after any overflow eviction). Overflow evicts the
/// stash bucket's policy victim, which leaves the cache (stash is terminal).
fn demote_to_stash<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	clear_target_hit(&mut item);
	let stash_idx = get_index(&item.key, core.stash_size);
	let bucket = &mut core.stash_buckets[stash_idx];
	debug_assert!(
		bucket.get_position(&item.key).is_none(),
		"demote_to_stash: key={:?} already resident in stash",
		item.key
	);
	if bucket.is_full() {
		if let Some(e) = bucket.evict_item() {
			debug!("demote_to_stash: stash={stash_idx} evicted_key={:?}", e.key);
		}
	}
	if bucket.size() >= bucket.get_max_size() {
		// Zero-capacity bucket: nothing can be held.
		return;
	}
	let mid = bucket.size() / 2;
	debug!("demote_to_stash: key={:?} stash={stash_idx} pos={mid}", item.key);
	bucket.get_items_mut().insert(mid, item);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Stash landings use the
/// mid-bucket demotion slot.
fn demote_from_normal_bucket<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, source_idx: usize)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		demote_to_stash(core, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		clear_target_hit(&mut item);
		if let Some(cascade) = core.put_at(Slot::Target, RETRY_POS, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			demote_to_stash(core, cascade);
		}
	}
}

fn promotion_get<'a, K, V>(core: &'a mut SegmentCore<K, V>, key: &K) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in stash -> target={target_idx} (MRU)");
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, data);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let bucket = &core.buckets[target_idx];
		let is_hottest = Some(item_idx) == bucket.promotion_candidate_index();
		let was_hit = has_target_hit(&bucket.get_items()[item_idx]);
		if is_hottest && was_hit {
			debug!("promotion_get: key={key:?} found in target as hottest+hit -> probing={probing_idx}");
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote, mark hit)");
			core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter |= TARGET_HIT_FLAG;
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
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
	fn stash_hit_inserts_at_back_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Promotion lands at MRU: keys[1] after keys[0].
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn first_target_hit_at_mru_does_not_promote() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// keys[1] sits at MRU but has no target hit yet: stays in target.
		seg.get(&keys[1]);
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_none());
	}

	#[test]
	fn target_hit_hottest_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// First target hit marks; second (at MRU, marked) promotes to probing.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
		let idx = seg.core.buckets[probing_idx].get_position(&keys[1]).unwrap();
		assert!(!has_target_hit(&seg.core.buckets[probing_idx].get_items()[idx]));
	}

	#[test]
	fn marked_but_not_mru_does_not_promote() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		seg.get(&keys[1]); // mark keys[1]
		seg.get(&keys[0]); // keys[0] to MRU, keys[1] now LRU
		seg.get(&keys[1]); // marked but not MRU -> stays, moves to MRU
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_none());
	}

	#[test]
	fn probing_hit_touches_in_place() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
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
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Sole resident of target → at MRU; first get marks, second promotes.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());
	}

	#[test]
	fn cascade_demotion_probing_to_full_target_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
		assert!(seg.core.buckets[target_idx].is_full());

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		let evicted_key = probing_keys[0];
		let stash_idx = get_index(&evicted_key, seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[stash_idx].get_position(&evicted_key).is_some()
				|| seg.core.buckets[probing_idx].get_position(&evicted_key).is_some()
				|| seg.core.buckets[target_idx].get_position(&evicted_key).is_some(),
			"evicted probing item should be demoted to stash or still in cache"
		);
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	#[test]
	fn last_bucket_probing_wraps_to_zero() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = seg_size - 1;
		let probing_idx = (target_idx + 1) % seg_size;
		assert_eq!(probing_idx, 0, "sanity: wraparound math holds");
		let keys = find_keys_for_bucket(target_idx, seg_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}

	#[test]
	fn evicted_from_probing_cascades_to_target() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 2);
		let k = target_keys[0];
		let trigger = target_keys[1];
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		seg.core_mut().buckets[probing_idx].put(Item::new(k, 99));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 50));
		assert!(seg.core().buckets[probing_idx].is_full());

		promote_to_probing(seg.core_mut(), Item::new(trigger, 200));

		let in_target = seg.core().buckets[target_idx].get_position(&k).is_some();
		let in_stash = seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some());
		let still_in_probing = seg.core().buckets[probing_idx].get_position(&k).is_some();

		assert!(!still_in_probing);
		assert!(in_target);
		assert!(!in_stash);
	}

	#[test]
	fn target_retry_lands_at_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let pre = target_keys[0];
		let foreign = target_keys[1];
		let trigger = target_keys[2];
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		seg.core_mut().buckets[target_idx].put(Item::new(pre, 1));
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 3));

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99));

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items.iter().position(|it| it.key == foreign).expect("foreign in true target");
		assert_eq!(pos, 0);
	}

	#[test]
	fn direct_to_stash_lands_mid_bucket() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let victim = target_keys[0];
		let resident = target_keys[1];
		let trigger = target_keys[2];
		let stash_idx = get_index(&victim, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 8)
			.into_iter()
			.find(|k| !target_keys.contains(k))
			.expect("distinct stash_pre");

		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));

		let len_before = seg.core().stash_buckets[stash_idx].size();
		promote_to_target(seg.core_mut(), Item::new(trigger, 99));

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items.iter().position(|it| it.key == victim).expect("victim in stash");
		assert_eq!(pos, len_before / 2);
	}

	#[test]
	fn mid_stash_demotion_evicts_when_full() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let keys = find_keys_for_bucket(0, stash_size, 3);

		seg.core_mut().stash_buckets[0].put(Item::new(keys[0], 1));
		seg.core_mut().stash_buckets[0].put(Item::new(keys[1], 2));
		assert!(seg.core().stash_buckets[0].is_full());

		demote_to_stash(seg.core_mut(), Item::new(keys[2], 3));
		let b = &seg.core().stash_buckets[0];
		assert!(b.size() <= b.get_max_size());
		assert!(b.get_position(&keys[2]).is_some());
		assert!(b.get_position(&keys[0]).is_none(), "LRU stash resident evicted");
	}
}
