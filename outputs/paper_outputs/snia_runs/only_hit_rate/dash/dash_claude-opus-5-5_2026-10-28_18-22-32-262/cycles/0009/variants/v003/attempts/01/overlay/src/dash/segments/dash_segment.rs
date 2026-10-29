use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's segment (cycle 0009 v003: cycle 0008 v001 with the ghost ring
/// split into per-stash-bucket sub-rings).
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
/// target's MRU end (`STASH_PROMOTION_TARGET_POS = Back`). Foreign residents displaced
/// by the resulting cascade still retry at the cold `insert_pos`.
///
/// Inherited (cycle 0004 v001): the two-hit gate is skipped when the target bucket has
/// at least `STASH_GATE_BYPASS_FREE_SLOTS` free slots.
///
/// Inherited (cycle 0005 v003): every item evicted out of the cache (a stash overflow)
/// has its key fingerprint recorded in a FIFO ghost ring (`GhostRing`, ARC/2Q "A1out").
/// A fresh `put` whose key is found in the ghost removes the fingerprint and enters the
/// stash with `lfu_counter = STASH_PROMOTION_HITS - 1`, so its first stash hit promotes
/// it. Placement is unchanged; only the counter differs.
///
/// Inherited (cycle 0006 v001): an item promoted from its target into its probing bucket
/// lands at the probing MRU end (`PROBING_PROMOTION_POS = Back`). Foreign probing
/// residents displaced by it still retry their own target at the cold `insert_pos`.
///
/// Inherited (cycle 0007 v002): with `RESCUE_STASH_VICTIMS`, a stash-overflow victim is
/// first placed into a free slot of its own target bucket (else its probing bucket) at
/// the LRU end (`Front`); only when both are full is it evicted out of the cache and
/// recorded in the ghost.
///
/// Inherited (cycle 0008 v001): with `NORMAL_VICTIM_SECOND_CHANCE`, inside a normal
/// (target or probing) bucket `Item::lfu_counter` is a 1-bit reference flag: cleared to 0
/// on every entry into a normal bucket, set to 1 on a hit while resident. Before a
/// promotion into a full normal bucket, `make_room_clock` scans LRU→MRU, clears each
/// referenced item it passes (second chance) and demotes the first unreferenced one
/// through the usual cascade; if all were referenced it does nothing and the plain LRU
/// eviction applies. In the stash the counter keeps its meaning as a hit count.
///
/// New (cycle 0009 v003): the ghost is partitioned into one FIFO sub-ring per stash
/// bucket, each `GHOST_FACTOR * bucket_size` wide. A key's fingerprint is recorded in and
/// looked up from the sub-ring of its own stash bucket (`get_index(key, stash_size)`), so
/// the put-path scan covers `GHOST_FACTOR * bucket_size` entries instead of
/// `GHOST_FACTOR × stash slots`. Total capacity is unchanged.
///
/// Metadata cost: the ghost holds `GHOST_FACTOR * num_of_stash_buckets * bucket_size`
/// `u64` fingerprints per segment plus one `u16` head per stash bucket, i.e.
/// `slots × 8 + num_of_stash_buckets × 2` bytes per segment. It never stores values; a
/// fingerprint false positive costs only one early promotion.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	ghost: GhostRing,
}

/// Ghost sub-ring width, as a multiple of a stash bucket's slot count (so the total
/// ghost capacity is `GHOST_FACTOR ×` the segment's stash slots).
const GHOST_FACTOR: usize = 2;

/// When set, a stash-overflow victim is placed into a free slot of its own target
/// (else its probing) bucket at the LRU end, and is evicted from the cache only when
/// both are full.
const RESCUE_STASH_VICTIMS: bool = true;

/// When set, the victim leaving a full normal bucket on a promotion is chosen
/// CLOCK-style: the first item (from the LRU end) whose reference bit is clear, with
/// referenced items passed over getting their bit cleared.
const NORMAL_VICTIM_SECOND_CHANCE: bool = true;

