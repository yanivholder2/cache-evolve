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

/// Lower saturation bound of the per-segment bypass feedback score.
pub const BYPASS_SCORE_MIN: i8 = -16;
/// Upper saturation bound of the per-segment bypass feedback score.
pub const BYPASS_SCORE_MAX: i8 = 15;

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

/// Mask of the bits strictly below position `i` (`i < 64`).
#[inline]
fn low_mask(i: usize) -> u64 {
	if i == 0 {
		0
	} else {
		(1u64 << i) - 1
	}
}

/// Variant cycle/0009/v003 (H03): bypass feedback state.
///
/// * `ghost_admitted` bitmap — one bit per normal-bucket slot, positional and
///   mirrored through every slot shift the segment performs (Front insert,
///   removal, policy reorder on hit). A set bit marks an item that entered
///   target via a ghost-hit bypass and has not yet been re-hit.
/// * `score` — signed saturating counter in `[BYPASS_SCORE_MIN, BYPASS_SCORE_MAX]`.
///   +1 when a flagged slot is re-hit, -1 when a flagged slot is evicted or
///   demoted out of its bucket before being re-hit.
///
/// The bit lives here, not in `lfu_counter`; the counter cap is unchanged.
#[derive(Debug)]
struct Feedback {
	words: Box<[u64]>,
	words_per_bucket: usize,
	bits_per_bucket: usize,
	score: i8,
}

impl Feedback {
	fn new(num_buckets: usize, bucket_size: usize) -> Self {
		let words_per_bucket = bucket_size.div_ceil(64).max(1);
		Feedback {
			words: vec![0u64; num_buckets * words_per_bucket].into_boxed_slice(),
			words_per_bucket,
			bits_per_bucket: bucket_size,
			score: 0,
		}
	}

	fn reward(&mut self) {
		if self.score < BYPASS_SCORE_MAX {
			self.score += 1;
		}
	}

	fn penalize(&mut self) {
		if self.score > BYPASS_SCORE_MIN {
			self.score -= 1;
		}
	}

	/// `true` if any slot of normal bucket `b` is flagged.
	#[inline]
	fn any(&self, b: usize) -> bool {
		let s = b * self.words_per_bucket;
		self.words.get(s..s + self.words_per_bucket).is_some_and(|w| w.iter().any(|&x| x != 0))
	}

	#[inline]
	fn get(&self, b: usize, i: usize) -> bool {
		if i >= self.bits_per_bucket {
			return false;
		}
		let w = b * self.words_per_bucket + i / 64;
		self.words.get(w).is_some_and(|&x| (x >> (i % 64)) & 1 == 1)
	}

	#[inline]
	fn set(&mut self, b: usize, i: usize, v: bool) {
		if i >= self.bits_per_bucket {
			return;
		}
		let w = b * self.words_per_bucket + i / 64;
		if let Some(x) = self.words.get_mut(w) {
			let m = 1u64 << (i % 64);
			if v {
				*x |= m;
			} else {
				*x &= !m;
			}
		}
	}

	/// Read and clear the bit at slot `i` of bucket `b` (no shift).
	fn take(&mut self, b: usize, i: usize) -> bool {
		let v = self.get(b, i);
		if v {
			self.set(b, i, false);
		}
		v
	}

	/// Mirror `items.remove(i)`: return the removed slot's bit and shift the
	/// higher slots down by one.
	fn remove(&mut self, b: usize, i: usize) -> bool {
		let bits = self.bits_per_bucket;
		if bits == 0 || i >= bits || !self.any(b) {
			return false;
		}
		let v = self.get(b, i);
		if self.words_per_bucket == 1 {
			if let Some(w) = self.words.get_mut(b) {
				let x = *w;
				let low = x & low_mask(i);
				let high = ((x >> i) >> 1) << i;
				*w = low | high;
			}
		} else {
			for j in i..bits - 1 {
				let n = self.get(b, j + 1);
				self.set(b, j, n);
			}
			self.set(b, bits - 1, false);
		}
		v
	}

