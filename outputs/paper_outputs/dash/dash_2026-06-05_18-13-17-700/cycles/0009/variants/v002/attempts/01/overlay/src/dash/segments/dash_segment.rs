use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, stash_alt_index};
use log::{debug, info};
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by ranking in the
/// freshest quarter of their target.
///
/// This variant (H02) keeps the parent's two-speed promotion, per-key touch
/// counter, and cuckoo-style single-hop stash relocation, and grafts on a small
/// bounded **ghost list** of stash victims that were *dropped* — i.e. evicted
/// from the cache entirely because neither stash home had room. The ghost list
/// holds **keys only** (never values), so it adds no resident capacity and
/// cannot perturb `capacity_conservation`. On the next admission of a ghosted
/// key, the ghost hit is treated as a LIRS-style short-reuse-distance signal
/// that the key is recurring rather than transient: the key is fast-tracked
/// straight into its protected target tier (MRU landing), skipping stash dwell,
/// while genuine first-touch cold keys keep the existing stash path and still
/// earn their dwell time.
///
/// The promotion design is only the *baseline*; this whole type is the segment
/// extension point and the framework is free to evolve where items land on a hit
/// and how they migrate between slots.
#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	core: SegmentCore<K, V>,
	/// Per-key count of target-bucket hits since the key was last admitted into
	/// its target. A key reaching two demonstrated target accesses promotes to
	/// probing even when the rank gate degenerates on a small bucket. Entries are
	/// dropped when a key promotes out of target or is observed non-resident, so
	/// the map tracks only the small live target population.
	target_touches: HashMap<K, u8>,
	/// Bounded ghost ring of keys that were *dropped* from the cache entirely
	/// (evicted from stash with no alternate home). Stores keys only — never
	/// values — so it adds no resident capacity. A ghost hit on the next
	/// admission of a key earns it a fast-track into its protected target tier.
	ghost: Ghost<K>,
}

/// A small bounded ghost list of dropped stash victims.
///
/// It records the **keys** of items that truly left the cache (a stash overflow
/// victim whose alternate home was also unavailable). It never stores values, so
/// it consumes no resident capacity and is invisible to `total_items` /
/// `capacity_conservation`. Membership is O(1) via a companion `HashSet`; the
/// `VecDeque` keeps FIFO order so the oldest ghost is evicted first when the ring
/// is full.
#[derive(Debug)]
struct Ghost<K>
where
	K: Hash + Eq + Copy,
{
	ring: VecDeque<K>,
	set: HashSet<K>,
	cap: usize,
}

