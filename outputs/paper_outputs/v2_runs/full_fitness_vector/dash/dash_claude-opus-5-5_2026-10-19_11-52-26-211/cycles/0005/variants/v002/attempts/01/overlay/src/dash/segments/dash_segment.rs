use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. A stash→target insert lands at the LRU end
/// (`InsertPosition::Front`), so an item must climb before it can promote again.
/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a
/// one-more-chance step between adjacent tiers.
///
/// Items demoted from a normal bucket into the stash land at the stash's MRU end
/// (`InsertPosition::Back`), SLRU-style: they already proved reuse, so they should
/// outlive never-reused new arrivals rather than be the next stash victim.
///
/// Brand-new `put` arrivals land at the stash's LRU end ([`FRESH_PUT_STASH_POS`] =
/// `Front`), making the stash a scan-resistant probationary queue: a fresh item is
/// the next victim of its stash bucket unless it is read again (and thus promoted
/// out of the stash) before that bucket next overflows.
///
/// **Two-choice stash:** every stash insert tries the key's primary stash bucket `s0`,
/// then its neighbour `s1 = (s0 + 1) % stash_size`, and only evicts (from `s0`) when both
/// are full. Lookups check `s0` then `s1`.
///
/// **Victim rescue:** when a stash insert evicts (both choices full), the victim is
/// re-homed at the LRU end of its own target bucket, else its probing bucket, if that
/// bucket has a free slot (so the rescue itself never evicts). Otherwise it leaves the cache.
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

/// Stash placement for brand-new `put` arrivals: the LRU end, so never-reused one-hit
/// items are evicted before reuse-proven demotees (which enter at [`DEMOTE_TO_STASH_POS`]).
const FRESH_PUT_STASH_POS: InsertPosition = InsertPosition::Front;

/// When `true`, a stash-overflow victim (both stash choices full) is re-homed into a
/// non-full target/probing bucket instead of leaving the cache. When `false`, behaviour
/// is exactly two-choice stash only.
const RESCUE_STASH_VICTIMS: bool = true;

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

	/// Same semantics as the default trait `put` (precondition: key not resident),
	/// except that a fresh insert lands at the stash's LRU end, uses the two-choice
	/// stash, and any overflow victim may be rescued into a non-full normal bucket.
	fn put(&mut self, item: Item<K, V>) {
		let core = &mut self.core;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?} -> stash {FRESH_PUT_STASH_POS:?}", item.key);
		stash_insert_with_rescue(core, FRESH_PUT_STASH_POS, item);
	}
}

// ── Two-choice stash ───────────────────────────────────────────────────────

/// Inserts `item` at `pos` into its primary stash bucket `s0` if it has room, else into
/// the neighbour `s1` if it has room, else into `s0` evicting per policy. Returns the
/// evicted item (only possible when both choices were full).
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let (s0, s1) = core.stash_indices(&item.key);
	if !core.stash_buckets[s0].is_full() {
		core.stash_buckets[s0].put_at(item, pos);
		return None;
	}
	if s1 != s0 && !core.stash_buckets[s1].is_full() {
		debug!("stash_insert: key={:?} s0={s0} full -> s1={s1}", item.key);
		core.stash_buckets[s1].put_at(item, pos);
		return None;
	}
	let evicted = core.stash_buckets[s0].put_at(item, pos).1;
	if let Some(ref e) = evicted {
		debug!("stash_insert: s0={s0} s1={s1} full, evicted key={:?}", e.key);
	}
	evicted
}

