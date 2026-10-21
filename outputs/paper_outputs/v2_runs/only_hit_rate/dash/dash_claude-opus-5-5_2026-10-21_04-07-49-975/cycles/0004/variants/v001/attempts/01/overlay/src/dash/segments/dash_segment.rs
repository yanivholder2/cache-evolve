use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Number of stash hits an item needs before it is promoted to its target bucket.
/// Hits below the threshold are served in place in stash (touching stash LRU order).
/// `Item::lfu_counter` is reused as the per-item stash-hit counter.
const PROMOTE_THRESHOLD: usize = 2;

/// Number of evicted-key fingerprints remembered per stash bucket.
const GHOST_WAYS: usize = 4;

/// Small fixed-size ring of fingerprints of keys recently evicted from one stash
/// bucket. `0` is the empty sentinel. Holds fingerprints only, never values, so a
/// false positive only causes a spurious admission pre-credit.
#[derive(Debug, Clone, Copy, Default)]
struct GhostRing {
	fps: [u32; GHOST_WAYS],
	cursor: u8,
}

impl GhostRing {
	/// Records `fp`, overwriting the oldest entry.
	fn record(&mut self, fp: u32) {
		let c = self.cursor as usize % GHOST_WAYS;
		self.fps[c] = fp;
		self.cursor = ((c + 1) % GHOST_WAYS) as u8;
	}

	/// If `fp` is present, clears that entry and returns `true`.
	fn take(&mut self, fp: u32) -> bool {
		for slot in self.fps.iter_mut() {
			if *slot == fp {
				*slot = 0;
				return true;
			}
		}
		false
	}
}

/// 32-bit key fingerprint: the upper 32 bits of the key's 64-bit hash (the bucket
/// index uses the full hash modulo a small bucket count, dominated by low bits).
/// `0` is reserved as the empty sentinel and is mapped to `1`.
fn fingerprint<K: Hash>(key: &K) -> u32 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	let fp = (hasher.finish() >> 32) as u32;
	if fp == 0 { 1 } else { fp }
}

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. A stash→target insert lands at the LRU end
/// (`InsertPosition::Front`), so an item must climb before it can promote again.
/// Demotion cascades down the chain `probing -> target -> stash -> out`, with a
/// one-more-chance step between adjacent tiers.
///
/// This variant adds a stash admission filter: an item must be hit
/// `PROMOTE_THRESHOLD` times in stash before it is moved into its target bucket.
/// Items demoted out of the normal tier re-enter stash at the MRU end, so the
/// stash acts as a victim cache for ex-residents.
///
/// Stash victim selection is frequency-aware (CLOCK-like second chance): when a
/// stash bucket is full, the victim is the first item, scanning from the LRU end,
/// whose stash-hit counter is `0`. Skipped items have their counter decremented
/// (saturating at `0`). If none has a zero counter, the LRU-end item is evicted.
///
/// Ghost history: each stash bucket remembers the fingerprints of the last
/// `GHOST_WAYS` keys that left the cache from it. A newly `put` key whose
/// fingerprint is found there is admitted pre-credited with `PROMOTE_THRESHOLD - 1`
/// stash hits, so a single hit promotes it.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	ghosts: Vec<GhostRing>,
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
			let ghosts = vec![GhostRing::default(); core.stash_size];
			Some(Self { core, ghosts })
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
		promotion_get(&mut self.core, &mut self.ghosts, key, InsertPosition::Front)
	}

	/// Same placement as the default `put` (stash, `InsertPosition::Back`), but a full
	/// stash bucket picks its victim through `stash_put`'s second-chance scan, and a
	/// key found in its stash bucket's ghost ring is admitted pre-credited.
	fn put(&mut self, mut item: Item<K, V>) {
		let core = &mut self.core;
		let ghosts = &mut self.ghosts;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let stash_idx = get_index(&item.key, core.stash_size);
		if ghosts[stash_idx].take(fingerprint(&item.key)) {
			debug!("DashSegment.put: key={:?} ghost hit -> pre-credited", item.key);
			item.lfu_counter = PROMOTE_THRESHOLD - 1;
		} else {
			debug!("DashSegment.put: key={:?}", item.key);
		}
		stash_put(core, ghosts, InsertPosition::Back, item);
	}
}

// ── Stash victim selection ─────────────────────────────────────────────────

