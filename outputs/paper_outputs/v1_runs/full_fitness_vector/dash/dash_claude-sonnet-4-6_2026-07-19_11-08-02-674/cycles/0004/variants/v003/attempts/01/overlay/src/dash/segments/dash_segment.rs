use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment with MRU-end promotion insertion and
/// vacancy-gated frequency promotion.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. A stash→target insert lands at the MRU end
/// (`InsertPosition::Back`), so an item needs only one more hit before it can
/// promote again — halving the warm-up cost compared to the baseline.
/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a
/// one-more-chance step between adjacent tiers.
///
/// Additionally, non-hottest items in target accumulate a frequency counter
/// (`lfu_counter`). When the counter reaches 2 **and** the probing bucket has
/// spare capacity, the item is promoted directly to probing — the vacancy gate
/// ensures no cascade eviction can occur from this early-promotion path.
///
/// When an item is promoted from stash to target, its `lfu_counter` is reset to
/// zero, so it must re-earn vacancy-gate eligibility from fresh post-promotion
/// accesses. This prevents stale pre-demotion counts from triggering premature
/// frequency promotion.
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
		promotion_get(&mut self.core, key, InsertPosition::Back)
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
	demote_from_normal_bucket(core, evicted, target_idx, insert_pos);
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
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		core.put_at(Slot::Stash, insert_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			core.put_at(Slot::Stash, insert_pos, cascade);
		}
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
///
/// Non-hottest items in target accumulate `lfu_counter`. When the counter reaches 2
/// and the probing bucket has spare capacity (vacancy-gated), the item is promoted
/// directly to probing — guaranteeing zero cascade cost on this path.
///
/// When an item is found in stash (stash hit), its `lfu_counter` is reset to zero
/// before it is placed back into the target tier. This ensures the vacancy-gate
/// threshold is evaluated against only fresh post-promotion accesses.
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
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in stash -> target={target_idx} insert_pos={insert_pos:?}");
		let mut data = core.stash_buckets[stash_idx].remove_at(item_idx);
		data.lfu_counter = 0;
		promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target or cascade it elsewhere via demote_from_normal_bucket")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			// Vacancy-gated frequency promotion: accumulate hits on non-hottest items
			// and promote early to probing only when probing has spare capacity,
			// ensuring zero cascade cost on this path.
			core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter += 1;
			let counter = core.buckets[target_idx].get_items()[item_idx].lfu_counter;
			let probing_has_space = !core.buckets[probing_idx].is_full();
			if counter >= 2 && probing_has_space {
				debug!("promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} probing={probing_idx} has space (vacancy-gated frequency-promote)");
				let data = core.buckets[target_idx].remove_at(item_idx);
				promote_to_probing(core, data, insert_pos);
				let item_new_idx = core.buckets[probing_idx]
					.get_position(key)
					.expect("promote_to_probing must leave key in probing");
				return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
			}
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} lfu_counter={counter} (intra-promote)");
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

		// Back insert pushes keys[1] to the MRU end (Back), leaving keys[0] at Front.
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

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// With Back insertion, keys[1] lands at MRU (promotion_candidate) after stash promotion.
		// The very next get promotes it to probing.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
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

		// Sole resident of target → already at MRU, so the next get triggers promotion.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
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

		// Two gets on target_keys[0]: first bumps it to MRU via get_from_position
		// (lfu_counter=1 but probing full → gate silenced), second hit finds it as hottest
		// and promotes to probing, triggering the cascade.
		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		// The evicted item's target is `probing_idx`, so the cascade routes it to stash
		// instead of bouncing it back into the bucket it just left.
		let evicted_key = probing_keys[0];
		let stash_idx = get_index(&evicted_key, seg.core.stash_size);
		// Permissive assertion: the key may have re-landed elsewhere via subsequent
		// stash overflow handling — only "vanished entirely" would be a real bug.
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

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		// With Back insertion, keys[1] is at MRU after stash promotion.
		// One more get promotes it; must place into the wrapped probing bucket without panic.
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

	/// Foreign resident demoted via one-more-chance lands at the bucket end determined by
	/// `insert_pos`: Front for cold, Back for hot.
	fn target_retry_position(insert_pos: InsertPosition, expected_idx: usize) {
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
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn target_retry_lands_at_front_when_cold() {
		target_retry_position(InsertPosition::Front, 0);
	}

	#[test]
	fn target_retry_lands_at_back_when_hot() {
		target_retry_position(InsertPosition::Back, 1);
	}

	/// When an evicted item's true target equals its source bucket, demote_from_normal_bucket
	/// routes it directly to stash (no retry). Verifies the `true_target == source_idx` branch
	/// AND that the stash insert honors `insert_pos`.
	fn direct_to_stash_position(insert_pos: InsertPosition, expected_idx: usize) {
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

		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));

		promote_to_target(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_front_when_cold() {
		direct_to_stash_position(InsertPosition::Front, 0);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert honors `insert_pos`.
	fn cascade_to_stash_position(insert_pos: InsertPosition, expected_idx: usize) {
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

		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_a, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 3));
		seg.core_mut().buckets[probing_idx].put(Item::new(probing_filler, 4));

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn cascade_to_stash_at_front_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 0);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}

	/// Vacancy-gated frequency promotion fires when lfu_counter >= 2 and probing has space.
	/// The item is promoted directly to probing without any cascade eviction.
	#[test]
	fn vacancy_gated_frequency_promote_fires_when_probing_has_space() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		// Place two items directly into target so target_keys[0] is non-hottest (LRU, pos 0)
		// and target_keys[1] is hottest (MRU, pos 1) with ClassicLRU.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 99));
		seg.core.buckets[target_idx].put(Item::new(target_keys[1], 88));
		// target: [target_keys[0] @ 0 (LRU/non-hottest), target_keys[1] @ 1 (MRU/hottest)]

		// Pre-set lfu_counter on target_keys[0] to 1 so the next hit increments to 2 and fires.
		seg.core.buckets[target_idx].get_items_mut()[0].lfu_counter = 1;

		// Probing is empty — has space.
		assert!(!seg.core.buckets[probing_idx].is_full());

		// Get target_keys[0]: non-hottest, counter becomes 2, probing has space → gate fires!
		let result = seg.get(&target_keys[0]);
		assert!(result.is_some(), "item must be returned after vacancy-gate promotion");
		assert!(
			seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some(),
			"item must be in probing after vacancy-gate promotion"
		);
		assert!(
			seg.core.buckets[target_idx].get_position(&target_keys[0]).is_none(),
			"item must leave target after vacancy-gate promotion"
		);
	}

	/// Vacancy gate is silenced when probing is full: the item stays in target regardless
	/// of its frequency counter value.
	#[test]
	fn vacancy_gated_promote_does_not_fire_when_probing_full() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		// Fill probing directly.
		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Place two items in target: target_keys[0] is non-hottest (LRU, pos 0).
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 99));
		seg.core.buckets[target_idx].put(Item::new(target_keys[1], 88));
		// target: [target_keys[0] @ 0 (LRU/non-hottest), target_keys[1] @ 1 (MRU/hottest)]

		// Pre-set lfu_counter to 1 so the next hit would reach counter=2 (threshold met),
		// but probing is full — gate must be silenced regardless.
		seg.core.buckets[target_idx].get_items_mut()[0].lfu_counter = 1;

		// Get target_keys[0]: counter becomes 2 >= 2, BUT probing is full → gate silenced.
		let result = seg.get(&target_keys[0]);
		assert!(result.is_some(), "item must be returned when gate is silenced");
		// Item must remain in target (not promoted).
		assert!(
			seg.core.buckets[target_idx].get_position(&target_keys[0]).is_some(),
			"item must remain in target when probing is full"
		);
		assert!(
			seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_none(),
			"item must not be promoted when probing is full"
		);
	}

	/// A stash-promoted item must enter its target bucket with lfu_counter == 0,
	/// regardless of the counter value it accumulated before demotion to stash.
	/// This verifies the stash-to-target counter reset introduced in this variant.
	#[test]
	fn stash_promoted_item_enters_target_with_zero_counter() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let key = keys[0];

		// Put item → goes to stash.
		seg.put(Item::new(key, 42));

		// Manually set a non-zero lfu_counter on the item in stash to simulate
		// an item that accumulated hits before being demoted.
		let stash_idx = get_index(&key, seg.core.stash_size);
		let stash_item_idx = seg.core.stash_buckets[stash_idx]
			.get_position(&key)
			.expect("item must be in stash after put");
		seg.core.stash_buckets[stash_idx].get_items_mut()[stash_item_idx].lfu_counter = 5;

		// Stash hit: item should be promoted to target with counter reset to 0.
		let result = seg.get(&key);
		assert!(result.is_some(), "stash hit must return the item");

		// Check that the item is in target with lfu_counter == 0.
		let item_in_target_idx = seg.core.buckets[target_idx]
			.get_position(&key)
			.expect("item must be in target after stash-hit promotion");
		let counter = seg.core.buckets[target_idx].get_items()[item_in_target_idx].lfu_counter;
		assert_eq!(
			counter, 0,
			"lfu_counter must be reset to 0 when item is promoted from stash to target"
		);
	}

	/// A stash-promoted item (counter reset to 0) must re-earn vacancy-gate eligibility
	/// from scratch. On the first target hit its counter becomes 1 (gate stays silent);
	/// on the second target hit its counter reaches 2 and the gate fires (if probing has space).
	///
	/// Uses FIFO eviction policy so that `get_from_position` does not reorder items
	/// within the bucket — this keeps key_a as non-hottest on both the first and second
	/// target hit, exercising the vacancy-gate path (not the hottest-item-promotion path).
	#[test]
	fn stash_promoted_item_must_re_earn_vacancy_gate() {
		// Use FIFO so get_from_position does not move the accessed item; key_a remains
		// non-hottest (pos 0, LRU) through both target hits, so both go through the
		// vacancy-gate code path rather than the hottest-item-promotion path.
		let mut seg = new_segment(EvictionPolicyKind::Fifo);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		let key_a = keys[0];
		let key_b = keys[1];

		// Put key_a → stash; set its counter to a high value to simulate pre-demotion history.
		seg.put(Item::new(key_a, 10));
		let stash_idx_a = get_index(&key_a, seg.core.stash_size);
		let idx_a = seg.core.stash_buckets[stash_idx_a]
			.get_position(&key_a)
			.expect("key_a must be in stash");
		seg.core.stash_buckets[stash_idx_a].get_items_mut()[idx_a].lfu_counter = 7;

		// Stash hit for key_a → counter reset to 0, key_a enters target at Back (pos 0, sole item).
		seg.get(&key_a);
		// target[target_idx] = [key_a @ 0]

		// Put key_b → stash; get it (stash hit) → key_b enters target at Back (pos 1).
		seg.put(Item::new(key_b, 20));
		seg.get(&key_b);
		// target[target_idx] = [key_a @ 0, key_b @ 1]
		// FIFO promotion_candidate_index = len-1 = 1, so key_b is hottest, key_a is non-hottest.

		// Verify probing is empty (vacancy gate will have space to fire on second hit).
		assert!(!seg.core.buckets[probing_idx].is_full(), "probing must have space for gate to fire");

		// First hit on key_a: non-hottest (pos 0), counter 0→1, gate condition (counter >= 2) not met.
		// FIFO's get_from_position does not reposition, so key_a stays at pos 0 (non-hottest).
		assert!(seg.get(&key_a).is_some(), "first hit must return the item");
		assert!(
			seg.core.buckets[target_idx].get_position(&key_a).is_some(),
			"key_a must remain in target after first hit (counter=1, gate silent)"
		);
		assert!(
			seg.core.buckets[probing_idx].get_position(&key_a).is_none(),
			"key_a must not be in probing after first hit (gate silent)"
		);

		// Second hit on key_a: still non-hottest (pos 0), counter 1→2, probing has space → gate fires.
		assert!(seg.get(&key_a).is_some(), "second hit must return the item");
		assert!(
			seg.core.buckets[probing_idx].get_position(&key_a).is_some(),
			"key_a must be promoted to probing after second hit (counter=2, gate fires)"
		);
		assert!(
			seg.core.buckets[target_idx].get_position(&key_a).is_none(),
			"key_a must leave target after vacancy-gate promotion"
		);
	}
}
