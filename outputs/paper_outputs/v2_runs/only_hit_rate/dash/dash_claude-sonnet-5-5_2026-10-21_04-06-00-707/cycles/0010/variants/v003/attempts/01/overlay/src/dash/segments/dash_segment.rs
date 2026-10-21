use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Window length (in final evictions) for the adaptive admission decision.
const WINDOW: u32 = 4096;
/// Filter bits per slot of capacity.
const FILTER_BITS_PER_SLOT: usize = 8;

/// Baseline promotion segment plus an adaptive second-sighting admission gate.
///
/// Every item carries a reuse flag (`lfu_counter > 0`, set on any hit). Each time an
/// item leaves the segment we count whether it was ever hit. Over a window of
/// `WINDOW` evictions, if more than half were never-hit, the next window requires a
/// second sighting (one-bit filter, set on first miss, cleared on admit) before a
/// key is admitted; otherwise all misses are admitted.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	filter: Vec<u64>,
	filter_bits: usize,
	strict: bool,
	win_total: u32,
	win_never_hit: u32,
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
			let filter_bits = (core.max_capacity() * FILTER_BITS_PER_SLOT).max(64);
			Some(Self {
				core,
				filter: vec![0u64; filter_bits.div_ceil(64)],
				filter_bits,
				strict: false,
				win_total: 0,
				win_never_hit: 0,
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

	fn bit_index(&self, key: &K) -> usize {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		(h.finish() as usize) % self.filter_bits
	}

	/// Record the fate of an item that left the segment; roll the window when full.
	fn record_eviction(&mut self, evicted: Option<Item<K, V>>) {
		let Some(e) = evicted else { return };
		self.win_total += 1;
		if e.lfu_counter == 0 {
			self.win_never_hit += 1;
		}
		if self.win_total >= WINDOW {
			self.strict = self.win_never_hit * 2 > self.win_total;
			self.win_total = 0;
			self.win_never_hit = 0;
		}
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
		let mut evicted = None;
		let res = promotion_get(&mut self.core, key, InsertPosition::Front, &mut evicted);
		if let Some(e) = evicted {
			// inline (disjoint-field) accounting while `res` borrows `self.core`
			self.win_total += 1;
			if e.lfu_counter == 0 {
				self.win_never_hit += 1;
			}
			if self.win_total >= WINDOW {
				self.strict = self.win_never_hit * 2 > self.win_total;
				self.win_total = 0;
				self.win_never_hit = 0;
			}
		}
		res
	}

	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		if self.strict {
			let bit = self.bit_index(&item.key);
			let (w, m) = (bit / 64, 1u64 << (bit % 64));
			if self.filter[w] & m == 0 {
				// first sighting: remember, do not admit
				self.filter[w] |= m;
				return;
			}
			self.filter[w] &= !m;
		}
		debug!("Segment.put: key={:?}", item.key);
		let evicted = self.core.put_at(Slot::Stash, InsertPosition::Back, item);
		self.record_eviction(evicted);
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────

/// Moves `item` into its target bucket. Returns an item that left the segment, if any.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let evicted = core.put_at(Slot::Target, insert_pos, item)?;
	demote_from_normal_bucket(core, evicted, target_idx, insert_pos)
}

/// Moves `item` into its probing bucket. Returns an item that left the segment, if any.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let evicted = core.put_at(Slot::Probing, insert_pos, item)?;
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos)
}

/// One-more-chance demotion; stash overflow is the end of the line and is returned.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	source_idx: usize,
	insert_pos: InsertPosition,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let true_target = get_index(&item.key, core.segment_size);
	if true_target == source_idx {
		core.put_at(Slot::Stash, insert_pos, item)
	} else if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
		core.put_at(Slot::Stash, insert_pos, cascade)
	} else {
		None
	}
}

fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
	out_evicted: &mut Option<Item<K, V>>,
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
		data.lfu_counter = 1; // reuse flag
		*out_evicted = promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx}")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			let mut data = core.buckets[target_idx].remove_at(item_idx);
			data.lfu_counter = 1;
			*out_evicted = promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx}")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			core.buckets[target_idx].get_items_mut()[item_idx].lfu_counter = 1;
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		core.buckets[probing_idx].get_items_mut()[item_idx].lfu_counter = 1;
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	None
}
