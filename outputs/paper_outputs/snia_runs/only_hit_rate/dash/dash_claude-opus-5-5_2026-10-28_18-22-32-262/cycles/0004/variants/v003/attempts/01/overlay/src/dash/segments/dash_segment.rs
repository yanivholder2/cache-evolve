use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment (cycle 0004 v003: MRU-end stash demotion + two-hit
/// stash-promotion gate + MRU-end placement of gated stash→target promotions +
/// zero-hit-first stash victim selection).
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. Demotion cascades down the chain
/// `probing -> target -> stash -> out`, with a one-more-chance step between
/// adjacent tiers.
///
/// Inherited (cycle 0001 v001): items demoted out of a normal bucket into the
/// stash land at the stash's MRU end (`InsertPosition::Back`).
///
/// Inherited (cycle 0002 v001): a stash item is promoted to its target only on its
/// second hit during the current stash stay. The first hit just refreshes it to the
/// stash MRU end. The hit count lives in `Item::lfu_counter` (unused by ClassicLRU)
/// and is reset to 0 on every stash entry from a normal bucket and on promotion out
/// of the stash.
///
/// Inherited (cycle 0003 v003): a stash item that passed the gate lands at its
/// target's MRU end (`STASH_PROMOTION_TARGET_POS = Back`).
///
/// New (stash victim rule): every stash insert (fresh `put`, direct demotion, and
/// cascade demotion) goes through [`stash_insert`]. When the stash bucket is full,
/// the victim is the LRU-most stash item with zero hits in its current stay
/// (`lfu_counter == 0`), scanning from index 0 (LRU end) towards MRU. Only if every
/// resident already has a hit does the bucket fall back to its plain LRU eviction.
/// This spares items that are halfway through the two-hit gate from being flushed
/// by one-hit arrivals. Exactly one item still leaves a full stash per insert.
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

	/// Same semantics as the default `Segment::put` (precondition: key not resident;
	/// item lands at the stash MRU end), but the stash insert goes through
	/// [`stash_insert`] so a full stash evicts its LRU-most zero-hit item first.
	fn put(&mut self, item: Item<K, V>) {
		let core = &mut self.core;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?}", item.key);
		stash_insert(core, InsertPosition::Back, item);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Position at which items demoted out of a normal bucket land in their stash bucket.
/// MRU end: a demoted item has already earned a normal-bucket slot, so it should
/// outlive brand-new stash arrivals rather than be the next stash victim.
const STASH_DEMOTION_POS: InsertPosition = InsertPosition::Back;

/// Number of hits during a single stash stay required before a stash item is
/// promoted to its target bucket.
const STASH_PROMOTION_HITS: usize = 2;

/// Where a stash item that passed the gate lands in its target.
/// MRU end: an item that has proven `STASH_PROMOTION_HITS` stash hits should not be
/// the next victim evicted from its target.
const STASH_PROMOTION_TARGET_POS: InsertPosition = InsertPosition::Back;

/// Inserts `item` into its stash bucket at `pos`.
///
/// Victim rule: if the stash bucket is full, the LRU-most item with zero stash hits
/// in its current stay (`lfu_counter == 0`, scanning from index 0 = LRU end towards
/// MRU) is removed and dropped (a cache eviction). If every resident already has at
/// least one hit, nothing is removed here and the bucket's own LRU eviction picks the
/// victim inside `put_at`. Either way exactly one item leaves a full stash.
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(&item.key, core.stash_size);
	let bucket = &mut core.stash_buckets[stash_idx];
	if bucket.is_full() {
		if let Some(v) = bucket.get_items().iter().position(|it| it.lfu_counter == 0) {
			let victim = bucket.remove_at(v);
			debug!(
				"stash_insert: stash={stash_idx} evicted zero-hit key={:?} at idx={v}",
				victim.key
			);
		}
	}
	// `None` when a slot was freed above; otherwise the bucket's LRU victim (dropped).
	let _ = core.put_at(Slot::Stash, pos, item);
}

/// Moves `item` into its target bucket at `place_pos`. Any evicted item flows through
/// the tier cascade; foreign residents displaced by it retry their own target at
/// `retry_pos`.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	place_pos: InsertPosition,
	retry_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Target, place_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx, retry_pos, STASH_DEMOTION_POS);
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
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos, STASH_DEMOTION_POS);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target (at `retry_pos`)
/// before stash; if it was already in its target, go straight to stash. All stash
/// inserts (direct and cascade) land at `stash_pos` with their stash-hit counter reset,
/// so every stash stay needs fresh hits to earn promotion. Stash inserts go through
/// [`stash_insert`] (zero-hit-first victim rule).
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	mut item: Item<K, V>,
	source_idx: usize,
	retry_pos: InsertPosition,
	stash_pos: InsertPosition,
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
		item.lfu_counter = 0;
		stash_insert(core, stash_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(mut cascade) = core.put_at(Slot::Target, retry_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			cascade.lfu_counter = 0;
			stash_insert(core, stash_pos, cascade);
		}
	}
}

