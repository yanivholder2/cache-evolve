use super::{probing2_index, Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Frequency threshold (H02): a target resident is promoted into the protected
/// probing tier only after it has proven this many hits during its current
/// target residency. Below the threshold it climbs positionally instead.
const PROMOTE_AT: u8 = 2;

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing tier only by proving they're warm
/// enough. A stash→target insert lands at the MRU end (`InsertPosition::Back`).
/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a
/// one-more-chance step between adjacent tiers.
///
/// **H03 — 3-wide associativity.** The probing tier is widened from one bucket
/// (`target+1`) to two (`target+1` *and* the secondary probing slot `target+2`),
/// so warm keys that collide on the same target coexist instead of thrashing.
/// The secondary slot is non-wrapping (see [`probing2_index`]): keys near the
/// top of the ring keep the baseline 2-wide neighborhood, which is required to
/// preserve the subject's slot-legality invariant. On a hit a key may now be
/// found in target, the primary probing bucket, or the secondary probing bucket;
/// on promotion, when the primary probing bucket is full the item prefers the
/// secondary bucket (if it has room) before triggering the demotion cascade.
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

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, key, InsertPosition::Back)
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing tier only by proving they're warm enough (PROMOTE_AT hits in this
// target residency). The probing tier spans two buckets (`target+1` and the
// non-wrapping `target+2`). Demotion cascades down the chain `probing -> target
// -> stash -> out`, with a one-more-chance step between adjacent tiers.

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

