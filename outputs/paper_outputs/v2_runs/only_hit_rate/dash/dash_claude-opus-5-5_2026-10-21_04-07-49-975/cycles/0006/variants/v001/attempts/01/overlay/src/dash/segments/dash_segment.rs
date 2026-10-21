use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Number of stash hits an item needs before it is promoted to its target bucket.
/// Hits below the threshold are served in place in stash (touching stash LRU order).
/// `Item::lfu_counter` is reused as the per-item stash-hit counter.
const PROMOTE_THRESHOLD: usize = 2;

/// Bimodal insertion period for fresh keys entering the stash (BIP, Qureshi et al.).
/// One in every `BIP_PERIOD` fresh puts lands at the stash MRU end; all others land
/// at the LRU end, where they are the first eviction candidates unless reused.
const BIP_PERIOD: u32 = 16;

/// Multiplier on the ghost table size relative to the segment's slot count (rounded
/// up to a power of two). Must be a power of two so the table size stays a power of
/// two and `idx & mask` indexing remains valid. `4` ≈ four caches' worth of
/// stash-eviction fingerprints summed over all segments.
const GHOST_SCALE: usize = 4;
const _: () = assert!(GHOST_SCALE.is_power_of_two());

/// Ghost table slot count for a segment of `capacity` item slots:
/// `next_power_of_two(capacity) × GHOST_SCALE` (always a power of two).
#[inline]
fn ghost_slots_for(capacity: usize) -> usize {
	capacity.max(1).next_power_of_two() * GHOST_SCALE
}

// ── Ghost fingerprint table ────────────────────────────────────────────────

/// Per-segment direct-mapped table of 16-bit fingerprints of keys recently evicted
/// out of the cache (i.e. out of stash). `0` means empty.
///
/// Sized to `GHOST_SCALE` (= 4) × the segment's total slot count (normal + stash)
/// rounded up to a power of two, so summed over all segments it covers roughly four
/// caches' worth of evictions. Direct-mapped overwrite is the only aging mechanism.
///
/// **Added metadata:** `2 bytes × slots()` per segment, i.e.
/// `2 × GHOST_SCALE × next_pow2(segment slots)` (heap, allocated once at
/// construction; lookups and writes never allocate). See [`Ghost::metadata_bytes`]
/// and [`DashSegment::ghost_metadata_bytes`].
pub(crate) struct Ghost {
	table: Box<[u16]>,
	mask: usize,
}

impl Debug for Ghost {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let used = self.table.iter().filter(|&&fp| fp != 0).count();
		f.debug_struct("Ghost").field("slots", &self.table.len()).field("used", &used).finish()
	}
}

impl Ghost {
	fn new(slots: usize) -> Self {
		let slots = slots.max(1).next_power_of_two();
		Self { table: vec![0u16; slots].into_boxed_slice(), mask: slots - 1 }
	}

	/// Secondary hash: the bucket hash remixed with a Fibonacci multiplier, then the
	/// index taken from bits 32.. and the fingerprint from bits 48..63 — different
	/// bits from the `hash % size` used for bucket selection.
	#[inline]
	fn locate<K: Hash + Debug>(&self, key: &K) -> (usize, u16) {
		let h = (hash(key) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		let idx = ((h >> 32) as usize) & self.mask;
		let fp = (h >> 48) as u16;
		(idx, if fp == 0 { 1 } else { fp })
	}

	/// Records `key` as evicted, overwriting whatever occupied its slot.
	#[inline]
	fn record<K: Hash + Debug>(&mut self, key: &K) {
		let (idx, fp) = self.locate(key);
		self.table[idx] = fp;
	}

	/// Returns `true` (and clears the slot) if `key`'s fingerprint is present.
	#[inline]
	fn take<K: Hash + Debug>(&mut self, key: &K) -> bool {
		let (idx, fp) = self.locate(key);
		if self.table[idx] == fp {
			self.table[idx] = 0;
			true
		} else {
			false
		}
	}

	pub(crate) fn slots(&self) -> usize {
		self.table.len()
	}

	/// Heap bytes held by the ghost table (2 bytes per slot).
	pub(crate) fn metadata_bytes(&self) -> usize {
		self.table.len() * std::mem::size_of::<u16>()
	}
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
/// Stash admission filter: an item must be hit `PROMOTE_THRESHOLD` times in stash
/// before it is moved into its target bucket. Items demoted out of the normal tier
/// re-enter stash at the MRU end, so the stash acts as a victim cache for ex-residents.
///
/// Stash victim selection is CLOCK-like second chance from the LRU end.
///
/// Fresh keys (from `put`) enter stash bimodally (BIP), **unless** their fingerprint
/// is found in the segment's ghost table of recent stash evictions: such returning
/// keys enter at the stash MRU end (counter 0), overriding BIP.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Per-segment counter of fresh (non-ghost) puts, driving bimodal stash insertion.
	bip_tick: u32,
	/// Fingerprints of keys recently evicted out of stash.
	ghost: Ghost,
}

impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new DashSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			let ghost = Ghost::new(ghost_slots_for(core.max_capacity()));
			info!(
				"DashSegment.try_new: ghost_slots={} ghost_metadata_bytes={}",
				ghost.slots(),
				ghost.metadata_bytes()
			);
			Some(Self { core, bip_tick: 0, ghost })
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

