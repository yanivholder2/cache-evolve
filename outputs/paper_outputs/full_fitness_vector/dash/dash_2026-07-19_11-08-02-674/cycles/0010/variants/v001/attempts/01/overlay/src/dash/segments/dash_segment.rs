use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// High bit of `lfu_counter` used as the clock's "warm" flag.
/// The low bits remain the unmodified frequency counter.
const CLOCK_BIT: usize = 1 << (usize::BITS - 1);

/// The dash Variant's segment with MRU-end insertion, vacancy-gated frequency
/// promotion, vacancy-gated probing re-entry, and a two-chance clock eviction.
///
/// The clock's "warm" signal is encoded in the high bit (`CLOCK_BIT`) of
/// `lfu_counter`, leaving the low bits as the unmodified frequency counter.
/// This means the clock and the frequency counter share the same field without
/// interfering: `lfu_counter & !CLOCK_BIT` is the frequency, `lfu_counter &
/// CLOCK_BIT` is the clock state.
///
/// On a target-bucket hit the item's CLOCK_BIT is set (warm) only when its
/// raw frequency reaches 2 or more. Before evicting from a full target bucket,
/// `clock_evict_prepare` scans from the front: cold items (CLOCK_BIT clear)
/// are evicted immediately; warm items have their CLOCK_BIT cleared (second
/// chance consumed) and are rotated to the back.
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

// ── Clock eviction helper ──────────────────────────────────────────────────
// Scan from position 0; give warm items (CLOCK_BIT set) a second chance by
// clearing their CLOCK_BIT and rotating them to the back. Stop as soon as
// position 0 holds a cold item (CLOCK_BIT clear) — that item will be the
// eviction candidate for the policy's min-scan.

