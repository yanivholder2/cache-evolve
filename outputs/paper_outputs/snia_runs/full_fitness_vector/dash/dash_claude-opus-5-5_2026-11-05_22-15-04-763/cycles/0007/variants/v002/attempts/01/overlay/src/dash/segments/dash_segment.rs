use super::{stash_bit_from_hash, Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment (two-hit stash promotion + MRU stash demotion +
/// probing overflow on second stash hit + room-gated target→probing promotion +
/// stash presence mask).
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. A stash→target insert lands at the LRU end
/// (`InsertPosition::Front`), so an item must climb before it can promote again.
/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a
/// one-more-chance step between adjacent tiers.
///
/// Two-hit gate: an item in stash must be hit **twice** before it is promoted
/// into the normal tier. The first stash hit only marks the item (via
/// `lfu_counter`) and moves it to the stash MRU end; the second promotes it.
/// Items demoted into stash have their mark cleared so they must prove
/// themselves again.
///
/// MRU demotion: items demoted out of the normal tier enter stash at the MRU
/// end (`InsertPosition::Back`), giving proven items a full stash residency in
/// which to earn re-promotion.
///
/// Probing overflow (cycle 3 H02): on the second stash hit, if the target is
/// full but the probing bucket has free space, the item lands in probing
/// instead of evicting a target resident. Only when both are full does the
/// promotion go to target and trigger the demotion cascade.
///
/// Room-gated target→probing promotion (cycle 4 H02): the hottest target item
/// is promoted into probing only when the probing bucket has free space. If
/// probing is full the item is touched in place in target (stays at target
/// MRU), so normal-tier hits never start the probing eviction cascade.
///
/// Stash presence mask (cycle 7 H02): each stash bucket carries a conservative
/// 64-bit presence word (`SegmentCore::stash_masks`). `get` tests the key's
/// bit first and skips the stash `get_position` scan when it is clear. Lookup
/// order (stash first) and all placement decisions are unchanged.
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
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Target, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx);
}