	/// Added per-segment metadata from the ghost table: `2 × ghost slots` bytes
	/// (= `2 × GHOST_SCALE × next_pow2(segment slots)`).
	pub fn ghost_metadata_bytes(&self) -> usize {
		self.ghost.metadata_bytes()
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

	/// Fresh keys go to stash. A ghost hit (the key was recently evicted out of stash)
	/// lands at the MRU end (`Back`) with counter 0 and clears the ghost slot. Otherwise
	/// BIP: every `BIP_PERIOD`-th put lands at `Back`, all others at `Front`. A full
	/// stash bucket picks its victim through `stash_put`'s second-chance scan *before*
	/// the insert, so the incoming key itself is never the victim.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let pos = if self.ghost.take(&item.key) {
			item.lfu_counter = 0;
			InsertPosition::Back
		} else {
			self.bip_tick = self.bip_tick.wrapping_add(1);
			if self.bip_tick % BIP_PERIOD == 0 {
				InsertPosition::Back
			} else {
				InsertPosition::Front
			}
		};
		debug!("DashSegment.put: key={:?} pos={pos:?}", item.key);
		stash_put(&mut self.core, &mut self.ghost, pos, item);
	}
}

// ── Stash victim selection ─────────────────────────────────────────────────

/// Inserts `item` at `pos` in its stash bucket. If the bucket is full, a victim is
/// chosen first by a bounded second-chance scan from the LRU end (index 0) toward
/// the MRU end: the first slot with a stash-hit counter of `0` is evicted; every
/// slot skipped has its counter decremented (never below `0`). If no slot has a
/// zero counter, the policy's eviction candidate is evicted. The evicted item leaves
/// the cache completely, so its fingerprint is recorded in `ghost`.
/// Returns the evicted item, if any.
fn stash_put<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
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
		ghost.record(&e.key);
	}
	out
}

// ── Promotion logic ────────────────────────────────────────────────────────

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
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
	demote_from_normal_bucket(core, ghost, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket. Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
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
	demote_from_normal_bucket(core, ghost, evicted, probing_idx, insert_pos);
}

/// Insert position for items demoted from the normal tier back into stash (MRU end):
/// the stash acts as a victim cache for ex-residents.
const DEMOTE_TO_STASH_POS: InsertPosition = InsertPosition::Back;