/// Moves `item` into the probing tier. The item prefers the primary probing
/// bucket (`target+1`); if that bucket is full but the secondary probing bucket
/// (`target+2`, non-wrapping) exists and has room, the item lands there with no
/// eviction. Only when both probing buckets are full (or the secondary slot does
/// not exist) does inserting into the primary bucket trigger the demotion
/// cascade. Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let probing2_idx = probing2_index(target_idx, core.segment_size);

	// Conflict relief (H03): if the primary probing bucket is saturated, prefer
	// the secondary probing bucket when it has room rather than evicting a warm
	// resident of the primary bucket. The item came from `target` (removed before
	// this call), so by the uniqueness invariant it is resident nowhere else and
	// this placement cannot duplicate it.
	if core.buckets[probing_idx].is_full() {
		if let Some(p2) = probing2_idx {
			if !core.buckets[p2].is_full() {
				// `p2` has room, so this put never evicts.
				core.buckets[p2].put_at(item, insert_pos);
				return;
			}
		}
	}

	let Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx`
/// (placed there as someone else's probing or secondary-probing slot), retry in
/// its own true target before stash; if it was already in its true target, go
/// straight to stash. Cascade evictions land in stash. The 3-wide neighborhood
/// needs no special-casing here: `true_target` is recomputed from the key, so an
/// item evicted from either probing bucket is routed to its true target, while an
/// item evicted from its own target goes to stash.
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
	let probing2_idx = probing2_index(target_idx, core.segment_size);

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
		// Frequency gate (H02): bump the resident's heat counter, then promote it
		// into the protected probing tier only once it has reached PROMOTE_AT hits
		// during this target residency. The signal is the already-present-but-
		// otherwise-unused `u8 lfu_counter`, so it costs no extra metadata.
		let heat = core.buckets[target_idx].get_items()[item_idx]
			.lfu_counter
			.saturating_add(1);
		core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter = heat;

		if heat >= PROMOTE_AT {
			debug!("promotion_get: key={key:?} reached PROMOTE_AT in target -> probing tier");
			let mut data = core.buckets[target_idx].remove_at(item_idx);
			// Heat is per-tier-residency: clear it as the item leaves target.
			data.lfu_counter = 0;
			promote_to_probing(core, data, insert_pos);

			// The promoted key now lives in the primary probing bucket or, under
			// H03 conflict relief, in the secondary probing bucket. Search both.
			if let Some(i) = core.buckets[probing_idx].get_position(key) {
				return Some(&core.buckets[probing_idx].get_items()[i]);
			}
			if let Some(p2) = probing2_idx {
				if let Some(i) = core.buckets[p2].get_position(key) {
					return Some(&core.buckets[p2].get_items()[i]);
				}
			}
			unreachable!("promotion_get: key={key:?} not found after promote_to_probing -- the item must remain resident in the primary or secondary probing bucket")
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} below PROMOTE_AT (intra-promote)");
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	// H03: the secondary probing bucket is a first-class residence — a hit here
	// touches the item in place, exactly like the primary probing bucket.
	if let Some(p2) = probing2_idx {
		if let Some(item_idx) = core.buckets[p2].get_position(key) {
			debug!("promotion_get: key={key:?} found in secondary probing bucket={p2}");
			return Some(core.buckets[p2].get_from_position(item_idx));
		}
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

		// Two target hits reach PROMOTE_AT == 2 and promote to the probing tier.
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
	fn secondary_probing_hit_touches_in_place() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		// small_dash_settings has 4 normal buckets, so target 0 has a non-wrapping
		// secondary probing slot at bucket 2.
		let p2 = probing2_index(target_idx, seg.core.segment_size).expect("target 0 has a secondary probing slot");
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.core.buckets[p2].put(Item::new(keys[0], 10));

		let val = seg.get(&keys[0]).unwrap().value;
		assert_eq!(val, 10);
		assert!(seg.core.buckets[p2].get_position(&keys[0]).is_some());
	}

	#[test]
	fn promote_prefers_secondary_when_primary_full() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let bsize = settings.bucket_size;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let p2 = probing2_index(target_idx, seg_size).expect("secondary slot exists");

		// Fill the primary probing bucket with native residents (true target ==
		// probing_idx) so a promotion from target cannot evict-and-retry them and
		// must instead fall through to the secondary probing bucket, which is empty.
		let probing_keys = find_keys_for_bucket(probing_idx, seg_size, bsize);
		for &k in &probing_keys {
			seg.core_mut().buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core().buckets[probing_idx].is_full());

		let target_keys = find_keys_for_bucket(target_idx, seg_size, 2);
		seg.put(Item::new(target_keys[0], 1));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 2));
		seg.get(&target_keys[1]);
		// Drive target_keys[1] to PROMOTE_AT (== 2 target hits) so it promotes.
		seg.get(&target_keys[1]);
		seg.get(&target_keys[1]);

		assert!(
			seg.core().buckets[p2].get_position(&target_keys[1]).is_some(),
			"promoted key should land in the secondary probing bucket when primary is full"
		);
		// And it must remain reachable via get.
		assert!(seg.get(&target_keys[1]).is_some());
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

	/// An item evicted from its primary probing bucket must cascade to its true
	/// target (one-more-chance), not jump straight to stash. Under H03 both probing
	/// buckets are filled so promotion cannot divert to the secondary slot and is
	/// forced to evict from the primary probing bucket.
	#[test]
	fn evicted_from_probing_cascades_to_target() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let bsize = settings.bucket_size;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let p2 = probing2_index(target_idx, seg_size).expect("secondary slot exists");

		let target_keys = find_keys_for_bucket(target_idx, seg_size, 2);
		let k = target_keys[0]; // foreign resident of `probing_idx`; true target == target_idx
		let trigger = target_keys[1]; // true target == target_idx

		// Saturate the secondary probing bucket so promotion cannot divert there.
		let p2_fillers = find_keys_for_bucket(p2, seg_size, bsize);
		for &f in &p2_fillers {
			seg.core_mut().buckets[p2].put(Item::new(f, f * 10));
		}
		assert!(seg.core().buckets[p2].is_full());

		// Fill the primary probing bucket: `k` first so it is the LRU victim, then a
		// native filler whose true target is `probing_idx`.
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];
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
}