	/// Mirror `items.insert(i, item)`: shift slots `>= i` up by one and set
	/// slot `i` to `v`.
	fn insert(&mut self, b: usize, i: usize, v: bool) {
		let bits = self.bits_per_bucket;
		if i >= bits {
			return;
		}
		if !v && !self.any(b) {
			return;
		}
		if self.words_per_bucket == 1 {
			if let Some(w) = self.words.get_mut(b) {
				let x = *w;
				let lm = low_mask(i);
				let low = x & lm;
				let high = (x & !lm) << 1;
				let bits_mask = if bits >= 64 { !0u64 } else { (1u64 << bits) - 1 };
				*w = (low | high | ((v as u64) << i)) & bits_mask;
			}
		} else {
			for j in (i + 1..bits).rev() {
				let p = self.get(b, j - 1);
				self.set(b, j, p);
			}
			self.set(b, i, v);
		}
	}

	/// Mirror a policy reorder that moved the item at `from` to `to`
	/// (ClassicLRU move-to-back, Swap one-step-forward; in-place policies are
	/// `from == to`).
	fn move_bit(&mut self, b: usize, from: usize, to: usize) {
		if from == to {
			return;
		}
		let v = self.remove(b, from);
		self.insert(b, to, v);
	}
}

/// Outcome of the bypass gate on a ghost hit's target bucket.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum BypassGate {
	/// Target has a free slot: bypass with no eviction.
	Free,
	/// Target is full but the victim scan found a cold (counter-0) slot: bypass,
	/// evicting the slot at this index.
	Cold(usize),
	/// H03: target is full and the tail window is all warm, but the segment's
	/// bypass feedback score is positive: bypass, evicting the policy's
	/// min-counter victim at this index.
	Score(usize),
	/// Target is full, every tail-window slot is warm, and the score is not
	/// positive: no bypass.
	Blocked,
}

/// Single-pass gate + victim choice for a ghost-hit bypass into `bucket`.
/// The victim scan is the bucket policy's own eviction scan (for `ClassicLRU`:
/// the min-counter slot over the 8-slot LRU tail window, ties to the older
/// slot), so "the chosen victim has counter 0" is exactly "some tail-window
/// slot has counter 0". When it does not, a positive `score` still admits.
fn bypass_gate<K, V>(bucket: &dyn Bucket<K, V>, score: i8) -> BypassGate
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	if !bucket.is_full() {
		return BypassGate::Free;
	}
	match bucket.eviction_candidate_index() {
		Some(v) if bucket.get_items()[v].lfu_counter == 0 => BypassGate::Cold(v),
		Some(v) if score > 0 => BypassGate::Score(v),
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
///
/// Variant cycle/0009/v003 (H03): the gate learns from outcomes. Each bypass
/// marks its landing slot `ghost_admitted`; a re-hit on a marked slot raises a
/// per-segment `bypass_score`, an eviction/demotion of a marked slot lowers it.
/// While the score is positive, returners also bypass all-warm windows,
/// evicting the normal min-counter victim.
///
/// Metadata added: one bit per normal-bucket slot (one `u64` per 64-slot
/// bucket) plus one `i8` per segment.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	ghost: Ghost,
	fb: Feedback,
}

impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Creates a new DashSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			let normal_bucket_size = core.buckets.first().map_or(0, |b| b.get_max_size());
			let normal_cap = core.buckets.len() * normal_bucket_size;
			let stash_cap = core.stash_buckets.len() * core.stash_buckets.first().map_or(0, |b| b.get_max_size());
			let ghost = Ghost::new(normal_cap + stash_cap);
			let fb = Feedback::new(core.buckets.len(), normal_bucket_size);
			info!(
				"DashSegment.try_new: ghost_slots={} ghost_admitted_words={}",
				ghost.slots.len(),
				fb.words.len()
			);
			Some(Self { core, ghost, fb })
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
		promotion_get(&mut self.core, &mut self.ghost, &mut self.fb, key, InsertPosition::Front)
	}

	/// Fresh keys enter the stash at `Back` with counter 0. A key remembered by
	/// the ghost table is admitted with `lfu_counter = 1`. It bypasses the stash
	/// into its target bucket at `Front` if the target has a free slot, a cold
	/// (counter-0) tail-window victim, or the segment's bypass score is positive;
	/// the landing slot is marked `ghost_admitted` and any evicted victim flows
	/// through the normal demotion cascade. Otherwise it enters the stash at
	/// `Back` and its ghost entry is kept.
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
		let gate = bypass_gate(&*self.core.buckets[target_idx], self.fb.score);
		if gate == BypassGate::Blocked {
			debug!(
				"DashSegment.put: key={:?} ghost hit, target={target_idx} warm, score={} -> stash, lfu_counter=1",
				item.key, self.fb.score
			);
			put_into_stash(&mut self.core, &mut self.ghost, InsertPosition::Back, item);
			return;
		}

		debug!(
			"DashSegment.put: key={:?} ghost hit -> target={target_idx} ({gate:?}, score={}), lfu_counter=1",
			item.key, self.fb.score
		);
		self.ghost.clear(idx);
		let key = item.key;
		let evicted = match gate {
			BypassGate::Cold(v) | BypassGate::Score(v) => {
				let victim = self.core.buckets[target_idx].remove_at(v);
				if self.fb.remove(target_idx, v) {
					self.fb.penalize();
				}
				Some(victim)
			}
			_ => None,
		};
		// The bucket now has room, so this insert does not run a second victim scan.
		let extra = normal_insert(&mut self.core, &mut self.fb, target_idx, item, InsertPosition::Front, true);
		debug_assert!(extra.is_none(), "put: bypass insert into target={target_idx} must not evict");
		if let Some(victim) = evicted {
			demote_from_normal_bucket(
				&mut self.core,
				&mut self.ghost,
				&mut self.fb,
				victim,
				target_idx,
				InsertPosition::Front,
			);
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

/// Inserts a non-resident `item` into normal bucket `bidx` at `pos`, evicting
/// the policy's victim first when full (same semantics as `Bucket::put_at` for
/// an absent key), while mirroring the slot shift in the `ghost_admitted`
/// bitmap. The inserted slot's bit is `flag`. Evicting a flagged slot lowers
/// the bypass score. Returns the evicted item, if any.
fn normal_insert<K, V>(
	core: &mut SegmentCore<K, V>,
	fb: &mut Feedback,
	bidx: usize,
	item: Item<K, V>,
	pos: InsertPosition,
	flag: bool,
) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let bucket = &mut core.buckets[bidx];
	let mut evicted = None;
	if bucket.is_full() {
		if let Some(v) = bucket.eviction_candidate_index() {
			let e = bucket.remove_at(v);
			if fb.remove(bidx, v) {
				fb.penalize();
				debug!("normal_insert: ghost-admitted key={:?} evicted unhit, score={}", e.key, fb.score);
			}
			debug!("normal_insert: bucket={bidx} pos={pos:?} evicted_key={:?}", e.key);
			evicted = Some(e);
		}
	}
	match pos {
		InsertPosition::Back => {
			let at = bucket.size();
			bucket.get_items_mut().push(item);
			fb.insert(bidx, at, flag);
		}
		InsertPosition::Front => {
			bucket.get_items_mut().insert(0, item);
			fb.insert(bidx, 0, flag);
		}
	}
	evicted
}

/// A hit on normal bucket `b` at `pos` that stays in the bucket: rewards the
/// bypass score if the slot was ghost-admitted (clearing the bit), lets the
/// policy reorder, and mirrors that reorder in the bitmap.
fn touch<'a, K, V>(core: &'a mut SegmentCore<K, V>, fb: &mut Feedback, b: usize, pos: usize) -> &'a Item<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	if fb.take(b, pos) {
		fb.reward();
		debug!("touch: ghost-admitted slot re-hit in bucket={b}, score={}", fb.score);
	}
	if !fb.any(b) {
		return core.buckets[b].get_from_position(pos);
	}
	let hit: *const Item<K, V> = core.buckets[b].get_from_position(pos);
	let items = core.buckets[b].get_items();
	let new_pos = items.iter().position(|it| std::ptr::eq(it, hit)).unwrap_or(pos);
	fb.move_bit(b, pos, new_pos);
	&items[new_pos]
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
	fb: &mut Feedback,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = normal_insert(core, fb, target_idx, item, insert_pos, false) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, fb, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket. Any evicted item flows through the tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
	fb: &mut Feedback,
	item: Item<K, V>,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let Some(evicted) = normal_insert(core, fb, probing_idx, item, insert_pos, false) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, fb, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
