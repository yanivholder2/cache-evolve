use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_bucket::tag_of_hash;
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
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
/// **Two-choice stash:** every stash insert goes through [`stash_insert_h`], which
/// fills the key's primary stash bucket `s0` first, overflows into the neighbour
/// `s1 = (s0 + 1) % stash_size` when `s0` is full, and evicts from `s0` only when
/// both are full. Lookups check `s0` first and `s1` last (after target and probing),
/// since `s1` only ever holds overflow.
///
/// **Target-only stash-victim rescue:** every stash insert site goes through
/// [`stash_insert_with_rescue`]. When the stash insert evicts a victim, the victim is
/// placed at the LRU end (`Front`) of its own target bucket if that bucket has a hole;
/// otherwise it is dropped. Rescue never writes into a probing bucket (so `foreign`
/// and the probing gate are untouched), never evicts, and never cascades.
///
/// **Borrowed-count filter:** `SegmentCore::borrowed[s1]` counts the items in `s1`
/// that overflowed from `s0`. When it is zero, no key with primary `s0` can be in
/// `s1`, so `get` skips the `s1` scan entirely.
///
/// **Foreign-count filter:** `SegmentCore::foreign[p]` counts the items in normal
/// bucket `p` that are there as their probing slot (target `p - 1`). When it is zero,
/// no key with target `p - 1` can be in `p`, so `get` skips the probing scan entirely.
///
/// **Concrete buckets:** `SegmentCore` stores its buckets by value as `DashBucket`, so
/// every probe below (`get_position_tagged`, `is_full`, `promotion_candidate_index`, ...)
/// is a statically dispatched, inlinable call with no `Box` indirection.
///
/// **Slot tags:** the key is hashed once per lookup / insert; the stash, target and
/// probing indices and a one-byte slot tag (top hash byte) all derive from that hash.
/// Each bucket probe compares the tag against the bucket header's inline tag array
/// first and only loads the item buffer for tag matches.
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
	/// except that a fresh insert lands at the stash's LRU end and uses two-choice
	/// placement. An item evicted by that insert is rescued into its target bucket if
	/// the target has a hole, else it leaves the cache.
	fn put(&mut self, item: Item<K, V>) {
		let core = &mut self.core;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?} -> stash {FRESH_PUT_STASH_POS:?}", item.key);
		let h = hash(&item.key);
		stash_insert_with_rescue(core, FRESH_PUT_STASH_POS, item, h);
	}
}

/// Two-choice stash insert: fill `s0` if it has room, else `s1` (if distinct) if it has
/// room, else insert into `s0` and return its eviction.
/// `h` is the caller's `hash(&item.key)`; the stash index and slot tag derive from it.
fn stash_insert_h<K, V>(
	core: &mut SegmentCore<K, V>,
	pos: InsertPosition,
	item: Item<K, V>,
	h: usize,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let s0 = h % core.stash_size;
	let s1 = core.stash_neighbor(s0);
	if !core.stash_buckets[s0].is_full() {
		return core.put_stash_at_index_h(s0, pos, item, h);
	}
	if s1 != s0 && !core.stash_buckets[s1].is_full() {
		debug!("stash_insert: key={:?} s0={s0} full -> s1={s1}", item.key);
		return core.put_stash_at_index_h(s1, pos, item, h);
	}
	core.put_stash_at_index_h(s0, pos, item, h)
}