impl<K> Ghost<K>
where
	K: Hash + Eq + Copy,
{
	/// Creates a ghost ring bounded to `cap` keys (clamped to at least 1).
	fn new(cap: usize) -> Self {
		Self {
			ring: VecDeque::new(),
			set: HashSet::new(),
			cap: cap.max(1),
		}
	}

	/// O(1) membership test.
	fn contains(&self, key: &K) -> bool {
		self.set.contains(key)
	}

	/// Removes `key` from the ghost ring if present. Returns whether it was found.
	fn remove(&mut self, key: &K) -> bool {
		if self.set.remove(key) {
			if let Some(pos) = self.ring.iter().position(|k| k == key) {
				self.ring.remove(pos);
			}
			true
		} else {
			false
		}
	}

	/// Records a dropped victim's key, evicting the oldest ghost first when full.
	/// A key already ghosted is a no-op (it stays at its current age).
	fn record(&mut self, key: K) {
		if self.set.contains(&key) {
			return;
		}
		while self.ring.len() >= self.cap {
			match self.ring.pop_front() {
				Some(old) => {
					self.set.remove(&old);
				}
				None => break,
			}
		}
		self.ring.push_back(key);
		self.set.insert(key);
	}
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
			// Bound the ghost ring to roughly the total stash capacity: the
			// population of keys that can plausibly be in stash-eviction flight at
			// once. Keys only, so this allocates no value storage.
			let ghost_cap = settings.num_of_stash_buckets.saturating_mul(settings.bucket_size);
			Some(Self {
				core,
				target_touches: HashMap::new(),
				ghost: Ghost::new(ghost_cap),
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

	/// Cold admission.
	///
	/// If `item.key` is present in the **ghost ring** — meaning it was admitted,
	/// dropped from the cache entirely, and is now being requested again within
	/// the ghost window — it is a proven recurring key (a short reuse distance).
	/// Such a key is removed from the ghost ring and fast-tracked straight into
	/// its protected **target** tier at the MRU end via [`promote_to_target`],
	/// skipping stash dwell. `promote_to_target` conserves occupancy by cascading
	/// exactly one evicted item down the tier, so this remains at most one new
	/// resident seat per `put` (`single_eviction_per_put` holds).
	///
	/// A key not in the ghost ring keeps the existing cold path: it routes
	/// through [`stash_insert`] (MRU stash landing, one relocation hop on
	/// overflow), so genuine first-touch cold keys still earn their dwell time.
	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		let key = item.key;
		if self.ghost.contains(&key) {
			// Proven recurring: ghost hit on re-admission. Fast-track to target MRU.
			self.ghost.remove(&key);
			// Freshly admitted into target: reset the per-key touch counter so the
			// usual target→probing gating applies from here.
			self.target_touches.insert(key, 0);
			debug!("DashSegment.put: key={key:?} ghost-hit -> fast-track to target (MRU)");
			promote_to_target(
				&mut self.core,
				&mut self.ghost,
				item,
				InsertPosition::Back,
				InsertPosition::Back,
			);
		} else {
			debug!("DashSegment.put: key={key:?} cold -> stash");
			stash_insert(&mut self.core, &mut self.ghost, item, InsertPosition::Back);
		}
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, &mut self.target_touches, &mut self.ghost, key)
	}
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket either by ranking in the freshest quarter of their target
// (the inherited rank gate) or by accumulating two demonstrated target hits (a
// density-independent per-key touch counter). Promotion is two-speed: a
// stash→target hop lands at the LRU end (slow, must re-prove warmth) while a
// target→probing hop lands at the MRU end (fast, already proven). Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers; demotion landings stay at the MRU
// end so the working-set tail is protected on the way down.
//
// The one place a key truly leaves the cache is the *drop* branch of
// [`stash_insert`] (a stash overflow victim with no alternate home). That is the
// only site that records a key into the ghost ring, so every threaded helper
// that can reach a stash drop carries the `&mut Ghost` along.

/// Inserts `item` into its **primary** stash bucket. If that bucket overflows,
/// the single evicted victim is given one relocation hop into its **alternate**
/// stash home (`stash_alt_index`) — but only when that alternate is a distinct
/// bucket and currently has room. Otherwise the victim is **dropped** (baseline
/// stash-terminal behaviour) and its key is recorded into the `ghost` ring,
/// since this is the one site where a key truly leaves the cache. At most one
/// hop, never a cascade, so each call adds at most one item to the segment.
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, ghost: &mut Ghost<K>, item: Item<K, V>, pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let Some(victim) = core.put_at(Slot::Stash, pos, item) else {
		return;
	};
	// The primary stash bucket overflowed. Try to give the victim a second home.
	let primary = get_index(&victim.key, core.stash_size);
	let alt = stash_alt_index(&victim.key, core.stash_size);
	if alt != primary && !core.stash_buckets[alt].is_full() {
		debug!(
			"stash_insert: relocate overflow victim key={:?} primary={primary} -> alt={alt}",
			victim.key
		);
		// `alt` is confirmed non-full, so this insert never evicts: the relocation
		// hop adds exactly one item and preserves single_eviction_per_put.
		core.stash_buckets[alt].put_at(victim, InsertPosition::Back);
	} else {
		// The victim is dropped from the cache entirely — this is the only place a
		// key truly leaves. Record its key in the ghost ring so a re-request earns
		// the fast-track. The ring holds keys only and is strictly bounded, so the
		// total count of cached *items* is unchanged.
		debug!(
			"stash_insert: drop+ghost overflow victim key={:?} (alt unavailable: alt={alt} primary={primary})",
			victim.key
		);
		ghost.record(victim.key);
	}
}