/// One-more-chance demotion. If the item was a foreign resident at `source_idx`, retry
/// in its own true target before stash; if it was already in its target, go straight
/// to stash. Items demoted to stash are pre-credited with `PROMOTE_THRESHOLD - 1`.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
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
		stash_put(core, ghost, DEMOTE_TO_STASH_POS, item);
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
			stash_put(core, ghost, DEMOTE_TO_STASH_POS, cascade);
		}
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	ghost: &mut Ghost,
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
		promote_to_target(core, ghost, data, insert_pos);

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
			promote_to_probing(core, ghost, data, insert_pos);

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
	fn ghost_sized_to_scaled_segment_capacity_pow2() {
		let seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let cap = seg.core().max_capacity();
		let expected = cap.next_power_of_two() * GHOST_SCALE;
		assert!(expected.is_power_of_two());
		assert_eq!(seg.ghost.slots(), expected);
		assert_eq!(seg.ghost_metadata_bytes(), 2 * expected);
	}

	#[test]
	fn ghost_record_take_roundtrip() {
		let mut g = Ghost::new(64);
		assert!(!g.take(&42_i64));
		g.record(&42_i64);
		assert!(g.take(&42_i64), "recorded key must match");
		assert!(!g.take(&42_i64), "match clears the slot");
	}

	/// A key evicted out of stash is recorded in the ghost; when it returns as a fresh
	/// put it lands at the MRU end with counter 0, overriding BIP.
	#[test]
	fn ghost_hit_lands_at_mru_end() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 2);
		for &k in &keys[..bsize] {
			seg.core_mut().stash_buckets[0].put(Item::new(k, k));
		}
		seg.bip_tick = 0;
		seg.put(Item::new(keys[bsize], 0)); // evicts keys[0] -> ghost
		assert!(!seg.core().contains_key(&keys[0]));
		// Make room without touching the ghost (direct removal of keys[bsize]).
		let removed = seg.core_mut().stash_buckets[0].remove_at(0);
		assert_eq!(removed.key, keys[bsize]);
		let tick_before = seg.bip_tick;
		seg.put(Item::new(keys[0], 7));
		let b = &seg.core().stash_buckets[0];
		let last = b.get_items().last().unwrap();
		assert_eq!(last.key, keys[0], "ghost hit must land at MRU end");
		assert_eq!(last.lfu_counter, 0, "ghost hit is not pre-credited");
		assert_eq!(b.size(), bsize);
		assert_eq!(seg.bip_tick, tick_before, "ghost hit does not advance BIP");
	}

	/// BIP: a non-period fresh put into a full stash bucket lands at the LRU end
	/// (index 0), evicting exactly one item (the scan's victim) and no more.
	#[test]
	fn bip_fresh_put_lands_at_lru_end() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 1);
		for &k in &keys[..bsize] {
			seg.core_mut().stash_buckets[0].put(Item::new(k, k));
		}
		assert!(seg.core().stash_buckets[0].is_full());
		seg.bip_tick = 0; // next put -> tick 1 -> Front path
		seg.put(Item::new(keys[bsize], 0));

		let b = &seg.core().stash_buckets[0];
		assert_eq!(b.get_items()[0].key, keys[bsize], "fresh key must sit at LRU end");
		assert_eq!(b.size(), bsize, "exactly one eviction");
		assert!(b.get_position(&keys[0]).is_none(), "LRU zero-counter item is the victim");
		for &k in &keys[1..bsize] {
			assert!(b.get_position(&k).is_some(), "no further eviction");
		}
	}

	/// BIP: every `BIP_PERIOD`-th fresh put lands at the MRU end.
	#[test]
	fn bip_period_put_lands_at_mru_end() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let bsize = settings.bucket_size;
		let keys = find_keys_for_bucket(0, stash_size, bsize + 1);
		for &k in &keys[..bsize] {
			seg.core_mut().stash_buckets[0].put(Item::new(k, k));
		}
		seg.bip_tick = BIP_PERIOD - 1; // next put is the 16th -> Back path
		seg.put(Item::new(keys[bsize], 0));

		let b = &seg.core().stash_buckets[0];
		assert_eq!(b.get_items()[bsize - 1].key, keys[bsize], "16th put must sit at MRU end");
		assert_eq!(b.size(), bsize);
	}

	/// BIP cadence from a fresh segment: puts 1..15 take the Front path, put 16 takes Back.
	#[test]
	fn bip_cadence_from_fresh_segment() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::<i64, i64>::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let keys = find_keys_for_bucket(0, stash_size, BIP_PERIOD as usize);
		assert_eq!(keys.len(), BIP_PERIOD as usize);
		for &k in &keys[..BIP_PERIOD as usize - 1] {
			seg.put(Item::new(k, k));
			let b = &seg.core().stash_buckets[0];
			assert_eq!(b.get_items()[0].key, k, "non-period put must land at LRU end");
		}
		let last = keys[BIP_PERIOD as usize - 1];
		seg.put(Item::new(last, last));
		let b = &seg.core().stash_buckets[0];
		assert_eq!(b.get_items().last().unwrap().key, last, "16th put must land at MRU end");
	}

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
		seg.core_mut().stash_buckets[0].get_items_mut()[0].lfu_counter = 1;
		seg.put(Item::new(keys[bsize], 0));

		let b = &seg.core().stash_buckets[0];
		let p0 = b.get_position(&keys[0]).expect("credited LRU item must be protected");
		assert_eq!(b.get_items()[p0].lfu_counter, 0, "skipped item must be aged");
		if bsize >= 2 {
			assert!(b.get_position(&keys[1]).is_none(), "first zero-counter item is the victim");
		}
		assert!(b.get_position(&keys[bsize]).is_some());
		assert_eq!(b.size(), bsize);
	}

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

		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

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

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		seg.get(&keys[1]);
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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 200), InsertPosition::Back);

		let in_target = seg.core().buckets[target_idx].get_position(&k).is_some();
		let in_stash = seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some());
		let still_in_probing = seg.core().buckets[probing_idx].get_position(&k).is_some();

		assert!(!still_in_probing, "sanity: k was evicted from its probing bucket");
		assert!(in_target, "promotion demotion from probing must place item into its target bucket");
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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos);

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

		promote_to_target(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos);

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

		promote_to_probing(&mut seg.core, &mut seg.ghost, Item::new(trigger, 99), insert_pos);

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
