use super::{Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::{age_tail_window, record_hit, LRU_TAIL_WINDOW};
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

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
/// Variant cycle/0004/v003 (H03): every eviction — from a normal bucket (the
/// victim demoted toward stash) or from a stash bucket (the item leaving the
/// cache) — is followed, immediately after the victim's removal and before the
/// arriving item is inserted, by `age_tail_window` on that bucket under
/// `ClassicLRU`. The victim itself keeps the counter it had when it left.
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
	/// Creates a new DashSegment with fallible initialization.
	pub fn try_new(settings: &DashSettings) -> Option<Self> {
		if let Some(core) = SegmentCore::try_new(settings) {
			info!("DashSegment.try_new");
			Some(Self { core })
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
		promotion_get(&mut self.core, key, InsertPosition::Front)
	}

	/// Same placement as the default `put` (stash, `Back`), but a stash eviction
	/// it causes ages the stash bucket's tail window.
	fn put(&mut self, item: Item<K, V>) {
		debug_assert!(
			!self.core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?}", item.key);
		put_at_aged(&mut self.core, Slot::Stash, InsertPosition::Back, item);
	}
}

// ── Eviction-pressure ageing ───────────────────────────────────────────────

/// Equivalent of `SegmentCore::put_at`, except that when the destination bucket
/// is full the victim (per `eviction_candidate_index`) is removed here, and —
/// under `ClassicLRU` — the bucket's tail window is aged right after the
/// removal, before `item` is inserted. Returns the evicted item, if any.
fn put_at_aged<K, V>(core: &mut SegmentCore<K, V>, slot: Slot, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let bucket = match slot {
		Slot::Stash => {
			let idx = get_index(&item.key, core.stash_size);
			&mut core.stash_buckets[idx]
		}
		Slot::Target => {
			let idx = get_index(&item.key, core.segment_size);
			&mut core.buckets[idx]
		}
		Slot::Probing => {
			let target_idx = get_index(&item.key, core.segment_size);
			let probing_idx = (target_idx + 1) % core.segment_size;
			&mut core.buckets[probing_idx]
		}
	};

	let mut evicted = None;
	if bucket.is_full() && bucket.get_position(&item.key).is_none() {
		if let Some(victim_idx) = bucket.eviction_candidate_index() {
			let victim = bucket.remove_at(victim_idx);
			if bucket.get_eviction_policy().name() == "ClassicLRU" {
				age_tail_window(bucket.get_items_mut(), LRU_TAIL_WINDOW);
			}
			debug!("put_at_aged: slot={slot:?} pos={pos:?} evicted_key={:?}", victim.key);
			evicted = Some(victim);
		}
	}
	let (_, extra) = bucket.put_at(item, pos);
	debug_assert!(extra.is_none(), "put_at_aged: bucket evicted again after making room");
	evicted.or(extra)
}

// ── Promotion logic ────────────────────────────────────────────────────────
// Promotion treats target and probing as a tier: items earn their way into the
// probing bucket only by proving they're the hottest in their target. Demotion
// cascades down the chain `probing -> target -> stash -> out`, with a
// one-more-chance step between adjacent tiers.

/// Moves `item` into its target bucket. Any evicted item flows through the tier cascade.
fn promote_to_target<K, V>(core: &mut SegmentCore<K, V>, item: Item<K, V>, insert_pos: InsertPosition)
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let target_idx = get_index(&item.key, core.segment_size);
	let Some(evicted) = put_at_aged(core, Slot::Target, insert_pos, item) else {
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
	let Some(evicted) = put_at_aged(core, Slot::Probing, insert_pos, item) else {
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
		put_at_aged(core, Slot::Stash, insert_pos, item);
	} else {
		debug!(
			"demote_from_normal_bucket: key={:?} probing={source_idx} -> target={true_target}",
			item.key
		);
		if let Some(cascade) = put_at_aged(core, Slot::Target, insert_pos, item) {
			debug!(
				"demote_from_normal_bucket: cascade key={:?} target={true_target} -> stash",
				cascade.key
			);
			put_at_aged(core, Slot::Stash, insert_pos, cascade);
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
		// Count the stash hit before the item leaves stash, so it carries a
		// non-zero frequency into the normal tier.
		record_hit(core.stash_buckets[stash_idx].get_items_mut(), item_idx);
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
			// Count the promoting hit too; the counter travels with the item.
			record_hit(core.buckets[target_idx].get_items_mut(), item_idx);
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	fn new_segment(policy: EvictionPolicyKind) -> DashSegment<i64, i64> {
		DashSegment::new(&small_dash_settings(policy))
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_idx = get_index(&1_i64, seg.core().stash_size);
		assert!(seg.core().stash_buckets[stash_idx].get_position(&1).is_some());
	}

	#[test]
	fn stash_eviction_ages_survivors_not_newcomer() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let max = seg.core.stash_buckets[0].get_max_size();
		let keys = find_keys_for_bucket(0, seg.core.stash_size, max + 1);
		for &k in &keys[..max] {
			seg.put(Item::new(k, k));
		}
		for it in seg.core.stash_buckets[0].get_items_mut().iter_mut() {
			it.lfu_counter = 2;
		}
		seg.put(Item::new(keys[max], keys[max]));
		let items = seg.core.stash_buckets[0].get_items();
		assert_eq!(items.len(), max);
		assert!(items.iter().all(|i| i.key != keys[0]), "oldest tie must be the victim");
		for (i, it) in items[..max - 1].iter().enumerate() {
			let expected = if i < LRU_TAIL_WINDOW { 1 } else { 2 };
			assert_eq!(it.lfu_counter, expected, "survivor slot {i}");
		}
		assert_eq!(items[max - 1].key, keys[max]);
		assert_eq!(items[max - 1].lfu_counter, 0);
	}

	#[test]
	fn non_lru_policy_eviction_does_not_age() {
		let mut seg = new_segment(EvictionPolicyKind::Fifo);
		let max = seg.core.stash_buckets[0].get_max_size();
		let keys = find_keys_for_bucket(0, seg.core.stash_size, max + 1);
		for &k in &keys[..max] {
			seg.put(Item::new(k, k));
		}
		for it in seg.core.stash_buckets[0].get_items_mut().iter_mut() {
			it.lfu_counter = 2;
		}
		seg.put(Item::new(keys[max], keys[max]));
		let items = seg.core.stash_buckets[0].get_items();
		assert!(items[..max - 1].iter().all(|i| i.lfu_counter == 2));
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
	fn stash_hit_inserts_at_front_of_target() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg.core.segment_size, 3);

		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);

		// Cold insert pushes keys[1] to Front, displacing keys[0] backward.
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
