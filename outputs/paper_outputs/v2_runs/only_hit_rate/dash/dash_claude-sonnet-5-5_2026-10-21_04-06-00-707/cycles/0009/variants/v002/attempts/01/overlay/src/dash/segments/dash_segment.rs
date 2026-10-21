use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Number of one-hit-wonder filter slots (u8 each => 512 bytes per segment). Power of two.
const FILTER_SLOTS: usize = 512;

/// Ghost table entries (direct-mapped). Power of two.
const GHOST_SLOTS: usize = 4096;
/// Valid marker bit in a ghost entry: [valid:1 @16+2][fp:16 @2][counter:2 @0].
const GHOST_VALID: u32 = 1 << 18;

/// Maximum value of the saturating reuse counter (2 bits).
const MAX_REUSE: usize = 3;

type Ghost = [u32; GHOST_SLOTS];

#[inline]
fn ghost_slot_fp(key_hash: usize) -> (usize, u32) {
	let key_hash = key_hash as u64;
	let slot = ((key_hash >> 40) as usize) & (GHOST_SLOTS - 1);
	let fp = ((key_hash >> 16) & 0xFFFF) as u32;
	(slot, fp)
}

/// The dash Subject's segment (variant: MRU-insert promotion + MRU-insert admission
/// + one-hit-wonder admission filter + 2-bit reuse counter with aging
/// + ghost table restoring the reuse counter of recently evicted keys
/// + scan resistance: a fully-hot stash bucket is not displaced by a second-sighting key).
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Fingerprints of keys seen once while the stash bucket was full. 0 = empty.
	filter: [u8; FILTER_SLOTS],
	/// Ghost of evicted victims: fingerprint + counter at eviction time.
	ghost: Box<Ghost>,
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
			Some(Self {
				core,
				filter: [0; FILTER_SLOTS],
				ghost: Box::new([0; GHOST_SLOTS]),
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
		promotion_get(&mut self.core, &mut self.ghost, key, InsertPosition::Back)
	}

	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let mut init_counter = 0usize;
		let stash_idx = get_index(&item.key, self.core.stash_size);
		if self.core.stash_buckets[stash_idx].is_full() {
			// Eviction would be needed: apply the one-hit-wonder filter.
			// Use high hash bits (distinct from the low bits used for bucket selection).
			let h = hash(&item.key);
			let slot = (h >> 32) & (FILTER_SLOTS - 1);
			let fp = (((h >> 16) & 0xFF) as u8) | 1;
			if self.filter[slot] != fp {
				debug!("DashSegment.put: key={:?} first sighting, skip", item.key);
				self.filter[slot] = fp;
				return;
			}

			// Scan resistance: if every resident is hot (counter > 0), drop the
			// newcomer and leave its filter bit set so a further sighting is needed.
			let all_hot = self.core.stash_buckets[stash_idx]
				.get_items()
				.iter()
				.all(|i| i.lfu_counter > 0);
			if all_hot {
				debug!("DashSegment.put: key={:?} bucket fully hot, drop", item.key);
				return;
			}
			self.filter[slot] = 0;

			// Passed the filter: consult the ghost table.
			let (gs, gfp) = ghost_slot_fp(h);
			let e = self.ghost[gs];
			if e & GHOST_VALID != 0 && ((e >> 2) & 0xFFFF) == gfp {
				init_counter = ((e & 3) as usize).max(1);
				self.ghost[gs] = 0;
			}
		}
		debug!("DashSegment.put: key={:?} -> stash MRU", item.key);
		let mut item = item;
		item.lfu_counter = init_counter;
		put_stash(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
	}
}

// ── Reuse-counter victim selection ─────────────────────────────────────────

/// Saturating increment of the reuse counter on a hit.
#[inline]
fn bump<K, V>(item: &mut Item<K, V>)
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	if item.lfu_counter < MAX_REUSE {
		item.lfu_counter += 1;
	}
}

/// Record an evicted victim in the ghost table.
#[inline]
fn record_ghost<K, V>(ghost: &mut Ghost, victim: &Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let (gs, gfp) = ghost_slot_fp(hash(&victim.key));
	ghost[gs] = GHOST_VALID | (gfp << 2) | (victim.lfu_counter.min(MAX_REUSE) as u32);
}

/// Inserts `item` into its stash bucket. If the bucket is full, first evicts the first
/// slot (scan order) whose counter is 0; if none, ages every slot by 1 and evicts the
/// first slot that reached 0 (or the first slot if none did).
fn put_stash<K, V>(core: &mut SegmentCore<K, V>, ghost: &mut Ghost, pos: InsertPosition, item: Item<K, V>)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let idx = get_index(&item.key, core.stash_size);
	let bucket = &mut core.stash_buckets[idx];
	if bucket.is_full() {
		let victim = bucket.get_items().iter().position(|i| i.lfu_counter == 0);
		match victim {
			Some(v) => {
				let removed = bucket.remove_at(v);
				record_ghost(ghost, &removed);
			}
			None => {
				let mut first_zero: Option<usize> = None;
				for (n, i) in bucket.get_items_mut().iter_mut().enumerate() {
					if i.lfu_counter > 0 {
						i.lfu_counter -= 1;
					}
					if i.lfu_counter == 0 && first_zero.is_none() {
						first_zero = Some(n);
					}
				}
				let removed = bucket.remove_at(first_zero.unwrap_or(0));
				record_ghost(ghost, &removed);
			}
		}
	}
	core.put_at(Slot::Stash, pos, item);
}

// ── Promotion logic ────────────────────────────────────────────────────────

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

/// One-more-chance demotion. If the item was a foreign resident at `source_idx`,
/// retry in its own true target before stash; if already in its target, go to stash.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		put_stash(core, ghost, insert_pos, item);
	} else if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
		put_stash(core, ghost, insert_pos, cascade);
	}
}

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
		let mut data = core.stash_buckets[stash_idx].remove_at(item_idx);
		bump(&mut data);
		promote_to_target(core, ghost, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx}")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		bump(&mut core.buckets[target_idx].get_items_mut()[item_idx]);
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, ghost, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx}")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		bump(&mut core.buckets[probing_idx].get_items_mut()[item_idx]);
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	None
}