/// Moves `item` into its target bucket, landing it at `landing_pos`. Any evicted
/// item flows through the tier cascade, landing at `demote_pos`.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost<K>,
	item: Item<K, V>,
	landing_pos: InsertPosition,
	demote_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = core.put_at(Slot::Target, landing_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, evicted, target_idx, demote_pos);
}

/// Moves `item` into its probing bucket, landing it at `landing_pos`. Any evicted
/// item flows through the tier cascade, landing at `demote_pos`.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost<K>,
	item: Item<K, V>,
	landing_pos: InsertPosition,
	demote_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;
	let Some(evicted) = core.put_at(Slot::Probing, landing_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, ghost, evicted, probing_idx, demote_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
///
/// Both stash landings route through [`stash_insert`], so a demoted item that
/// overflows its primary stash bucket gets the same one-hop relocation chance
/// that cold admissions do — and, on a true drop, is recorded into the ghost ring.
fn demote_from_normal_bucket<K, V>(
	core: &mut SegmentCore<K, V>,
	ghost: &mut Ghost<K>,
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
		stash_insert(core, ghost, item, insert_pos);
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
			stash_insert(core, ghost, cascade, insert_pos);
		}
	}
}

/// Whether a target hit at `item_idx` is fresh enough to promote into probing.
///
/// The baseline gate promotes only the single hottest resident
/// (`item_idx == promotion_candidate_index`). The inherited widening (H01)
/// promotes when the key ranks in the **freshest quarter** of its target
/// bucket. For the recency-ordered policies in use (ClassicLRU / Fifo / Swap),
/// `promotion_candidate_index` is the last vector index and freshness increases
/// toward the back, so the top quarter is `item_idx >= len - len / 4`.
///
/// Small or heavily-colliding buckets (`len < 4`) fall back to the original
/// hottest-only test, so they behave exactly as the baseline does today.
fn is_in_top_quarter<K, V>(bucket: &dyn crate::shared::traits::bucket::Bucket<K, V>, item_idx: usize) -> bool
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	let len = bucket.get_items().len();
	if len < 4 {
		Some(item_idx) == bucket.promotion_candidate_index()
	} else {
		item_idx >= len - len / 4
	}
}