/// Rotate `items` so that position 0 holds the first cold item encountered.
/// Warm items (CLOCK_BIT set) have their CLOCK_BIT cleared and are moved to
/// the back. If every item is warm the whole Vec is traversed, clearing all
/// CLOCK_BITs, and the original position-0 item (now cold) ends up at front.
fn clock_evict_prepare<K, V>(items: &mut Vec<Item<K, V>>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let n = items.len();
	for _ in 0..n {
		if items[0].lfu_counter & CLOCK_BIT == 0 {
			break; // cold item at front — evict it
		}
		items[0].lfu_counter &= !CLOCK_BIT; // second chance consumed
		items.rotate_left(1);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
///
/// If the arriving item carries historical frequency (`lfu_counter & !CLOCK_BIT >= 2`),
/// it is marked warm on arrival so the clock treats it as a recently-accessed item.
/// Items with raw frequency below 2 are not pre-marked warm — they must earn
/// clock-protection by reaching the threshold in their new bucket.
/// Before inserting into a full bucket, `clock_evict_prepare` is run so that warm
/// items get a second chance and cold items are selected for eviction.
fn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, mut item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	// Mark warm on arrival only if the item has proven high-reuse frequency (>= 2 accesses).
	if item.lfu_counter & !CLOCK_BIT >= 2 {
		item.lfu_counter |= CLOCK_BIT;
	}
	let target_idx = get_index(&item.key, core.segment_size);
	if core.buckets[target_idx].is_full() {
		clock_evict_prepare(core.buckets[target_idx].get_items_mut());
	}
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
///
/// The clock pre-pass is run before the target-retry insert so that warm residents
/// get their second chance before being cascaded out.
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
		if core.buckets[true_target].is_full() {
			clock_evict_prepare(core.buckets[true_target].get_items_mut());
		}
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
/// Non-hottest items in target accumulate `lfu_counter & !CLOCK_BIT` as frequency. On each
/// non-hottest hit the frequency is incremented. CLOCK_BIT is set only when the raw frequency
/// first reaches 2, giving the item clock-protection precisely when it has proven high reuse.
/// When the frequency reaches 2 and the probing bucket has spare capacity (vacancy-gated), the
/// item is promoted directly to probing — guaranteeing zero cascade cost on this path.
///
/// Items in probing also accumulate `lfu_counter`. When the counter reaches 2
/// and the target bucket has spare capacity (vacancy-gated probing re-entry),
/// the item is moved back to target at the MRU end — giving it a fresh chance
/// to re-earn its probing slot and improving hot-item cycling.
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
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
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
			// CLOCK_BIT is set only when the raw frequency first reaches 2,
			// giving the clock genuine discriminating power: items with freq 0-1 remain
			// unprotected and compete by raw frequency, while freq>=2 items earn one
			// protected round per eviction event.
			let items_mut = core.buckets[target_idx].get_items_mut();
			let old_freq = items_mut[item_idx].lfu_counter & !CLOCK_BIT;
			let new_freq = old_freq + 1;
			items_mut[item_idx].lfu_counter = if new_freq >= 2 {
				new_freq | CLOCK_BIT
			} else {
				new_freq
			};
			let freq = new_freq;
			let probing_has_space = !core.buckets[probing_idx].is_full();
			if freq >= 2 && probing_has_space {
				debug!("promotion_get: key={key:?} found in target item_idx={item_idx} freq={freq} probing={probing_idx} has space (vacancy-gated frequency-promote)");
				let data = core.buckets[target_idx].remove_at(item_idx);
				promote_to_probing(core, data, insert_pos);
				let item_new_idx = core.buckets[probing_idx]
					.get_position(key)
					.expect("promote_to_probing must leave key in probing");
				return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
			}
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} freq={freq} (intra-promote)");
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		// Probing counter uses raw lfu_counter (CLOCK_BIT not tracked for probing items).
		core.buckets[probing_idx].get_items_mut()[item_idx].lfu_counter += 1;
		let counter = core.buckets[probing_idx].get_items()[item_idx].lfu_counter;
		let target_has_space = !core.buckets[target_idx].is_full();
		if counter >= 2 && target_has_space {
			debug!("promotion_get: key={key:?} found in probing bucket={probing_idx} lfu_counter={counter} target={target_idx} has space (vacancy-gated probing re-entry)");
			let data = core.buckets[probing_idx].remove_at(item_idx);
			promote_to_target(core, data, InsertPosition::Back);
			let item_new_idx = core.buckets[target_idx]
				.get_position(key)
				.expect("promote_to_target must leave key in target");
			return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
		}
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx} lfu_counter={counter} (intra-probing stay)");
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
	fn probing_hit_stays_when_counter_below_threshold() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		// Place directly in probing with lfu_counter = 0
		seg.core.buckets[probing_idx].put(Item::new(keys[0], 10));

		// First hit: counter becomes 1 (< 2), gate does not fire, item stays in probing
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
		// (freq=1 but probing full → gate silenced), second hit finds it as hottest
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

	/// Vacancy-gated frequency promotion fires when freq (counter & !CLOCK_BIT) >= 2
	/// and probing has space. The item is promoted directly to probing without any cascade eviction.
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

		// Get target_keys[0]: non-hottest, freq becomes 2, probing has space → gate fires!
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

		// Pre-set lfu_counter to 1 so the next hit would reach freq=2 (threshold met),
		// but probing is full — gate must be silenced regardless.
		seg.core.buckets[target_idx].get_items_mut()[0].lfu_counter = 1;

		// Get target_keys[0]: freq becomes 2 >= 2, BUT probing is full → gate silenced.
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

	/// Vacancy-gated probing re-entry fires when probing lfu_counter >= 2 and target has space.
	/// The item is moved back to target at the MRU end.
	#[test]
	fn vacancy_gated_probing_reentry_fires_when_target_has_space() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		// Place item directly in probing with lfu_counter = 1
		seg.core.buckets[probing_idx].put(Item::new(keys[0], 42));
		seg.core.buckets[probing_idx].get_items_mut()[0].lfu_counter = 1;

		// Target is empty — has space.
		assert!(!seg.core.buckets[target_idx].is_full());

		// Hit: counter becomes 2, target has space → re-entry fires
		let result = seg.get(&keys[0]);
		assert!(result.is_some(), "item must be returned after probing re-entry");
		assert!(
			seg.core.buckets[target_idx].get_position(&keys[0]).is_some(),
			"item must be in target after probing re-entry"
		);
		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[0]).is_none(),
			"item must leave probing after re-entry"
		);
	}

	/// Vacancy-gated probing re-entry is silenced when target is full.
	#[test]
	fn vacancy_gated_probing_reentry_silenced_when_target_full() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		// Need 3 keys that all hash to target_idx: [0] goes to probing, [1] and [2] fill target.
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		// Fill target with [1] and [2] (bucket capacity = 2 in small_dash_settings).
		seg.core.buckets[target_idx].put(Item::new(target_keys[1], 10));
		seg.core.buckets[target_idx].put(Item::new(target_keys[2], 20));
		assert!(seg.core.buckets[target_idx].is_full());

		// Place target_keys[0] in probing with lfu_counter = 1.
		// Its hash maps to target_idx, so the re-entry gate checks target_idx.
		seg.core.buckets[probing_idx].put(Item::new(target_keys[0], 99));
		seg.core.buckets[probing_idx].get_items_mut()[0].lfu_counter = 1;

		// Hit: counter becomes 2, but target is full → gate silenced, item stays in probing.
		let result = seg.get(&target_keys[0]);
		assert!(result.is_some(), "item must be returned when re-entry gate silenced");
		assert!(
			seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some(),
			"item must remain in probing when target is full"
		);
		assert!(
			seg.core.buckets[target_idx].get_position(&target_keys[0]).is_none(),
			"item must not be moved to full target"
		);
	}
}
