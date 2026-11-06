use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::record_hit;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Minimum number of slots in each segment's ghost table.
pub const GHOST_MIN_SLOTS: usize = 64;
/// Reserved fingerprint value marking an empty ghost slot.
const GHOST_EMPTY: u16 = 0;

/// splitmix64 finalizer, used to derive the ghost-table index from bits that
/// are independent of the fingerprint (top 16 bits of the raw hash).
#[inline]
fn splitmix64(mut h: u64) -> u64 {
	h ^= h >> 30;
	h = h.wrapping_mul(0xbf58476d1ce4e5b9);
	h ^= h >> 27;
	h = h.wrapping_mul(0x94d049bb133111eb);
	h ^= h >> 31;
	h
}

/// Per-segment direct-mapped table of fingerprints of keys recently dropped
/// from the stash (evicted out of the cache). A key re-put while its
/// fingerprint still sits in its slot is admitted with `lfu_counter = 1`
/// (2Q A1out / ARC-ghost style).
///
/// Variant cycle/0005/v001 (H01): the table is sized to the segment's resident
/// item capacity (rounded up to a power of two, floor `GHOST_MIN_SLOTS`), so
/// ghost reach matches cache reach. A collision overwrites the older entry,
/// which approximates FIFO ageing at table scale; lookup is one compare.
#[derive(Debug)]
struct Ghost {
	slots: Box<[u16]>,
	mask: usize,
}

impl Ghost {
	/// Allocate a table of at least `capacity` slots (power of two, floor
	/// `GHOST_MIN_SLOTS`), all empty.
	fn new(capacity: usize) -> Self {
		let len = capacity.max(GHOST_MIN_SLOTS).next_power_of_two();
		Ghost { slots: vec![GHOST_EMPTY; len].into_boxed_slice(), mask: len - 1 }
	}

	/// Slot index and 16-bit fingerprint for `key`, from a single hash.
	/// The fingerprint is the top 16 bits of the raw hash (unused by placement);
	/// a raw 0 is remapped to 1 so 0 can mark an empty slot. The index comes
	/// from a splitmix64 mix of the hash, independent of the fingerprint bits.
	fn ghost_key<K: Hash + Debug>(&self, key: &K) -> (usize, u16) {
		let h = hash(key) as u64;
		let fp = (h >> 48) as u16;
		let fp = if fp == GHOST_EMPTY { 1 } else { fp };
		let idx = (splitmix64(h) as usize) & self.mask;
		(idx, fp)
	}

	/// Record a fingerprint at `idx`, overwriting whatever was there.
	fn record(&mut self, idx: usize, fp: u16) {
		self.slots[idx] = fp;
	}

	/// Returns `true` if `slots[idx] == fp`, without clearing it.
	fn contains(&self, idx: usize, fp: u16) -> bool {
		self.slots[idx] == fp
	}

	/// Empties the slot at `idx`.
	fn clear(&mut self, idx: usize) {
		self.slots[idx] = GHOST_EMPTY;
	}
}

/// Outcome of the H02 (cycle 0008) bypass gate on a ghost hit's target bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BypassGate {
	/// Target has a free slot: bypass with no eviction.
	Free,
	/// Target is full but the victim scan found a cold (counter-0) slot: bypass,
	/// evicting the slot at this index.
	Cold(usize),
	/// Target is full and every tail-window slot is warm: no bypass.
	Blocked,
}