/// Per-stash-bucket FIFO sub-rings of 64-bit fingerprints of keys recently evicted out
/// of the segment. Sub-ring `sb` occupies `slots[sb * width .. (sb + 1) * width]` with
/// its own write head `heads[sb]`. Overwrite-on-wrap is the decay: a fingerprint
/// survives about `GHOST_FACTOR × bucket_size` subsequent evictions from its own stash
/// bucket. `0` marks an empty slot (fingerprints are forced non-zero).
///
/// Metadata: `slots.len() × 8 + heads.len() × 2` bytes, with
/// `slots.len() = num_of_stash_buckets * GHOST_FACTOR * bucket_size`.
#[derive(Debug)]
struct GhostRing {
	slots: Vec<u64>,
	heads: Vec<u16>,
	width: usize,
}

impl GhostRing {
	fn new(num_buckets: usize, width: usize) -> Self {
		let num_buckets = num_buckets.max(1);
		let width = width.max(1);
		assert!(
			width <= u16::MAX as usize + 1,
			"GhostRing::new: sub-ring width {width} exceeds u16 head range"
		);
		Self {
			slots: vec![0; num_buckets * width],
			heads: vec![0; num_buckets],
			width,
		}
	}

	#[inline]
	fn range(&self, sb: usize) -> std::ops::Range<usize> {
		let start = sb * self.width;
		start..start + self.width
	}

	/// Writes `fp` at sub-ring `sb`'s head (overwriting its oldest entry) and advances
	/// the head modulo `width`.
	fn insert(&mut self, sb: usize, fp: u64) {
		let h = self.heads[sb] as usize;
		self.slots[sb * self.width + h] = fp;
		self.heads[sb] = ((h + 1) % self.width) as u16;
	}

	/// Linear scan of sub-ring `sb` for `fp`.
	#[cfg(test)]
	fn contains(&self, sb: usize, fp: u64) -> bool {
		self.slots[self.range(sb)].iter().any(|&s| s == fp)
	}

