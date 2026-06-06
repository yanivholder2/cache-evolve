use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Frequency threshold (H01/H02): a target resident is promoted into the
/// protected probing tier only after it has proven this many hits during its
/// current target residency. Below the threshold it climbs positionally instead.
const PROMOTE_AT: u8 = 2;

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're warm
/// enough. A stash→target insert lands at the MRU end (`InsertPosition::Back`).
/// Demotion is **single-hop** (H03): a normal-bucket eviction always falls
/// straight to its stash, with no one-more-chance retry into the true target.
///
/// H01: the promotion heat that gates target→probing now lives in the
/// segment-owned `SegmentCore::target_heat` side table, keyed by the resident
/// key, rather than inside `Item`. The gate decisions are unchanged — heat is
/// bumped on every target hit and the item promotes once it reaches
/// `PROMOTE_AT` — but `Item<u64, u64>` no longer carries a `u8` counter, so it
/// stays at the 16-byte floor.
///
/// The promotion gate is only the *baseline* design; this whole type is the
/// segment extension point and the framework is free to evolve where items land
/// on a hit and how they migrate between slots.
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

	/// Routes a fresh (non-resident) item to its stash bucket, mirroring the
	/// default `Segment::put`. The only addition is clearing any stale promotion
	/// heat for this key (H01): when the per-`Item` `lfu_counter` was dropped on
	/// full eviction in the parent, a re-inserted key always started at heat 0.
	/// The side table outlives the `Item`, so we clear the key's heat here to
	/// reproduce that exact reset and keep the gate decisions bit-for-bit
	/// identical. (During normal residency `put` is never called for a resident
	/// key, so heat carries across slot moves just as the parent counter did.)
	fn put(&mut self, item: Item<K, V>) {
		let core = self.core_mut();
		core.target_heat.remove(&item.key);
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("Segment.put: key={:?}", item.key);
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, key, InsertPosition::Back)
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're warm enough (PROMOTE_AT hits in this
// target residency). Demotion is single-hop (H03): a normal-bucket eviction
// always lands directly in its stash.

/// Moves `item` into its target bucket. Any evicted item is demoted to stash.
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

/// Moves `item` into its probing bucket. Any evicted item is demoted to stash.
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

