use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Baseline promotion segment, with a widened probe neighbourhood: overflow from the
/// target tier tries the next bucket, then the previous bucket, before the stash.
/// Lookups search stash, target, next and previous buckets.
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
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			info!("DashSegment.try_new");
			Some(Self { core })
		} else {
			log::error!("DashSegment.try_new: core initialization failed");
			None
		}
	}

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
		promotion_get(&mut self.core, key, InsertPosition::Front)
	}
}

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

/// Overflow placement: next bucket, then previous bucket (only if they have room),
/// and only then the stash (which evicts if full).
fn overflow_put<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let n = core.segment_size;
	let target_idx = get_index(&item.key, n);
	let next_idx = (target_idx + 1) % n;
	let prev_idx = (target_idx + n - 1) % n;
	for idx in [next_idx, prev_idx] {
		if !core.buckets[idx].is_full() {
			core.buckets[idx].put_at(item, insert_pos);
			return;
		}
	}
	core.put_at(Slot::Stash, insert_pos, item);
}

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
		debug!("demote_from_normal_bucket: key={:?} target={source_idx} -> overflow", item.key);
		overflow_put(core, item, insert_pos);
	} else {
		if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
			overflow_put(core, cascade, insert_pos);
		}
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
	let prev_idx = (target_idx + core.segment_size - 1) % core.segment_size;

	if let Some(item_idx) = core.stash_buckets[stash_idx].get_position(key) {
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, data, insert_pos);

		// The key was just placed in target (or cascaded into next/prev/stash).
		for idx in [target_idx, probing_idx, prev_idx] {
			if let Some(p) = core.buckets[idx].get_position(key) {
				return Some(&core.buckets[idx].get_items()[p]);
			}
		}
		let Some(p) = core.stash_buckets[stash_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} lost after stash promotion")
		};
		return Some(&core.stash_buckets[stash_idx].get_items()[p]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		let is_hottest = Some(item_idx) == core.buckets[target_idx].promotion_candidate_index();
		if is_hottest {
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			for idx in [probing_idx, target_idx, prev_idx] {
				if let Some(p) = core.buckets[idx].get_position(key) {
					return Some(&core.buckets[idx].get_items()[p]);
				}
			}
			let Some(p) = core.stash_buckets[stash_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} lost after probing promotion")
			};
			return Some(&core.stash_buckets[stash_idx].get_items()[p]);
		} else {
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	if let Some(item_idx) = core.buckets[prev_idx].get_position(key) {
		return Some(core.buckets[prev_idx].get_from_position(item_idx));
	}

	debug!("promotion_get: key={key:?} miss");
	None
}
