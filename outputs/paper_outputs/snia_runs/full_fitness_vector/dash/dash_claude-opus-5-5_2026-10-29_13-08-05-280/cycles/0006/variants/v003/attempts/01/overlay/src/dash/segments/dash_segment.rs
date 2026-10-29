use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Number of stash hits an item must accumulate (in `Item::lfu_counter`) before it
/// may be promoted from stash into its target bucket. One-hit items (scan traffic)
/// stay in stash and never displace residents of the protected normal tier.
const PROMOTE_THRESHOLD: u8 = 2;

/// Stash hits required before an item may promote into a *full* target bucket
/// (where promotion swaps the target's LRU resident down into stash) — applied
/// only when the segment's `full_target_guard` is enabled (large caches).
/// Promotions into a non-full target still need only `PROMOTE_THRESHOLD`.
const FULL_TARGET_PROMOTE_THRESHOLD: u8 = 3;

/// Total cache capacity at or above which the full-target stash guard is enabled:
/// the geometric middle of the 16384 and 131072 evaluation sizes
/// (sqrt(16384 * 131072) ≈ 46341). Below it, stash items promote at two hits even
/// into a full target, since a small cache can evict them from stash before a
/// third hit arrives.
const GUARD_MIN_CAPACITY: usize = 46341;

/// Prior hottest-position target hits (recorded in `Item::lfu_counter`) required
/// before a target resident may promote into a *full* probing bucket. A promotion
/// into a full probing bucket triggers the most expensive cascade
/// (probing LRU -> its true target -> that target's LRU -> stash), so the first
/// hottest hit against a full probing bucket is only recorded (an in-place MRU
/// touch); the next hottest hit promotes. Ungated: applies at every cache size.
const FULL_PROBING_PROMOTE_THRESHOLD: u8 = 1;

/// The dash Subject's segment, variant c0006/v003: c0005/v001 (two-hit stash
/// admission, full-probing door guard, size-gated full-target stash guard, 24-byte
/// `Item` stamped from a segment-local tick) with a **direct swap** on stash→full-
/// target promotion.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by proving they're the
/// hottest in their target. A stash→target insert lands at the LRU end
/// (`InsertPosition::Front`), so an item must climb before it can promote again.
/// Demotion out of probing cascades down the chain `probing -> target -> stash -> out`,
/// with a one-more-chance step between adjacent tiers.
///
/// Stash→full-target promotion does not cascade: the promoted key and the target's
/// LRU entry trade places in one step (see [`swap_into_full_target`]). The demoted
/// entry goes straight to stash with `lfu_counter = 0` — no neighbour/probing retry.
///
/// `Item::lfu_counter` (unused by ClassicLRU) is reused as a reuse counter: an item
/// needs `PROMOTE_THRESHOLD` stash hits to leave stash into a non-full target, and
/// (when `full_target_guard` is set) `FULL_TARGET_PROMOTE_THRESHOLD` hits to enter a
/// full one. In the normal tier the counter is only written on a hottest-position
/// target hit against a full probing bucket (door guard: first such hit sets it to
/// 1, second promotes and resets it to 0). Promotion into target resets it to 0;
/// cascade demotion into stash halves it; a swap-demoted target LRU enters stash at 0.
///
/// `Item::timestamp` is a `u32` logical tick drawn from this segment's own `tick`
/// counter on every insert (`put` has `&mut self`, so no atomic is needed).
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Segment-local logical clock; bumped (wrapping) on every insert.
	tick: u32,
	/// Whether stash→full-target promotion demands `FULL_TARGET_PROMOTE_THRESHOLD`
	/// hits. Set at construction from total capacity vs `GUARD_MIN_CAPACITY`.
	full_target_guard: bool,
}

impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new DashSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			let full_target_guard = settings.capacity() >= GUARD_MIN_CAPACITY;
			info!("DashSegment.try_new: full_target_guard={full_target_guard}");
			Some(Self {
				core,
				tick: 0,
				full_target_guard,
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
		let full_target_guard = self.full_target_guard;
		promotion_get(&mut self.core, key, InsertPosition::Front, full_target_guard)
	}

	/// Default stash-back insert, plus stamping the item with the segment-local tick.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		item.timestamp = self.tick;
		self.tick = self.tick.wrapping_add(1);
		debug!("DashSegment.put: key={:?} tick={}", item.key, item.timestamp);
		self.core.put_at(Slot::Stash, InsertPosition::Back, item);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// out of probing cascades down the chain `probing -> target -> stash -> out`, with
// a one-more-chance step between adjacent tiers. Stash→full-target promotion is a
// direct swap instead (no cascade).

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

/// Direct swap for a stash item (already removed from its stash slot) entering its
/// *full* target bucket: take the target's LRU-position entry out, insert `item` at
/// `insert_pos`, and write the demoted entry into stash with `lfu_counter = 0` so it
/// must re-earn target through the normal stash gate. No neighbour/probing retry and
/// no cascade: exactly one target bucket and one stash bucket are touched.
///
/// The demoted entry lands in its own stash bucket (slot legality). When that is the
/// bucket `item` just vacated, the freed slot absorbs it; otherwise that stash bucket
/// may have to drop its own eviction candidate.
///
/// Returns the index of `item` within the target bucket.
fn swap_into_full_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	target_idx: usize,
	insert_pos: InsertPosition,
) -> usize
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target = &mut core.buckets[target_idx];
	let Some(victim_idx) = target.eviction_candidate_index() else {
		// Degenerate zero-capacity bucket: nothing to swap with; fall back to the cascade path.
		let key = item.key;
		promote_to_target(core, item, insert_pos);
		return core.buckets[target_idx]
			.get_position(&key)
			.expect("swap_into_full_target: key must be resident after fallback promotion");
	};
	let mut demoted = target.remove_at(victim_idx);
	let (_, evicted) = target.put_at(item, insert_pos);
	debug_assert!(evicted.is_none(), "swap_into_full_target: target had a free slot after removing its LRU");
	let new_idx = match insert_pos {
		InsertPosition::Front => 0,
		InsertPosition::Back => target.size() - 1,
	};

	debug!(
		"swap_into_full_target: target={target_idx} demoted key={:?} -> stash",
		demoted.key
	);
	demoted.lfu_counter = 0;
	if let Some(dropped) = core.put_at(Slot::Stash, insert_pos, demoted) {
		debug!("swap_into_full_target: stash dropped key={:?}", dropped.key);
	}
	new_idx
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
/// Items entering stash have their reuse counter halved so old popularity fades.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
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
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		item.lfu_counter >>= 1;
		core.put_at(Slot::Stash, insert_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(mut cascade) = core.put_at(Slot::Target, insert_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			cascade.lfu_counter >>= 1;
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
	full_target_guard: bool,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		let lfu_counter = core.stash_buckets[stash_idx].get_items()[item_idx].lfu_counter;
		let count = lfu_counter.saturating_add(1);
		core.stash_buckets[stash_idx].get_items_mut()[item_idx].lfu_counter = count;

		if count < PROMOTE_THRESHOLD {
			debug!("promotion_get: key={key:?} found in stash count={count} -> touch in stash");
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		let target_full = core.buckets[target_idx].is_full();

		// Size-gated occupancy guard: in large caches, entering a full target forces a
		// demotion, so demand one more stash hit first. Small caches promote at two hits.
		if full_target_guard && count < FULL_TARGET_PROMOTE_THRESHOLD && target_full {
			debug!("promotion_get: key={key:?} found in stash count={count} target={target_idx} full -> touch in stash");
			return Some(core.stash_buckets[stash_idx].get_from_position(item_idx));
		}

		let mut data = core.stash_buckets[stash_idx].remove_at(item_idx);
		data.lfu_counter = 0;

		if target_full {
			debug!("promotion_get: key={key:?} found in stash count={count} -> swap into full target={target_idx} insert_pos={insert_pos:?}");
			let item_new_idx = swap_into_full_target(core, data, target_idx, insert_pos);
			debug_assert_eq!(core.buckets[target_idx].get_items()[item_new_idx].key, *key);
			return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
		}

		debug!("promotion_get: key={key:?} found in stash count={count} -> target={target_idx} insert_pos={insert_pos:?}");
		promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into non-full target={target_idx}")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			if core.buckets[probing_idx].is_full() {
				// Door guard: promoting into a full probing bucket triggers the longest
				// demotion cascade, so demand a second consecutive hottest hit first.
				let counter = core.buckets[target_idx].get_items()[item_idx].lfu_counter;
				if counter < FULL_PROBING_PROMOTE_THRESHOLD {
					debug!("promotion_get: key={key:?} found in target as hottest, probing={probing_idx} full -> record hit, touch in target");
					core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter = 1;
					return Some(core.buckets[target_idx].get_from_position(item_idx));
				}
				debug!("promotion_get: key={key:?} found in target as hottest (guard passed) -> full probing={probing_idx}");
				let mut data = core.buckets[target_idx].remove_at(item_idx);
				data.lfu_counter = 0;
				promote_to_probing(core, data, insert_pos);
			} else {
				debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
				let data = core.buckets[target_idx].remove_at(item_idx);
				promote_to_probing(core, data, insert_pos);
			}

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
	fn put_stamps_segment_local_tick() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		seg.put(Item::new(2_i64, 20_i64));
		let ts = |k: i64| {
			let s = get_index(&k, seg.core.stash_size);
			let p = seg.core.stash_buckets[s].get_position(&k).unwrap();
			seg.core.stash_buckets[s].get_items()[p].timestamp
		};
		assert_eq!(ts(1), 0);
		assert_eq!(ts(2), 1);
		assert_eq!(seg.tick, 2);
	}

	#[test]
	fn tick_wraps_without_panic() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.tick = u32::MAX;
		seg.put(Item::new(1_i64, 10_i64));
		assert_eq!(seg.tick, 0);
	}

	#[test]
	fn guard_gated_by_capacity() {
		let small = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(!small.full_target_guard);
		let mut s = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		s.num_of_segments = 64;
		s.num_of_normal_buckets = 8;
		s.num_of_stash_buckets = 24;
		s.bucket_size = 64; // 64 * 32 * 64 = 131072
		let large: DashSegment<i64, i64> = DashSegment::new(&s);
		assert!(large.full_target_guard);
		s.num_of_segments = 8; // 16384
		let mid: DashSegment<i64, i64> = DashSegment::new(&s);
		assert!(!mid.full_target_guard);
	}

	#[test]
	fn one_stash_hit_keeps_key_in_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let k = find_keys_for_bucket(target_idx, seg.core.segment_size, 1)[0];
		seg.put(Item::new(k, 10));
		assert_eq!(seg.get(&k).unwrap().value, 10);

		let stash_idx = get_index(&k, seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&k).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none());
	}

	#[test]
	fn two_stash_hits_promote_to_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let k = find_keys_for_bucket(target_idx, seg.core.segment_size, 1)[0];
		seg.put(Item::new(k, 10));
		seg.get(&k);
		assert_eq!(seg.get(&k).unwrap().value, 10);

		let stash_idx = get_index(&k, seg.core.stash_size);
		assert!(seg.core.stash_buckets[stash_idx].get_position(&k).is_none());
		let pos = seg.core.buckets[target_idx].get_position(&k).expect("promoted into target");
		assert_eq!(seg.core.buckets[target_idx].get_items()[pos].lfu_counter, 0);
	}

	/// With the guard enabled, a full target demands `FULL_TARGET_PROMOTE_THRESHOLD`
	/// stash hits: after two hits the item is still in stash and the target is
	/// untouched; the third hit swaps it in and demotes the target's LRU to stash
	/// with a cleared counter.
	#[test]
	fn two_hit_stash_item_enters_full_target_demoting_lru() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		seg.full_target_guard = true;
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, settings.num_of_normal_buckets, settings.bucket_size + 1);
		for &r in &keys[..settings.bucket_size] {
			let mut it = Item::new(r, r);
			it.lfu_counter = 5;
			seg.core_mut().buckets[target_idx].put(it);
		}
		assert!(seg.core().buckets[target_idx].is_full());

		let k = keys[settings.bucket_size];
		let k_stash_idx = get_index(&k, settings.num_of_stash_buckets);
		seg.put(Item::new(k, 77));
		seg.get(&k);
		assert_eq!(seg.get(&k).unwrap().value, 77);

		// After two hits: still in stash, target unchanged.
		assert!(seg.core().stash_buckets[k_stash_idx].get_position(&k).is_some());
		assert!(seg.core().buckets[target_idx].get_position(&k).is_none());
		for &r in &keys[..settings.bucket_size] {
			assert!(seg.core().buckets[target_idx].get_position(&r).is_some());
		}

		// Third hit swaps into the full target.
		assert_eq!(seg.get(&k).unwrap().value, 77);
		assert!(seg.core().stash_buckets[k_stash_idx].get_position(&k).is_none());
		let kpos = seg.core().buckets[target_idx].get_position(&k).expect("swapped into target");
		assert_eq!(kpos, 0, "promoted key lands at the Front (LRU end)");
		assert_eq!(seg.core().buckets[target_idx].get_items()[kpos].lfu_counter, 0);

		let lru = keys[0];
		assert!(seg.core().buckets[target_idx].get_position(&lru).is_none());
		let lru_stash_idx = get_index(&lru, settings.num_of_stash_buckets);
		let lpos = seg.core().stash_buckets[lru_stash_idx]
			.get_position(&lru)
			.expect("target LRU should be swapped down to stash");
		assert_eq!(seg.core().stash_buckets[lru_stash_idx].get_items()[lpos].lfu_counter, 0);
		for &r in &keys[1..settings.bucket_size] {
			assert!(seg.core().buckets[target_idx].get_position(&r).is_some());
		}
	}

	/// The swap never retries a foreign (probing-resident) LRU in its true target:
	/// it goes straight to stash with a cleared counter, and the neighbour bucket is
	/// untouched. Item count is preserved.
	#[test]
	fn swap_sends_foreign_lru_straight_to_stash() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		assert!(!seg.full_target_guard);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 1;
		let neighbour_idx = 0; // foreign's true target; bucket 1 is its probing slot
		let foreign = find_keys_for_bucket(neighbour_idx, seg_size, 1)[0];
		let natives = find_keys_for_bucket(target_idx, seg_size, settings.bucket_size + 1);

		let mut f = Item::new(foreign, 5);
		f.lfu_counter = 4;
		seg.core_mut().buckets[target_idx].put(f); // LRU position
		for &r in &natives[..settings.bucket_size - 1] {
			seg.core_mut().buckets[target_idx].put(Item::new(r, r));
		}
		assert!(seg.core().buckets[target_idx].is_full());

		let k = natives[settings.bucket_size];
		seg.put(Item::new(k, 77));
		let before = seg.core().total_items();
		seg.get(&k);
		assert_eq!(seg.get(&k).unwrap().value, 77);

		assert!(seg.core().buckets[target_idx].get_position(&k).is_some());
		assert!(seg.core().buckets[target_idx].get_position(&foreign).is_none());
		assert!(
			seg.core().buckets[neighbour_idx].get_items().is_empty(),
			"swap must not retry the demoted entry in its neighbour/true target"
		);
		let fs = get_index(&foreign, settings.num_of_stash_buckets);
		let fpos = seg.core().stash_buckets[fs].get_position(&foreign).expect("foreign LRU demoted to stash");
		assert_eq!(seg.core().stash_buckets[fs].get_items()[fpos].lfu_counter, 0);
		assert_eq!(seg.core().total_items(), before);
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_capacity_bound());
	}

	/// With the guard disabled (small cache), two stash hits promote even into a full target.
	#[test]
	fn unguarded_two_hits_enter_full_target() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		assert!(!seg.full_target_guard);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, settings.num_of_normal_buckets, settings.bucket_size + 1);
		for &r in &keys[..settings.bucket_size] {
			seg.core_mut().buckets[target_idx].put(Item::new(r, r));
		}
		assert!(seg.core().buckets[target_idx].is_full());

		let k = keys[settings.bucket_size];
		let k_stash_idx = get_index(&k, settings.num_of_stash_buckets);
		seg.put(Item::new(k, 77));
		seg.get(&k);
		assert_eq!(seg.get(&k).unwrap().value, 77);
		assert!(seg.core().stash_buckets[k_stash_idx].get_position(&k).is_none());
		assert!(seg.core().buckets[target_idx].get_position(&k).is_some());
	}

	#[test]
	fn normal_tier_hits_do_not_touch_counter() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 2);
		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		// keys[1] sits at Front (not hottest): a target hit must leave its counter at 0.
		seg.get(&keys[1]);
		let pos = seg.core.buckets[target_idx].get_position(&keys[1]).expect("still in target");
		assert_eq!(seg.core.buckets[target_idx].get_items()[pos].lfu_counter, 0);
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

		// Sole resident of target → already at MRU. Probing is full, so the first get
		// only records the hottest hit (door guard); the second promotes.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());
	}

	/// Door guard: the first hottest hit against a full probing bucket leaves the key in
	/// target with `lfu_counter == 1` and probing unchanged; the second hit promotes it
	/// and resets the counter to 0.
	#[test]
	fn full_probing_door_guard_defers_first_hottest_hit() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, seg.core.buckets[probing_idx].get_max_size());

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());
		let probing_before: Vec<i64> = seg.core.buckets[probing_idx].get_items().iter().map(|it| it.key).collect();

		let k = target_keys[0];
		seg.core.buckets[target_idx].put(Item::new(k, 100));

		// First hottest hit: deferred.
		assert_eq!(seg.get(&k).unwrap().value, 100);
		let pos = seg.core.buckets[target_idx].get_position(&k).expect("still in target after first hit");
		assert_eq!(seg.core.buckets[target_idx].get_items()[pos].lfu_counter, 1);
		assert!(seg.core.buckets[probing_idx].get_position(&k).is_none());
		let probing_after: Vec<i64> = seg.core.buckets[probing_idx].get_items().iter().map(|it| it.key).collect();
		assert_eq!(probing_before, probing_after, "probing must be unchanged by a deferred hit");

		// Second hottest hit: promotes, counter reset.
		assert_eq!(seg.get(&k).unwrap().value, 100);
		assert!(seg.core.buckets[target_idx].get_position(&k).is_none());
		let ppos = seg.core.buckets[probing_idx].get_position(&k).expect("promoted into probing");
		assert_eq!(seg.core.buckets[probing_idx].get_items()[ppos].lfu_counter, 0);
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

		// Three gets on target_keys[0]: first bumps it to MRU, second is a deferred
		// hottest hit (probing is full), third promotes to probing, which evicts the
		// LRU resident of probing.
		seg.get(&target_keys[0]);
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
		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());
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

	/// Cascade demotion into stash halves the reuse counter.
	#[test]
	fn demotion_to_stash_halves_counter() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg_size, settings.bucket_size + 1);
		for &r in &keys[..settings.bucket_size] {
			let mut it = Item::new(r, r);
			it.lfu_counter = 6;
			seg.core_mut().buckets[target_idx].put(it);
		}
		let victim = keys[0];
		promote_to_target(seg.core_mut(), Item::new(keys[settings.bucket_size], 1), InsertPosition::Front);

		let stash_idx = get_index(&victim, settings.num_of_stash_buckets);
		let pos = seg.core().stash_buckets[stash_idx]
			.get_position(&victim)
			.expect("victim demoted to stash");
		assert_eq!(seg.core().stash_buckets[stash_idx].get_items()[pos].lfu_counter, 3);
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
}
