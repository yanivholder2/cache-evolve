use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::record_hit;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
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

	/// Returns `true` and clears the slot if `slots[idx] == fp`.
	fn take(&mut self, idx: usize, fp: u16) -> bool {
		let slot = &mut self.slots[idx];
		if *slot == fp {
			*slot = GHOST_EMPTY;
			true
		} else {
			false
		}
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
/// remembered enters the stash (same position as before) with
/// `lfu_counter = 1`, so it outlasts one-touch arrivals in the tail scan.
///
/// Variant cycle/0005/v001 (H01): the ghost is a direct-mapped table sized to
/// the segment's item capacity instead of a 64-entry linear-scan ring.
///
/// Variant cycle/0006/v003 (H03): an item demoted from a normal bucket into
/// the stash has its `lfu_counter` halved at the tier boundary, so stale
/// normal-tier frequency no longer shields it from the stash tail scan.
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

	/// Same placement as the default (stash, `Back`); only the admission
	/// counter changes when the key is remembered by the ghost table.
	fn put(&mut self, mut item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let (idx, fp) = self.ghost.ghost_key(&item.key);
		if self.ghost.take(idx, fp) {
			debug!("DashSegment.put: key={:?} ghost hit -> lfu_counter=1", item.key);
			item.lfu_counter = 1;
		} else {
			debug!("DashSegment.put: key={:?}", item.key);
		}
		put_into_stash(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
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
		seg.put(Item::new(victim, victim));
		assert_eq!(stash_counter(&seg, victim), Some(1), "ghost hit must admit with counter 1");
	}

	#[test]
	fn ghost_collision_keeps_only_latest() {
		let mut g = Ghost::new(GHOST_MIN_SLOTS);
		let idx = 5;
		g.record(idx, 111);
		g.record(idx, 222);
		assert!(!g.take(idx, 111), "older fingerprint at the same index must be overwritten");
		assert!(g.take(idx, 222));
		assert!(!g.take(idx, 222), "a matched slot is cleared");
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

		// Fill the target bucket: each get promotes stash -> target with counter >= 1.
		for &k in &keys[..cap] {
			seg.put(Item::new(k, k));
			let c = seg.get(&k).unwrap().lfu_counter;
			assert!(c >= 1);
		}
		assert_eq!(seg.core.buckets[target_idx].size(), cap);

		// Give the LRU-end resident a counter of 6; every other resident gets a
		// higher count so the 6 is unambiguously the tail-scan victim.
		{
			let items = seg.core.buckets[target_idx].get_items_mut();
			items[0].lfu_counter = 6;
			for it in items.iter_mut().skip(1) {
				it.lfu_counter = 7;
			}
		}
		let before: Vec<(i64, usize)> =
			seg.core.buckets[target_idx].get_items().iter().map(|it| (it.key, it.lfu_counter)).collect();

		// Promote one more key into the full target to force a demotion.
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