/// Single-pass gate + victim choice for a ghost-hit bypass into `bucket`.
/// The victim scan is the bucket policy's own eviction scan (for `ClassicLRU`:
/// the min-counter slot over the 8-slot LRU tail window, ties to the older
/// slot), so "the chosen victim has counter 0" is exactly "some tail-window
/// slot has counter 0". The returned index is the one evicted on the bypass,
/// so the window is read once.
fn bypass_gate<K, V>(bucket: &dyn Bucket<K, V>) -> BypassGate
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	if !bucket.is_full() {
		return BypassGate::Free;
	}
	match bucket.eviction_candidate_index() {
		Some(v) if bucket.get_items()[v].lfu_counter == 0 => BypassGate::Cold(v),
		Some(_) => BypassGate::Blocked,
		// A full bucket with no items only happens for max_size 0; nothing to evict.
		None => BypassGate::Free,
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
/// Variant cycle/0002/v002 (H02): every hit is counted in `lfu_counter`,
/// including the stash hit that promotes an item to target and the
/// hottest-in-target hit that promotes it to probing. The counter travels with
/// the `Item` through every promotion and demotion, so the frequency-aware
/// victim choice in the eviction policy sees an item's true reuse history.
///
/// Variant cycle/0004/v002 (H02): a per-segment ghost remembers the
/// fingerprints of keys dropped out of the stash. A key re-put while still
/// remembered is admitted with `lfu_counter = 1`.
///
/// Variant cycle/0005/v001 (H01): the ghost is a direct-mapped table sized to
/// the segment's item capacity instead of a 64-entry linear-scan ring.
///
/// Variant cycle/0006/v003 (H03): an item demoted from a normal bucket into
/// the stash has its `lfu_counter` halved at the tier boundary, so stale
/// normal-tier frequency no longer shields it from the stash tail scan.
///
/// Variant cycle/0007/v002 (H02): a ghost-hit `put` skips the stash and lands
/// directly in the key's target bucket at the LRU end (`Front`), exactly as a
/// stash→target promotion does (ARC/2Q: a ghost hit goes to the frequency
/// tier). Fresh keys still enter the stash at `Back` with counter 0.
///
/// Variant cycle/0008/v002 (H02): the stash bypass is selective. It is taken
/// only when the target has a free slot or its tail-window victim is cold
/// (counter 0); otherwise the ghost-hit key enters the stash at `Back` with
/// counter 1 and its ghost entry is kept, so a later return can retry.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
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
			let normal_cap = core.buckets.len() * core.buckets.first().map_or(0, |b| b.get_max_size());
			let stash_cap = core.stash_buckets.len() * core.stash_buckets.first().map_or(0, |b| b.get_max_size());
			let ghost = Ghost::new(normal_cap + stash_cap);
			info!("DashSegment.try_new: ghost_slots={}", ghost.slots.len());
			Some(Self { core, ghost })
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

	/// Fresh keys enter the stash at `Back` with counter 0. A key remembered by
	/// the ghost table is admitted with `lfu_counter = 1`. It bypasses the stash
	/// into its target bucket at `Front` only if the target has a free slot or a
	/// cold (counter-0) tail-window victim; the evicted victim then flows through
	/// the normal demotion cascade. Otherwise it enters the stash at `Back` and
	/// its ghost entry is kept.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let (idx, fp) = self.ghost.ghost_key(&item.key);
		if !self.ghost.contains(idx, fp) {
			debug!("DashSegment.put: key={:?}", item.key);
			put_into_stash(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
			return;
		}

		item.lfu_counter = 1;
		let target_idx = get_index(&item.key, self.core.segment_size);
		let gate = bypass_gate(&*self.core.buckets[target_idx]);
		if gate == BypassGate::Blocked {
			debug!("DashSegment.put: key={:?} ghost hit, target={target_idx} warm -> stash, lfu_counter=1", item.key);
			put_into_stash(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
			return;
		}

		debug!("DashSegment.put: key={:?} ghost hit -> target={target_idx} ({gate:?}), lfu_counter=1", item.key);
		self.ghost.clear(idx);
		let key = item.key;
		let evicted = match gate {
			BypassGate::Cold(v) => Some(self.core.buckets[target_idx].remove_at(v)),
			_ => None,
		};
		// The bucket now has room, so this insert does not run a second victim scan.
		let (_, extra) = self.core.buckets[target_idx].put_at(item, InsertPosition::Front);
		debug_assert!(extra.is_none(), "put: bypass insert into target={target_idx} must not evict");
		if let Some(victim) = evicted {
			demote_from_normal_bucket(&mut self.core, &mut self.ghost, victim, target_idx, InsertPosition::Front);
		}
		debug_assert!(
			self.core.buckets[target_idx].get_position(&key).is_some(),
			"put: ghost-hit key={key:?} must be resident in target={target_idx}"
		);
	}
}

/// Inserts `item` into its stash bucket; any stash victim leaves the cache, so
/// its fingerprint is recorded in the ghost table.
fn put_into_stash<K, V>(core: &mut SegmentCore<K, V>, ghost: &mut Ghost, insert_pos: InsertPosition, item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	if let Some(dropped) = core.put_at(Slot::Stash, insert_pos, item) {
		let (idx, fp) = ghost.ghost_key(&dropped.key);
		ghost.record(idx, fp);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, ghost: &mut Ghost, item: Item<K, V>, insert_pos: InsertPosition)
where
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
fn promote_to_probing<K, V>(core: &mut SegmentCore<K, V>, ghost: &mut Ghost, item: Item<K, V>, insert_pos: InsertPosition)
where
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

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
/// Normal-bucket victims stay resident (demoted to stash) and are not ghosted; only
/// the stash item those demotions push out of the cache is.
///
/// H03: an item crossing from the normal tier into the stash has its
/// `lfu_counter` halved (tier-boundary ageing). The probing→own-target retry
/// stays in the normal tier and keeps its counter.
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
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		item.lfu_counter /= 2;
		put_into_stash(core, ghost, insert_pos, item);
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
			cascade.lfu_counter /= 2;
			put_into_stash(core, ghost, insert_pos, cascade);
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
		debug!("promotion_get: key={key:?} found in stash -> target={target_idx} insert_pos={insert_pos:?}");
		// Count the stash hit (with bucket ageing) before the item leaves stash,
		// so it carries a non-zero frequency into the normal tier.
		record_hit(core.stash_buckets[stash_idx].get_items_mut(), item_idx);
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
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
			// Count the promoting hit too; the counter travels with the item.
			record_hit(core.buckets[target_idx].get_items_mut(), item_idx);
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

	fn stash_counter(seg: &DashSegment<i64, i64>, key: i64) -> Option<usize> {
		let idx = get_index(&key, seg.core.stash_size);
		let pos = seg.core.stash_buckets[idx].get_position(&key)?;
		Some(seg.core.stash_buckets[idx].get_items()[pos].lfu_counter)
	}

	fn target_counter(seg: &DashSegment<i64, i64>, key: i64) -> Option<usize> {
		let idx = get_index(&key, seg.core.segment_size);
		let pos = seg.core.buckets[idx].get_position(&key)?;
		Some(seg.core.buckets[idx].get_items()[pos].lfu_counter)
	}

	fn is_ghosted(seg: &DashSegment<i64, i64>, key: i64) -> bool {
		let (idx, fp) = seg.ghost.ghost_key(&key);
		seg.ghost.contains(idx, fp)
	}

	/// Fill target 0 to capacity with counters `first` (LRU end) and `rest`,
	/// then ghost one more key mapping to target 0. Returns (target snapshot, ghosted key).
	fn full_target_with_ghosted_key(
		seg: &mut DashSegment<i64, i64>,
		first: usize,
		rest: usize,
	) -> (Vec<(i64, usize)>, i64) {
		let target_idx = 0;
		let seg_size = seg.core.segment_size;
		let stash_size = seg.core.stash_size;
		let cap = seg.core.buckets[target_idx].get_max_size();

		let fillers = find_keys_for_bucket(target_idx, seg_size, cap);
		assert_eq!(fillers.len(), cap);
		for &k in &fillers {
			seg.put(Item::new(k, k));
			seg.get(&k);
		}
		assert_eq!(seg.core.buckets[target_idx].size(), cap);
		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			items[0].lfu_counter = first;
			for it in items.iter_mut().skip(1) {
				it.lfu_counter = rest;
			}
		}
		let before: Vec<(i64, usize)> =
			seg.core.buckets[target_idx].get_items().iter().map(|it| (it.key, it.lfu_counter)).collect();

		let stash_cap = seg.core.stash_buckets[0].get_max_size();
		let mut candidates = (0i64..100_000).filter(|k| !fillers.contains(k) && get_index(k, seg_size) == target_idx);
		let first_key = candidates.next().expect("candidate key");
		let s = get_index(&first_key, stash_size);
		let mut overflow = vec![first_key];
		overflow.extend(candidates.filter(|k| get_index(k, stash_size) == s).take(stash_cap));
		assert_eq!(overflow.len(), stash_cap + 1);
		for &k in &overflow {
			seg.put(Item::new(k, k));
		}
		let dropped: Vec<i64> = overflow.iter().copied().filter(|k| !seg.core.contains_key(k)).collect();
		assert_eq!(dropped.len(), 1, "exactly one key must have overflowed the stash bucket");
		assert_eq!(seg.core.buckets[target_idx].size(), cap, "target untouched by stash puts");
		(before, dropped[0])
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	#[test]
	fn fresh_key_starts_with_counter_zero() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(7_i64, 70_i64));
		assert_eq!(stash_counter(&seg, 7), Some(0));
	}

	#[test]
	fn fresh_put_still_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let key = 42_i64;
		seg.put(Item::new(key, key));
		assert_eq!(stash_counter(&seg, key), Some(0), "fresh key must land in stash with counter 0");
		assert_eq!(target_counter(&seg, key), None, "fresh key must not land in target");
		let target_idx = get_index(&key, seg.core.segment_size);
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		assert!(seg.core.buckets[probing_idx].get_position(&key).is_none());
	}

	#[test]
	fn key_evicted_from_stash_and_reput_starts_with_counter_one() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let cap = seg.core.stash_buckets[0].get_max_size();
		let keys = find_keys_for_bucket(0, seg.core.stash_size, cap + 1);
		for &k in &keys {
			seg.put(Item::new(k, k));
		}
		let dropped: Vec<i64> = keys.iter().copied().filter(|k| !seg.core.contains_key(k)).collect();
		assert_eq!(dropped.len(), 1, "exactly one key must have overflowed the stash bucket");
		let victim = dropped[0];
		// Target has free room, so the bypass is taken.
		seg.put(Item::new(victim, victim));
		let t = get_index(&victim, seg.core.segment_size);
		let pos = seg.core.buckets[t].get_position(&victim).expect("ghost-hit key must land in its target bucket");
		assert_eq!(seg.core.buckets[t].get_items()[pos].lfu_counter, 1, "ghost hit must admit with counter 1");
		assert_eq!(stash_counter(&seg, victim), None, "ghost-hit key must not also be in stash");
		assert!(!is_ghosted(&seg, victim), "bypass consumes the ghost entry");
		assert_eq!(seg.get(&victim).map(|it| it.value), Some(victim));
	}

	#[test]
	fn ghost_hit_into_warm_full_target_falls_back_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (before, ghosted) = full_target_with_ghosted_key(&mut seg, 6, 7);
		let stash_idx = get_index(&ghosted, seg.core.stash_size);
		let stash_before: Vec<i64> = seg.core.stash_buckets[stash_idx].get_items().iter().map(|it| it.key).collect();

		seg.put(Item::new(ghosted, ghosted));
		assert_eq!(target_counter(&seg, ghosted), None, "warm target must block the bypass");
		assert_eq!(stash_counter(&seg, ghosted), Some(1), "fallback lands in stash with counter 1");
		// The fallback insert may drop another stash key, whose fingerprint could
		// overwrite the same ghost slot; only check retention when it does not.
		let (gidx, _) = seg.ghost.ghost_key(&ghosted);
		let overwritten = stash_before
			.iter()
			.copied()
			.filter(|k| !seg.core.contains_key(k))
			.any(|k| seg.ghost.ghost_key(&k).0 == gidx);
		if !overwritten {
			assert!(is_ghosted(&seg, ghosted), "fallback keeps the ghost entry");
		}

		let after: Vec<(i64, usize)> =
			seg.core.buckets[0].get_items().iter().map(|it| (it.key, it.lfu_counter)).collect();
		assert_eq!(before, after, "target must be untouched by the fallback");
		assert!(seg.core.check_slot_legality());
		assert!(seg.core.check_capacity_bound());
	}

	#[test]
	fn ghost_hit_into_full_target_with_cold_slot_bypasses() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (before, ghosted) = full_target_with_ghosted_key(&mut seg, 0, 7);
		let target_idx = 0;

		seg.put(Item::new(ghosted, ghosted));
		assert_eq!(target_counter(&seg, ghosted), Some(1));
		assert_eq!(stash_counter(&seg, ghosted), None);
		assert_eq!(seg.core.buckets[target_idx].get_items()[0].key, ghosted, "bypass lands at Front");
		assert!(!is_ghosted(&seg, ghosted));

		let evicted: Vec<(i64, usize)> =
			before.iter().copied().filter(|(k, _)| seg.core.buckets[target_idx].get_position(k).is_none()).collect();
		assert_eq!(evicted.len(), 1, "exactly one resident must have been demoted");
		let (demoted, old) = evicted[0];
		assert_eq!(old, 0, "the cold resident must be the victim");
		assert_eq!(stash_counter(&seg, demoted), Some(0));
		assert!(seg.core.check_slot_legality());
		assert!(seg.core.check_capacity_bound());
	}

	#[test]
	fn bypass_gate_reports_free_cold_blocked() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert_eq!(bypass_gate(&*seg.core.buckets[0]), BypassGate::Free);
		let cap = seg.core.buckets[0].get_max_size();
		let keys = find_keys_for_bucket(0, seg.core.segment_size, cap);
		for &k in &keys {
			seg.core.buckets[0].put(Item::new(k, k));
		}
		for it in seg.core.buckets[0].get_items_mut().iter_mut() {
			it.lfu_counter = 3;
		}
		assert_eq!(bypass_gate(&*seg.core.buckets[0]), BypassGate::Blocked);
		let w = cap.min(crate::shared::eviction_policy::LRU_TAIL_WINDOW) - 1;
		seg.core.buckets[0].get_items_mut()[w].lfu_counter = 0;
		assert_eq!(bypass_gate(&*seg.core.buckets[0]), BypassGate::Cold(w));
	}

	#[test]
	fn ghost_collision_keeps_only_latest() {
		let mut g = Ghost::new(GHOST_MIN_SLOTS);
		let idx = 5;
		g.record(idx, 111);
		g.record(idx, 222);
		assert!(!g.contains(idx, 111), "older fingerprint at the same index must be overwritten");
		assert!(g.contains(idx, 222));
		g.clear(idx);
		assert!(!g.contains(idx, 222), "a cleared slot no longer matches");
	}

	#[test]
	fn ghost_table_is_power_of_two_and_covers_capacity() {
		let seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let len = seg.ghost.slots.len();
		assert!(len.is_power_of_two(), "ghost table length {len} must be a power of two");
		assert!(len >= seg.core.max_capacity(), "ghost table length {len} must cover item capacity");
		assert!(len >= GHOST_MIN_SLOTS);
		assert_eq!(seg.ghost.mask, len - 1);

		for cap in [0usize, 1, 63, 64, 65, 2048, 3000] {
			let g = Ghost::new(cap);
			assert!(g.slots.len().is_power_of_two());
			assert!(g.slots.len() >= cap.max(GHOST_MIN_SLOTS));
		}
	}

	#[test]
	fn fingerprint_never_empty() {
		let g = Ghost::new(GHOST_MIN_SLOTS);
		for k in 0..10_000_i64 {
			let (idx, fp) = g.ghost_key(&k);
			assert_ne!(fp, GHOST_EMPTY);
			assert!(idx < g.slots.len());
		}
	}

	#[test]
	fn stash_hit_counts_and_carries_counter() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		seg.put(Item::new(keys[0], 10));
		let c = seg.get(&keys[0]).unwrap().lfu_counter;
		assert_eq!(c, 1, "stash hit must be counted and survive promotion");
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_some());
	}

	#[test]
	fn demotion_to_stash_halves_counter() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let cap = seg.core.buckets[target_idx].get_max_size();
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, cap + 1);
		assert_eq!(keys.len(), cap + 1, "need enough keys for target bucket");

		for &k in &keys[..cap] {
			seg.put(Item::new(k, k));
			let c = seg.get(&k).unwrap().lfu_counter;
			assert!(c >= 1);
		}
		assert_eq!(seg.core.buckets[target_idx].size(), cap);

		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			items[0].lfu_counter = 6;
			for it in items.iter_mut().skip(1) {
				it.lfu_counter = 7;
			}
		}
		let before: Vec<(i64, usize)> =
			seg.core.buckets[target_idx].get_items().iter().map(|it| (it.key, it.lfu_counter)).collect();

		let newcomer = keys[cap];
		seg.put(Item::new(newcomer, newcomer));
		seg.get(&newcomer);
		assert!(seg.core.buckets[target_idx].get_position(&newcomer).is_some());

		let evicted: Vec<(i64, usize)> =
			before.iter().copied().filter(|(k, _)| seg.core.buckets[target_idx].get_position(k).is_none()).collect();
		assert_eq!(evicted.len(), 1, "exactly one resident must have been demoted");
		let (demoted, old) = evicted[0];
		assert_eq!(old, 6, "the counter-6 resident must be the victim");
		assert_eq!(stash_counter(&seg, demoted), Some(3), "demotion to stash must halve the counter");
		assert_eq!(stash_counter(&seg, demoted), Some(old / 2));
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
		seg.put(Item::new(keys[1], 20));
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
		seg.put(Item::new(keys[1], 20));
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
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}
}
