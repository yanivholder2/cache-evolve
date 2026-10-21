use super::{alt_stash_index, Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::{Bucket, InsertPosition};
use crate::shared::utils::{get_index, hash};
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// Number of one-hit-wonder filter slots (u8 each => 512 bytes per segment). Power of two.
const FILTER_SLOTS: usize = 512;

/// The dash Subject's segment (variant: MRU-insert promotion + MRU-insert admission
/// + one-hit-wonder admission filter, plus power-of-two-choices stash placement).
///
/// Admitted items go to the less-loaded of two hash-derived stash buckets
/// (primary, then an independently hashed alternate); lookups probe both.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Fingerprints of keys seen once while both candidate stash buckets were full. 0 = empty.
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
		let primary = get_index(&item.key, self.core.stash_size);
		let alt = alt_stash_index(&item.key, self.core.stash_size);
		// Power of two choices: fewer occupied slots wins, ties go to primary.
		let chosen = if self.core.stash_buckets[alt].size() < self.core.stash_buckets[primary].size() {
			alt
		} else {
			primary
		};
		if self.core.stash_buckets[chosen].is_full() {
			// Both candidates full: eviction needed, apply the one-hit-wonder filter.
			let h = hash(&item.key);
			let slot = (h >> 32) & (FILTER_SLOTS - 1);
			let fp = (((h >> 16) & 0xFF) as u8) | 1;
			if self.filter[slot] != fp {
				debug!("DashSegment.put: key={:?} first sighting, skip", item.key);
				self.filter[slot] = fp;
				return;
			}
			self.filter[slot] = 0;
		}
		debug!("DashSegment.put: key={:?} -> stash[{chosen}] MRU", item.key);
		self.core.stash_buckets[chosen].put_at(item, InsertPosition::Back);
	}
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
		core.put_at(Slot::Stash, insert_pos, item);
	} else if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
		core.put_at(Slot::Stash, insert_pos, cascade);
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
	let alt_idx = alt_stash_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	// Probe primary stash bucket first, then the alternate.
	let stash_hit = if let Some(i) = core.stash_buckets[stash_idx].get_position(key) {
		Some((stash_idx, i))
	} else {
		core.stash_buckets[alt_idx].get_position(key).map(|i| (alt_idx, i))
	};
	if let Some((sidx, item_idx)) = stash_hit {
		let data = core.stash_buckets[sidx].remove_at(item_idx);
		promote_to_target(core, data, insert_pos);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx}")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
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
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	None
}