/// Normal-bucket victims stay resident (demoted to stash) and are not ghosted; only
/// the stash item those demotions push out of the cache is.
///
/// H03 (c0006): an item crossing from the normal tier into the stash has its
/// `lfu_counter` halved (tier-boundary ageing). The probing→own-target retry
/// stays in the normal tier and keeps its counter.
///
/// H03 (c0009): the `ghost_admitted` bit of `item` was already consumed (and
/// scored) by whoever removed it; the retry insert is unflagged, and a flagged
/// cascade victim lowers the score inside `normal_insert`.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost,
	fb: &mut Feedback,
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
		if let Some(mut cascade) = normal_insert(core, fb, true_target, item, insert_pos, false) {
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
	fb: &mut Feedback,
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
		promote_to_target(core, ghost, fb, data, insert_pos);

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
			if fb.remove(target_idx, item_idx) {
				fb.reward();
				debug!("promotion_get: ghost-admitted key={key:?} re-hit, score={}", fb.score);
			}
			promote_to_probing(core, ghost, fb, data, insert_pos);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
			return Some(touch(core, fb, target_idx, item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		return Some(touch(core, fb, probing_idx, item_idx));
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
		assert!(seg.fb.get(t, pos), "bypass landing slot must be ghost_admitted");
		assert_eq!(stash_counter(&seg, victim), None, "ghost-hit key must not also be in stash");
		assert!(!is_ghosted(&seg, victim), "bypass consumes the ghost entry");
		assert_eq!(seg.fb.score, 0);
		assert_eq!(seg.get(&victim).map(|it| it.value), Some(victim));
		assert_eq!(seg.fb.score, 1, "re-hit of a ghost-admitted slot rewards the score");
		assert!(!seg.fb.any(t), "re-hit clears the ghost_admitted bit");
		// A second hit does not reward again.
		seg.get(&victim);
		assert_eq!(seg.fb.score, 1);
	}

	#[test]
	fn ghost_hit_into_warm_full_target_falls_back_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (before, ghosted) = full_target_with_ghosted_key(&mut seg, 6, 7);
		assert_eq!(seg.fb.score, 0);
		let stash_idx = get_index(&ghosted, seg.core.stash_size);
		let stash_before: Vec<i64> = seg.core.stash_buckets[stash_idx].get_items().iter().map(|it| it.key).collect();

		seg.put(Item::new(ghosted, ghosted));
		assert_eq!(target_counter(&seg, ghosted), None, "warm target must block the bypass");
		assert_eq!(stash_counter(&seg, ghosted), Some(1), "fallback lands in stash with counter 1");
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
	fn ghost_hit_into_warm_full_target_bypasses_when_score_positive() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (before, ghosted) = full_target_with_ghosted_key(&mut seg, 6, 7);
		seg.fb.score = 2;
		let target_idx = 0;

		seg.put(Item::new(ghosted, ghosted));
		assert_eq!(target_counter(&seg, ghosted), Some(1), "positive score admits into warm target");
		assert_eq!(stash_counter(&seg, ghosted), None);
		assert_eq!(seg.core.buckets[target_idx].get_items()[0].key, ghosted, "bypass lands at Front");
		assert!(seg.fb.get(target_idx, 0), "landing slot is ghost_admitted");
		assert!(!is_ghosted(&seg, ghosted));

		let evicted: Vec<(i64, usize)> =
			before.iter().copied().filter(|(k, _)| seg.core.buckets[target_idx].get_position(k).is_none()).collect();
		assert_eq!(evicted.len(), 1, "exactly one resident must have been demoted");
		assert_eq!(evicted[0].1, 6, "the min-counter resident must be the victim");
		assert_eq!(seg.fb.score, 2, "victim was not ghost-admitted, score unchanged");
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
	fn evicting_unhit_ghost_admitted_slot_penalizes() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let (_, ghosted) = full_target_with_ghosted_key(&mut seg, 0, 7);
		let target_idx = 0;
		seg.put(Item::new(ghosted, ghosted));
		assert!(seg.fb.get(target_idx, 0));
		// Make the ghost-admitted slot the coldest so the next target insert evicts it.
		seg.core.buckets[target_idx].get_items_mut()[0].lfu_counter = 0;
		let newcomer = (0i64..100_000)
			.find(|k| get_index(k, seg.core.segment_size) == target_idx && !seg.core.contains_key(k))
			.unwrap();
		let extra = normal_insert(&mut seg.core, &mut seg.fb, target_idx, Item::new(newcomer, newcomer), InsertPosition::Front, false);
		assert_eq!(extra.map(|e| e.key), Some(ghosted));
		assert_eq!(seg.fb.score, -1, "evicting an unhit ghost-admitted slot lowers the score");
		assert!(!seg.fb.any(target_idx));
	}

	#[test]
	fn bypass_gate_reports_free_cold_score_blocked() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert_eq!(bypass_gate(&*seg.core.buckets[0], 0), BypassGate::Free);
		let cap = seg.core.buckets[0].get_max_size();
		let keys = find_keys_for_bucket(0, seg.core.segment_size, cap);
		for &k in &keys {
			seg.core.buckets[0].put(Item::new(k, k));
		}
		for it in seg.core.buckets[0].get_items_mut().iter_mut() {
			it.lfu_counter = 3;
		}
		assert_eq!(bypass_gate(&*seg.core.buckets[0], 0), BypassGate::Blocked);
		assert_eq!(bypass_gate(&*seg.core.buckets[0], -5), BypassGate::Blocked);
		assert_eq!(bypass_gate(&*seg.core.buckets[0], 1), BypassGate::Score(0));
		let w = cap.min(crate::shared::eviction_policy::LRU_TAIL_WINDOW) - 1;
		seg.core.buckets[0].get_items_mut()[w].lfu_counter = 0;
		assert_eq!(bypass_gate(&*seg.core.buckets[0], 0), BypassGate::Cold(w));
		assert_eq!(bypass_gate(&*seg.core.buckets[0], 1), BypassGate::Cold(w));
	}

	#[test]
	fn feedback_score_saturates() {
		let mut fb = Feedback::new(1, 4);
		for _ in 0..100 {
			fb.reward();
		}
		assert_eq!(fb.score, BYPASS_SCORE_MAX);
		for _ in 0..100 {
			fb.penalize();
		}
		assert_eq!(fb.score, BYPASS_SCORE_MIN);
	}

	fn bits(fb: &Feedback, b: usize, n: usize) -> Vec<bool> {
		(0..n).map(|i| fb.get(b, i)).collect()
	}

	#[test]
	fn feedback_bits_mirror_vec_shifts() {
		for size in [4usize, 64, 70] {
			let mut fb = Feedback::new(3, size);
			let mut model: Vec<bool> = Vec::new();
			let b = 1;
			// Front inserts.
			for v in [true, false, true, false] {
				model.insert(0, v);
				fb.insert(b, 0, v);
			}
			assert_eq!(bits(&fb, b, size)[..model.len()], model[..], "size={size}");
			// Back insert.
			model.push(true);
			fb.insert(b, model.len() - 1, true);
			let n = model.len().min(size);
			assert_eq!(bits(&fb, b, size)[..n], model[..n], "size={size}");
			model.truncate(n);
			// Remove from the middle.
			let r = model.remove(1);
			assert_eq!(fb.remove(b, 1), r);
			assert_eq!(bits(&fb, b, size)[..model.len()], model[..]);
			// Move to back (ClassicLRU) and one step forward (Swap).
			let last = model.len() - 1;
			let v = model.remove(0);
			model.push(v);
			fb.move_bit(b, 0, last);
			assert_eq!(bits(&fb, b, size)[..model.len()], model[..]);
			model.swap(2, 1);
			fb.move_bit(b, 2, 1);
			assert_eq!(bits(&fb, b, size)[..model.len()], model[..]);
			// No bits beyond len, other buckets untouched.
			assert!(bits(&fb, b, size)[model.len()..].iter().all(|&x| !x));
			assert!(!fb.any(0) && !fb.any(2));
		}
	}

	#[test]
	fn feedback_full_bucket_insert_drops_top() {
		let mut fb = Feedback::new(1, 64);
		fb.set(0, 63, true);
		fb.insert(0, 0, false);
		assert!(!fb.any(0), "bit shifted past capacity is dropped");
		fb.set(0, 63, true);
		assert!(fb.remove(0, 63));
		assert!(!fb.any(0));
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