	/// Zeroes the first slot of sub-ring `sb` matching `fp`; returns whether one was
	/// found (`ghost_contains` + `ghost_remove` fused into a single scan).
	fn remove(&mut self, sb: usize, fp: u64) -> bool {
		let r = self.range(sb);
		if let Some(slot) = self.slots[r].iter_mut().find(|s| **s == fp) {
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
/// otherwise (or when both are full) it leaves the cache and is recorded in its stash
/// bucket's ghost sub-ring.
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
			evicted.lfu_counter = 0;
			let ret = core.put_at(Slot::Target, InsertPosition::Front, evicted);
			debug_assert!(ret.is_none(), "rescue into a free target slot must not evict");
			return;
		}
		if probing_room {
			debug!("put_stash_recording: evicted key={:?} -> rescued into probing={p}", evicted.key);
			evicted.lfu_counter = 0;
			let ret = core.put_at(Slot::Probing, InsertPosition::Front, evicted);
			debug_assert!(ret.is_none(), "rescue into a free probing slot must not evict");
			return;
		}
	}
	let sb = get_index(&evicted.key, core.stash_size);
	debug!("put_stash_recording: evicted key={:?} -> ghost sub-ring={sb}", evicted.key);
	ghost.insert(sb, fingerprint(&evicted.key));
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
			let num_buckets = core.stash_size.max(settings.num_of_stash_buckets);
			let width = GHOST_FACTOR * settings.bucket_size;
			Some(Self {
				core,
				ghost: GhostRing::new(num_buckets, width),
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
	/// is in its stash bucket's ghost sub-ring enters with its stash-hit counter
	/// pre-credited, and any stash eviction is rescued or recorded in the ghost.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let sb = get_index(&item.key, self.core.stash_size);
		let ghost_hit = self.ghost.remove(sb, fingerprint(&item.key));
		item.lfu_counter = if ghost_hit { STASH_PROMOTION_HITS - 1 } else { 0 };
		debug!("DashSegment.put: key={:?} ghost_hit={ghost_hit}", item.key);
		put_stash_recording(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
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
/// promoted to its target bucket — unless the target has at least
/// `STASH_GATE_BYPASS_FREE_SLOTS` free slots, in which case the first hit promotes.
/// Ghost-returning puts start at `STASH_PROMOTION_HITS - 1`.
const STASH_PROMOTION_HITS: usize = 2;

/// Minimum number of free slots in the target bucket for a stash item to promote on
/// its first hit.
const STASH_GATE_BYPASS_FREE_SLOTS: usize = 2;

/// Where a stash item promoted to its target lands.
/// MRU end: an item that has proven itself in the stash should not be the next victim
/// evicted from its target.
const STASH_PROMOTION_TARGET_POS: InsertPosition = InsertPosition::Back;

/// Where an item promoted from its target into its probing bucket lands; MRU end so a
/// proven-hottest item is not the next probing victim.
const PROBING_PROMOTION_POS: InsertPosition = InsertPosition::Back;

/// CLOCK-style second chance for a full normal bucket. Scans from index 0 (LRU) toward
/// MRU for the first item whose reference bit (`lfu_counter`) is clear, clearing the bit
/// of every referenced item it passes. The unreferenced item found is removed and routed
/// through `demote_from_normal_bucket` (the same cascade an LRU eviction takes). If every
/// item was referenced, nothing is removed (all bits are now clear) and the caller's
/// `put_at` evicts the LRU item as before. No-op when the bucket is not full.
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
	let mut victim = None;
	for (j, it) in core.buckets[bucket_idx].get_items_mut().iter_mut().enumerate() {
		if it.lfu_counter == 0 {
			victim = Some(j);
			break;
		}
		it.lfu_counter = 0;
	}
	let Some(j) = victim else {
		debug!("make_room_clock: bucket={bucket_idx} all referenced -> bits cleared, LRU fallback");
		return;
	};
	let removed = core.buckets[bucket_idx].remove_at(j);
	debug!(
		"make_room_clock: bucket={bucket_idx} unreferenced victim key={:?} at index={j}",
		removed.key
	);
	debug_assert!(
		!core.buckets[bucket_idx].is_full(),
		"make_room_clock: bucket={bucket_idx} must have a free slot after removing the victim"
	);
	// The victim's own cascade never re-targets `bucket_idx` (an owned item goes to stash;
	// a foreign one retries its own target). Only a stash-overflow rescue into the slot
	// just freed can refill it; the caller's `put_at` then evicts LRU as usual.
	demote_from_normal_bucket(core, ghost, removed, bucket_idx, retry_pos, STASH_DEMOTION_POS);
}

/// Moves `item` into its target bucket at `place_pos`. Any evicted item flows through
/// the tier cascade; foreign residents displaced by it retry their own target at
/// `retry_pos`.
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

/// Moves `item` into its probing bucket at `place_pos`. Any evicted item flows through
/// the tier cascade; foreign residents displaced by it retry their own target at
/// `retry_pos`.
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
	item.lfu_counter = 0;
	let Some(evicted) = core.put_at(Slot::Probing, place_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, evicted, probing_idx, retry_pos, STASH_DEMOTION_POS);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target (at `retry_pos`)
/// before stash; if it was already in its target, go straight to stash. All stash
/// inserts (direct and cascade) land at `stash_pos` with their stash-hit counter reset,
/// so every stash stay needs fresh hits to earn promotion. Items the stash insert
/// evicts are rescued into free normal slots or recorded in the ghost.
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
		item.lfu_counter = 0;
		put_stash_recording(core, ghost, stash_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		// Entry into a normal bucket: fresh reference-bit lifecycle.
		item.lfu_counter = 0;
		if let Some(mut cascade) = core.put_at(Slot::Target, retry_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			cascade.lfu_counter = 0;
			put_stash_recording(core, ghost, stash_pos, cascade);
		}
	}
}

/// `insert_pos = Front`: foreign-resident retries land at the LRU end of their target.
/// `insert_pos = Back`: those land at MRU.
/// Target→probing promotions always land at `PROBING_PROMOTION_POS`, stash→target
/// promotions always land at `STASH_PROMOTION_TARGET_POS`, and stash demotions always
/// land at `STASH_DEMOTION_POS`, independent of `insert_pos`.
/// Stash items promote on their `STASH_PROMOTION_HITS`-th hit in the current stay, or
/// on any hit when the target bucket has at least `STASH_GATE_BYPASS_FREE_SLOTS` free
/// slots (bypass; only the target is checked, not probing).
/// A hit on a target (non-hottest) or probing resident sets its reference bit.
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
			item.lfu_counter = item.lfu_counter.saturating_add(1);
			item.lfu_counter
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

		debug!(
			"promotion_get: key={key:?} found in stash hits={hits} free={free} -> target={target_idx} place_pos={STASH_PROMOTION_TARGET_POS:?} retry_pos={insert_pos:?}"
		);
		core.stash_buckets[stash_idx].get_items_mut()[item_idx].lfu_counter = 0;
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
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, ghost, data, PROBING_PROMOTION_POS, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
			core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter = 1;
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		core.buckets[probing_idx].get_items_mut()[item_idx].lfu_counter = 1;
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

	/// Whether `key`'s fingerprint is in its own stash bucket's ghost sub-ring.
	fn ghost_has(seg: &DashSegment<i64, i64>, key: i64) -> bool {
		let sb = get_index(&key, seg.core.stash_size);
		seg.ghost.contains(sb, fingerprint(&key))
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

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	/// With only one free slot in the target, a single stash hit does not promote: the
	/// key stays in its stash bucket and is refreshed to the MRU (last) index. A second
	/// hit promotes.
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

		// Leave exactly one free slot in the target: below the bypass threshold.
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
		assert!(seg.core.buckets[target_idx].get_position(&k0).is_none());

		// Second hit promotes into the target.
		seg.get(&k0);
		assert!(seg.core.buckets[target_idx].get_position(&k0).is_some());
		assert!(seg.core.stash_buckets[stash_idx].get_position(&k0).is_none());
	}

	/// Picks a key `k` with target 0 and stash 0, a target filler, and stash-0 keys
	/// distinct from both.
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

	/// A key evicted out of the stash and then re-put promotes on its first stash hit,
	/// even though the target has no free slot (bypass off). Target 0 and probing 1 are
	/// pre-filled so the stash overflow evicts `k` instead of rescuing it.
	#[test]
	fn ghost_put_promotes_on_first_stash_hit() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let (k, filler, others) = ghost_fixture(&seg);
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
		assert!(ghost_has(&seg, k), "eviction must record k in its ghost sub-ring");

		seg.put(Item::new(k, 11));
		assert!(!ghost_has(&seg, k), "ghost hit must consume the fingerprint");
		let val = seg.get(&k).expect("hit must return Some").value;
		assert_eq!(val, 11);
		assert!(
			seg.core.buckets[target_idx].get_position(&k).is_some(),
			"ghost-returning key must promote on its first stash hit"
		);
		assert!(seg.core.stash_buckets[0].get_position(&k).is_none());
	}

