use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::info;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Tiny multiplicative hasher (cheap, deterministic) for the doorkeeper.
struct MixHasher(u64);

impl Hasher for MixHasher {
	fn finish(&self) -> u64 {
		let mut x = self.0;
		x ^= x >> 33;
		x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
		x ^= x >> 33;
		x
	}
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0 ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
		}
	}
	fn write_i64(&mut self, i: i64) {
		self.0 = (self.0 ^ i as u64).wrapping_mul(0x9e37_79b9_7f4a_7c15);
	}
	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0 ^ i).wrapping_mul(0x9e37_79b9_7f4a_7c15);
	}
}

/// Fixed 512-bit (one 64-byte cache line) 2-probe doorkeeper. Both probe indices
/// come from a single 64-bit hash (low 9 bits and next 9 bits). Aging reset is
/// triggered by a running count of newly set bits (> 256, i.e. 50% fill) and
/// simply zeroes the 8 words.
#[derive(Debug)]
#[repr(align(64))]
struct Doorkeeper {
	words: [u64; 8],
	set_bits: u32,
}

const DOOR_LIMIT: u32 = 256;

impl Doorkeeper {
	fn new() -> Self {
		Self {
			words: [0u64; 8],
			set_bits: 0,
		}
	}

	/// Returns true if the key was already seen (admit). Otherwise records it.
	#[inline]
	fn seen_or_record(&mut self, h: u64) -> bool {
		let a = (h & 511) as usize;
		let b = ((h >> 9) & 511) as usize;
		let (wa, ma) = (a >> 6, 1u64 << (a & 63));
		let (wb, mb) = (b >> 6, 1u64 << (b & 63));
		let ta = self.words[wa] & ma != 0;
		let tb = self.words[wb] & mb != 0;
		if ta && tb {
			return true;
		}
		if !ta {
			self.words[wa] |= ma;
			self.set_bits += 1;
		}
		// Re-check: a and b may share the same word/bit.
		if self.words[wb] & mb == 0 {
			self.words[wb] |= mb;
			self.set_bits += 1;
		}
		if self.set_bits > DOOR_LIMIT {
			self.words = [0u64; 8];
			self.set_bits = 0;
		}
		false
	}
}

/// Promotion-design segment. Variant v001: `get` uses `InsertPosition::Back`, so
/// promotions and demotions land at the MRU end of their destination bucket.
/// `put` consults a one-cache-line 2-probe doorkeeper once the destination stash
/// bucket is full: a key is admitted only on its second sighting.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	door: Doorkeeper,
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
			Some(Self { core, door: Doorkeeper::new() })
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
		let core = &mut self.core;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present",
			item.key
		);
		let stash_idx = get_index(&item.key, core.stash_size);
		let full = {
			let b = &core.stash_buckets[stash_idx];
			b.size() >= b.get_max_size()
		};
		if full {
			let mut h = MixHasher(0xcbf2_9ce4_8422_2325);
			item.key.hash(&mut h);
			if !self.door.seen_or_record(h.finish()) {
				return;
			}
		}
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	}
}

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
		core.put_at(Slot::Stash, insert_pos, item);
	} else if let Some(cascade) = core.put_at(Slot::Target, insert_pos, item) {
		core.put_at(Slot::Stash, insert_pos, cascade);
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
			let data = core.buckets[target_idx].remove_at(item_idx);
			promote_to_probing(core, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
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
