use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment (cycle 0010 v001: cycle 0009 v001 + a 2-bit persistent,
/// aged reuse level chooses the victim of a full normal bucket).
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
/// stash MRU end. The hit count lives in the low bits of `Item::lfu_counter` and is
/// reset to 0 on every stash entry from a normal bucket and on promotion out of the
/// stash.
///
/// Inherited (cycle 0003 v003): a stash item that passed the gate lands at its
/// target's MRU end (`STASH_PROMOTION_TARGET_POS = Back`). Foreign residents displaced
/// by the resulting cascade still retry at the cold `insert_pos`.
///
/// Inherited (cycle 0004 v001): the two-hit gate is skipped when the target bucket has
/// at least `STASH_GATE_BYPASS_FREE_SLOTS` free slots.
///
/// Inherited (cycle 0005 v003): every item evicted out of the cache (a stash overflow)
/// has its key fingerprint recorded in a FIFO ghost ring (`GhostRing`, ARC/2Q "A1out").
/// A fresh `put` whose key is found in the ghost removes the fingerprint and enters the
/// stash with its hit count at `STASH_PROMOTION_HITS - 1`, so its first stash hit
/// promotes it.
///
/// Inherited (cycle 0006 v001): an item promoted from its target into its probing bucket
/// lands at the probing MRU end (`PROBING_PROMOTION_POS = Back`).
///
/// Inherited (cycle 0007 v002): with `RESCUE_STASH_VICTIMS`, a stash-overflow victim is
/// first placed into a free slot of its own target bucket (else its probing bucket) at
/// the LRU end (`Front`); only when both are full is it evicted out of the cache and
/// recorded in the ghost.
///
/// Inherited (cycle 0008 v001): inside a normal bucket the low bits of `lfu_counter`
/// are a 1-bit reference flag: cleared on entry into a normal bucket, set on a hit.
///
/// Inherited (cycle 0009 v001): a ghost-hit `put` sets `GHOST_ORIGIN_FLAG`; a flagged
/// stash promotee enters its target with reference bit 1 (`CARRY_GHOST_HISTORY`).
///
/// New (cycle 0010): `lfu_counter` bits `LEVEL_SHIFT..LEVEL_SHIFT+2` hold a reuse level
/// (0..=`LEVEL_MAX`). It is raised (saturating) on every hit — stash, target and
/// probing — and is preserved across every move inside the cache (stash entry, rescue,
/// stash→target and target→probing promotion, foreign retry); it is lost only when the
/// item leaves the cache. With `PERSISTENT_REUSE_LEVEL`, `make_room_clock` evicts the
/// lowest-level item of a full normal bucket (ties: prefer reference bit 0, then the
/// lower/LRU index) and ages every other item (level − 1, reference bit cleared), so
/// no item stays pinned for more than `LEVEL_MAX` contested promotions without hits.
///
/// Metadata cost: the ghost holds `GHOST_FACTOR * num_of_stash_buckets * bucket_size`
/// `u64` fingerprints per segment. The level reuses spare bits of `lfu_counter`.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	ghost: GhostRing,
}

/// Ghost ring capacity, as a multiple of the segment's total stash slots.
const GHOST_FACTOR: usize = 2;

/// When set, a stash-overflow victim is placed into a free slot of its own target
/// (else its probing) bucket at the LRU end, and is evicted from the cache only when
/// both are full.
const RESCUE_STASH_VICTIMS: bool = true;

/// When set, the victim leaving a full normal bucket on a promotion is chosen by
/// `make_room_clock` instead of plain LRU.
const NORMAL_VICTIM_SECOND_CHANCE: bool = true;

/// High bit of `Item::lfu_counter` (`usize`) marking a stash item that entered via a
/// ghost hit. Only meaningful in the stash; masked off for hit counting.
const GHOST_ORIGIN_FLAG: usize = 1 << (usize::BITS - 1);

/// When set, a ghost-origin item promoted from the stash enters its target with the
/// reference bit set (1) instead of clear (0).
const CARRY_GHOST_HISTORY: bool = true;

/// Low bits of `lfu_counter`: the stash hit count, or the reference bit in a normal
/// bucket.
const LOW_MASK: usize = 0xFFFF;

/// Reuse-level field of `lfu_counter`.
const LEVEL_SHIFT: u32 = 16;
const LEVEL_MASK: usize = 0b11 << LEVEL_SHIFT;
const LEVEL_MAX: usize = 3;

/// When set, the persistent aged reuse level chooses normal-bucket victims; when clear,
/// the cycle 0009 v001 CLOCK scan is used.
const PERSISTENT_REUSE_LEVEL: bool = true;

/// Reuse level stored in `c`.
fn level(c: usize) -> usize {
	(c & LEVEL_MASK) >> LEVEL_SHIFT
}