/// [`stash_insert`], then — if it evicted a victim (both stash choices full) — try to
/// re-home the victim at the LRU end of its target bucket, else its probing bucket, only
/// when that bucket has a free slot. The rescue never evicts, so it never cascades.
/// Otherwise the victim leaves the cache.
fn stash_insert_with_rescue<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let Some(victim) = stash_insert(core, pos, item) else {
		return;
	};
	if !RESCUE_STASH_VICTIMS {
		debug!("stash_insert_with_rescue: key={:?} dropped (rescue disabled)", victim.key);
		return;
	}
	let t = get_index(&victim.key, core.segment_size);
	let p = (t + 1) % core.segment_size;
	if !core.buckets[t].is_full() {
		debug!("stash_insert_with_rescue: key={:?} rescued -> target={t}", victim.key);
		let evicted = core.put_at(Slot::Target, InsertPosition::Front, victim);
		debug_assert!(evicted.is_none(), "rescue into non-full target must not evict");
	} else if !core.buckets[p].is_full() {
		debug!("stash_insert_with_rescue: key={:?} rescued -> probing={p}", victim.key);
		let evicted = core.put_at(Slot::Probing, InsertPosition::Front, victim);
		debug_assert!(evicted.is_none(), "rescue into non-full probing must not evict");
	} else {
		debug!("stash_insert_with_rescue: key={:?} dropped (target/probing full)", victim.key);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Stash placement for items demoted out of a normal bucket. SLRU rationale: an item
/// leaving the protected tier (target/probing) has proven reuse, so it enters the
/// probationary tier (stash) at its MRU end, behind never-reused new arrivals in
/// eviction order — not at the LRU end where the very next stash overflow would evict it.
const DEMOTE_TO_STASH_POS: InsertPosition = InsertPosition::Back;

/// Moves `item` into its target bucket (`target_idx`, precomputed by the caller as
/// `get_index(&item.key, segment_size)`). Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	target_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(target_idx, get_index(&item.key, core.segment_size));
	let Some(evicted) = core.put_at(Slot::Target, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket (`probing_idx`, precomputed by the caller as
/// `(target_idx + 1) % segment_size`). Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	probing_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(
		probing_idx,
		(get_index(&item.key, core.segment_size) + 1) % core.segment_size
	);
	let Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
///
/// `insert_pos` governs only the one-more-chance retry into the true target; every stash
/// insert uses [`DEMOTE_TO_STASH_POS`] (MRU end), per SLRU, via the two-choice stash with
/// victim rescue.
///
/// Never writes into the bucket at `source_idx`: a native victim goes to stash, and a
/// foreign victim's true target is necessarily `source_idx - 1` (mod segment_size),
/// which differs from `source_idx` since segment_size >= 2. A rescue only writes into a
/// non-full bucket, and `source_idx` is full right after the insert that evicted from it.
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
		stash_insert_with_rescue(core, DEMOTE_TO_STASH_POS, item);
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
			stash_insert_with_rescue(core, DEMOTE_TO_STASH_POS, cascade);
		}
	}
}