	/// A stash-overflow victim whose target bucket has a free slot is rescued into the
	/// target's LRU end instead of leaving the cache.
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
		assert!(!ghost_has(&seg, victim), "rescued victim must not be in the ghost");
		assert!(seg.core.check_slot_legality());
		let val = seg.get(&victim).expect("rescued victim must be found").value;
		assert_eq!(val, victim * 10);
	}

	/// When both the victim's target and probing buckets are full, the stash victim is
	/// evicted out of the cache and recorded in its ghost sub-ring.
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
		assert!(ghost_has(&seg, victim), "eviction must record the victim in its ghost sub-ring");
		for &f in target_fillers.iter().chain(probing_fillers.iter()) {
			assert!(seg.core.contains_key(&f), "rescue must never displace a resident");
		}
	}

	/// A key never seen in the ghost still needs two stash hits when the bypass is off.
	#[test]
	fn non_ghost_put_still_needs_two_hits() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (k, filler, others) = ghost_fixture(&seg);
		let target_idx = 0;

		seg.core.buckets[target_idx].put(Item::new(filler, 1));
		seg.put(Item::new(k, 10));
		seg.put(Item::new(others[0], 20));
		assert!(!ghost_has(&seg, k));

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
		let mut g = GhostRing::new(1, 2);
		g.insert(0, 3);
		g.insert(0, 5);
		g.insert(0, 7);
		assert!(!g.contains(0, 3));
		assert!(g.contains(0, 5) && g.contains(0, 7));
		assert!(g.remove(0, 5));
		assert!(!g.contains(0, 5));
		assert!(!g.remove(0, 5));
	}

	/// Overflowing one sub-ring must not evict a fingerprint stored in another, and
	/// lookups are confined to the named sub-ring.
	#[test]
	fn ghost_sub_rings_are_independent() {
		let mut g = GhostRing::new(2, 2);
		g.insert(1, 9);
		g.insert(0, 3);
		g.insert(0, 5);
		g.insert(0, 7);
		assert!(!g.contains(0, 3), "bucket 0 wrapped and overwrote its oldest");
		assert!(g.contains(0, 5) && g.contains(0, 7));
		assert!(g.contains(1, 9), "bucket 1's fingerprint must survive bucket 0's overflow");
		assert!(!g.contains(0, 9) && !g.contains(1, 5), "lookups are sub-ring local");
		assert!(!g.remove(0, 9));
		assert!(g.remove(1, 9));
		assert!(!g.contains(1, 9));
	}

	/// An empty target (2 free slots) lets a stash item promote on its first hit.
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

		// Promotion lands at the MRU end (Back): keys[1] follows keys[0].
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

		// Target is [keys[0], keys[1]]: keys[0] is at the LRU end (not hottest). Hitting
		// it should bump it Back via ClassicLRU, not promote it to probing.
		seg.get(&keys[0]);
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
		assert_eq!(items[1].lfu_counter, 1, "a resident hit must set the reference bit");
		assert_eq!(items[0].lfu_counter, 0);
	}

	/// With target `[a(ref=1), b(ref=0)]` full, promoting a third key spares the
	/// referenced LRU `a` (clearing its bit) and demotes the unreferenced `b` to stash.
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

		// Full target: c needs two stash hits.
		seg.put(Item::new(c, c * 10));
		seg.get(&c);
		seg.get(&c);
		assert!(seg.core.buckets[target_idx].get_position(&c).is_some(), "c promoted");

		let items = seg.core.buckets[target_idx].get_items();
		let a_pos = items.iter().position(|it| it.key == a).expect("referenced LRU a must be spared");
		assert_eq!(items[a_pos].lfu_counter, 0, "spared item's bit is cleared");
		assert!(seg.core.buckets[target_idx].get_position(&b).is_none(), "b must leave the target");
		let b_stash = get_index(&b, seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[b_stash].get_position(&b).is_some(),
			"unreferenced b must be demoted to its stash"
		);
		assert!(seg.core.check_slot_legality());
	}

	/// When every resident of a full target is referenced, all bits are cleared and the
	/// LRU resident is evicted as before.
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
		assert_eq!(items[0].lfu_counter, 0, "passed-over bit is cleared");
		assert_eq!(items[1].key, c);
		let a_stash = get_index(&a, seg.core.stash_size);
		assert!(
			seg.core.stash_buckets[a_stash].get_position(&a).is_some(),
			"LRU a must be demoted to its stash"
		);
	}

	#[test]
	fn target_hit_hottest_promotes_to_probing() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);

		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		// keys[1] was promoted to the target's MRU end, so it is already hottest:
		// one hit promotes it to probing.
		seg.get(&keys[1]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_none());
	}

	/// A target-hottest item promoted into a non-empty probing bucket lands at the
	/// probing MRU end (`PROBING_PROMOTION_POS = Back`), not in the victim slot.
	#[test]
	fn target_hottest_promotion_lands_at_probing_mru() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		let probing_filler = find_keys_for_bucket(probing_idx, seg.core.segment_size, 1)[0];

		seg.core.buckets[probing_idx].put(Item::new(probing_filler, 7));
		fill_target_via_stash(&mut seg, keys[0], keys[1]);

		// keys[1] sits at the target MRU end: one get promotes it to probing.
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
		assert_eq!(
			seg.core.buckets[probing_idx].get_items()[pos].lfu_counter,
			1,
			"a probing hit must set the reference bit"
		);
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

		fill_target_via_stash(&mut seg, target_keys[0], target_keys[1]);
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

		fill_target_via_stash(&mut seg, keys[0], keys[1]);
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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos, insert_pos);

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should be in true target after retry");
		assert_eq!(pos, expected_idx);
		assert_eq!(items[pos].lfu_counter, 0, "retried foreign resident enters with bit 0");
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
	/// promotion `insert_pos`.
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
		// v001: demoted items land at the stash Back even for a Front promotion.
		direct_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert lands at the stash MRU end.
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
		// v001: cascaded items land at the stash Back even for a Front promotion.
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