/// `c` with its level field replaced by `l` (clamped to `LEVEL_MAX`); all other bits of
/// `c` (low bits, ghost flag) are kept.
fn with_level(c: usize, l: usize) -> usize {
	(c & !LEVEL_MASK) | (l.min(LEVEL_MAX) << LEVEL_SHIFT)
}

/// Low bits of `c` (stash hit count or reference bit).
fn low(c: usize) -> usize {
	c & LOW_MASK
}

/// `c` with its level raised by one (saturating at `LEVEL_MAX`) when
/// `PERSISTENT_REUSE_LEVEL` is set.
fn raise_level(c: usize) -> usize {
	if PERSISTENT_REUSE_LEVEL {
		with_level(c, (level(c) + 1).min(LEVEL_MAX))
	} else {
		c
	}
}

/// FIFO ring of 64-bit fingerprints of keys recently evicted out of the segment.
/// Overwrite-on-wrap is the decay: a fingerprint survives about
/// `GHOST_FACTOR × stash slots` subsequent evictions. `0` marks an empty slot
/// (fingerprints are forced non-zero).
#[derive(Debug)]
struct GhostRing {
	slots: Vec<u64>,
	head: usize,
}

impl GhostRing {
	fn new(len: usize) -> Self {
		Self {
			slots: vec![0; len.max(1)],
			head: 0,
		}
	}

	/// Writes `fp` at `head` (overwriting the oldest entry) and advances `head`.
	fn insert(&mut self, fp: u64) {
		self.slots[self.head] = fp;
		self.head = (self.head + 1) % self.slots.len();
	}

	/// Linear scan for `fp`.
	#[cfg(test)]
	fn contains(&self, fp: u64) -> bool {
		self.slots.iter().any(|&s| s == fp)
	}

	/// Zeroes the first slot matching `fp`; returns whether one was found.
	fn remove(&mut self, fp: u64) -> bool {
		if let Some(slot) = self.slots.iter_mut().find(|s| **s == fp) {
			*slot = 0;
			true
		} else {
			false
		}
	}
}

/// Non-zero 64-bit fingerprint of `key` (the hasher behind `get_index`).
fn fingerprint<K: Hash + Debug>(key: &K) -> u64 {
	(hash(key) as u64) | 1
}

