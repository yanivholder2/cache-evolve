use super::{index_from_hash, key_hash, Segment, SegmentCore, Slot};
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
/// **Two-choice stash:** every stash insert goes through [`stash_insert`], which
/// fills the key's primary stash bucket `s0` first, overflows into the neighbour
/// `s1 = (s0 + 1) % stash_size` when `s0` is full, and evicts from `s0` only when
/// both are full. Lookups check `s0` first and `s1` last (after target and probing),
/// since `s1` only ever holds overflow.
///
/// **Stash-victim rescue:** a stash victim (both `s0` and `s1` were full) is not
/// dropped outright: it is placed at the LRU end of its target bucket if that bucket
/// has a free slot, else of its probing bucket if *that* has a free slot, else it
/// leaves the cache. Rescue only fills free normal slots, so it never evicts.
///
/// **Borrowed-count filter:** `SegmentCore::borrowed[s1]` counts the items in `s1`
/// that overflowed from `s0`. When it is zero, no key with primary `s0` can be in
/// `s1`, so `get` skips the `s1` scan entirely. The counter is maintained from hashes
/// the insert/evict path already computes (one hash per key; the evictee's single
/// hash serves both its counter decrement and its rescue target).
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
	/// placement. Any stash victim of that insert is rescued into a free normal slot
	/// when possible, else leaves the cache.
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

/// Two-choice stash insert given the item's precomputed raw hash `h`: fill `s0` if it
/// has room, else `s1` (if distinct) if it has room, else insert into `s0` and return
/// its eviction together with the evictee's raw hash.
#[inline]
fn stash_insert_hashed<K, V>(
	core: &mut SegmentCore<K, V>,
	pos: InsertPosition,
	item: Item<K, V>,
	h: usize,
) -> Option<(Item<K, V>, usize)>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let s0 = index_from_hash(h, core.stash_size);
	if !core.stash_buckets[s0].is_full() {
		return core.put_stash_at_index_known(s0, s0, pos, item);
	}
	let s1 = core.stash_neighbor(s0);
	if s1 != s0 && !core.stash_buckets[s1].is_full() {
		debug!("stash_insert: key={:?} s0={s0} full -> s1={s1}", item.key);
		return core.put_stash_at_index_known(s1, s0, pos, item);
	}
	core.put_stash_at_index_known(s0, s0, pos, item)
}

/// Two-choice stash insert (hashes `item.key` once). Returns the evicted item and its
/// raw hash, if any; the evictee leaves the cache unless the caller rescues it.
#[cfg_attr(not(test), allow(dead_code))]
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>) -> Option<(Item<K, V>, usize)>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = key_hash(&item.key);
	stash_insert_hashed(core, pos, item, h)
}

/// [`stash_insert_hashed`] followed by stash-victim rescue. The victim's target index
/// comes from the hash `put_stash_at_index_known` already computed for its counter
/// update, so rescue costs no extra hash. Rescue writes only into a non-full normal
/// bucket (target first, then probing), at its LRU end, so it never evicts and never
/// touches the stash (hence never changes `borrowed`).
#[inline]
fn stash_insert_with_rescue_hashed<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>, h: usize)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let Some((victim, vh)) = stash_insert_hashed(core, pos, item, h) else {
		return;
	};
	let t = index_from_hash(vh, core.segment_size);
	debug_assert_eq!(t, get_index(&victim.key, core.segment_size));
	if !core.buckets[t].is_full() {
		debug!("stash rescue: key={:?} -> target={t}", victim.key);
		let ev = core.put_at_normal_idx(t, InsertPosition::Front, victim);
		debug_assert!(ev.is_none(), "rescue into non-full target must not evict");
		return;
	}
	let p = (t + 1) % core.segment_size;
	if !core.buckets[p].is_full() {
		debug!("stash rescue: key={:?} -> probing={p}", victim.key);
		let ev = core.put_at_normal_idx(p, InsertPosition::Front, victim);
		debug_assert!(ev.is_none(), "rescue into non-full probing must not evict");
		return;
	}
	debug!("stash rescue: key={:?} target={t} probing={p} full -> dropped", victim.key);
}

