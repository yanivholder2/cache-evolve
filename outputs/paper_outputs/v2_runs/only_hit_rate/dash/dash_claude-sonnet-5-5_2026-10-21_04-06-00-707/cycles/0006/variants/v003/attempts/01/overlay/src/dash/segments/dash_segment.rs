use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU8, AtomicUsize, Ordering};

const ROWS: usize = 4;
const MAX_COUNT: u8 = 15; // 4-bit saturating counter
const ROW_MULS: [u64; ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0x27D4_EB2F_1656_67C5,
];

/// Count-min sketch of 4-bit counters (one nibble value per `AtomicU8`), relaxed atomics.
#[derive(Debug)]
struct FreqSketch {
	width_mask: usize,
	table: Vec<AtomicU8>,
	additions: AtomicUsize,
	sample_size: usize,
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = (capacity.max(1) * 4).next_power_of_two();
		let table = (0..width * ROWS).map(|_| AtomicU8::new(0)).collect();
		Self {
			width_mask: width - 1,
			table,
			additions: AtomicUsize::new(0),
			sample_size: capacity.max(1) * 10,
		}
	}

	fn base_hash<K: Hash>(key: &K) -> u64 {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		h.finish()
	}

	fn slot(&self, base: u64, row: usize) -> usize {
		let x = base.wrapping_mul(ROW_MULS[row]);
		let x = x ^ (x >> 29);
		let x = x.wrapping_mul(ROW_MULS[(row + 1) % ROWS]);
		let idx = ((x >> 32) as usize) & self.width_mask;
		row * (self.width_mask + 1) + idx
	}

	fn estimate<K: Hash>(&self, key: &K) -> u8 {
		let base = Self::base_hash(key);
		(0..ROWS)
			.map(|r| self.table[self.slot(base, r)].load(Ordering::Relaxed))
			.min()
			.unwrap_or(0)
	}

	fn record<K: Hash>(&self, key: &K) {
		let base = Self::base_hash(key);
		for r in 0..ROWS {
			let c = &self.table[self.slot(base, r)];
			let v = c.load(Ordering::Relaxed);
			if v < MAX_COUNT {
				c.store(v + 1, Ordering::Relaxed);
			}
		}
		if self.additions.fetch_add(1, Ordering::Relaxed) + 1 >= self.sample_size {
			self.age();
		}
	}

	fn age(&self) {
		self.additions.store(0, Ordering::Relaxed);
		for c in &self.table {
			c.store(c.load(Ordering::Relaxed) >> 1, Ordering::Relaxed);
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
/// Variant: `put` is gated by a count-min frequency comparison against the
/// would-be stash victim.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	sketch: FreqSketch,
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
			let sketch = FreqSketch::new(core.max_capacity());
			Some(Self { core, sketch })
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
		self.sketch.record(key);
		promotion_get(&mut self.core, key, InsertPosition::Front)
	}

	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let stash_idx = get_index(&item.key, self.core.stash_size);
		let victim_key = {
			let bucket = &self.core.stash_buckets[stash_idx];
			if bucket.is_full() {
				bucket
					.eviction_candidate_index()
					.map(|i| bucket.get_items()[i].key)
			} else {
				None
			}
		};
		// Insert attempt counts toward the candidate's frequency.
		self.sketch.record(&item.key);
		if let Some(vk) = victim_key {
			if self.sketch.estimate(&item.key) <= self.sketch.estimate(&vk) {
				debug!("DashSegment.put: reject key={:?} (not hotter than victim {:?})", item.key, vk);
				return;
			}
		}
		self.core.put_at(Slot::Stash, InsertPosition::Back, item);
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

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
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
		debug!(
			"demote_from_normal_bucket: key={:?} target={source_idx} -> stash",
			item.key
		);
		core.put_at(Slot::Stash, insert_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			core.put_at(Slot::Stash, insert_pos, cascade);
		}
	}
}

/// `insert_pos = Front`: promotions land at LRU end; the item must earn MRU before promoting again.
/// `insert_pos = Back`: promotions land at MRU; one more access promotes to the next tier.
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
		debug!("promotion_get: key={key:?} found in stash -> target={target_idx} insert_pos={insert_pos:?}");
		let data = core.stash_buckets[stash_idx].remove_at(item_idx);
		promote_to_target(core, data, insert_pos);

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
			promote_to_probing(core, data, insert_pos);

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