/// Stash insert. Any item the stash evicts is rescued into a free slot of its own
/// target (else probing) bucket at the LRU end when `RESCUE_STASH_VICTIMS` is set;
/// otherwise (or when both are full) it leaves the cache and is recorded in the ghost.
fn put_stash_recording<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
	pos: InsertPosition,
	item: Item<K, V>,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let Some(mut evicted) = core.put_at(Slot::Stash, pos, item) else {
		return;
	};
	if RESCUE_STASH_VICTIMS {
		let t = get_index(&evicted.key, core.segment_size);
		let p = (t + 1) % core.segment_size;
		let target_room = core.buckets[t].get_items().len() < core.buckets[t].get_max_size();
		let probing_room = core.buckets[p].get_items().len() < core.buckets[p].get_max_size();
		if target_room {
			debug!("put_stash_recording: evicted key={:?} -> rescued into target={t}", evicted.key);
			evicted.lfu_counter = with_level(0, level(evicted.lfu_counter));
			let ret = core.put_at(Slot::Target, InsertPosition::Front, evicted);
			debug_assert!(ret.is_none(), "rescue into a free target slot must not evict");
			return;
		}
		if probing_room {
			debug!("put_stash_recording: evicted key={:?} -> rescued into probing={p}", evicted.key);
			evicted.lfu_counter = with_level(0, level(evicted.lfu_counter));
			let ret = core.put_at(Slot::Probing, InsertPosition::Front, evicted);
			debug_assert!(ret.is_none(), "rescue into a free probing slot must not evict");
			return;
		}
	}
	debug!("put_stash_recording: evicted key={:?} -> ghost", evicted.key);
	ghost.insert(fingerprint(&evicted.key));
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
			let ghost_len = GHOST_FACTOR * settings.num_of_stash_buckets * settings.bucket_size;
			Some(Self {
				core,
				ghost: GhostRing::new(ghost_len),
			})
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
		promotion_get(&mut self.core, &mut self.ghost, key, InsertPosition::Front)
	}

	/// Same placement as the default `put` (stash, `Back`), but a key whose fingerprint
	/// is in the ghost enters with its stash-hit counter pre-credited and the
	/// ghost-origin flag set, and any stash eviction is rescued or recorded in the ghost.
	/// A newly put item always starts at reuse level 0.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let ghost_hit = self.ghost.remove(fingerprint(&item.key));
		item.lfu_counter = if ghost_hit {
			(STASH_PROMOTION_HITS - 1) | GHOST_ORIGIN_FLAG
		} else {
			0
		};
		debug!("DashSegment.put: key={:?} ghost_hit={ghost_hit}", item.key);
		put_stash_recording(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────

/// Position at which items demoted out of a normal bucket land in their stash bucket.
const STASH_DEMOTION_POS: InsertPosition = InsertPosition::Back;

/// Number of hits during a single stash stay required before a stash item is
/// promoted to its target bucket — unless the target has at least
/// `STASH_GATE_BYPASS_FREE_SLOTS` free slots, in which case the first hit promotes.
const STASH_PROMOTION_HITS: usize = 2;

/// Minimum number of free slots in the target bucket for a stash item to promote on
/// its first hit.
const STASH_GATE_BYPASS_FREE_SLOTS: usize = 2;

/// Where a stash item promoted to its target lands.
const STASH_PROMOTION_TARGET_POS: InsertPosition = InsertPosition::Back;

/// Where an item promoted from its target into its probing bucket lands.
const PROBING_PROMOTION_POS: InsertPosition = InsertPosition::Back;

/// Frees one slot of a full normal bucket before a promotion into it. No-op when the
/// bucket is not full.
///
/// With `PERSISTENT_REUSE_LEVEL`: the victim is the item with the lowest reuse level;
/// among equal minimal levels an item with reference bit 0 is preferred, then the lower
/// (LRU-side) index. Every other item is aged (level − 1 saturating, reference bit
/// cleared). A victim is always removed and routed through `demote_from_normal_bucket`.
///
/// Without it: the cycle 0009 v001 CLOCK scan (first clear reference bit from LRU,
/// clearing passed bits; all referenced → nothing removed, LRU fallback by the caller).
fn make_room_clock<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
	bucket_idx: usize,
	retry_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	if !NORMAL_VICTIM_SECOND_CHANCE || !core.buckets[bucket_idx].is_full() {
		return;
	}
	let victim = if PERSISTENT_REUSE_LEVEL {
		let items = core.buckets[bucket_idx].get_items_mut();
		let min_level = items
			.iter()
			.map(|it| level(it.lfu_counter))
			.min()
			.expect("make_room_clock: full bucket is non-empty");
		let j = items
			.iter()
			.position(|it| level(it.lfu_counter) == min_level && low(it.lfu_counter) == 0)
			.or_else(|| items.iter().position(|it| level(it.lfu_counter) == min_level))
			.expect("make_room_clock: a minimal-level item exists");
		for (i, it) in items.iter_mut().enumerate() {
			if i != j {
				it.lfu_counter = with_level(0, level(it.lfu_counter).saturating_sub(1));
			}
		}
		Some(j)
	} else {
		let mut victim = None;
		for (j, it) in core.buckets[bucket_idx].get_items_mut().iter_mut().enumerate() {
			if low(it.lfu_counter) == 0 {
				victim = Some(j);
				break;
			}
			it.lfu_counter = with_level(0, level(it.lfu_counter));
		}
		victim
	};
	let Some(j) = victim else {
		debug!("make_room_clock: bucket={bucket_idx} all referenced -> bits cleared, LRU fallback");
		return;
	};
	let removed = core.buckets[bucket_idx].remove_at(j);
	debug!(
		"make_room_clock: bucket={bucket_idx} victim key={:?} at index={j} level={}",
		removed.key,
		level(removed.lfu_counter)
	);
	debug_assert!(
		!core.buckets[bucket_idx].is_full(),
		"make_room_clock: bucket={bucket_idx} must have a free slot after removing the victim"
	);
	demote_from_normal_bucket(core, ghost, removed, bucket_idx, retry_pos, STASH_DEMOTION_POS);
}

/// Moves `item` into its target bucket at `place_pos`. Any evicted item flows through
/// the tier cascade; foreign residents displaced by it retry their own target at
/// `retry_pos`. Does not touch `item.lfu_counter`.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
	item: Item<K, V>,
	place_pos: InsertPosition,
	retry_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	make_room_clock(core, ghost, target_idx, retry_pos);
	let Some(evicted) = core.put_at(Slot::Target, place_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, evicted, target_idx, retry_pos, STASH_DEMOTION_POS);
}

/// Moves `item` into its probing bucket at `place_pos` (reference bit cleared, level
/// kept). Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
	mut item: Item<K, V>,
	place_pos: InsertPosition,
	retry_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	make_room_clock(core, ghost, probing_idx, retry_pos);
	item.lfu_counter = with_level(0, level(item.lfu_counter));
	let Some(evicted) = core.put_at(Slot::Probing, place_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, evicted, probing_idx, retry_pos, STASH_DEMOTION_POS);
}

/// One-more-chance demotion. A foreign resident retries its own true target (at
/// `retry_pos`) before stash; an owned item goes straight to stash. Every entry resets
/// the low bits (and the ghost flag) but keeps the reuse level.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
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
		item.lfu_counter = with_level(0, level(item.lfu_counter));
		put_stash_recording(core, ghost, stash_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		item.lfu_counter = with_level(0, level(item.lfu_counter));
		if let Some(mut cascade) = core.put_at(Slot::Target, retry_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			cascade.lfu_counter = with_level(0, level(cascade.lfu_counter));
			put_stash_recording(core, ghost, stash_pos, cascade);
		}
	}
}