/// Moves `item` into its probing bucket. Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target (LRU end) before
/// stash; if it was already in its target, go straight to stash. Items entering stash
/// land at the stash MRU end.
///
/// Every item entering stash has its stash-hit mark (`lfu_counter`) cleared, so it
/// must be hit twice in stash before it is promoted back into the normal tier.
///
/// Stash inserts go through `SegmentCore::put_at(Slot::Stash, ..)`, which keeps
/// the stash presence mask up to date (OR on insert, recompute on eviction).
fn demote_from_normal_bucket<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, source_idx: usize)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash (MRU)",
			item.key
		);
		item.lfu_counter = 0;
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(mut cascade) = core.put_at(Slot::Target, InsertPosition::Front, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash (MRU)",
				cascade.key
			);
			cascade.lfu_counter = 0;
			core.put_at(Slot::Stash, InsertPosition::Back, cascade);
		}
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
///
/// Stash hits are two-hit gated: the first hit marks the item and touches it in
/// place (stash MRU under ClassicLRU); the second hit promotes it into the
/// normal tier — target if it has room, else probing if it has room, else target
/// with the demotion cascade.
///
/// Target hits promote the hottest item into probing only when probing has free
/// space; otherwise the item is touched in place in target.
///
/// The stash probe is guarded by the stash presence mask: if the key's bit is
/// clear in `stash_masks[stash_idx]` the key cannot be in that stash bucket,
/// so the linear `get_position` scan is skipped.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	// One hash for all three indexes and the stash bit (identical to `get_index`).
	let h = hash(key);
	let stash_idx = h % core.stash_size;
	let target_idx = h % core.segment_size;
	let probing_idx = (target_idx + 1) % core.segment_size;

	let maybe_in_stash = core.stash_masks[stash_idx] & stash_bit_from_hash(h) != 0;
	let stash_pos = if maybe_in_stash {
		core.stash_buckets[stash_idx].get_position(key)
	} else {
		debug_assert!(
			core.stash_buckets[stash_idx].get_position(key).is_none(),
			"promotion_get: stash mask false negative for key={key:?} stash={stash_idx}"
		);
		None
	};

	if let Some(item_idx) = stash_pos {
		let counter = core.stash_buckets[stash_idx].get_items()[item_idx].lfu_counter;
		if counter == 0 {
			debug!("promotion_get: key={key:?} first stash hit -> mark and stay in stash={stash_idx}");
			{
				let item = &mut core.stash_buckets[stash_idx].get_items_mut()[item_idx];
				item.lfu_counter = item.lfu_counter.max(1);
			}
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		let mut data = core.stash_buckets[stash_idx].remove_at(item_idx);
		core.refresh_stash_mask(stash_idx);
		data.lfu_counter = 0;

		let max = core.buckets[target_idx].get_max_size();
		let t_len = core.buckets[target_idx].size();
		let p_len = core.buckets[probing_idx].size();

		if t_len >= max && p_len < max {
			debug!(
				"promotion_get: key={key:?} second stash hit, target={target_idx} full -> probing={probing_idx} overflow insert_pos={insert_pos:?}"
			);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after overflow insert into probing={probing_idx} -- probing had room, so no eviction can have displaced it")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		}

		debug!("promotion_get: key={key:?} second stash hit -> target={target_idx} insert_pos={insert_pos:?}");
		promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target or cascade it elsewhere via demote_from_normal_bucket")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		let probing_has_room = core.buckets[probing_idx].size() < core.buckets[probing_idx].get_max_size();
		if is_hottest && probing_has_room {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx} (has room)");
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- probing had room, so no eviction can have displaced it")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		}
		if is_hottest {
			debug!(
				"promotion_get: key={key:?} found in target as hottest but probing={probing_idx} full -> stay in target={target_idx}"
			);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
		}
		return Some(core.buckets[target_idx].get_from_position(item_idx));
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

	/// Two stash hits promote a freshly put item into its target bucket.
	fn put_and_promote(seg: &mut DashSegment<i64, i64>, key: i64, value: i64) {
		seg.put(Item::new(key, value));
		seg.get(&key);
		seg.get(&key);
	}

	fn bucket_keys(seg: &DashSegment<i64, i64>, idx: usize) -> Vec<i64> {
		seg.core.buckets[idx].get_items().iter().map(|i| i.key).collect()
	}

	/// Every resident stash key has its bit set in its bucket's mask.
	fn assert_masks_conservative(seg: &DashSegment<i64, i64>) {
		for (s, b) in seg.core.stash_buckets.iter().enumerate() {
			for it in b.get_items() {
				assert_ne!(
					seg.core.stash_masks[s] & SegmentCore::<i64, i64>::stash_bit(&it.key),
					0,
					"stash mask false negative: key={} stash={s}",
					it.key
				);
			}
		}
	}

	#[test]
	fn stash_mask_conservative_under_workload() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..5000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = (x % 97) as i64;
			if seg.get(&key).is_none() {
				seg.put(Item::new(key, key * 3));
			}
			assert_masks_conservative(&seg);
		}
	}

	#[test]
	fn stash_mask_cleared_after_promotion_out_of_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let key = 7_i64;
		let stash_idx = get_index(&key, seg.core.stash_size);
		seg.put(Item::new(key, 70));
		seg.get(&key);
		seg.get(&key);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_none());
		assert_eq!(seg.core.stash_masks[stash_idx], 0, "empty stash bucket has an empty mask");
		assert_eq!(seg.get(&key).unwrap().value, 70);
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	#[test]
	fn single_stash_hit_stays_in_stash_at_mru() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let stash_idx = 0;
		let keys = find_keys_for_bucket(stash_idx, stash_size, 2);
		let (a, b) = (keys[0], keys[1]);

		seg.put(Item::new(a, 10));
		seg.put(Item::new(b, 20));

		let val = seg.get(&a).unwrap().value;
		assert_eq!(val, 10);

		let target_idx = get_index(&a, seg.core.segment_size);
		assert!(seg.core.buckets[target_idx].get_position(&a).is_none());
		let items = seg.core.stash_buckets[stash_idx].get_items();
		assert_eq!(items.last().unwrap().key, a, "first stash hit moves item to stash MRU");
		assert!(items.last().unwrap().lfu_counter >= 1, "first stash hit marks the item");
	}

	#[test]
	fn second_stash_hit_promotes_to_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let key = 7_i64;
		seg.put(Item::new(key, 70));
		seg.get(&key);
		let stash_idx = get_index(&key, seg.core.stash_size);
		let target_idx = get_index(&key, seg.core.segment_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_some());

		let val = seg.get(&key).unwrap().value;
		assert_eq!(val, 70);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_none());
		let pos = seg.core.buckets[target_idx].get_position(&key).expect("promoted into target");
		assert_eq!(seg.core.buckets[target_idx].get_items()[pos].lfu_counter, 0);
	}

	/// Target full, probing has room: the second stash hit overflows into probing
	/// without evicting any target resident.
	#[test]
	fn second_stash_hit_uses_probing_when_target_full() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let max = seg.core.buckets[target_idx].get_max_size();
		let keys = find_keys_for_bucket(target_idx, seg_size, max + 1);
		assert_eq!(keys.len(), max + 1, "sanity: enough keys for target");
		let fillers = &keys[..max];
		let key = keys[max];

		for &k in fillers {
			seg.core.buckets[target_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[target_idx].is_full());
		let before = bucket_keys(&seg, target_idx);

		seg.put(Item::new(key, 123));
		seg.get(&key);
		let val = seg.get(&key).unwrap().value;
		assert_eq!(val, 123);

		assert!(
			seg.core.buckets[probing_idx].get_position(&key).is_some(),
			"overflowed promotion must land in probing"
		);
		assert!(seg.core.buckets[target_idx].get_position(&key).is_none());
		assert_eq!(bucket_keys(&seg, target_idx), before, "target contents unchanged");
		for &k in fillers {
			assert!(
				seg.core.stash_buckets.iter().all(|b| b.get_position(&k).is_none()),
				"no target resident may be demoted to stash"
			);
		}
		let stash_idx = get_index(&key, seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_none());

		// Subsequent hits find it in probing.
		assert_eq!(seg.get(&key).unwrap().value, 123);
	}

	/// Target and probing both full: the second stash hit falls back to target,
	/// and the target's LRU is demoted to stash MRU with its mark cleared.
	#[test]
	fn second_stash_hit_falls_back_to_target_when_both_full() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let stash_size = seg.core.stash_size;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let max = seg.core.buckets[target_idx].get_max_size();
		let keys = find_keys_for_bucket(target_idx, seg_size, max + 1);
		let fillers = &keys[..max];
		let key = keys[max];
		let probing_fillers = find_keys_for_bucket(probing_idx, seg_size, max);

		for &k in fillers {
			let mut it = Item::new(k, k * 10);
			it.lfu_counter = 4;
			seg.core.buckets[target_idx].put(it);
		}
		for &k in &probing_fillers {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[target_idx].is_full());
		assert!(seg.core.buckets[probing_idx].is_full());
		let former_lru = fillers[0];
		let probing_before = bucket_keys(&seg, probing_idx);

		seg.put(Item::new(key, 321));
		seg.get(&key);
		let val = seg.get(&key).unwrap().value;
		assert_eq!(val, 321);

		assert!(seg.core.buckets[target_idx].get_position(&key).is_some(), "key promoted into target");
		assert!(seg.core.buckets[probing_idx].get_position(&key).is_none());
		assert_eq!(bucket_keys(&seg, probing_idx), probing_before, "probing untouched");
		assert!(seg.core.buckets[target_idx].get_position(&former_lru).is_none());

		let lru_stash = get_index(&former_lru, stash_size);
		let items = seg.core.stash_buckets[lru_stash].get_items();
		let last = items.last().expect("stash holds the demoted LRU");
		assert_eq!(last.key, former_lru, "former target LRU at stash MRU end");
		assert_eq!(last.lfu_counter, 0, "demotion clears the stash-hit mark");
		assert_masks_conservative(&seg);
	}

	#[test]
	fn lfu_policy_still_requires_two_stash_hits() {
		let mut seg = new_segment(EvictionPolicyKind::Lfu);
		let key = 11_i64;
		seg.put(Item::new(key, 1));
		seg.get(&key);
		let stash_idx = get_index(&key, seg.core.stash_size);
		let target_idx = get_index(&key, seg.core.segment_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_some());
		seg.get(&key);
		assert!(seg.core.buckets[target_idx].get_position(&key).is_some());
	}

	#[test]
	fn stash_hit_inserts_at_front_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		put_and_promote(&mut seg, keys[0], 10);
		put_and_promote(&mut seg, keys[1], 20);

		// Cold insert pushes keys[1] to Front, displacing keys[0] backward.
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
	}

	#[test]
	fn target_hit_not_hottest_promotes_within_bucket() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		put_and_promote(&mut seg, keys[0], 10);
		put_and_promote(&mut seg, keys[1], 20);

		// keys[1] starts at Front (cold). Hitting it should bump it Back via ClassicLRU,
		// not promote it to probing.
		seg.get(&keys[1]);
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn target_hit_hottest_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		put_and_promote(&mut seg, keys[0], 10);
		put_and_promote(&mut seg, keys[1], 20);

		// First hit moves keys[1] to MRU; second hit (already at MRU) promotes to probing.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
	}

	/// With an empty probing bucket, the hottest target item is still promoted
	/// into probing (the room gate is open).
	#[test]
	fn target_hottest_promotes_when_probing_has_room() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let k = find_keys_for_bucket(target_idx, seg.core.segment_size, 1)[0];

		assert_eq!(seg.core.buckets[probing_idx].size(), 0, "sanity: probing empty");
		// Sole resident of target → at MRU, so it is the promotion candidate.
		seg.core.buckets[target_idx].put(Item::new(k, 100));

		let val = seg.get(&k).unwrap().value;
		assert_eq!(val, 100);
		assert!(seg.core.buckets[probing_idx].get_position(&k).is_some(), "promoted into probing");
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none());
		assert!(seg.core.stash_buckets.iter().all(|b| b.size() == 0), "no demotion to stash");
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

	/// Probing full: the hottest target item stays in target (no promotion,
	/// no probing eviction); probing contents are unchanged.
	#[test]
	fn probing_full_hottest_target_item_stays_in_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let max = seg.core.buckets[probing_idx].get_max_size();
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, max);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());
		let probing_before = bucket_keys(&seg, probing_idx);

		// Sole resident of target → already at MRU, so it is the promotion candidate.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
		let val = seg.get(&target_keys[0]).unwrap().value;
		assert_eq!(val, 100);

		assert!(seg.core.buckets[target_idx].get_position(&target_keys[0]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_none());
		assert_eq!(bucket_keys(&seg, probing_idx), probing_before, "probing contents unchanged");
	}

	/// Full target + full probing: repeated hits on the hottest target item no
	/// longer start the probing → target → stash cascade.
	#[test]
	fn probing_full_hottest_target_hit_does_not_cascade() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let max = seg.core.buckets[probing_idx].get_max_size();
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, max);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, max);

		for &k in &target_keys {
			seg.core.buckets[target_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[target_idx].is_full());
		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());
		let probing_before = bucket_keys(&seg, probing_idx);

		let hot = target_keys[0];
		seg.get(&hot);
		let val = seg.get(&hot).unwrap().value;
		assert_eq!(val, hot * 10);

		assert!(seg.core.buckets[target_idx].get_position(&hot).is_some(), "hot item stays in target");
		assert!(seg.core.buckets[probing_idx].get_position(&hot).is_none());
		assert_eq!(bucket_keys(&seg, probing_idx), probing_before, "probing contents unchanged");
		for &k in target_keys.iter().chain(probing_keys.iter()) {
			assert!(
				seg.core.stash_buckets.iter().all(|b| b.get_position(&k).is_none()),
				"no normal-tier item may be demoted to stash"
			);
		}
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	/// Regression for the last-bucket probing wraparound (commit 4b0b763): keys whose target
	/// is `segment_size - 1` must probe at bucket 0, not at `segment_size` (out of bounds).
	#[test]
	fn last_bucket_probing_wraps_to_zero() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = seg_size - 1;
		let probing_idx = (target_idx + 1) % seg_size;
		assert_eq!(probing_idx, 0, "sanity: wraparound math holds");
		let keys = find_keys_for_bucket(target_idx, seg_size, 2);

		put_and_promote(&mut seg, keys[0], 10);
		put_and_promote(&mut seg, keys[1], 20);
		// Two-get promote: must place into the wrapped probing bucket without panic.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}

	/// An item evicted from its probing bucket must cascade to its true target
	/// (one-more-chance), not jump straight to stash.
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

		// `k` placed first so it is the LRU victim when bucket 1 next evicts.
		seg.core_mut().buckets[probing_idx].put(Item::new(k, 99));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 50));
		assert!(seg.core().buckets[probing_idx].is_full());
		assert!(seg.core().buckets[target_idx].get_position(&k).is_none());

		promote_to_probing(seg.core_mut(), Item::new(trigger, 200), InsertPosition::Back);

		let in_target = seg.core().buckets[target_idx].get_position(&k).is_some();
		let in_stash = seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some());
		let still_in_probing = seg.core().buckets[probing_idx].get_position(&k).is_some();

		assert!(!still_in_probing, "sanity: k was evicted from its probing bucket");
		assert!(
			in_target,
			"promotion demotion from probing must place item into its target bucket"
		);
		assert!(!in_stash, "promotion demotion must not bypass the target tier");
	}

	/// Foreign resident demoted via one-more-chance always lands at the LRU end (Front)
	/// of its true target, regardless of the promoted item's `insert_pos`.
	fn target_retry_position(insert_pos: InsertPosition) {
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

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should be in true target after retry");
		assert_eq!(pos, 0);
	}

	#[test]
	fn target_retry_lands_at_front_when_cold() {
		target_retry_position(InsertPosition::Front);
	}

	#[test]
	fn target_retry_lands_at_front_when_hot() {
		target_retry_position(InsertPosition::Back);
	}

	/// When an evicted item's true target equals its source bucket, demote_from_normal_bucket
	/// routes it directly to stash (no retry). Verifies the `true_target == source_idx` branch
	/// AND that the stash insert lands at the stash MRU end.
	fn direct_to_stash_position(insert_pos: InsertPosition) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let victim = target_keys[0];
		let resident = target_keys[1];
		let trigger = target_keys[2];
		let stash_idx = crate::shared::utils::get_index(&victim, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 1)[0];

		seg.core_mut().put_at(Slot::Stash, InsertPosition::Back, Item::new(stash_pre, 7));
		let mut marked_victim = Item::new(victim, 1);
		marked_victim.lfu_counter = 5;
		seg.core_mut().buckets[target_idx].put(marked_victim);
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));

		promote_to_target(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, stash_items.len() - 1, "demotion into stash lands at MRU");
		assert_eq!(
			stash_items[pos].lfu_counter, 0,
			"demotion into stash must clear the stash-hit mark"
		);
		assert_masks_conservative(&seg);
	}

	#[test]
	fn direct_to_stash_at_mru() {
		direct_to_stash_position(InsertPosition::Front);
		direct_to_stash_position(InsertPosition::Back);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert lands at stash MRU.
	fn cascade_to_stash_position(insert_pos: InsertPosition) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 4);
		let target_filler_a = target_keys[0];
		let target_filler_b = target_keys[1];
		let foreign = target_keys[2]; // foreign at probing; true_target = target_idx
		let trigger = target_keys[3];
		let probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		// `target_filler_a` is the LRU of target — evicted on retry, cascades to stash.
		let cascaded = target_filler_a;
		let stash_idx = crate::shared::utils::get_index(&cascaded, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 4)
			.into_iter()
			.find(|k| *k != cascaded)
			.expect("a distinct stash_pre must exist");

		seg.core_mut().put_at(Slot::Stash, InsertPosition::Back, Item::new(stash_pre, 7));
		let mut marked = Item::new(target_filler_a, 1);
		marked.lfu_counter = 3;
		seg.core_mut().buckets[target_idx].put(marked);
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 3));
		seg.core_mut().buckets[probing_idx].put(Item::new(probing_filler, 4));

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, stash_items.len() - 1, "cascade into stash lands at MRU");
		assert_eq!(
			stash_items[pos].lfu_counter, 0,
			"cascade into stash must clear the stash-hit mark"
		);
		assert_masks_conservative(&seg);
	}

	#[test]
	fn cascade_to_stash_at_mru() {
		cascade_to_stash_position(InsertPosition::Front);
		cascade_to_stash_position(InsertPosition::Back);
	}
}