/// Single-hop demotion (H03). A normal-bucket eviction always lands directly in
/// its stash, regardless of whether the evicted item was a foreign resident in
/// someone else's probing slot.
///
/// The parent's *one-more-chance* step is removed: it used to re-insert a
/// foreign resident (`true_target != source_idx`) into its true target, and if
/// that target was full it cascaded the target's LRU victim down to the stash
/// with a second `put_at`. That retry-plus-cascade was located (cycle 0007) as
/// the put-p99 bound — a single put could trigger two extra normal-bucket
/// inserts. Collapsing it to one unconditional stash hop removes that tail work.
/// The cross-cycle insight that hit-rate is capacity/compulsory-bound and
/// conflict-insensitive on this trace predicts no hit-rate cost from dropping
/// this conflict-mitigation mechanism. `insert_pos` is still honored so cold/hot
/// stash placement stays Front/Back. `source_idx` is retained on the signature
/// (the callers still pass it) but is no longer needed to choose the route.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let _ = source_idx;
	debug!(
		"demote_from_normal_bucket: key={:?} -> stash (single-hop)",
		item.key
	);
	core.put_at(Slot::Stash, insert_pos, item);
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
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
		// Frequency gate (H01): the promotion heat lives in the segment-owned
		// `target_heat` side table rather than inside `Item`, so `Item` stays
		// 16 bytes. Bump this target resident's heat, then promote it into the
		// protected probing tier once it has reached PROMOTE_AT hits during this
		// target residency. Read/modify/write the side-table entry in place of
		// the old inline `lfu_counter` field.
		let heat = {
			let h = core.target_heat.entry(*key).or_insert(0);
			*h = h.saturating_add(1);
			*h
		};

		if heat >= PROMOTE_AT {
			debug!("promotion_get: key={key:?} reached PROMOTE_AT in target -> probing={probing_idx}");
			// Heat is per-tier-residency: clear it as the item leaves target,
			// matching the parent's `data.lfu_counter = 0` reset on promotion.
			core.target_heat.remove(key);
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} below PROMOTE_AT (intra-promote)");
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

		// Back-insert promotion: keys[1] lands at the MRU end after keys[0].
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn target_hit_below_threshold_promotes_within_bucket() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]); // stash -> target, heat 0
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]); // stash -> target, heat 0

		// First target hit bumps heat to 1 (< PROMOTE_AT): stays in target.
		seg.get(&keys[1]);
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_none());
	}

	#[test]
	fn target_hit_at_threshold_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Two target hits reach PROMOTE_AT == 2 and promote to probing.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
	}

	/// H01: heat is cleared when a key leaves the cache (re-`put`), so a
	/// re-inserted key starts cold and needs the full PROMOTE_AT hits again.
	#[test]
	fn reinserted_key_starts_with_cleared_heat() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]); // stash -> target, heat 0
		seg.get(&keys[0]); // heat 1 (< PROMOTE_AT): stays in target
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());

		// Simulate the key leaving and being re-inserted: heat must reset.
		seg.core_mut().buckets[target_idx].remove(&keys[0]);
		seg.put(Item::new(keys[0], 10));
		assert_eq!(seg.core().target_heat.get(&keys[0]), None);

		seg.get(&keys[0]); // stash -> target, heat 0 again
		seg.get(&keys[0]); // heat 1: still in target, not promoted
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&keys[0]).is_none());
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
		// Two-get promote: must place into the wrapped probing bucket without panic.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}

	/// H03 single-hop: an item evicted from its probing bucket goes straight to
	/// its stash, with no one-more-chance retry into its true target.
	#[test]
	fn evicted_from_probing_demotes_to_stash() {
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
		assert!(in_stash, "single-hop demotion must route the eviction straight to stash");
		assert!(!in_target, "single-hop demotion must not retry into the true target");
	}

	/// H03 single-hop: a foreign resident evicted from someone else's probing slot
	/// lands directly in its own stash bucket at the end determined by `insert_pos`
	/// (Front for cold, Back for hot) — no target retry.
	fn foreign_to_stash_position(insert_pos: InsertPosition, expected_idx: usize) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 2);
		let foreign = target_keys[0]; // true_target = target_idx, but resident at probing_idx
		let trigger = target_keys[1];
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		// Pre-seed foreign's stash bucket so Front/Back placement is observable.
		let stash_idx = crate::shared::utils::get_index(&foreign, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 2)
			.into_iter()
			.find(|k| *k != foreign)
			.expect("a distinct stash_pre must exist");
		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));

		// `foreign` is the LRU resident of probing_idx (evicted first on overflow).
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 3));

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should land directly in its stash after single-hop demotion");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn foreign_to_stash_at_front_when_cold() {
		foreign_to_stash_position(InsertPosition::Front, 0);
	}

	#[test]
	fn foreign_to_stash_at_back_when_hot() {
		foreign_to_stash_position(InsertPosition::Back, 1);
	}

	/// When an evicted item's true target equals its source bucket, demote_from_normal_bucket
	/// routes it directly to stash. Verifies the stash insert honors `insert_pos`.
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

	/// H03 single-hop: with the retry+cascade removed, a probing eviction no
	/// longer pushes a target resident down to stash. The evicted foreign item
	/// lands in its own stash and the formerly-cascaded target resident stays put.
	#[test]
	fn single_hop_does_not_cascade_target_resident() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 4);
		let target_filler_a = target_keys[0];
		let target_filler_b = target_keys[1];
		let foreign = target_keys[2]; // true_target = target_idx; resident at probing_idx
		let trigger = target_keys[3];
		let probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		// Fill the true target so the parent would have cascaded its LRU on retry.
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_a, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));
		assert!(seg.core().buckets[target_idx].is_full());
		// `foreign` is the LRU of probing_idx (evicted first on overflow).
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 3));
		seg.core_mut().buckets[probing_idx].put(Item::new(probing_filler, 4));
		assert!(seg.core().buckets[probing_idx].is_full());

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), InsertPosition::Back);

		// The evicted foreign item goes straight to its stash (single hop).
		let stash_idx = crate::shared::utils::get_index(&foreign, stash_size);
		assert!(
			seg.core().stash_buckets[stash_idx].get_position(&foreign).is_some(),
			"evicted foreign item must land in stash via single-hop demotion"
		);
		// No cascade: the target's LRU resident is untouched (no retry into target).
		assert!(
			seg.core().buckets[target_idx].get_position(&target_filler_a).is_some(),
			"single-hop demotion must not cascade a target resident down to stash"
		);
	}
}