/// Position of a freshly inserted item in a bucket of length `len` after a
/// `Bucket::put_at(.., insert_pos)` that evicted (if full) *before* inserting.
#[inline]
fn inserted_position(insert_pos: InsertPosition, len: usize) -> usize {
	match insert_pos {
		InsertPosition::Front => 0,
		InsertPosition::Back => len - 1,
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
/// Demotions into the stash always land at the stash's MRU end regardless of `insert_pos`.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some((stash_idx, item_idx)) = core.find_in_stash(key) {
		debug!("promotion_get: key={key:?} found in stash={stash_idx} -> target={target_idx} insert_pos={insert_pos:?}");
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, data, target_idx, insert_pos);

		// The cascade never writes back into target_idx (rescues only fill non-full
		// buckets, and target_idx is full whenever a cascade runs), so the new item sits
		// exactly where the insert put it.
		let items = core.buckets[target_idx].get_items();
		let pos = inserted_position(insert_pos, items.len());
		debug_assert_eq!(core.buckets[target_idx].get_position(key), Some(pos));
		return Some(&items[pos]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, probing_idx, insert_pos);

			// The cascade never writes back into probing_idx (a foreign victim retries in
			// target_idx; rescues only fill non-full buckets), so the new item sits exactly
			// where the insert put it.
			let items = core.buckets[probing_idx].get_items();
			let pos = inserted_position(insert_pos, items.len());
			debug_assert_eq!(core.buckets[probing_idx].get_position(key), Some(pos));
			return Some(&items[pos]);
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
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings, KEY_SEARCH_RANGE};

	fn new_segment(policy: EvictionPolicyKind) -> DashSegment<i64, i64> {
		DashSegment::new(&small_dash_settings(policy))
	}

	/// Fills normal bucket `idx` to capacity with keys native to it (target == idx) that
	/// are not in `avoid` and not already resident.
	fn fill_normal(core: &mut SegmentCore<i64, i64>, idx: usize, avoid: &[i64]) {
		for k in find_keys_for_bucket(idx, core.segment_size, KEY_SEARCH_RANGE as usize) {
			if core.buckets[idx].is_full() {
				break;
			}
			if avoid.contains(&k) || core.contains_key(&k) {
				continue;
			}
			core.buckets[idx].put(Item::new(k, k));
		}
		assert!(core.buckets[idx].is_full(), "fill_normal: not enough keys for bucket {idx}");
	}

	/// Fills stash bucket `idx` to capacity with keys whose primary stash bucket is `idx`
	/// that are not in `avoid` and not already resident.
	fn fill_stash(core: &mut SegmentCore<i64, i64>, idx: usize, avoid: &[i64]) {
		for k in find_keys_for_bucket(idx, core.stash_size, KEY_SEARCH_RANGE as usize) {
			if core.stash_buckets[idx].is_full() {
				break;
			}
			if avoid.contains(&k) || core.contains_key(&k) {
				continue;
			}
			core.stash_buckets[idx].put(Item::new(k, k));
		}
		assert!(core.stash_buckets[idx].is_full(), "fill_stash: not enough keys for stash {idx}");
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_size = seg.core().stash_size;
		let stash_idx = get_index(&1_i64, stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());

		// A second fresh key in the same stash bucket lands at the LRU end (index 0).
		let second = find_keys_for_bucket(stash_idx, stash_size, 4)
			.into_iter()
			.find(|k| *k != 1)
			.expect("a second key for the same stash bucket must exist");
		seg.put(Item::new(second, 20));
		assert_eq!(seg.core().stash_buckets[stash_idx].get_position(&second), Some(0));
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	/// A demoted (reuse-proven) stash resident outlives fresh arrivals: on stash overflow
	/// the fresh arrival at the LRU end is evicted, not the demotee.
	#[test]
	fn fresh_put_evicted_before_demoted_item() {
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
		let fresh: Vec<i64> = find_keys_for_bucket(stash_idx, stash_size, 16)
			.into_iter()
			.filter(|k| !target_keys.contains(k))
			.take(2)
			.collect();
		assert_eq!(fresh.len(), 2);

		// Demote `victim` into the stash via target overflow.
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));
		let trigger_target = get_index(&trigger, seg_size);
		promote_to_target(seg.core_mut(), Item::new(trigger, 99), trigger_target, InsertPosition::Front);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&victim).is_some());

		// Block the two-choice neighbour and the fresh victim's rescue destinations so the
		// overflow below still evicts fresh[0] out of the cache.
		let avoid = [victim, resident, trigger, fresh[0], fresh[1]];
		let (_, s1) = seg.core().stash_indices(&victim);
		fill_stash(seg.core_mut(), s1, &avoid);
		let f0_t = get_index(&fresh[0], seg_size);
		fill_normal(seg.core_mut(), f0_t, &avoid);
		fill_normal(seg.core_mut(), (f0_t + 1) % seg_size, &avoid);

		// Fill the stash bucket with a fresh put (bucket_size = 2).
		seg.put(Item::new(fresh[0], 10));
		assert!(seg.core().stash_buckets[stash_idx].is_full());

		// Overflow with one more fresh put.
		seg.put(Item::new(fresh[1], 11));

		let stash = &seg.core().stash_buckets[stash_idx];
		assert!(stash.get_position(&victim).is_some(), "demoted item must survive");
		assert!(stash.get_position(&fresh[0]).is_none(), "oldest fresh arrival must be evicted");
		assert_eq!(stash.get_position(&fresh[1]), Some(0), "new arrival sits at LRU end");
		assert!(!seg.core().contains_key(&fresh[0]));
	}

	/// Both stash choices full and the victim's target has one free slot: the overflow
	/// victim is rescued to target index 0 and stays readable.
	#[test]
	fn stash_overflow_victim_rescued_to_target() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let x = find_keys_for_bucket(0, settings.num_of_stash_buckets, 1)[0];
		let (s0, s1) = seg.core().stash_indices(&x);
		assert_ne!(s0, s1);
		fill_stash(seg.core_mut(), s0, &[x]);
		fill_stash(seg.core_mut(), s1, &[x]);
		let victim_idx = seg.core().stash_buckets[s0].eviction_candidate_index().unwrap();
		let victim = seg.core().stash_buckets[s0].get_items()[victim_idx].key;

		// Leave exactly one free slot in the victim's target bucket.
		let t = get_index(&victim, seg_size);
		let filler = find_keys_for_bucket(t, seg_size, KEY_SEARCH_RANGE as usize)
			.into_iter()
			.find(|k| *k != x && !seg.core().contains_key(k))
			.expect("a filler key for the victim's target must exist");
		seg.core_mut().buckets[t].put(Item::new(filler, filler));
		assert_eq!(seg.core().buckets[t].size(), settings.bucket_size - 1);

		let before = seg.core().total_items();
		seg.put(Item::new(x, 42));

		assert_eq!(seg.core().total_items(), before + 1, "rescue keeps the victim resident");
		assert_eq!(seg.core().buckets[t].get_position(&victim), Some(0));
		assert!(seg.core().stash_buckets[s0].get_position(&x).is_some());
		assert!(seg.core().check_slot_legality());
		assert_eq!(seg.get(&victim).map(|it| it.key), Some(victim));
		assert_eq!(seg.get(&x).map(|it| it.value), Some(42));
	}

	/// `s0` full and `s1` free: the fresh put lands in `s1`; no eviction, no rescue.
	#[test]
	fn fresh_put_uses_neighbour_stash_when_primary_full() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let x = find_keys_for_bucket(0, settings.num_of_stash_buckets, 1)[0];
		let (s0, s1) = seg.core().stash_indices(&x);
		fill_stash(seg.core_mut(), s0, &[x]);
		let s0_keys: Vec<i64> = seg.core().stash_buckets[s0].get_items().iter().map(|i| i.key).collect();

		seg.put(Item::new(x, 5));

		assert!(seg.core().stash_buckets[s1].get_position(&x).is_some(), "x must land in s1");
		let after: Vec<i64> = seg.core().stash_buckets[s0].get_items().iter().map(|i| i.key).collect();
		assert_eq!(after, s0_keys, "s0 must be untouched");
		assert!(seg.core().buckets.iter().all(|b| b.size() == 0), "no rescue into normal buckets");
		assert!(seg.core().contains_key(&x));
		assert!(seg.core().check_slot_legality());
		assert_eq!(seg.get(&x).map(|it| it.value), Some(5));
	}

	/// Both stash choices, the victim's target, and its probing bucket are all full: the
	/// victim leaves the cache.
	#[test]
	fn stash_overflow_victim_dropped_when_all_full() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let x = find_keys_for_bucket(0, settings.num_of_stash_buckets, 1)[0];
		let (s0, s1) = seg.core().stash_indices(&x);
		fill_stash(seg.core_mut(), s0, &[x]);
		fill_stash(seg.core_mut(), s1, &[x]);
		let victim_idx = seg.core().stash_buckets[s0].eviction_candidate_index().unwrap();
		let victim = seg.core().stash_buckets[s0].get_items()[victim_idx].key;
		let t = get_index(&victim, seg_size);
		fill_normal(seg.core_mut(), t, &[x]);
		fill_normal(seg.core_mut(), (t + 1) % seg_size, &[x]);

		let before = seg.core().total_items();
		seg.put(Item::new(x, 1));

		assert_eq!(seg.core().total_items(), before, "one in, one out");
		assert!(!seg.core().contains_key(&victim), "victim must leave the cache");
		assert!(seg.get(&victim).is_none());
		assert!(seg.core().contains_key(&x));
	}

	#[test]
	fn stash_hit_inserts_at_front_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Cold insert pushes keys[1] to Front, displacing keys[0] backward.
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
	}

	/// Stash hit on a key whose target bucket is full: the victim is evicted before the
	/// Front insert, so the returned reference is the requested key at index 0 of target.
	#[test]
	fn stash_hit_on_full_target_returns_item_at_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg_size, settings.bucket_size + 1);
		let k = keys[settings.bucket_size];
		for &f in &keys[..settings.bucket_size] {
			seg.core_mut().buckets[target_idx].put(Item::new(f, f * 10));
		}
		assert!(seg.core().buckets[target_idx].is_full());

		seg.put(Item::new(k, 777));
		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 777)));
		assert_eq!(seg.core().buckets[target_idx].get_position(&k), Some(0));
		assert!(seg.core().buckets[target_idx].is_full());
		assert!(seg.core().check_slot_legality());
	}

	#[test]
	fn target_hit_not_hottest_promotes_within_bucket() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

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

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// First hit moves keys[1] to MRU; second hit (already at MRU) promotes to probing.
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

		// Two gets on target_keys[0]: first bumps it to MRU, second promotes to probing,
		// which evicts the LRU resident of probing.
		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		// The evicted item's target is `probing_idx`, so the cascade routes it to stash
		// (either two-choice bucket) instead of bouncing it back into the bucket it just left.
		// Permissive assertion: only "vanished entirely" would be a real bug.
		let evicted_key = probing_keys[0];
		assert!(
			seg.core.contains_key(&evicted_key),
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

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 200), trigger_probing, InsertPosition::Back);

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

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), trigger_probing, insert_pos);

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
	/// AND that the stash insert always lands at the stash's MRU end (Back), regardless of
	/// the caller's `insert_pos`.
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

		let trigger_target = get_index(&trigger, seg_size);
		promote_to_target(seg.core_mut(), Item::new(trigger, 99), trigger_target, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_back_when_cold() {
		direct_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert lands at the stash's MRU end
	/// (Back), regardless of the caller's `insert_pos`.
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

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), trigger_probing, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn cascade_to_stash_at_back_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