/// Stash insert with **target-only, scan-free victim rescue**.
///
/// Runs [`stash_insert_h`]. If it evicts a victim, the victim is hashed once and, if
/// its target bucket `t = hv % segment_size` has a hole, inserted at the target's LRU
/// end (`Front`) via `put_at_h(Slot::Target, ..)` (which keeps the tag array in sync
/// and leaves `foreign` unchanged, since the victim is native to `t`). Otherwise the
/// victim is dropped, as before.
///
/// Rescue never writes into a probing bucket, only fills an existing hole (so it never
/// evicts and never cascades into [`demote_from_normal_bucket`]), and never revisits
/// the stash.
#[inline]
fn stash_insert_with_rescue<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>, h: usize)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let Some(victim) = stash_insert_h(core, pos, item, h) else {
		return;
	};
	let hv = hash(&victim.key);
	let t = hv % core.segment_size;
	if !core.buckets[t].is_full() {
		debug!("stash_insert_with_rescue: victim key={:?} -> target={t} (Front)", victim.key);
		let evicted = core.put_at_h(Slot::Target, InsertPosition::Front, victim, hv);
		debug_assert!(evicted.is_none(), "rescue into a non-full target must not evict");
	} else {
		debug!("stash_insert_with_rescue: victim key={:?} target={t} full -> dropped", victim.key);
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
#[allow(dead_code)]
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	target_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = hash(&item.key);
	promote_to_target_h(core, item, target_idx, insert_pos, h);
}

/// [`promote_to_target`] with the caller's key hash `h`.
#[inline]
fn promote_to_target_h<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	target_idx: usize,
	insert_pos: InsertPosition,
	h: usize,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(target_idx, get_index(&item.key, core.segment_size));
	let Some(evicted) = core.put_at_h(Slot::Target, insert_pos, item, h) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket (`probing_idx`, precomputed by the caller as
/// `(target_idx + 1) % segment_size`). Any evicted item flows through the tier cascade.
#[allow(dead_code)]
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	probing_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = hash(&item.key);
	promote_to_probing_h(core, item, probing_idx, insert_pos, h);
}

/// [`promote_to_probing`] with the caller's key hash `h`.
#[inline]
fn promote_to_probing_h<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	probing_idx: usize,
	insert_pos: InsertPosition,
	h: usize,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(
		probing_idx,
		(get_index(&item.key, core.segment_size) + 1) % core.segment_size
	);
	let Some(evicted) = core.put_at_h(Slot::Probing, insert_pos, item, h) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
///
/// `insert_pos` governs only the one-more-chance retry into the true target; every stash
/// insert uses [`DEMOTE_TO_STASH_POS`] (MRU end), per SLRU, with two-choice placement and
/// target-only victim rescue ([`stash_insert_with_rescue`]).
///
/// Never writes into the bucket at `source_idx`: a native victim goes to stash, and a
/// foreign victim's true target is necessarily `source_idx - 1` (mod segment_size),
/// which differs from `source_idx` since segment_size >= 2. A rescued stash victim only
/// fills a hole, and every bucket written on this path is full when a stash insert
/// happens, so rescue cannot land in it either.
///
/// Keeps `foreign` exact: `item` just left `source_idx`, and a cascade evictee leaves
/// `true_target`; each is reported via `note_normal_removal` with its own target index.
///
/// Each evictee is hashed exactly once here (no hash was in hand for it); its target
/// index, stash index and slot tag all derive from that hash.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = hash(&item.key);
	let true_target = h % core.segment_size;
	core.note_normal_removal(source_idx, true_target);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		stash_insert_with_rescue(core, DEMOTE_TO_STASH_POS, item, h);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(cascade) = core.put_at_h(Slot::Target, insert_pos, item, h) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			// The cascade evictee left `true_target`; it may itself be foreign there.
			let ch = hash(&cascade.key);
			let cascade_target = ch % core.segment_size;
			core.note_normal_removal(true_target, cascade_target);
			stash_insert_with_rescue(core, DEMOTE_TO_STASH_POS, cascade, ch);
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