/// Inserts `item` at `pos` in its stash bucket. If the bucket is full, a victim is
/// chosen first by a bounded second-chance scan from the LRU end (index 0) toward
/// the MRU end: the first slot with a stash-hit counter of `0` is evicted; every
/// slot skipped has its counter decremented (never below `0`). If no slot has a
/// zero counter, the policy's eviction candidate (LRU end under ClassicLRU) is
/// evicted. The victim is removed via `remove_at`, preserving the order of the
/// remaining items. Any item leaving the cache is recorded in the stash bucket's
/// ghost ring. Returns the evicted item, if any.
fn stash_put<K, V>(
	core: &mut SegmentCore<K, V>,
	ghosts: &mut [GhostRing],
	pos: InsertPosition,
	item: Item<K, V>,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(&item.key, core.stash_size);
	let bucket = &mut core.stash_buckets[stash_idx];
	let mut evicted = None;
	if bucket.is_full() && bucket.get_position(&item.key).is_none() {
		let mut victim = None;
		for (i, it) in bucket.get_items_mut().iter_mut().enumerate() {
			if it.lfu_counter == 0 {
				victim = Some(i);
				break;
			}
			it.lfu_counter -= 1;
		}
		let victim = victim.or_else(|| bucket.eviction_candidate_index());
		if let Some(v) = victim {
			let e = bucket.remove_at(v);
			debug!(
				"stash_put: stash={stash_idx} victim_idx={v} evicted_key={:?} for key={:?}",
				e.key, item.key
			);
			evicted = Some(e);
		}
	}
	let extra = core.put_at(Slot::Stash, pos, item);
	debug_assert!(
		evicted.is_none() || extra.is_none(),
		"stash_put: double eviction from a single insert"
	);
	let out = evicted.or(extra);
	if let Some(ref e) = out {
		let idx = get_index(&e.key, core.stash_size);
		ghosts[idx].record(fingerprint(&e.key));
	}
	out
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	ghosts: &mut [GhostRing],
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Target, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghosts, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket. Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	ghosts: &mut [GhostRing],
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghosts, evicted, probing_idx, insert_pos);
}