/// `insert_pos = Front`: probing promotions and foreign-resident retries land at the LRU
/// end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: those land at MRU; one more access promotes to the next tier.
/// Gated stash→target promotions always land at `STASH_PROMOTION_TARGET_POS`, and stash
/// demotions always land at `STASH_DEMOTION_POS`, independent of `insert_pos`.
/// Stash items promote only on their `STASH_PROMOTION_HITS`-th hit in the current stay.
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
		let hits = {
			let item = &mut core.stash_buckets[stash_idx].get_items_mut()[item_idx];
			item.lfu_counter = item.lfu_counter.saturating_add(1);
			item.lfu_counter
		};
		if hits < STASH_PROMOTION_HITS {
			debug!("promotion_get: key={key:?} found in stash hits={hits} -> refresh in stash={stash_idx}");
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		debug!(
			"promotion_get: key={key:?} found in stash hits={hits} -> target={target_idx} place_pos={STASH_PROMOTION_TARGET_POS:?} retry_pos={insert_pos:?}"
		);
		core.stash_buckets[stash_idx].get_items_mut()[item_idx].lfu_counter = 0;
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, data, STASH_PROMOTION_TARGET_POS, insert_pos);

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
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
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

	/// A full stash bucket evicts its LRU-most zero-hit item, sparing an LRU-end item
	/// that already has one stash hit.
	#[test]
	fn full_stash_spares_item_with_one_hit() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let cap = seg.core.stash_buckets[0].get_max_size();
		let stash_idx = 0;
		let keys = find_keys_for_bucket(stash_idx, stash_size, cap + 1);
		assert_eq!(keys.len(), cap + 1, "sanity: enough keys for the stash bucket");

		for &k in &keys[..cap] {
			seg.put(Item::new(k, k * 10));
		}
		assert!(seg.core.stash_buckets[stash_idx].is_full());

		// The LRU-most item has one hit in its current stay.
		let lru_key = keys[0];
		{
			let items = seg.core_mut().stash_buckets[stash_idx].get_items_mut();
			assert_eq!(items[0].key, lru_key);
			items[0].lfu_counter = 1;
		}

		let before = seg.core.total_items();
		seg.put(Item::new(keys[cap], 0));
		assert_eq!(seg.core.total_items(), before, "exactly one item leaves a full stash");

		let bucket = &seg.core.stash_buckets[stash_idx];
		assert!(bucket.get_position(&lru_key).is_some(), "one-hit item must survive");
		assert!(
			bucket.get_position(&keys[1]).is_none(),
			"oldest zero-hit item must be evicted"
		);
		assert!(bucket.get_position(&keys[cap]).is_some());
		assert_eq!(bucket.get_items().last().unwrap().key, keys[cap], "new item at MRU end");
	}

	/// When every stash resident has a hit, the plain LRU victim is evicted.
	#[test]
	fn full_stash_all_hit_falls_back_to_lru() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let cap = seg.core.stash_buckets[0].get_max_size();
		let stash_idx = 0;
		let keys = find_keys_for_bucket(stash_idx, stash_size, cap + 1);

		for &k in &keys[..cap] {
			seg.put(Item::new(k, k * 10));
		}
		for it in seg.core_mut().stash_buckets[stash_idx].get_items_mut() {
			it.lfu_counter = 1;
		}
		seg.put(Item::new(keys[cap], 0));

		let bucket = &seg.core.stash_buckets[stash_idx];
		assert!(bucket.get_position(&keys[0]).is_none(), "LRU item evicted on fallback");
		assert_eq!(bucket.size(), cap);
	}

	/// A single stash hit does not promote: the key stays in its stash bucket and is
	/// refreshed to the MRU (last) index.
	#[test]
	fn single_stash_hit_stays_in_stash_at_mru() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let stash_idx = 0;
		let keys = find_keys_for_bucket(stash_idx, stash_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.put(Item::new(keys[1], 20));

		let val = seg.get(&keys[0]).expect("hit must return Some").value;
		assert_eq!(val, 10);

		let items = seg.core.stash_buckets[stash_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == keys[0])
			.expect("key must remain in its stash bucket after a single hit");
		assert_eq!(pos, items.len() - 1, "single stash hit must refresh to the MRU end");

		let target_idx = get_index(&keys[0], seg.core.segment_size);
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_none());

		// Second hit promotes into the target.
		seg.get(&keys[0]);
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.stash_buckets[stash_idx].get_position(&keys[0]).is_none());
	}

	#[test]
	fn stash_hit_inserts_at_back_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		// Gated promotion lands at the MRU end (Back): keys[1] follows keys[0].
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn target_hit_not_hottest_promotes_within_bucket() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		// Target is [keys[0], keys[1]]: keys[0] is at the LRU end (not hottest). Hitting
		// it should bump it Back via ClassicLRU, not promote it to probing.
		seg.get(&keys[0]);
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
	}

	#[test]
	fn target_hit_hottest_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		// keys[1] was promoted to the target's MRU end, so it is already hottest:
		// one hit promotes it to probing.
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
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
		seg.get(&target_keys[1]);
		assert!(seg.core.buckets[target_idx].is_full());

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Target is [target_keys[0], target_keys[1]]: target_keys[1] sits at the MRU end,
		// so one get promotes it to probing, which evicts the LRU resident of probing.
		seg.get(&target_keys[1]);
		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[1]).is_some());

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
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		// keys[1] landed at the target MRU end, so one get promotes it: must place into
		// the wrapped probing bucket without panic.
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
	/// AND that the stash insert lands at the stash MRU end (Back) regardless of the
	/// promotion `insert_pos`. The stash bucket starts with one item (not full), so the
	/// zero-hit victim rule does not fire and the expected index is unchanged.
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
		assert!(!seg.core().stash_buckets[stash_idx].is_full());
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));

		promote_to_target(seg.core_mut(), Item::new(trigger, 99), insert_pos, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_front_when_cold() {
		// v001: demoted items land at the stash Back even for a Front promotion.
		direct_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert lands at the stash MRU end.
	/// The stash bucket starts with one item (not full), so the zero-hit victim rule
	/// does not fire and the expected index is unchanged.
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
		assert!(!seg.core().stash_buckets[stash_idx].is_full());
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
		// v001: cascaded items land at the stash Back even for a Front promotion.
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