/// Stash-hit branch shared by the `s0` and `s1` lookups: removes the item at `item_idx`
/// of stash bucket `stash_idx` (via the borrowed-count-aware helper), promotes it into
/// `target_idx`, and returns it. `h` is the lookup's key hash.
#[inline]
fn stash_hit<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	stash_idx: usize,
	item_idx: usize,
	target_idx: usize,
	insert_pos: InsertPosition,
	h: usize,
) -> &'a Item<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug!("promotion_get: key={key:?} found in stash={stash_idx} -> target={target_idx} insert_pos={insert_pos:?}");
	let data = core.remove_stash_at(stash_idx, item_idx);
	promote_to_target_h(core, data, target_idx, insert_pos, h);

	// The cascade never writes back into target_idx (it is full whenever a cascade
	// reaches the stash, so rescue skips it), so the new item sits exactly where the
	// insert put it.
	let items = core.buckets[target_idx].get_items();
	let pos = inserted_position(insert_pos, items.len());
	debug_assert_eq!(core.buckets[target_idx].get_position(key), Some(pos));
	&items[pos]
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
/// Demotions into the stash always land at the stash's MRU end regardless of `insert_pos`.
///
/// Lookup order: stash `s0` → target → probing (skipped when `foreign[probing] == 0`) →
/// stash `s1` (overflow-only, so checked last, and skipped when `borrowed[s1] == 0`).
///
/// The key is hashed once; `s0`, `target_idx` and the probe tag all derive from it, and
/// every bucket probe is tag-filtered.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = hash(key);
	let tag = tag_of_hash(h);
	let s0 = h % core.stash_size;
	let target_idx = h % core.segment_size;

	if let Some(item_idx) = core.stash_buckets[s0].get_position_tagged(key, tag) {
		return Some(stash_hit(core, key, s0, item_idx, target_idx, insert_pos, h));
	}

	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = core.buckets[target_idx].get_position_tagged(key, tag) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
			// The removed item is native to target_idx, so `foreign` is unchanged.
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing_h(core, data, probing_idx, insert_pos, h);

			// The cascade never writes back into probing_idx (a foreign victim retries in
			// target_idx, and rescue only targets a victim's own target bucket, which is
			// never probing_idx while probing_idx is full), so the new item sits exactly
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

	// `foreign[probing_idx] == 0` proves no key whose target is target_idx is resident in
	// probing_idx, so the scan can be skipped without missing a resident key.
	if core.foreign[probing_idx] != 0 {
		if let Some(item_idx) = core.buckets[probing_idx].get_position_tagged(key, tag) {
			debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
			return Some(core.buckets[probing_idx].get_from_position(item_idx));
		}
	}

	// Neighbour stash bucket, computed lazily: only reached on an otherwise-miss, and
	// only scanned when it holds at least one item borrowed from s0. `borrowed[s1] == 0`
	// proves no key with primary s0 is resident in s1.
	let s1 = core.stash_neighbor(s0);
	if s1 != s0 && core.borrowed[s1] != 0 {
		if let Some(item_idx) = core.stash_buckets[s1].get_position_tagged(key, tag) {
			return Some(stash_hit(core, key, s1, item_idx, target_idx, insert_pos, h));
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

	/// Asserts every structural checker on the segment core.
	fn assert_all_invariants(seg: &DashSegment<i64, i64>, ctx: &str) {
		let core = seg.core();
		assert!(core.check_slot_legality(), "slot legality: {ctx}");
		assert!(core.check_capacity_bound(), "capacity bound: {ctx}");
		assert!(core.check_borrowed_counts(), "borrowed drift: {ctx}");
		assert!(core.check_foreign_counts(), "foreign drift: {ctx}");
		assert!(core.check_tags_consistent(), "tag drift: {ctx}");
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
	/// (both s0 and s1 full) the fresh arrival at s0's LRU end is evicted from the stash,
	/// not the demotee. The evicted fresh arrival is then rescued into its target only if
	/// that target has a hole.
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
		let s1 = (stash_idx + 1) % stash_size;
		let fresh: Vec<i64> = find_keys_for_bucket(stash_idx, stash_size, 16)
			.into_iter()
			.filter(|k| !target_keys.contains(k))
			.take(2)
			.collect();
		assert_eq!(fresh.len(), 2);
		let s1_fill: Vec<i64> = find_keys_for_bucket(s1, stash_size, 16)
			.into_iter()
			.filter(|k| !target_keys.contains(k) && !fresh.contains(k))
			.take(settings.bucket_size)
			.collect();
		assert_eq!(s1_fill.len(), settings.bucket_size);

		// Pre-fill the neighbour stash bucket so overflow must evict from s0.
		for &k in &s1_fill {
			seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, k));
		}
		assert!(seg.core().stash_buckets[s1].is_full());

		// Demote `victim` into the stash via target overflow.
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));
		let trigger_target = get_index(&trigger, seg_size);
		promote_to_target(seg.core_mut(), Item::new(trigger, 99), trigger_target, InsertPosition::Front);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&victim).is_some());

		// Fill the stash bucket with a fresh put (bucket_size = 2).
		seg.put(Item::new(fresh[0], 10));
		assert!(seg.core().stash_buckets[stash_idx].is_full());

		let f0_target = get_index(&fresh[0], seg_size);
		let f0_rescuable = !seg.core().buckets[f0_target].is_full();

		// Overflow with one more fresh put.
		seg.put(Item::new(fresh[1], 11));

		let stash = &seg.core().stash_buckets[stash_idx];
		assert!(stash.get_position(&victim).is_some(), "demoted item must survive");
		assert!(stash.get_position(&fresh[0]).is_none(), "oldest fresh arrival must leave the stash");
		assert_eq!(stash.get_position(&fresh[1]), Some(0), "new arrival sits at LRU end");
		if f0_rescuable {
			assert_eq!(
				seg.core().buckets[f0_target].get_position(&fresh[0]),
				Some(0),
				"stash victim with a target hole is rescued to target LRU end"
			);
		} else {
			assert!(!seg.core().contains_key(&fresh[0]), "stash victim with full target is dropped");
		}
		for &k in &s1_fill {
			assert!(seg.core().contains_key(&k), "s1 residents untouched");
		}
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_tags_consistent());
	}

	// ── Two-choice stash ────────────────────────────────────────────────────

	/// With s0 full and s1 free, a fresh put lands in s1 and evicts nothing.
	#[test]
	fn two_choice_fresh_put_overflows_to_neighbor() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		let keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		for &k in &keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		assert!(seg.core().stash_buckets[s0].is_full());
		let before = seg.core().total_items();

		let k = keys[settings.bucket_size];
		seg.put(Item::new(k, 42));

		assert_eq!(seg.core().total_items(), before + 1, "no eviction");
		assert!(seg.core().stash_buckets[s1].get_position(&k).is_some());
		for &f in &keys[..settings.bucket_size] {
			assert!(seg.core().contains_key(&f));
		}
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_borrowed_counts());
	}

	/// With both s0 and s1 full, s0's LRU is evicted from the stash. Its target bucket is
	/// empty here, so the victim is rescued into the target's LRU end.
	#[test]
	fn two_choice_both_full_evicts_s0_lru() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let seg_size = settings.num_of_normal_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		let s0_keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		let s1_keys = find_keys_for_bucket(s1, stash_size, settings.bucket_size);
		for &k in &s1_keys {
			seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, k));
		}
		for &k in &s0_keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		assert!(seg.core().stash_buckets[s0].is_full());
		assert!(seg.core().stash_buckets[s1].is_full());
		// Fresh puts land at Front, so the LRU (index 0) of s0 is the most recent fresh put.
		let lru = seg.core().stash_buckets[s0].get_items()[0].key;

		let k = s0_keys[settings.bucket_size];
		seg.put(Item::new(k, 42));

		assert!(seg.core().stash_buckets[s0].get_position(&lru).is_none(), "s0 LRU must leave the stash");
		assert!(seg.core().stash_buckets[s1].get_position(&lru).is_none());
		let t = get_index(&lru, seg_size);
		assert_eq!(seg.core().buckets[t].get_position(&lru), Some(0), "victim rescued into empty target");
		assert_eq!(seg.core().stash_buckets[s0].get_position(&k), Some(0));
		for &f in &s1_keys {
			assert!(seg.core().stash_buckets[s1].get_position(&f).is_some());
		}
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_tags_consistent());
	}

	// ── Target-only stash-victim rescue ─────────────────────────────────────

	/// Fills s1 (directly) and s0 (via fresh puts) so the next s0 put evicts s0's LRU.
	/// Returns `(s0_keys, s1_keys, lru)`.
	fn setup_full_stash_pair(seg: &mut DashSegment<i64, i64>, settings: &DashSettings) -> (Vec<i64>, Vec<i64>, i64) {
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		assert_ne!(s0, s1);
		let s0_keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		let s1_keys = find_keys_for_bucket(s1, stash_size, settings.bucket_size);
		for &k in &s1_keys {
			seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, k));
		}
		for &k in &s0_keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		assert!(seg.core().stash_buckets[s0].is_full());
		assert!(seg.core().stash_buckets[s1].is_full());
		let lru = seg.core().stash_buckets[s0].get_items()[0].key;
		(s0_keys, s1_keys, lru)
	}

	/// A stash victim whose target has a hole lands at target index 0 (LRU end), next to
	/// the existing target resident; nothing leaves the cache and all counters hold.
	#[test]
	fn rescue_victim_into_target_hole_lands_at_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		assert!(settings.bucket_size >= 2, "test needs room for a resident plus a hole");
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let (s0_keys, s1_keys, lru) = setup_full_stash_pair(&mut seg, &settings);
		let t = get_index(&lru, seg_size);
		let resident = find_keys_for_bucket(t, seg_size, 64)
			.into_iter()
			.find(|k| !s0_keys.contains(k) && !s1_keys.contains(k))
			.expect("a resident key for the victim's target must exist");
		seg.core_mut().put_normal_at_index(t, InsertPosition::Back, Item::new(resident, 5));
		assert!(!seg.core().buckets[t].is_full());
		let before = seg.core().total_items();

		let k = s0_keys[settings.bucket_size];
		seg.put(Item::new(k, 42));

		assert_eq!(seg.core().total_items(), before + 1, "rescue must keep the victim");
		assert_eq!(seg.core().buckets[t].get_position(&lru), Some(0));
		assert!(seg.core().buckets[t].get_position(&resident).is_some());
		assert!(seg.core().contains_key(&k));
		assert_all_invariants(&seg, "after rescue into hole");

		// The rescued victim is found by a tag-filtered get.
		assert_eq!(seg.get(&lru).map(|it| it.key), Some(lru));
		assert_all_invariants(&seg, "after get of rescued victim");
	}

	/// A stash victim whose target is full is dropped; the target is untouched.
	#[test]
	fn rescue_victim_with_full_target_is_dropped() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let (s0_keys, s1_keys, lru) = setup_full_stash_pair(&mut seg, &settings);
		let t = get_index(&lru, seg_size);
		let residents: Vec<i64> = find_keys_for_bucket(t, seg_size, 64)
			.into_iter()
			.filter(|k| !s0_keys.contains(k) && !s1_keys.contains(k))
			.take(settings.bucket_size)
			.collect();
		assert_eq!(residents.len(), settings.bucket_size);
		for &r in &residents {
			seg.core_mut().put_normal_at_index(t, InsertPosition::Back, Item::new(r, r));
		}
		assert!(seg.core().buckets[t].is_full());
		let target_before: Vec<i64> = seg.core().buckets[t].get_items().iter().map(|it| it.key).collect();
		let before = seg.core().total_items();

		let k = s0_keys[settings.bucket_size];
		seg.put(Item::new(k, 42));

		assert_eq!(seg.core().total_items(), before, "victim dropped, new key inserted");
		assert!(!seg.core().contains_key(&lru), "victim with full target must be dropped");
		let target_after: Vec<i64> = seg.core().buckets[t].get_items().iter().map(|it| it.key).collect();
		assert_eq!(target_before, target_after, "full target untouched");
		assert_all_invariants(&seg, "after dropped victim");
	}

	/// Random get-or-put ops under every policy and two bucket sizes: after each op every
	/// invariant checker holds, a put grows the segment by at most one item, and `get`
	/// finds exactly the resident keys.
	#[test]
	fn rescue_random_ops_keep_all_invariants() {
		for policy in EvictionPolicyKind::all() {
			for bucket_size in [2usize, 5] {
				let mut settings = small_dash_settings(policy);
				settings.bucket_size = bucket_size;
				let mut seg = DashSegment::new(&settings);
				let mut state: u64 = 0x5EED_0F_0A11_C0DE ^ bucket_size as u64;
				for _ in 0..6000 {
					state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					let r = (state >> 33) % 1000;
					let key = if r < 500 { (r % 16) as i64 } else { (r % 180) as i64 };
					let resident = seg.core().contains_key(&key);
					let before = seg.core().total_items();
					let hit = seg.get(&key).map(|it| it.key);
					assert_eq!(hit.is_some(), resident, "policy={policy:?} key={key}");
					assert_eq!(seg.core().total_items(), before, "get must conserve items");
					if hit.is_none() {
						seg.put(Item::new(key, key));
						let after = seg.core().total_items();
						assert!(after == before || after == before + 1, "single eviction per put");
						assert!(seg.core().contains_key(&key), "fresh put must be resident");
					}
					assert_all_invariants(&seg, &format!("policy={policy:?} bs={bucket_size} key={key}"));
				}
			}
		}
	}

	// ── Borrowed-count filter ───────────────────────────────────────────────

	/// (a) A fresh put that overflows into s1 bumps `borrowed[s1]` to 1; a `get` that
	/// promotes the key out of s1 drops it back to 0.
	#[test]
	fn borrowed_count_tracks_overflow_and_promotion() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		assert_ne!(s0, s1);
		let keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		for &k in &keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		assert_eq!(seg.core().borrowed[s1], 0);

		let k = keys[settings.bucket_size];
		seg.put(Item::new(k, 42));
		assert!(seg.core().stash_buckets[s1].get_position(&k).is_some());
		assert_eq!(seg.core().borrowed[s1], 1);
		assert!(seg.core().check_borrowed_counts());

		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 42)));
		assert!(seg.core().stash_buckets[s1].get_position(&k).is_none());
		assert_eq!(seg.core().borrowed[s1], 0);
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
	}

	/// (b) A borrowed item in s1 is evicted when s1's own native keys fill it: the
	/// counter goes down.
	#[test]
	fn borrowed_count_drops_when_native_insert_evicts_borrowed() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		assert_ne!(s0, s1);

		// Overflow one s0 key into s1.
		let s0_keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		for &k in &s0_keys {
			seg.put(Item::new(k, k));
		}
		let borrowed_key = s0_keys[settings.bucket_size];
		assert!(seg.core().stash_buckets[s1].get_position(&borrowed_key).is_some());
		assert_eq!(seg.core().borrowed[s1], 1);

		// Fill s1 with its own native keys at the MRU end: the borrowed item becomes
		// s1's LRU and is evicted by the native insert that overflows s1.
		let natives: Vec<i64> = find_keys_for_bucket(s1, stash_size, 64)
			.into_iter()
			.filter(|k| !s0_keys.contains(k))
			.take(settings.bucket_size)
			.collect();
		assert_eq!(natives.len(), settings.bucket_size);
		let mut evicted_borrowed = false;
		for &k in &natives {
			let evicted = seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, k));
			if evicted.map(|e| e.key) == Some(borrowed_key) {
				evicted_borrowed = true;
			}
			assert!(seg.core().check_borrowed_counts());
		}
		assert!(evicted_borrowed, "native insert must evict the borrowed item");
		assert!(
			seg.core().stash_buckets[s1].get_position(&borrowed_key).is_none(),
			"borrowed item must have been evicted from s1"
		);
		assert_eq!(seg.core().borrowed[s1], 0);
		assert!(seg.core().check_borrowed_counts());
	}

	/// (c) A random op sequence keeps `check_borrowed_counts()`, `check_foreign_counts()`
	/// and the bucket tag arrays exact after every op.
	#[test]
	fn borrowed_counts_exact_under_random_ops() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		// Simple deterministic LCG.
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..5000 {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let key = ((state >> 33) % 200) as i64;
			if seg.get(&key).is_none() {
				seg.put(Item::new(key, key));
			}
			assert!(seg.core().check_borrowed_counts(), "borrowed drift after key={key}");
			assert!(seg.core().check_foreign_counts(), "foreign drift after key={key}");
			assert!(seg.core().check_slot_legality());
			assert!(seg.core().check_tags_consistent(), "tag drift after key={key}");
		}
	}

	/// Random ops under every eviction policy and a larger bucket: every resident key is
	/// found by the tag-filtered `get`, and the tag arrays stay exact.
	#[test]
	fn tag_filter_finds_every_resident_under_all_policies() {
		for policy in EvictionPolicyKind::all() {
			let mut settings = small_dash_settings(policy);
			settings.bucket_size = 6;
			settings.num_of_stash_buckets = 3;
			let mut seg = DashSegment::new(&settings);
			let mut state: u64 = 0xABCD_EF01_2345_6789;
			for _ in 0..4000 {
				state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let r = (state >> 33) % 1000;
				let key = if r < 600 { (r % 20) as i64 } else { (r % 150) as i64 };
				let resident = seg.core().contains_key(&key);
				let hit = seg.get(&key).map(|it| it.key);
				assert_eq!(hit.is_some(), resident, "policy={policy:?} key={key}");
				if let Some(k) = hit {
					assert_eq!(k, key);
				} else {
					seg.put(Item::new(key, key));
				}
				assert!(seg.core().check_tags_consistent(), "policy={policy:?}");
				assert!(seg.core().check_foreign_counts());
				assert!(seg.core().check_borrowed_counts());
			}
		}
	}

	/// Random ops with a skewed key distribution (frequent hottest→probing promotions):
	/// the foreign counters stay exact and every resident key is found by `get`.
	#[test]
	fn foreign_counts_exact_under_skewed_random_ops() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut state: u64 = 0x1234_5678_9ABC_DEF0;
		for _ in 0..8000 {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let r = (state >> 33) % 1000;
			let key = if r < 700 { (r % 12) as i64 } else { (r % 120) as i64 };
			let resident = seg.core().contains_key(&key);
			let hit = seg.get(&key).is_some();
			assert_eq!(hit, resident, "get must find every resident key (key={key})");
			if !hit {
				seg.put(Item::new(key, key));
			}
			assert!(seg.core().check_foreign_counts(), "foreign drift after key={key}");
			assert!(seg.core().check_borrowed_counts());
		}
	}

	/// (d) A miss on a key whose s1 holds only native items returns None.
	#[test]
	fn miss_with_native_only_s1_returns_none() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		let probe = find_keys_for_bucket(s0, stash_size, 1)[0];
		let natives: Vec<i64> = find_keys_for_bucket(s1, stash_size, 16)
			.into_iter()
			.filter(|k| *k != probe)
			.take(settings.bucket_size)
			.collect();
		for &k in &natives {
			seg.put(Item::new(k, k));
		}
		assert_eq!(seg.core().borrowed[s1], 0);
		assert!(seg.get(&probe).is_none());
		for &k in &natives {
			assert!(seg.core().contains_key(&k));
		}
	}

	// ── Foreign-count filter ────────────────────────────────────────────────

	/// (a) A hottest→probing promotion sets `foreign[probing] == 1`; evicting that item
	/// back to its target via the one-more-chance cascade drops it back to 0.
	#[test]
	fn foreign_count_tracks_promotion_and_one_more_chance() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let k = find_keys_for_bucket(target_idx, seg_size, 1)[0];
		let natives: Vec<i64> = find_keys_for_bucket(probing_idx, seg_size, settings.bucket_size)
			.into_iter()
			.collect();

		// Sole resident of target → hottest, so the get promotes it into probing.
		seg.core_mut().buckets[target_idx].put(Item::new(k, 7));
		assert_eq!(seg.core().foreign[probing_idx], 0);
		assert_eq!(seg.get(&k).map(|it| it.value), Some(7));
		assert!(seg.core().buckets[probing_idx].get_position(&k).is_some());
		assert_eq!(seg.core().foreign[probing_idx], 1);
		assert!(seg.core().check_foreign_counts());

		// Fill the probing bucket with its own natives (k stays the LRU), then one more
		// native target insert evicts k, which retries in its own target.
		for &n in &natives[..settings.bucket_size - 1] {
			seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(n, n));
		}
		assert!(seg.core().buckets[probing_idx].is_full());
		let trigger = natives[settings.bucket_size - 1];
		promote_to_target(seg.core_mut(), Item::new(trigger, trigger), probing_idx, InsertPosition::Back);

		assert!(seg.core().buckets[probing_idx].get_position(&k).is_none());
		assert!(seg.core().buckets[target_idx].get_position(&k).is_some());
		assert_eq!(seg.core().foreign[probing_idx], 0);
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_tags_consistent());
	}

	/// (b) A miss on a key whose probing bucket holds only native items returns None.
	#[test]
	fn miss_with_native_only_probing_returns_none() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let probe = find_keys_for_bucket(target_idx, seg_size, 1)[0];
		let natives = find_keys_for_bucket(probing_idx, seg_size, settings.bucket_size);
		for &n in &natives {
			seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(n, n));
		}
		assert_eq!(seg.core().foreign[probing_idx], 0);
		assert!(seg.get(&probe).is_none());
		for &n in &natives {
			assert!(seg.core().contains_key(&n));
		}
	}

	/// (c) A probing hit still works when `foreign != 0`.
	#[test]
	fn probing_hit_with_foreign_nonzero() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let k = find_keys_for_bucket(target_idx, seg_size, 1)[0];
		let native = find_keys_for_bucket(probing_idx, seg_size, 1)[0];
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(native, 1));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(k, 55));
		assert_eq!(seg.core().foreign[probing_idx], 1);

		assert_eq!(seg.get(&k).map(|it| (it.key, it.value)), Some((k, 55)));
		assert!(seg.core().buckets[probing_idx].get_position(&k).is_some());
		assert_eq!(seg.core().foreign[probing_idx], 1);
		assert!(seg.core().check_foreign_counts());
	}

	/// A raw `get_items_mut` push (fixture path) disables a bucket's tags; the key is
	/// still found by the (now plain-scan) lookup.
	#[test]
	fn raw_pushed_item_still_found() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg_size, 2);
		seg.core_mut().buckets[target_idx].put(Item::new(keys[0], 1));
		seg.core_mut().buckets[target_idx].get_items_mut().push(Item::new(keys[1], 2));
		assert_eq!(seg.get(&keys[0]).map(|it| it.value), Some(1));
		assert!(seg.core().check_tags_consistent());
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
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
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
		assert!(seg.core.check_tags_consistent());
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
		assert_eq!(seg.core.foreign[probing_idx], 1);
		assert!(seg.core.check_foreign_counts());
	}

	#[test]
	fn probing_hit_touches_in_place() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);

		seg.core.put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(keys[0], 10));

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
		assert!(seg.core.check_foreign_counts());
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
		// instead of bouncing it back into the bucket it just left.
		let evicted_key = probing_keys[0];
		// Permissive assertion: the key may have re-landed elsewhere via subsequent
		// stash overflow handling — only "vanished entirely" would be a real bug.
		assert!(
			seg.core.contains_key(&evicted_key),
			"evicted probing item should be demoted to stash or still in cache"
		);
		assert!(seg.core.check_foreign_counts());
		assert!(seg.core.check_tags_consistent());
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
		assert_eq!(seg.core.foreign[0], 1);
		assert!(seg.core.check_foreign_counts());
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
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(k, 99));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(filler, 50));
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
		assert!(seg.core().check_foreign_counts());
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
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(foreign, 2));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(filler, 3));

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), trigger_probing, insert_pos);

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should be in true target after retry");
		assert_eq!(pos, expected_idx);
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_tags_consistent());
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

		seg.core_mut().put_stash_at_index(stash_idx, InsertPosition::Back, Item::new(stash_pre, 7));
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
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
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

		seg.core_mut().put_stash_at_index(stash_idx, InsertPosition::Back, Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_a, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(foreign, 3));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(probing_filler, 4));

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), trigger_probing, insert_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_tags_consistent());
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 1);
	}
}