/// Stash items promote on their `STASH_PROMOTION_HITS`-th hit in the current stay, or
/// on any hit when the target bucket has at least `STASH_GATE_BYPASS_FREE_SLOTS` free
/// slots. Every hit (stash, target, probing) raises the reuse level.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	ghost: &mut GhostRing,
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
			let flag = item.lfu_counter & GHOST_ORIGIN_FLAG;
			let lvl = level(item.lfu_counter);
			let count = low(item.lfu_counter).saturating_add(1).min(LOW_MASK);
			item.lfu_counter = raise_level(with_level(count | flag, lvl));
			count
		};
		let target_capacity = core.buckets[target_idx].get_max_size();
		let free = target_capacity.saturating_sub(core.buckets[target_idx].get_items().len());
		let target_has_room = free >= STASH_GATE_BYPASS_FREE_SLOTS;
		if hits < STASH_PROMOTION_HITS && !target_has_room {
			debug!(
				"promotion_get: key={key:?} found in stash hits={hits} free={free} -> refresh in stash={stash_idx}"
			);
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		{
			let item = &mut core.stash_buckets[stash_idx].get_items_mut()[item_idx];
			let flagged = item.lfu_counter & GHOST_ORIGIN_FLAG != 0;
			let bit = if flagged && CARRY_GHOST_HISTORY { 1 } else { 0 };
			item.lfu_counter = with_level(bit, level(item.lfu_counter));
			debug!(
				"promotion_get: key={key:?} found in stash hits={hits} free={free} ghost_origin={flagged} -> target={target_idx} place_pos={STASH_PROMOTION_TARGET_POS:?} retry_pos={insert_pos:?}"
			);
		}
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, ghost, data, STASH_PROMOTION_TARGET_POS, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target or cascade it elsewhere via demote_from_normal_bucket")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			debug!(
				"promotion_get: key={key:?} found in target as hottest -> probing={probing_idx} place_pos={PROBING_PROMOTION_POS:?} retry_pos={insert_pos:?}"
			);
			{
				let item = &mut core.buckets[target_idx].get_items_mut()[item_idx];
				item.lfu_counter = raise_level(item.lfu_counter);
			}
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, ghost, data, PROBING_PROMOTION_POS, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
			let item = &mut core.buckets[target_idx].get_items_mut()[item_idx];
			item.lfu_counter = raise_level(with_level(1, level(item.lfu_counter)));
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		let item = &mut core.buckets[probing_idx].get_items_mut()[item_idx];
		item.lfu_counter = raise_level(with_level(1, level(item.lfu_counter)));
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

	/// Fills an initially empty target bucket (2 slots) with `a` then `b`, both via
	/// the public put/get path: `a` promotes on its first hit (empty target → bypass),
	/// `b` needs two hits (1 free slot → gated). Result: target is `[a, b]`, `b` at MRU.
	fn fill_target_via_stash(seg: &mut DashSegment<i64, i64>, a: i64, b: i64) {
		let target_idx = get_index(&a, seg.core.segment_size);
		assert_eq!(target_idx, get_index(&b, seg.core.segment_size), "sanity: same target");
		seg.put(Item::new(a, a * 10));
		seg.get(&a);
		assert!(seg.core.buckets[target_idx].get_position(&a).is_some(), "bypass promotes a");
		seg.put(Item::new(b, b * 10));
		seg.get(&b);
		assert!(
			seg.core.buckets[target_idx].get_position(&b).is_none(),
			"one free slot: first hit must stay in stash"
		);
		seg.get(&b);
		assert!(seg.core.buckets[target_idx].get_position(&b).is_some(), "second hit promotes b");
	}

	/// First `count` keys with normal-bucket target `bucket_idx` that are not in `exclude`.
	fn keys_for_target_excluding(
		bucket_idx: usize,
		seg_size: usize,
		count: usize,
		exclude: &[i64],
	) -> Vec<i64> {
		let keys: Vec<i64> = find_keys_for_bucket(bucket_idx, seg_size, 200)
			.into_iter()
			.filter(|k| !exclude.contains(k))
			.take(count)
			.collect();
		assert_eq!(keys.len(), count, "not enough keys for target {bucket_idx}");
		keys
	}

	fn counter_of(seg: &DashSegment<i64, i64>, bucket: usize, key: i64) -> usize {
		let items = seg.core.buckets[bucket].get_items();
		items.iter().find(|it| it.key == key).expect("key must be in bucket").lfu_counter
	}

	#[test]
	fn level_helpers_round_trip() {
		let c = with_level(5 | GHOST_ORIGIN_FLAG, 2);
		assert_eq!(level(c), 2);
		assert_eq!(low(c), 5);
		assert_ne!(c & GHOST_ORIGIN_FLAG, 0);
		assert_eq!(level(with_level(c, 9)), LEVEL_MAX);
		assert_eq!(level(raise_level(with_level(0, LEVEL_MAX))), LEVEL_MAX);
		assert_eq!(level(raise_level(0)), 1);
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
		let seg_size = seg.core.segment_size;
		let stash_idx = 0;
		let target_idx = 0;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 200);
		let k0 = *target_keys
			.iter()
			.find(|k| get_index(*k, stash_size) == stash_idx)
			.expect("a key with target 0 and stash 0 must exist");
		let filler = *target_keys.iter().find(|k| **k != k0).expect("a filler key must exist");
		let other = find_keys_for_bucket(stash_idx, stash_size, 200)
			.into_iter()
			.find(|k| *k != k0 && *k != filler)
			.expect("a second stash key must exist");

		seg.core.buckets[target_idx].put(Item::new(filler, 1));

		seg.put(Item::new(k0, 10));
		seg.put(Item::new(other, 20));

		let val = seg.get(&k0).expect("hit must return Some").value;
		assert_eq!(val, 10);

		let items = seg.core.stash_buckets[stash_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == k0)
			.expect("key must remain in its stash bucket after a single hit");
		assert_eq!(pos, items.len() - 1, "single stash hit must refresh to the MRU end");
		assert_eq!(level(items[pos].lfu_counter), 1, "a stash hit raises the level");
		assert!(seg.core.buckets[target_idx].get_position(&k0).is_none());

		seg.get(&k0);
		assert!(seg.core.buckets[target_idx].get_position(&k0).is_some());
		assert!(seg.core.stash_buckets[stash_idx].get_position(&k0).is_none());
	}

	fn ghost_fixture(seg: &DashSegment<i64, i64>) -> (i64, i64, Vec<i64>) {
		let stash_size = seg.core.stash_size;
		let seg_size = seg.core.segment_size;
		let target_keys = find_keys_for_bucket(0, seg_size, 200);
		let k = *target_keys
			.iter()
			.find(|k| get_index(*k, stash_size) == 0)
			.expect("a key with target 0 and stash 0 must exist");
		let filler = *target_keys.iter().find(|x| **x != k).expect("a filler key must exist");
		let others: Vec<i64> = find_keys_for_bucket(0, stash_size, 200)
			.into_iter()
			.filter(|x| *x != k && *x != filler)
			.collect();
		(k, filler, others)
	}

	fn ghost_returning_key(seg: &mut DashSegment<i64, i64>) -> i64 {
		let seg_size = seg.core.segment_size;
		let (k, filler, others) = ghost_fixture(seg);
		let target_idx = 0;
		let probing_idx = 1;

		let mut exclude = others.clone();
		exclude.push(k);
		exclude.push(filler);
		let filler2 = keys_for_target_excluding(target_idx, seg_size, 1, &exclude)[0];
		exclude.push(filler2);
		let probing_fillers = keys_for_target_excluding(probing_idx, seg_size, 2, &exclude);

		seg.core.buckets[target_idx].put(Item::new(filler, 1));
		seg.core.buckets[target_idx].put(Item::new(filler2, 2));
		for &pf in &probing_fillers {
			seg.core.buckets[probing_idx].put(Item::new(pf, 3));
		}
		seg.put(Item::new(k, 10));
		for &o in &others {
			if !seg.core.contains_key(&k) {
				break;
			}
			seg.put(Item::new(o, o * 10));
		}
		assert!(!seg.core.contains_key(&k), "k must have been evicted out of the stash");
		assert!(seg.ghost.contains(fingerprint(&k)), "eviction must record k in the ghost");

		seg.put(Item::new(k, 11));
		assert!(!seg.ghost.contains(fingerprint(&k)), "ghost hit must consume the fingerprint");
		k
	}

	#[test]
	fn ghost_put_promotes_on_first_stash_hit() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let k = ghost_returning_key(&mut seg);
		let target_idx = 0;

		let val = seg.get(&k).expect("hit must return Some").value;
		assert_eq!(val, 11);
		let pos = seg.core.buckets[target_idx]
			.get_position(&k)
			.expect("ghost-returning key must promote on its first stash hit");
		let c = seg.core.buckets[target_idx].get_items()[pos].lfu_counter;
		assert_eq!(low(c), 1, "ghost-origin promotee must enter its target with reference bit 1");
		assert_eq!(c & GHOST_ORIGIN_FLAG, 0, "flag must not reach the target");
		assert!(seg.core.stash_buckets[0].get_position(&k).is_none());
		assert!(seg.core.check_slot_legality());
	}

	#[test]
	fn ghost_flag_does_not_change_gate() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let k = ghost_returning_key(&mut seg);
		let target_idx = 0;

		let stash_items = seg.core.stash_buckets[0].get_items();
		let it = stash_items.iter().find(|it| it.key == k).expect("k must be in stash 0");
		assert_ne!(it.lfu_counter & GHOST_ORIGIN_FLAG, 0, "ghost hit must set the flag");
		assert_eq!(low(it.lfu_counter), 1, "masked stash count must read 1");
		assert!(seg.core.buckets[target_idx].is_full(), "sanity: bypass is off");

		seg.get(&k);
		assert!(
			seg.core.buckets[target_idx].get_position(&k).is_some(),
			"first hit must promote a ghost-credited key"
		);
		for b in seg.core.stash_buckets.iter() {
			for it in b.get_items() {
				assert_eq!(it.lfu_counter & GHOST_ORIGIN_FLAG, 0, "flag must not remain on {:?}", it.key);
			}
		}
	}

	#[test]
	fn fresh_promotee_enters_with_bit_zero() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		fill_target_via_stash(&mut seg, keys[0], keys[1]);
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[1].key, keys[1]);
		assert_eq!(low(items[1].lfu_counter), 0, "fresh two-hit promotee must enter with bit 0");
		assert_eq!(low(items[0].lfu_counter), 0, "fresh bypass promotee must enter with bit 0");
		assert_eq!(level(items[1].lfu_counter), 2, "two stash hits carry level 2");
		assert_eq!(level(items[0].lfu_counter), 1, "one stash hit carries level 1");
	}

	#[test]
	fn stash_victim_rescued_into_free_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let seg_size = seg.core.segment_size;
		let keys = find_keys_for_bucket(0, stash_size, 3);
		assert_eq!(keys.len(), 3);
		let victim = keys[0];
		let t = get_index(&victim, seg_size);

		for &k in &keys {
			seg.put(Item::new(k, k * 10));
		}
		assert!(seg.core.stash_buckets[0].get_position(&victim).is_none(), "victim left the stash");
		let items = seg.core.buckets[t].get_items();
		assert_eq!(items[0].key, victim, "victim must be rescued into its target at index 0");
		assert!(!seg.ghost.contains(fingerprint(&victim)), "rescued victim must not be in the ghost");
		assert!(seg.core.check_slot_legality());
		let val = seg.get(&victim).expect("rescued victim must be found").value;
		assert_eq!(val, victim * 10);
	}

	#[test]
	fn stash_victim_evicted_when_target_and_probing_full() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let stash_size = seg.core.stash_size;
		let seg_size = seg.core.segment_size;
		let stash_keys = find_keys_for_bucket(0, stash_size, 200);
		let victim = stash_keys[0];
		let t = get_index(&victim, seg_size);
		let p = (t + 1) % seg_size;

		let target_fillers = keys_for_target_excluding(t, seg_size, 2, &stash_keys);
		let probing_fillers = keys_for_target_excluding(p, seg_size, 2, &stash_keys);
		for &f in &target_fillers {
			seg.core.buckets[t].put(Item::new(f, 1));
		}
		for &f in &probing_fillers {
			seg.core.buckets[p].put(Item::new(f, 2));
		}
		assert!(seg.core.buckets[t].is_full() && seg.core.buckets[p].is_full());

		for &k in &stash_keys[..3] {
			seg.put(Item::new(k, k * 10));
		}
		assert!(!seg.core.contains_key(&victim), "victim must be evicted out of the cache");
		assert!(seg.ghost.contains(fingerprint(&victim)), "eviction must record the victim in the ghost");
		for &f in target_fillers.iter().chain(probing_fillers.iter()) {
			assert!(seg.core.contains_key(&f), "rescue must never displace a resident");
		}
	}

	#[test]
	fn non_ghost_put_still_needs_two_hits() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (k, filler, others) = ghost_fixture(&seg);
		let target_idx = 0;

		seg.core.buckets[target_idx].put(Item::new(filler, 1));
		seg.put(Item::new(k, 10));
		seg.put(Item::new(others[0], 20));
		assert!(!seg.ghost.contains(fingerprint(&k)));

		seg.get(&k);
		let items = seg.core.stash_buckets[0].get_items();
		let pos = items.iter().position(|it| it.key == k).expect("k must stay in stash");
		assert_eq!(pos, items.len() - 1, "single stash hit must refresh to the MRU end");
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none());

		seg.get(&k);
		assert!(seg.core.buckets[target_idx].get_position(&k).is_some());
		assert!(seg.core.stash_buckets[0].get_position(&k).is_none());
	}

	#[test]
	fn ghost_ring_overwrites_oldest() {
		let mut g = GhostRing::new(2);
		g.insert(3);
		g.insert(5);
		g.insert(7);
		assert!(!g.contains(3));
		assert!(g.contains(5) && g.contains(7));
		assert!(g.remove(5));
		assert!(!g.contains(5));
		assert!(!g.remove(5));
	}

	#[test]
	fn stash_hit_bypasses_gate_when_target_has_two_free_slots() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let key = 1_i64;
		let target_idx = get_index(&key, seg.core.segment_size);
		let stash_idx = get_index(&key, seg.core.stash_size);
		seg.put(Item::new(key, 10));

		let val = seg.get(&key).expect("hit must return Some").value;
		assert_eq!(val, 10);
		assert!(seg.core.buckets[target_idx].get_position(&key).is_some());
		assert!(seg.core.stash_buckets[stash_idx].get_position(&key).is_none());
	}

	#[test]
	fn stash_hit_inserts_at_back_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[0]);
		assert_eq!(items[1].key, keys[1]);
	}

	#[test]
	fn target_hit_not_hottest_promotes_within_bucket() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		seg.get(&keys[0]);
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
		assert_eq!(low(items[1].lfu_counter), 1, "a resident hit must set the reference bit");
		assert_eq!(level(items[1].lfu_counter), 2, "a resident hit raises the level");
		assert_eq!(low(items[0].lfu_counter), 0);
	}

	/// With target `[a(ref=1), b(ref=0)]` at equal level, promoting a third key spares the
	/// referenced `a` (clearing its bit) and demotes the unreferenced `b` to stash.
	#[test]
	fn clock_spares_referenced_lru_resident() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let (a, b, c) = (keys[0], keys[1], keys[2]);

		fill_target_via_stash(&mut seg, a, b);
		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			assert_eq!(items[0].key, a);
			assert_eq!(items[1].key, b);
			items[0].lfu_counter = 1;
			items[1].lfu_counter = 0;
		}

		seg.put(Item::new(c, c * 10));
		seg.get(&c);
		seg.get(&c);
		assert!(seg.core.buckets[target_idx].get_position(&c).is_some(), "c promoted");

		let items = seg.core.buckets[target_idx].get_items();
		let a_pos = items.iter().position(|it| it.key == a).expect("referenced a must be spared");
		assert_eq!(low(items[a_pos].lfu_counter), 0, "spared item's bit is cleared");
		assert!(seg.core.buckets[target_idx].get_position(&b).is_none(), "b must leave the target");
		let b_stash = get_index(&b, seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[b_stash].get_position(&b).is_some(),
			"unreferenced b must be demoted to its stash"
		);
		assert!(seg.core.check_slot_legality());
	}

	/// When every resident has the same level and is referenced, the LRU resident is the
	/// victim and the other's bit is cleared.
	#[test]
	fn clock_falls_back_to_lru_when_all_referenced() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let (a, b, c) = (keys[0], keys[1], keys[2]);

		fill_target_via_stash(&mut seg, a, b);
		for it in seg.core.buckets[target_idx].get_items_mut().iter_mut() {
			it.lfu_counter = 1;
		}

		seg.put(Item::new(c, c * 10));
		seg.get(&c);
		seg.get(&c);

		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items.len(), 2);
		assert_eq!(items[0].key, b, "b survives, now at LRU");
		assert_eq!(low(items[0].lfu_counter), 0, "passed-over bit is cleared");
		assert_eq!(items[1].key, c);
		let a_stash = get_index(&a, seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[a_stash].get_position(&a).is_some(),
			"LRU a must be demoted to its stash"
		);
	}

	/// Target `[a(level 3), b(level 0)]` with `b` at MRU: promoting `c` evicts `b`, and
	/// `a`'s level drops to 2.
	#[test]
	fn min_level_victim_spares_hot_mru_neighbour() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let (a, b, c) = (keys[0], keys[1], keys[2]);

		seg.core.buckets[target_idx].put(Item::new(a, 1));
		seg.core.buckets[target_idx].put(Item::new(b, 2));
		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			assert_eq!(items[0].key, a);
			assert_eq!(items[1].key, b);
			items[0].lfu_counter = with_level(0, 3);
			items[1].lfu_counter = 0;
		}

		promote_to_target(&mut seg.core, &mut seg.ghost, Item::new(c, 3), InsertPosition::Back, InsertPosition::Front);

		assert!(seg.core.buckets[target_idx].get_position(&b).is_none(), "b must be the victim");
		let b_stash = get_index(&b, seg.core.stash_size);
		assert!(seg.core.stash_buckets[b_stash].get_position(&b).is_some(), "b demoted to stash");
		assert!(seg.core.buckets[target_idx].get_position(&c).is_some(), "c promoted");
		assert_eq!(level(counter_of(&seg, target_idx, a)), 2, "a is aged by one");
		assert!(seg.core.check_slot_legality());
	}

	/// A level-3 item survives exactly `LEVEL_MAX` contested promotions without hits and
	/// is the victim of the next one.
	#[test]
	fn aging_unpins_after_level_max_contests() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3 + LEVEL_MAX);
		let a = keys[0];

		seg.core.buckets[target_idx].put(Item::new(a, 1));
		seg.core.buckets[target_idx].put(Item::new(keys[1], 2));
		seg.core.buckets[target_idx].get_items_mut()[0].lfu_counter = with_level(0, LEVEL_MAX);

		for round in 0..=LEVEL_MAX {
			for sb in seg.core.stash_buckets.iter_mut() {
				sb.get_items_mut().clear();
			}
			let newcomer = keys[2 + round];
			promote_to_target(
				&mut seg.core,
				&mut seg.ghost,
				Item::new(newcomer, 9),
				InsertPosition::Back,
				InsertPosition::Front,
			);
			if round < LEVEL_MAX {
				assert!(
					seg.core.buckets[target_idx].get_position(&a).is_some(),
					"a must survive contest {round}"
				);
				assert_eq!(level(counter_of(&seg, target_idx, a)), LEVEL_MAX - 1 - round);
			} else {
				assert!(
					seg.core.buckets[target_idx].get_position(&a).is_none(),
					"a must be evicted after LEVEL_MAX contests"
				);
			}
			assert!(seg.core.buckets[target_idx].get_position(&newcomer).is_some());
		}
	}

	/// A level-2 key demoted to stash keeps level 2 there, and comes back to its target
	/// carrying the level raised by its stash hits.
	#[test]
	fn level_survives_stash_round_trip() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let (k, x, trigger) = (keys[0], keys[1], keys[2]);

		seg.core.buckets[target_idx].put(Item::new(k, 10));
		seg.core.buckets[target_idx].put(Item::new(x, 20));
		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			items[0].lfu_counter = with_level(0, 2);
			items[1].lfu_counter = with_level(0, 3);
		}

		promote_to_target(&mut seg.core, &mut seg.ghost, Item::new(trigger, 30), InsertPosition::Back, InsertPosition::Front);
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none(), "k is the min-level victim");
		let k_stash = get_index(&k, seg.core.stash_size);
		let pos = seg.core.stash_buckets[k_stash].get_position(&k).expect("k demoted to stash");
		let c = seg.core.stash_buckets[k_stash].get_items()[pos].lfu_counter;
		assert_eq!(level(c), 2, "level survives stash entry");
		assert_eq!(low(c), 0, "stash count reset on entry");

		// Target full: two stash hits promote k back.
		seg.get(&k);
		seg.get(&k);
		let pos = seg.core.buckets[target_idx].get_position(&k).expect("k promoted back");
		let c = seg.core.buckets[target_idx].get_items()[pos].lfu_counter;
		assert_eq!(level(c), LEVEL_MAX, "level 2 plus stash hits, saturated");
		assert_eq!(low(c), 0);
		assert!(seg.core.check_slot_legality());
	}

	#[test]
	fn target_hit_hottest_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
		let c = counter_of(&seg, probing_idx, keys[1]);
		assert_eq!(level(c), LEVEL_MAX, "two stash hits + hottest hit carried into probing");
		assert_eq!(low(c), 0);
	}

	#[test]
	fn target_hottest_promotion_lands_at_probing_mru() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		let probing_filler = find_keys_for_bucket(probing_idx, seg.core.segment_size, 1)[0];

		seg.core.buckets[probing_idx].put(Item::new(probing_filler, 7));
		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		seg.get(&keys[1]);

		let items = seg.core.buckets[probing_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == keys[1])
			.expect("promoted key must be in its probing bucket");
		assert_eq!(pos, items.len() - 1, "probing promotion must land at the MRU end");
		assert!(
			items.iter().any(|it| it.key == probing_filler),
			"probing filler must still be resident (bucket had room)"
		);
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
		let pos = seg.core.buckets[probing_idx].get_position(&keys[0]).expect("still in probing");
		let c = seg.core.buckets[probing_idx].get_items()[pos].lfu_counter;
		assert_eq!(low(c), 1, "a probing hit must set the reference bit");
		assert_eq!(level(c), 1, "a probing hit raises the level");
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

		fill_target_via_stash(&mut seg, target_keys[0], target_keys[1]);
		assert!(seg.core.buckets[target_idx].is_full());

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		seg.get(&target_keys[1]);
		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[1]).is_some());

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

		fill_target_via_stash(&mut seg, keys[0], keys[1]);
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
		assert!(seg.core().buckets[target_idx].get_position(&k).is_none());

		promote_to_probing(
			&mut seg.core,
			&mut seg.ghost,
			Item::new(trigger, 200),
			InsertPosition::Back,
			InsertPosition::Back,
		);

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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos, insert_pos);

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should be in true target after retry");
		assert_eq!(pos, expected_idx);
		assert_eq!(low(items[pos].lfu_counter), 0, "retried foreign resident enters with bit 0");
	}

	#[test]
	fn target_retry_lands_at_front_when_cold() {
		target_retry_position(InsertPosition::Front, 0);
	}

	#[test]
	fn target_retry_lands_at_back_when_hot() {
		target_retry_position(InsertPosition::Back, 1);
	}

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

		promote_to_target(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_front_when_cold() {
		direct_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

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
		let foreign = target_keys[2];
		let trigger = target_keys[3];
		let probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn cascade_to_stash_at_front_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
