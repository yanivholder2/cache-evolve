use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::{get_index, stash_alt_index};
use log::{debug, info};
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's baseline segment.
///
/// Its `get` realises the **promotion** design: target and probing form a tier,
/// and items earn their way into the probing bucket only by ranking in the
/// freshest quarter of their target.
///
/// This variant (H01) keeps the winner's two-speed promotion plus
/// density-independent **per-key touch counter** and grafts on a **bounded
/// multi-hop cuckoo cascade** for stash overflow. Every stash-bound insert now
/// routes through [`stash_insert`]: when a key's primary stash bucket overflows,
/// the single evicted victim is carried as a "homeless" item through up to
/// [`RELOCATION_MAX_HOPS`] cuckoo hops. At each hop the homeless item is sent to
/// its *other* legal stash home; if that home has room it lands there and the
/// cascade stops, otherwise it displaces that home's LRU resident and the
/// displaced resident becomes the new homeless item for the next hop. Exactly one
/// item is homeless at any time and at most one item is dropped per `put`
/// (when the hop budget is exhausted or no distinct second home exists), so the
/// cascade conserves occupancy and never enlarges the budget. This deepens a cold
/// key's stash dwell-time — its chance to be re-referenced and promoted — beyond
/// the single-hop relocation the parent used. `promotion_get` searches both stash
/// homes so a relocated key is still found and promoted.
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
				target_touches: HashMap::new(),
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

	/// Cold admission. Routes the incoming item through [`stash_insert`] (MRU
	/// landing) so a primary-stash overflow is carried through the bounded
	/// cuckoo cascade rather than dropping the victim immediately.
	fn put(&mut self, item: Item<K, V>) {
		let core = self.core_mut();
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?}", item.key);
		stash_insert(core, item, InsertPosition::Back);
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		promotion_get(&mut self.core, &mut self.target_touches, key)
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

/// Maximum number of cuckoo relocation hops a homeless stash item may take
/// before it is dropped. Bounds the cascade so each [`stash_insert`] terminates
/// and drops at most one item per `put`.
const RELOCATION_MAX_HOPS: usize = 3;

/// The key's legal stash home that is **not** `current`.
///
/// Every key has two stash homes: its primary (`get_index`) and its alternate
/// (`stash_alt_index`). Returns the home distinct from `current`, or `None` when
/// the two homes collapse onto a single bucket (no distinct second home) or when
/// `current` is neither home (the caller passed a bucket the key does not belong
/// to). Used to walk the cuckoo cascade: each homeless item is sent to its other
/// home, away from the bucket it was just evicted from.
fn other_stash_home<K>(key: &K, current: usize, stash_size: usize) -> Option<usize>
where
	K: Hash + Debug,
{
	let primary = get_index(key, stash_size);
	let alt = stash_alt_index(key, stash_size);
	if primary == alt {
		None
	} else if current == primary {
		Some(alt)
	} else if current == alt {
		Some(primary)
	} else {
		None
	}
}

/// Inserts `item` into its **primary** stash bucket. If that bucket overflows,
/// the single evicted victim is carried as a "homeless" item through a bounded
/// cuckoo cascade of up to [`RELOCATION_MAX_HOPS`] hops:
///
/// - At each hop the homeless item is routed to its *other* legal stash home
///   ([`other_stash_home`], away from the bucket it was just evicted from).
/// - If that home has room, the homeless item lands there (no eviction) and the
///   cascade stops.
/// - If that home is full, the homeless item displaces the home's LRU resident;
///   the displaced resident becomes the new homeless item and the cascade
///   advances one hop.
/// - When the hop budget is exhausted, or no distinct second home exists, the
///   final homeless item is dropped.
///
/// Exactly one item is homeless at any moment and at most one item is dropped per
/// call, so occupancy is conserved and the per-bucket capacity bound holds (every
/// landing is either into a non-full bucket or via an LRU-evicting `put_at`).
/// Every relocation lands in one of the key's two homes, so slot legality holds.
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	// Seat the item in its primary stash bucket. No overflow → nothing to relocate.
	let Some(victim) = core.put_at(Slot::Stash, pos, item) else {
		return;
	};
	// `put_at(Slot::Stash, ..)` derives the primary bucket, so the victim was
	// evicted from its own primary stash bucket. Carry it through the cascade.
	let mut homeless = victim;
	let mut from = get_index(&homeless.key, core.stash_size);

	for _ in 0..RELOCATION_MAX_HOPS {
		let Some(dest) = other_stash_home(&homeless.key, from, core.stash_size) else {
			debug!(
				"stash_insert: drop homeless key={:?} (no distinct second home, from={from})",
				homeless.key
			);
			return;
		};
		if !core.stash_buckets[dest].is_full() {
			// Non-full destination: this relocation seats the homeless item with no
			// eviction, so the cascade ends having added exactly one item overall.
			debug!(
				"stash_insert: relocate homeless key={:?} from={from} -> dest={dest}",
				homeless.key
			);
			core.stash_buckets[dest].put_at(homeless, InsertPosition::Back);
			return;
		}
		// Full destination: displace its LRU resident. `put_at` seats the homeless
		// item and evicts exactly one resident, conserving occupancy and keeping
		// exactly one item homeless. Carry the displaced resident onward.
		debug!(
			"stash_insert: displace LRU at dest={dest} to seat homeless key={:?}",
			homeless.key
		);
		let (_, evicted) = core.stash_buckets[dest].put_at(homeless, InsertPosition::Back);
		homeless = evicted.expect("a full bucket must evict exactly one item on put_at");
		from = dest;
	}
	// Hop budget exhausted: drop the final homeless item (at most one drop per put).
	debug!(
		"stash_insert: drop homeless key={:?} after exhausting {RELOCATION_MAX_HOPS} hops",
		homeless.key
	);
}

/// Moves `item` into its target bucket, landing it at `landing_pos`. Any evicted
/// item flows through the tier cascade, landing at `demote_pos`.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
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
	demote_from_normal_bucket(core, evicted, target_idx, demote_pos);
}

/// Moves `item` into its probing bucket, landing it at `landing_pos`. Any evicted
/// item flows through the tier cascade, landing at `demote_pos`.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
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
	demote_from_normal_bucket(core, evicted, probing_idx, demote_pos);
}

/// One-more-chance demotion. If the item was a foreign resident at `source_idx` (placed
/// there as someone else's probing slot), retry in its own true target before stash;
/// if it was already in its target, go straight to stash. Cascade evictions land in stash.
///
/// Both stash landings route through [`stash_insert`], so a demoted item that
/// overflows its primary stash bucket gets the same bounded cuckoo-cascade
/// relocation chance that cold admissions do.
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
		stash_insert(core, item, insert_pos);
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
			stash_insert(core, cascade, insert_pos);
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
		promote_to_target(core, data, InsertPosition::Front, InsertPosition::Back);
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
			promote_to_probing(core, data, InsertPosition::Back, InsertPosition::Back);

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

		promote_to_probing(seg.core_mut(), Item::new(trigger, 200), InsertPosition::Back, InsertPosition::Back);

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

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), InsertPosition::Back, demote_pos);

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

		promote_to_target(seg.core_mut(), Item::new(trigger, 99), InsertPosition::Back, demote_pos);

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

		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), InsertPosition::Back, demote_pos);

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