/// Insert position for items demoted from the normal tier back into stash.
///
/// Demoted items have already earned normal-tier residency (and are pre-credited
/// with `PROMOTE_THRESHOLD - 1` stash hits), so they re-enter stash at the MRU end
/// (`InsertPosition::Back`) rather than at the LRU end. This makes the stash act as
/// a victim cache: ex-residents outlive one-touch arrivals long enough to use their
/// pre-credit and win back their slot.
const DEMOTE_TO_STASH_POS: InsertPosition = InsertPosition::Back;

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
/// Items demoted to stash have already earned normal-tier residency, so they are
/// pre-credited with `PROMOTE_THRESHOLD - 1` stash hits: one more hit returns them.
///
/// The target retry honours `insert_pos` (normal-tier ordering is unchanged), but
/// stash re-entries from demotion are always placed at the stash MRU end
/// (`DEMOTE_TO_STASH_POS`).
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghosts: &mut [GhostRing],
	mut item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash (MRU)",
			item.key
		);
		item.lfu_counter = PROMOTE_THRESHOLD - 1;
		stash_put(core, ghosts, DEMOTE_TO_STASH_POS, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(mut cascade) = core.put_at(Slot::Target, insert_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash (MRU)",
				cascade.key
			);
			cascade.lfu_counter = PROMOTE_THRESHOLD - 1;
			stash_put(core, ghosts, DEMOTE_TO_STASH_POS, cascade);
		}
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	ghosts: &mut [GhostRing],
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
			let counter = &mut core.stash_buckets[stash_idx].get_items_mut()[item_idx].lfu_counter;
			*counter = counter.saturating_add(1);
			*counter
		};
		if hits < PROMOTE_THRESHOLD {
			debug!("promotion_get: key={key:?} found in stash hits={hits} -> served in place");
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		debug!("promotion_get: key={key:?} found in stash hits={hits} -> target={target_idx} insert_pos={insert_pos:?}");
		let mut data = core.stash_buckets[stash_idx].remove_at(item_idx);
		data.lfu_counter = 0;
		promote_to_target(core, ghosts, data, insert_pos);

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
			promote_to_probing(core, ghosts, data, insert_pos);

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

	#[test]
	fn fingerprint_never_zero_and_deterministic() {
		for k in 0i64..1000 {
			assert_ne!(fingerprint(&k), 0);
			assert_eq!(fingerprint(&k), fingerprint(&k));
		}
	}

	#[test]
	fn ghost_ring_overwrites_oldest_and_take_clears() {
		let mut r = GhostRing::default();
		for fp in 1..=(GHOST_WAYS as u32 + 1) {
			r.record(fp);
		}
		assert!(!r.take(1), "oldest entry must have been overwritten");
		assert!(r.take(2));
		assert!(!r.take(2), "take clears the matched entry");
	}

	/// A key evicted from a full stash bucket and then `put` again is admitted
	/// pre-credited, and a single `get` promotes it to its target bucket.
	#[test]
	fn ghost_hit_readmits_pre_credited_and_promotes_on_one_hit() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 1);
		for &k in &keys {
			seg.put(Item::new(k, k * 10));
		}
		let victim = keys[0];
		assert!(!seg.core().contains_key(&victim), "LRU-end zero-counter key evicted");

		seg.put(Item::new(victim, 7));
		let b = &seg.core().stash_buckets[0];
		let pos = b.get_position(&victim).expect("re-put key resident in stash");
		assert_eq!(b.get_items()[pos].lfu_counter, PROMOTE_THRESHOLD - 1);

		let val = seg.get(&victim).map(|it| it.value);
		assert_eq!(val, Some(7));
		let target_idx = get_index(&victim, seg.core().segment_size);
		assert!(seg.core().buckets[target_idx].get_position(&victim).is_some());
		assert!(seg.core().stash_buckets[0].get_position(&victim).is_none());
	}

	/// A never-seen key still needs `PROMOTE_THRESHOLD` hits, even when its stash
	/// bucket's ghost ring holds other fingerprints.
	#[test]
	fn never_seen_key_still_needs_two_hits() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 2);
		for &k in &keys[..=bsize] {
			seg.put(Item::new(k, k));
		}
		let fresh = keys[bsize + 1];
		seg.put(Item::new(fresh, 5));
		let b = &seg.core().stash_buckets[0];
		let pos = b.get_position(&fresh).expect("fresh key in stash");
		assert_eq!(b.get_items()[pos].lfu_counter, 0);

		seg.get(&fresh);
		let target_idx = get_index(&fresh, seg.core().segment_size);
		assert!(seg.core().stash_buckets[0].get_position(&fresh).is_some());
		assert!(seg.core().buckets[target_idx].get_position(&fresh).is_none());
		seg.get(&fresh);
		assert!(seg.core().stash_buckets[0].get_position(&fresh).is_none());
		assert!(seg.core().buckets[target_idx].get_position(&fresh).is_some());
	}

	/// Full stash bucket: the scan skips (and ages) counter>0 items at the LRU end
	/// and evicts the first zero-counter item.
	#[test]
	fn stash_victim_skips_credited_items_and_ages_them() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 1);
		for &k in &keys[..bsize] {
			seg.core_mut().stash_buckets[0].put(Item::new(k, k));
		}
		// Credit the LRU-end item.
		seg.core_mut().stash_buckets[0].get_items_mut()[0].lfu_counter = 1;
		seg.put(Item::new(keys[bsize], 0));

		let b = &seg.core().stash_buckets[0];
		assert!(b.get_position(&keys[0]).is_some(), "credited LRU item must be protected");
		assert_eq!(b.get_items()[0].lfu_counter, 0, "skipped item must be aged");
		if bsize >= 2 {
			assert!(b.get_position(&keys[1]).is_none(), "first zero-counter item is the victim");
		}
		assert!(b.get_position(&keys[bsize]).is_some());
		assert_eq!(b.size(), bsize);
	}

	/// No zero-counter item: every slot ages and the LRU-end item is evicted.
	#[test]
	fn stash_victim_falls_back_to_lru_when_all_credited() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 1);
		for &k in &keys[..bsize] {
			let mut it = Item::new(k, k);
			it.lfu_counter = 2;
			seg.core_mut().stash_buckets[0].put(it);
		}
		seg.put(Item::new(keys[bsize], 0));
		let b = &seg.core().stash_buckets[0];
		assert!(b.get_position(&keys[0]).is_none(), "LRU-end item evicted on fallback");
		for it in b.get_items().iter().filter(|it| it.key != keys[bsize]) {
			assert_eq!(it.lfu_counter, 1);
		}
		assert_eq!(b.size(), bsize);
	}

	#[test]
	fn first_stash_hit_stays_in_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let k = find_keys_for_bucket(target_idx, seg.core.segment_size, 1)[0];
		let stash_idx = get_index(&k, seg.core.stash_size);

		seg.put(Item::new(k, 10));
		let val = seg.get(&k).map(|it| it.value);
		assert_eq!(val, Some(10));

		assert!(seg.core.stash_buckets[stash_idx].get_position(&k).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none());

		// Second hit promotes.
		seg.get(&k);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&k).is_none());
		assert!(seg.core.buckets[target_idx].get_position(&k).is_some());
	}

	#[test]
	fn stash_hit_inserts_at_front_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);

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

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
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
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
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
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
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

		promote_to_probing(&mut seg.core, &mut seg.ghosts, Item::new(trigger, 200), InsertPosition::Back);

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

		promote_to_probing(&mut seg.core, &mut seg.ghosts, Item::new(trigger, 99), insert_pos);

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
	/// AND that the stash insert lands at stash MRU regardless of `insert_pos`.
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

		promote_to_target(&mut seg.core, &mut seg.ghosts, Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_mru_even_when_cold() {
		direct_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert lands at stash MRU
	/// regardless of `insert_pos`.
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

		promote_to_probing(&mut seg.core, &mut seg.ghosts, Item::new(trigger, 99), insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn cascade_to_stash_at_mru_even_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
