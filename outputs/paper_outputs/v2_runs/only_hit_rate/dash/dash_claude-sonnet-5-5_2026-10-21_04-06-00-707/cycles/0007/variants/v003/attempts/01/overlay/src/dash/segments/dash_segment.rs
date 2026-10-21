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

/// The dash Subject's segment (variant: MRU-insert promotion + MRU-insert admission
/// + one-hit-wonder admission filter, 4x larger filter, indexed by high hash bits
/// + CLOCK-style referenced bit for stash victim selection; the filter is bypassed
/// when the stash or target bucket has a free slot).
///
/// The referenced bit lives in the existing `Item::lfu_counter` field (0 = not hit since
/// insertion, nonzero = hit), so item/metadata size does not grow.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Fingerprints of keys seen once while the stash bucket was full. 0 = empty.
	filter: [u8; FILTER_SLOTS],
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
		promotion_get(&mut self.core, key, InsertPosition::Back)
	}

	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let stash_idx = get_index(&item.key, self.core.stash_size);
		let target_idx = get_index(&item.key, self.core.segment_size);
		// Use high hash bits (distinct from the low bits used for bucket selection).
		let h = hash(&item.key);
		let slot = (h >> 32) & (FILTER_SLOTS - 1);
		let fp = (((h >> 16) & 0xFF) as u8) | 1;
		let has_room = !self.core.stash_buckets[stash_idx].is_full()
			|| !self.core.buckets[target_idx].is_full();
		if has_room {
			// Free space available: admit immediately, still record in the filter.
			self.filter[slot] = fp;
		} else {
			// Eviction would be needed: apply the one-hit-wonder filter.
			if self.filter[slot] != fp {
				debug!("DashSegment.put: key={:?} first sighting, skip", item.key);
				self.filter[slot] = fp;
				return;
			}
			self.filter[slot] = 0;
		}
		debug!("DashSegment.put: key={:?} -> stash MRU", item.key);
		let mut item = item;
		item.lfu_counter = 0; // referenced bit cleared on insert
		put_stash(&mut self.core, InsertPosition::Back, item);
	}
}

// ── Referenced-bit victim selection ────────────────────────────────────────

/// Inserts `item` into its stash bucket. If the bucket is full, first evicts the first
/// slot (scan order) whose referenced bit is clear; if every slot is referenced, clears
/// all bits and lets the bucket's own policy pick the victim.
fn put_stash<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>)
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
				let _ = bucket.remove_at(v);
			}
			None => {
				for i in bucket.get_items_mut().iter_mut() {
					i.lfu_counter = 0;
				}
			}
		}
	}
	core.put_at(Slot::Stash, pos, item);
}

// ── Promotion logic ────────────────────────────────────────────────────────

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

/// One-more-chance demotion. If the item was a foreign resident at `source_idx`,
/// retry in its own true target before stash; if already in its target, go to stash.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		put_stash(core, insert_pos, item);
	} else if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
		put_stash(core, insert_pos, cascade);
	}
}

fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
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
		data.lfu_counter = 1; // referenced
		promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx}")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter = 1; // referenced
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx}")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		core.buckets[probing_idx].get_items_mut()[item_idx].lfu_counter = 1; // referenced
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	None
}