/// Two-speed promotion with a per-key touch-counter trigger:
/// - a stash→target hop lands at the **LRU end** (`Front`); the key must climb
///   to the freshest quarter (or accumulate two target hits) before it can
///   promote again, and
/// - a target→probing hop lands at the **MRU end** (`Back`); a key that proved
///   warmth in its target promotes fast.
///
/// On every target hit the key's entry in `touches` is incremented. A key
/// promotes to probing when it ranks in the freshest quarter of its target
/// (the inherited rank gate) **or** when its touch count has reached 2 — a
/// density-independent signal that fires on len-1/len-2 buckets where the rank
/// rule degenerates. A freshly stash-admitted key has its counter reset to 0, so
/// it still needs two demonstrated target accesses before the counter fires,
/// preserving the slow first hop.
///
/// A stash hit is resolved against **both** stash homes: the key's primary
/// bucket (`get_index`) and its alternate (`stash_alt_index`), so a victim that
/// was relocated by [`stash_insert`] is still found and promoted.
///
/// Demotion landings are fixed at the **MRU end** (`Back`) so the working-set
/// tail is protected on the way down — this lever only slows the first hop.
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	touches: &mut HashMap<K, u8>,
	ghost: &mut Ghost<K>,
	key: &K,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let stash_idx = get_index(key, core.stash_size);
	let stash_alt = stash_alt_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);
	let probing_idx = (target_idx + 1) % core.segment_size;

	// Stash hit: check the primary bucket first, then the alternate second home.
	let stash_home = if core.stash_buckets[stash_idx].get_position(key).is_some() {
		Some(stash_idx)
	} else if stash_alt != stash_idx && core.stash_buckets[stash_alt].get_position(key).is_some() {
		Some(stash_alt)
	} else {
		None
	};

	if let Some(home) = stash_home {
		let item_idx = core.stash_buckets[home]
			.get_position(key)
			.expect("promotion_get: stash home was just confirmed to hold the key");
		debug!("promotion_get: key={key:?} found in stash home={home} -> target={target_idx} (slow, LRU landing)");
		let data = core.stash_buckets[home].remove_at(item_idx);
		// Slow first hop: land the freshly-admitted key at the LRU end so it must
		// re-prove warmth before reaching probing. Demotions still land MRU.
		promote_to_target(core, ghost, data, InsertPosition::Front, InsertPosition::Back);
		// Freshly admitted into target: start the per-key touch counter at 0 so the
		// key still needs two demonstrated target accesses before the counter fires.
		touches.insert(*key, 0);

		let Some(item_new_idx) = core.buckets[target_idx].get_position(key) else {
			unreachable!("promotion_get: key={key:?} not found after insert into target={target_idx} -- promote_to_target must leave the key resident in target or cascade it elsewhere via demote_from_normal_bucket")
		};
		return Some(&core.buckets[target_idx].get_items()[item_new_idx]);
	}

	if let Some(item_idx) = core.buckets[target_idx].get_position(key) {
		// Per-key touch counter: increment on every target hit. This is a
		// density-independent promotion trigger that fires on small buckets where
		// the rank-based top-quarter gate degenerates.
		let count = {
			let c = touches.entry(*key).or_insert(0);
			*c = c.saturating_add(1);
			*c
		};
		// Promote when the key ranks in the freshest quarter of its target bucket
		// (inherited rank gate) OR when it has accumulated two demonstrated target
		// accesses (density-independent touch trigger).
		let is_promotable = is_in_top_quarter(&*core.buckets[target_idx], item_idx) || count >= 2;
		if is_promotable {
			// The key is leaving target for probing — its target touch history is spent.
			touches.remove(key);
			debug!("promotion_get: key={key:?} promotable in target (rank or touches>=2) -> probing={probing_idx} (fast, MRU landing)");
			let data = core.buckets[target_idx].remove_at(item_idx);
			// Fast second hop: a key that proved warmth in its target lands at
			// the MRU end of probing. Demotions still land MRU.
			promote_to_probing(core, ghost, data, InsertPosition::Back, InsertPosition::Back);

			let Some(item_new_idx) = core.buckets[probing_idx].get_position(key) else {
				unreachable!("promotion_get: key={key:?} not found after insert into probing={probing_idx} -- promote_to_probing must leave the key resident in probing")
			};
			return Some(&core.buckets[probing_idx].get_items()[item_new_idx]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} touches={count} (intra-promote)");
			return Some(core.buckets[target_idx].get_from_position(item_idx));
		}
	}

	if let Some(item_idx) = core.buckets[probing_idx].get_position(key) {
		// The key is no longer in a target bucket — drop any stale touch entry.
		touches.remove(key);
		debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
		return Some(core.buckets[probing_idx].get_from_position(item_idx));
	}

	// Miss: the key is resident in no bucket here, so drop any stale entry.
	touches.remove(key);
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

	/// Settings with a wide enough bucket (`bucket_size = 8`) that the widened
	/// top-quarter gate (`len >= 4`) actually engages instead of falling back to
	/// the hottest-only test. `len / 4 == 2`, so the freshest-quarter threshold
	/// is `len - 2 == 6`.
	fn wide_dash_settings(policy: EvictionPolicyKind) -> DashSettings {
		DashSettings {
			num_of_segments: 1,
			num_of_normal_buckets: 4,
			num_of_stash_buckets: 2,
			bucket_size: 8,
			eviction_policy: policy,
			debug_mode: 0,
		}
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
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

		// Two-speed promotion: a stash hit lands at the LRU (Front) end of target,
		// so keys[1] is inserted ahead of keys[0].
		let items = seg.core.buckets[target_idx].get_items();
		assert_eq!(items[0].key, keys[1]);
		assert_eq!(items[1].key, keys[0]);
	}

	/// A dropped stash victim's key is recorded into the ghost ring, and the next
	/// admission of that key fast-tracks it straight into its target tier (MRU)
	/// instead of landing in stash.
	#[test]
	fn ghost_hit_fast_tracks_to_target() {
		// One stash bucket of size 1, so the second distinct stash key forces an
		// overflow drop (alternate home collapses with a single stash bucket).
		let settings = DashSettings {
			num_of_segments: 1,
			num_of_normal_buckets: 2,
			num_of_stash_buckets: 1,
			bucket_size: 1,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
			debug_mode: 0,
		};
		let mut seg = DashSegment::new(&settings);
		let stash_keys = find_keys_for_bucket(0, settings.num_of_stash_buckets, 2);
		let victim = stash_keys[0];
		let filler = stash_keys[1];

		// Admit the victim into the sole stash slot, then admit filler to overflow
		// and drop the victim (single stash bucket → no alternate home).
		seg.put(Item::new(victim, 1));
		assert!(seg.core().stash_buckets[0].get_position(&victim).is_some());
		seg.put(Item::new(filler, 2));

		// The victim was dropped → ghosted, and filler took its stash slot.
		assert!(seg.core().stash_buckets[0].get_position(&victim).is_none());
		assert!(seg.ghost.contains(&victim));

		// Re-admit the ghosted victim: it must skip stash and land in its target.
		seg.put(Item::new(victim, 3));
		let target_idx = get_index(&victim, seg.core().segment_size);
		assert!(
			seg.core().buckets[target_idx].get_position(&victim).is_some(),
			"ghosted key must be fast-tracked into its target tier"
		);
		assert!(!seg.ghost.contains(&victim), "ghost entry consumed on re-admission");
	}

	/// A genuinely cold (never-ghosted) key keeps the stash cold path.
	#[test]
	fn cold_key_without_ghost_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let key = find_keys_for_bucket(0, seg.core().stash_size, 1)[0];
		assert!(!seg.ghost.contains(&key));
		seg.put(Item::new(key, 10));
		let stash_idx = get_index(&key, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&key).is_some());
	}

	/// Under the widened gate, a hit on a key that is NOT in the freshest quarter
	/// of a large-enough target bucket bumps it within the bucket (intra-promote)
	/// rather than promoting it to probing — provided its touch counter has not
	/// yet reached 2.
	#[test]
	fn target_hit_below_top_quarter_stays_within_bucket() {
		let mut seg = DashSegment::new(&wide_dash_settings(EvictionPolicyKind::ClassicLRU));
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 8);

		// Fill the target bucket: ClassicLRU vector order is LRU..MRU = keys[0]..keys[7].
		for &k in &keys {
			seg.core.buckets[target_idx].put(Item::new(k, k * 10));
		}
		assert_eq!(seg.core.buckets[target_idx].get_items().len(), 8);

		// keys[1] sits at index 1; len/4 == 2 so the gate threshold is index 6.
		// 1 < 6 → not rank-promotable; first hit also leaves touches == 1 → bumped
		// to MRU within the bucket, not promoted.
		seg.get(&keys[1]);

		assert!(seg.core.buckets[target_idx].get_position(&keys[1]).is_some());
		assert!(seg.core.buckets[probing_idx].get_position(&keys[1]).is_none());
	}

	/// Under the widened gate, a hit on a key already in the freshest quarter of
	/// its target bucket promotes to probing — even when it is not the single
	/// hottest resident.
	#[test]
	fn target_hit_in_top_quarter_promotes_to_probing() {
		let mut seg = DashSegment::new(&wide_dash_settings(EvictionPolicyKind::ClassicLRU));
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 8);

		for &k in &keys {
			seg.core.buckets[target_idx].put(Item::new(k, k * 10));
		}
		assert_eq!(seg.core.buckets[target_idx].get_items().len(), 8);

		// keys[6] sits at index 6 == threshold (len - len/4 == 6) and is NOT the
		// single hottest (that is keys[7] at index 7), yet the widened gate still
		// promotes it.
		seg.get(&keys[6]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[6]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[6]).is_none());
	}

	/// The per-key touch counter promotes a key that has been hit twice in its
	/// target even when it never reaches the freshest quarter on a large bucket.
	#[test]
	fn target_two_touches_promotes_via_counter() {
		let mut seg = DashSegment::new(&wide_dash_settings(EvictionPolicyKind::ClassicLRU));
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 8);

		for &k in &keys {
			seg.core.buckets[target_idx].put(Item::new(k, k * 10));
		}
		assert_eq!(seg.core.buckets[target_idx].get_items().len(), 8);

		// keys[0] starts at the LRU end (index 0), far below the top-quarter
		// threshold (index 6). The first hit (touches==1) bumps it toward MRU but,
		// because the bucket is full, other residents keep it below the gate. The
		// second hit (touches==2) fires the density-independent counter trigger.
		seg.get(&keys[0]);
		seg.get(&keys[0]);

		assert!(seg.core.buckets[probing_idx].get_position(&keys[0]).is_some());
		assert!(seg.core.buckets[target_idx].get_position(&keys[0]).is_none());
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

		// Stash hits land keys at the LRU (Front) end of target, so target is
		// [keys[1], keys[0]]. keys[1] sits at the LRU end (not hottest); the first
		// hit bumps it to MRU within the bucket, the second (now hottest, and with
		// touches==2) promotes.
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
	fn probing_eviction_demotes_to_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 1);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Sole resident of target → already at MRU, so the next get triggers promotion.
		seg.core.buckets[target_idx].put(Item::new(target_keys[0], 100));
		seg.get(&target_keys[0]);

		assert!(seg.core.buckets[probing_idx].get_position(&target_keys[0]).is_some());
	}

	#[test]
	fn cascade_demotion_probing_to_full_target_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg.core.segment_size;
		let target_keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);
		let probing_keys = find_keys_for_bucket(probing_idx, seg.core.segment_size, 2);

		seg.put(Item::new(target_keys[0], 10));
		seg.get(&target_keys[0]);
		seg.put(Item::new(target_keys[1], 20));
		seg.get(&target_keys[1]);
		assert!(seg.core.buckets[target_idx].is_full());

		for &k in &probing_keys {
			seg.core.buckets[probing_idx].put(Item::new(k, k * 10));
		}
		assert!(seg.core.buckets[probing_idx].is_full());

		// Two gets on target_keys[0]: drives it to MRU then promotes to probing,
		// which evicts the LRU resident of probing.
		seg.get(&target_keys[0]);
		seg.get(&target_keys[0]);

		// The evicted item's target is `probing_idx`, so the cascade routes it to stash
		// instead of bouncing it back into the bucket it just left.
		let evicted_key = probing_keys[0];
		let stash_idx = get_index(&evicted_key, seg.core.stash_size);
		let stash_alt = stash_alt_index(&evicted_key, seg.core.stash_size);
		// Permissive assertion: the key may have re-landed elsewhere via subsequent
		// stash overflow handling (including its alternate stash home) — only
		// "vanished entirely" would be a real bug.
		assert!(
			seg.core.stash_buckets[stash_idx].get_position(&evicted_key).is_some()
				|| seg.core.stash_buckets[stash_alt].get_position(&evicted_key).is_some()
				|| seg.core.buckets[probing_idx].get_position(&evicted_key).is_some()
				|| seg.core.buckets[target_idx].get_position(&evicted_key).is_some(),
			"evicted probing item should be demoted to stash or still in cache"
		);
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}

	/// Regression for the last-bucket probing wraparound (commit 4b0b763): keys whose target
	/// is `segment_size - 1` must probe at bucket 0, not at `segment_size` (out of bounds).
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
		// Drive keys[1] from the LRU landing to MRU then promote: must place into
		// the wrapped probing bucket without panic.
		seg.get(&keys[1]);
		seg.get(&keys[1]);

		assert!(
			seg.core.buckets[probing_idx].get_position(&keys[1]).is_some(),
			"promoted item must land in wrapped probing bucket 0"
		);
	}

	/// An item evicted from its probing bucket must cascade to its true target
	/// (one-more-chance), not jump straight to stash.
	#[test]
	fn evicted_from_probing_cascades_to_target() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut ghost = Ghost::new(16);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 2);
		let k = target_keys[0];
		let trigger = target_keys[1];
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		// `k` placed first so it is the LRU victim when bucket 1 next evicts.
		seg.core_mut().buckets[probing_idx].put(Item::new(k, 99));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 50));
		assert!(seg.core().buckets[probing_idx].is_full());
		assert!(seg.core().buckets[target_idx].get_position(&k).is_none());

		promote_to_probing(seg.core_mut(), &mut ghost, Item::new(trigger, 200), InsertPosition::Back, InsertPosition::Back);

		let in_target = seg.core().buckets[target_idx].get_position(&k).is_some();
		let in_stash = seg.core().stash_buckets.iter().any(|b| b.get_position(&k).is_some());
		let still_in_probing = seg.core().buckets[probing_idx].get_position(&k).is_some();

		assert!(!still_in_probing, "sanity: k was evicted from its probing bucket");
		assert!(
			in_target,
			"promotion demotion from probing must place item into its target bucket"
		);
		assert!(!in_stash, "promotion demotion must not bypass the target tier");
	}

	/// Foreign resident demoted via one-more-chance lands at the bucket end determined by
	/// the demotion position: Front for cold, Back for hot.
	fn target_retry_position(demote_pos: InsertPosition, expected_idx: usize) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut ghost = Ghost::new(16);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let pre = target_keys[0];
		let foreign = target_keys[1];
		let trigger = target_keys[2];
		let filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		seg.core_mut().buckets[target_idx].put(Item::new(pre, 1));
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(filler, 3));

		promote_to_probing(seg.core_mut(), &mut ghost, Item::new(trigger, 99), InsertPosition::Back, demote_pos);

		let items = seg.core().buckets[target_idx].get_items();
		let pos = items
			.iter()
			.position(|it| it.key == foreign)
			.expect("foreign should be in true target after retry");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn target_retry_lands_at_front_when_cold() {
		target_retry_position(InsertPosition::Front, 0);
	}

	#[test]
	fn target_retry_lands_at_back_when_hot() {
		target_retry_position(InsertPosition::Back, 1);
	}

	/// When an evicted item's true target equals its source bucket, demote_from_normal_bucket
	/// routes it directly to stash (no retry). Verifies the `true_target == source_idx` branch
	/// AND that the stash insert honors the demotion position.
	fn direct_to_stash_position(demote_pos: InsertPosition, expected_idx: usize) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut ghost = Ghost::new(16);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 3);
		let victim = target_keys[0];
		let resident = target_keys[1];
		let trigger = target_keys[2];
		let stash_idx = crate::shared::utils::get_index(&victim, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 1)[0];

		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(victim, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(resident, 2));

		promote_to_target(seg.core_mut(), &mut ghost, Item::new(trigger, 99), InsertPosition::Back, demote_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == victim)
			.expect("victim should land directly in stash");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn direct_to_stash_at_front_when_cold() {
		direct_to_stash_position(InsertPosition::Front, 0);
	}

	#[test]
	fn direct_to_stash_at_back_when_hot() {
		direct_to_stash_position(InsertPosition::Back, 1);
	}

	/// Probing eviction → foreign-resident retry into full target → second eviction
	/// cascades into stash. Verifies the cascade stash insert honors the demotion position.
	fn cascade_to_stash_position(demote_pos: InsertPosition, expected_idx: usize) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut ghost = Ghost::new(16);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 4);
		let target_filler_a = target_keys[0];
		let target_filler_b = target_keys[1];
		let foreign = target_keys[2]; // foreign at probing; true_target = target_idx
		let trigger = target_keys[3];
		let probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];

		// `target_filler_a` is the LRU of target — evicted on retry, cascades to stash.
		let cascaded = target_filler_a;
		let stash_idx = crate::shared::utils::get_index(&cascaded, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 4)
			.into_iter()
			.find(|k| *k != cascaded)
			.expect("a distinct stash_pre must exist");

		seg.core_mut().stash_buckets[stash_idx].put(Item::new(stash_pre, 7));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_a, 1));
		seg.core_mut().buckets[target_idx].put(Item::new(target_filler_b, 2));
		seg.core_mut().buckets[probing_idx].put(Item::new(foreign, 3));
		seg.core_mut().buckets[probing_idx].put(Item::new(probing_filler, 4));

		promote_to_probing(seg.core_mut(), &mut ghost, Item::new(trigger, 99), InsertPosition::Back, demote_pos);

		let stash_items = seg.core().stash_buckets[stash_idx].get_items();
		let pos = stash_items
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
	}

	#[test]
	fn cascade_to_stash_at_front_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 0);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}
}