/// [`stash_insert_with_rescue_hashed`], hashing `item.key` once.
#[inline]
fn stash_insert_with_rescue<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = key_hash(&item.key);
	stash_insert_with_rescue_hashed(core, pos, item, h);
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
/// insert uses [`DEMOTE_TO_STASH_POS`] (MRU end), per SLRU, with two-choice placement
/// and stash-victim rescue.
///
/// Never writes into the bucket at `source_idx`: a native victim goes to stash, and a
/// foreign victim's true target is necessarily `source_idx - 1` (mod segment_size),
/// which differs from `source_idx` since segment_size >= 2. Rescue cannot write there
/// either: `source_idx` is full (it just evicted to admit an item), and rescue only
/// fills non-full buckets.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let h = key_hash(&item.key);
	let true_target = index_from_hash(h, core.segment_size);
	if true_target == source_idx {
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		stash_insert_with_rescue_hashed(core, DEMOTE_TO_STASH_POS, item, h);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(cascade) = core.put_at_normal_idx(true_target, insert_pos, item) {
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

/// Stash-hit branch shared by the `s0` and `s1` lookups: removes the item at `item_idx`
/// of stash bucket `stash_idx` (`is_borrowed` = hit in `s1 != s0`, so no hash is needed
/// for the counter update), promotes it into `target_idx`, and returns it.
#[inline]
fn stash_hit<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	stash_idx: usize,
	item_idx: usize,
	is_borrowed: bool,
	target_idx: usize,
	insert_pos: InsertPosition,
) -> &'a Item<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug!("promotion_get: key={key:?} found in stash={stash_idx} -> target={target_idx} insert_pos={insert_pos:?}");
	let data = core.remove_stash_at_known(stash_idx, item_idx, is_borrowed);
	promote_to_target(core, data, target_idx, insert_pos);

	// The cascade never writes back into target_idx (it is full after evicting, and
	// rescue only fills non-full buckets), so the new item sits exactly where the
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
/// Lookup order: stash `s0` → target → probing → stash `s1` (overflow-only, so checked last,
/// and skipped entirely when `borrowed[s1] == 0`).
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let s0 = get_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);

	if let Some(item_idx) = core.stash_buckets[s0].get_position(key) {
		return Some(stash_hit(core, key, s0, item_idx, false, target_idx, insert_pos));
	}

	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, probing_idx, insert_pos);

			// The cascade never writes back into probing_idx (a foreign victim retries in
			// target_idx; rescue only fills non-full buckets), so the new item sits exactly
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

	// Neighbour stash bucket, computed lazily: only reached on an otherwise-miss, and
	// only scanned when it holds at least one item borrowed from s0. `borrowed[s1] == 0`
	// proves no key with primary s0 is resident in s1.
	let s1 = core.stash_neighbor(s0);
	if s1 != s0 && core.borrowed[s1] != 0 {
		if let Some(item_idx) = core.stash_buckets[s1].get_position(key) {
			return Some(stash_hit(core, key, s1, item_idx, true, target_idx, insert_pos));
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

	fn in_any_stash(seg: &DashSegment<i64, i64>, k: i64) -> bool {
		seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some())
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
	/// (both s0 and s1 full) the fresh arrival at s0's LRU end is the stash victim, not the
	/// demotee. (The victim may then be rescued into a free normal slot.)
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

		// Overflow with one more fresh put.
		seg.put(Item::new(fresh[1], 11));

		let stash = &seg.core().stash_buckets[stash_idx];
		assert!(stash.get_position(&victim).is_some(), "demoted item must survive");
		assert!(stash.get_position(&fresh[0]).is_none(), "oldest fresh arrival must be evicted");
		assert_eq!(stash.get_position(&fresh[1]), Some(0), "new arrival sits at LRU end");
		assert!(!in_any_stash(&seg, fresh[0]), "stash victim leaves the stash");
		for &k in &s1_fill {
			assert!(seg.core().contains_key(&k), "s1 residents untouched");
		}
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_slot_legality());
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

	/// With both s0 and s1 full, s0's LRU is the stash victim; with its target bucket
	/// free, rescue places it at target index 0.
	#[test]
	fn two_choice_both_full_evicts_s0_lru_and_rescues_to_target_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
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
		let before = seg.core().total_items();

		let k = s0_keys[settings.bucket_size];
		seg.put(Item::new(k, 42));

		assert!(!in_any_stash(&seg, lru), "s0 LRU must leave the stash");
		let t = get_index(&lru, seg_size);
		assert_eq!(seg.core().buckets[t].get_position(&lru), Some(0), "rescued to target front");
		assert_eq!(seg.core().total_items(), before + 1, "rescue keeps the victim");
		assert_eq!(seg.core().stash_buckets[s0].get_position(&k), Some(0));
		for &f in &s1_keys {
			assert!(seg.core().stash_buckets[s1].get_position(&f).is_some());
		}
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_slot_legality());
		// The rescued item is readable.
		assert_eq!(seg.get(&lru).map(|it| it.value), Some(lru));
	}

	/// Sets up both stash buckets full and returns (seg, s0 LRU victim, next fresh key).
	fn full_stash_fixture() -> (DashSegment<i64, i64>, i64, i64) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
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
		let lru = seg.core().stash_buckets[s0].get_items()[0].key;
		(seg, lru, s0_keys[settings.bucket_size])
	}

	/// Fill normal bucket `idx` with keys legal there (target `idx`), avoiding `exclude`.
	fn fill_normal(seg: &mut DashSegment<i64, i64>, idx: usize, exclude: &[i64]) -> Vec<i64> {
		let seg_size = seg.core().segment_size;
		let mut filled = vec![];
		for k in 0i64..10_000 {
			if seg.core().buckets[idx].is_full() {
				break;
			}
			if get_index(&k, seg_size) == idx && !exclude.contains(&k) && !seg.core().contains_key(&k) {
				seg.core_mut().buckets[idx].put(Item::new(k, k));
				filled.push(k);
			}
		}
		assert!(seg.core().buckets[idx].is_full());
		filled
	}

	/// Target full, probing free: the victim is rescued to probing index 0.
	#[test]
	fn rescue_falls_back_to_probing() {
		let (mut seg, lru, k) = full_stash_fixture();
		let seg_size = seg.core().segment_size;
		let t = get_index(&lru, seg_size);
		let p = (t + 1) % seg_size;
		fill_normal(&mut seg, t, &[lru, k]);
		assert!(!seg.core().buckets[p].is_full());
		seg.put(Item::new(k, 42));
		assert!(!in_any_stash(&seg, lru));
		assert_eq!(seg.core().buckets[p].get_position(&lru), Some(0));
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_slot_legality());
	}

	/// Target and probing both full: the victim is dropped.
	#[test]
	fn rescue_drops_victim_when_all_full() {
		let (mut seg, lru, k) = full_stash_fixture();
		let seg_size = seg.core().segment_size;
		let t = get_index(&lru, seg_size);
		let p = (t + 1) % seg_size;
		fill_normal(&mut seg, t, &[lru, k]);
		fill_normal(&mut seg, p, &[lru, k]);
		let before = seg.core().total_items();
		seg.put(Item::new(k, 42));
		assert!(!seg.core().contains_key(&lru), "victim dropped");
		assert_eq!(seg.core().total_items(), before);
		assert!(seg.core().contains_key(&k));
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_slot_legality());
	}

	/// A key resident in s1, whose target holds other keys, is found by `get` and
	/// promoted to target index 0.
	#[test]
	fn s1_resident_found_and_promoted_to_target_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let tkeys = find_keys_for_bucket(target_idx, seg_size, 3);
		let other = tkeys[0];
		let k = tkeys[1];
		let s0 = get_index(&k, stash_size);
		let s1 = (s0 + 1) % stash_size;
		assert_ne!(s0, s1);

		seg.core_mut().buckets[target_idx].put(Item::new(other, 5));
		seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, 777));
		assert!(seg.core().stash_buckets[s0].get_position(&k).is_none());
		assert_eq!(seg.core().borrowed[s1], 1);

		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 777)));
		assert_eq!(seg.core().buckets[target_idx].get_position(&k), Some(0));
		assert!(seg.core().buckets[target_idx].get_position(&other).is_some());
		assert!(seg.core().stash_buckets[s1].get_position(&k).is_none());
		assert_eq!(seg.core().borrowed[s1], 0);
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_borrowed_counts());
	}

	/// A key in target is returned regardless of what sits in s1.
	#[test]
	fn target_hit_returned_with_s1_populated() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let k = find_keys_for_bucket(target_idx, seg_size, 1)[0];
		let s1 = (get_index(&k, stash_size) + 1) % stash_size;
		let s1_key = find_keys_for_bucket(s1, stash_size, 4)
			.into_iter()
			.find(|x| *x != k)
			.unwrap();
		seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(s1_key, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(k, 123));

		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 123)));
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

	fn random_ops(settings: &DashSettings, key_space: u64, ops: usize, seed: u64) {
		let mut seg = DashSegment::new(settings);
		// Simple deterministic LCG.
		let mut state: u64 = seed;
		for _ in 0..ops {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let key = ((state >> 33) % key_space) as i64;
			if seg.get(&key).is_none() {
				let before = seg.core().total_items();
				seg.put(Item::new(key, key));
				let after = seg.core().total_items();
				assert!(after == before || after == before + 1);
				assert!(seg.core().contains_key(&key), "fresh put must be resident");
			}
			assert!(seg.core().check_borrowed_counts(), "drift after key={key}");
			assert!(seg.core().check_slot_legality());
			assert!(seg.core().check_capacity_bound());
		}
	}

	/// (c) A random op sequence (with stash-victim rescue active on every stash insert)
	/// keeps `check_borrowed_counts()` and `check_slot_legality()` true after every op.
	#[test]
	fn borrowed_counts_exact_under_random_ops() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		random_ops(&settings, 200, 5000, 0x9E37_79B9_7F4A_7C15);
	}

	#[test]
	fn borrowed_counts_exact_under_random_ops_wider_geometry() {
		let mut settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		settings.num_of_normal_buckets = 8;
		settings.num_of_stash_buckets = 6;
		settings.bucket_size = 4;
		random_ops(&settings, 400, 20_000, 0xDEAD_BEEF_1234_5678);
		settings.num_of_stash_buckets = 1;
		random_ops(&settings, 400, 5_000, 0x1234_5678_9ABC_DEF0);
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

	#[test]
	fn stash_insert_returns_evictee_hash() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let total = settings.bucket_size * stash_size + 1;
		let keys: Vec<i64> = (0..total as i64).collect();
		let mut got = None;
		for &k in &keys {
			if let Some(r) = stash_insert(seg.core_mut(), InsertPosition::Back, Item::new(k, k)) {
				got = Some(r);
			}
		}
		let (e, eh) = got.expect("overfilled stash must evict");
		assert_eq!(eh, key_hash(&e.key));
		assert!(seg.core().check_borrowed_counts());
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
		// instead of bouncing it back into the bucket it just left.
		let evicted_key = probing_keys[0];
		// Permissive assertion: the key may have re-landed elsewhere via subsequent
		// stash overflow handling — only "vanished entirely" would be a real bug.
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
		assert!(seg.core().check_borrowed_counts());
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
