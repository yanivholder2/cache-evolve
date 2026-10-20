use super::{find_in, Segment, SegmentCore, Slot};
use crate::dash::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use crate::shared::utils::get_index;
use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;

/// The dash Subject's baseline segment (promotion design, two-choice stash,
/// borrowed/foreign-count probe filters).
///
/// Storage: every bucket's slots live inline in one flat per-tier item array inside
/// [`SegmentCore`] (bucket `i` = `items[i*cap .. i*cap + len[i]]`), and the eviction
/// policy is a statically dispatched match on positions — no per-bucket heap buffer and
/// no boxed policy on the hot path. Observable behaviour is identical to the boxed-bucket
/// layout: same lookup order, same recency moves, same victims, same counters.
///
/// Lookup order: stash `s0` → target → probing (skipped when `foreign[probing] == 0`) →
/// stash `s1` (skipped when `borrowed[s1] == 0`).
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

/// Stash placement for brand-new `put` arrivals: the LRU end.
const FRESH_PUT_STASH_POS: InsertPosition = InsertPosition::Front;

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

	/// Precondition: key not resident. A fresh insert lands at the stash's LRU end with
	/// two-choice placement; any item evicted by that insert leaves the cache.
	fn put(&mut self, item: Item<K, V>) {
		let core = &mut self.core;
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("DashSegment.put: key={:?} -> stash {FRESH_PUT_STASH_POS:?}", item.key);
		stash_insert(core, FRESH_PUT_STASH_POS, item);
	}
}

/// Two-choice stash insert: fill `s0` if it has room, else `s1` (if distinct) if it has
/// room, else insert into `s0` and return its eviction (which leaves the cache).
#[inline]
fn stash_insert<K, V>(core: &mut SegmentCore<K, V>, pos: InsertPosition, item: Item<K, V>) -> Option<Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let (s0, s1) = core.stash_indices(&item.key);
	if !core.stash_buckets.is_full(s0) {
		return core.put_stash_at_index(s0, pos, item);
	}
	if s1 != s0 && !core.stash_buckets.is_full(s1) {
		debug!("stash_insert: key={:?} s0={s0} full -> s1={s1}", item.key);
		return core.put_stash_at_index(s1, pos, item);
	}
	core.put_stash_at_index(s0, pos, item)
}

/// Stash placement for items demoted out of a normal bucket (SLRU: MRU end).
const DEMOTE_TO_STASH_POS: InsertPosition = InsertPosition::Back;

/// Moves `item` into its target bucket `target_idx`. Any evicted item flows through the
/// tier cascade.
fn promote_to_target<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	target_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(target_idx, get_index(&item.key, core.segment_size));
	let Some(evicted) = core.put_at(Slot::Target, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, target_idx, insert_pos);
}

/// Moves `item` into its probing bucket `probing_idx`. Any evicted item flows through the
/// tier cascade.
fn promote_to_probing<K, V>(
	core: &mut SegmentCore<K, V>,
	item: Item<K, V>,
	probing_idx: usize,
	insert_pos: InsertPosition,
) where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug_assert_eq!(
		probing_idx,
		(get_index(&item.key, core.segment_size) + 1) % core.segment_size
	);
	let Some(evicted) = core.put_at(Slot::Probing, insert_pos, item) else {
		return;
	};
	demote_from_normal_bucket(core, evicted, probing_idx, insert_pos);
}

/// One-more-chance demotion. A foreign victim at `source_idx` retries in its own true
/// target before stash; a native victim goes straight to stash. Cascade evictions land in
/// stash. Never writes into `source_idx`. Keeps `foreign` exact.
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
	core.note_normal_removal(source_idx, true_target);
	if true_target == source_idx {
		debug!("demote_from_normal_bucket: key={:?} target={source_idx} -> stash", item.key);
		stash_insert(core, DEMOTE_TO_STASH_POS, item);
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
			let cascade_target = get_index(&cascade.key, core.segment_size);
			core.note_normal_removal(true_target, cascade_target);
			stash_insert(core, DEMOTE_TO_STASH_POS, cascade);
		}
	}
}

/// Position of a freshly inserted item in a bucket of length `len` after an insert at
/// `insert_pos` that evicted (if full) *before* inserting.
#[inline]
fn inserted_position(insert_pos: InsertPosition, len: usize) -> usize {
	match insert_pos {
		InsertPosition::Front => 0,
		InsertPosition::Back => len - 1,
	}
}

/// Stash-hit branch shared by the `s0` and `s1` lookups: removes the item at `item_idx`
/// of stash bucket `stash_idx`, promotes it into `target_idx`, and returns it.
#[inline]
fn stash_hit<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	stash_idx: usize,
	item_idx: usize,
	target_idx: usize,
	insert_pos: InsertPosition,
) -> &'a Item<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	debug!("promotion_get: key={key:?} found in stash={stash_idx} -> target={target_idx} insert_pos={insert_pos:?}");
	let data = core.remove_stash_at(stash_idx, item_idx);
	promote_to_target(core, data, target_idx, insert_pos);

	// The cascade never writes back into target_idx, so the new item sits exactly
	// where the insert put it.
	let items = core.buckets.slice(target_idx);
	let pos = inserted_position(insert_pos, items.len());
	debug_assert_eq!(find_in(items, key), Some(pos));
	&items[pos]
}

/// Promotion lookup over the flat bucket arrays.
///
/// Lookup order: stash `s0` → target → probing (skipped when `foreign[probing] == 0`) →
/// stash `s1` (skipped when `borrowed[s1] == 0`).
#[inline]
fn promotion_get<'a, K, V>(
	core: &'a mut SegmentCore<K, V>,
	key: &K,
	insert_pos: InsertPosition,
) -> Option<&'a Item<K, V>>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	let s0 = get_index(key, core.stash_size);
	let target_idx = get_index(key, core.segment_size);

	if let Some(item_idx) = find_in(core.stash_buckets.slice(s0), key) {
		return Some(stash_hit(core, key, s0, item_idx, target_idx, insert_pos));
	}

	let probing_idx = (target_idx + 1) % core.segment_size;

	if let Some(item_idx) = find_in(core.buckets.slice(target_idx), key) {
		let is_hottest = Some(item_idx) == core.buckets.hottest(target_idx);
		if is_hottest {
			debug!("promotion_get: key={key:?} found in target as hottest -> probing={probing_idx}");
			// The removed item is native to target_idx, so `foreign` is unchanged.
			let data = core.buckets.remove_at(target_idx, item_idx);
			promote_to_probing(core, data, probing_idx, insert_pos);

			// The cascade never writes back into probing_idx, so the new item sits
			// exactly where the insert put it.
			let items = core.buckets.slice(probing_idx);
			let pos = inserted_position(insert_pos, items.len());
			debug_assert_eq!(find_in(items, key), Some(pos));
			return Some(&items[pos]);
		} else {
			debug!("promotion_get: key={key:?} found in target item_idx={item_idx} (intra-promote)");
			return Some(core.buckets.touch(target_idx, item_idx));
		}
	}

	// `foreign[probing_idx] == 0` proves no key whose target is target_idx is resident in
	// probing_idx.
	if core.foreign[probing_idx] != 0 {
		if let Some(item_idx) = find_in(core.buckets.slice(probing_idx), key) {
			debug!("promotion_get: key={key:?} found in probing bucket={probing_idx}");
			return Some(core.buckets.touch(probing_idx, item_idx));
		}
	}

	// `borrowed[s1] == 0` proves no key with primary s0 is resident in s1.
	let s1 = core.stash_neighbor(s0);
	if s1 != s0 && core.borrowed[s1] != 0 {
		if let Some(item_idx) = find_in(core.stash_buckets.slice(s1), key) {
			return Some(stash_hit(core, key, s1, item_idx, target_idx, insert_pos));
		}
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

	/// Raw Back insert into normal bucket `i` with no counter bookkeeping (the old
	/// `buckets[i].put(item)`); callers use native keys so `foreign` stays exact.
	fn raw_put_normal(seg: &mut DashSegment<i64, i64>, i: usize, item: Item<i64, i64>) {
		seg.core_mut().buckets.put_at(i, item, InsertPosition::Back);
	}

	#[test]
	fn put_goes_to_stash() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		seg.put(Item::new(1_i64, 10_i64));
		let stash_size = seg.core().stash_size;
		let stash_idx = get_index(&1_i64, stash_size);
		assert!(seg.core().stash_position(stash_idx, &1).is_some());

		let second = find_keys_for_bucket(stash_idx, stash_size, 4)
			.into_iter()
			.find(|k| *k != 1)
			.expect("a second key for the same stash bucket must exist");
		seg.put(Item::new(second, 20));
		assert_eq!(seg.core().stash_position(stash_idx, &second), Some(0));
		assert!(seg.core().stash_position(stash_idx, &1).is_some());
	}

	#[test]
	fn two_choice_fresh_put_overflows_to_neighbor() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		let keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		for &k in &keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		assert!(seg.core().stash_is_full(s0));
		let before = seg.core().total_items();
		let k = keys[settings.bucket_size];
		seg.put(Item::new(k, 42));
		assert_eq!(seg.core().total_items(), before + 1, "no eviction");
		assert!(seg.core().stash_position(s1, &k).is_some());
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_borrowed_counts());
	}

	#[test]
	fn two_choice_both_full_evicts_s0_lru() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let stash_size = settings.num_of_stash_buckets;
		let s0 = 0;
		let s1 = (s0 + 1) % stash_size;
		let s0_keys = find_keys_for_bucket(s0, stash_size, settings.bucket_size + 1);
		let s1_keys = find_keys_for_bucket(s1, stash_size, settings.bucket_size);
		for &k in &s1_keys {
			seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, k));
		}
		for &k in &s0_keys[..settings.bucket_size] {
			seg.put(Item::new(k, k));
		}
		let lru = seg.core().stash_items(s0)[0].key;
		let k = s0_keys[settings.bucket_size];
		seg.put(Item::new(k, 42));
		assert!(!seg.core().contains_key(&lru), "s0 LRU must be evicted");
		assert_eq!(seg.core().stash_position(s0, &k), Some(0));
		assert!(seg.core().check_borrowed_counts());
	}

	#[test]
	fn s1_resident_found_and_promoted_to_target_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let tkeys = find_keys_for_bucket(target_idx, seg_size, 3);
		let other = tkeys[0];
		let k = tkeys[1];
		let s0 = get_index(&k, stash_size);
		let s1 = (s0 + 1) % stash_size;
		assert_ne!(s0, s1);

		raw_put_normal(&mut seg, target_idx, Item::new(other, 5));
		seg.core_mut().put_stash_at_index(s1, InsertPosition::Back, Item::new(k, 777));
		assert_eq!(seg.core().borrowed[s1], 1);

		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 777)));
		assert_eq!(seg.core().normal_position(target_idx, &k), Some(0));
		assert!(seg.core().normal_position(target_idx, &other).is_some());
		assert!(seg.core().stash_position(s1, &k).is_none());
		assert_eq!(seg.core().borrowed[s1], 0);
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
	}

	#[test]
	fn borrowed_counts_exact_under_random_ops() {
		for policy in EvictionPolicyKind::all() {
			let settings = small_dash_settings(policy);
			let mut seg = DashSegment::new(&settings);
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
			for _ in 0..5000 {
				state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let key = ((state >> 33) % 200) as i64;
				let resident = seg.core().contains_key(&key);
				let hit = seg.get(&key).map(|it| it.key);
				assert_eq!(hit.is_some(), resident, "policy={policy:?} key={key}");
				if let Some(k) = hit {
					assert_eq!(k, key);
				} else {
					seg.put(Item::new(key, key));
				}
				assert!(seg.core().check_borrowed_counts(), "borrowed drift after key={key}");
				assert!(seg.core().check_foreign_counts(), "foreign drift after key={key}");
				assert!(seg.core().check_slot_legality());
				assert!(seg.core().check_capacity_bound());
			}
		}
	}

	#[test]
	fn foreign_counts_exact_under_skewed_random_ops() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let mut state: u64 = 0x1234_5678_9ABC_DEF0;
		for _ in 0..8000 {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let r = (state >> 33) % 1000;
			let key = if r < 700 { (r % 12) as i64 } else { (r % 120) as i64 };
			let resident = seg.core().contains_key(&key);
			let hit = seg.get(&key).is_some();
			assert_eq!(hit, resident, "get must find every resident key (key={key})");
			if !hit {
				seg.put(Item::new(key, key));
			}
			assert!(seg.core().check_foreign_counts(), "foreign drift after key={key}");
			assert!(seg.core().check_borrowed_counts());
		}
	}

	#[test]
	fn foreign_count_tracks_promotion_and_one_more_chance() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let k = find_keys_for_bucket(target_idx, seg_size, 1)[0];
		let natives = find_keys_for_bucket(probing_idx, seg_size, settings.bucket_size);

		raw_put_normal(&mut seg, target_idx, Item::new(k, 7));
		assert_eq!(seg.get(&k).map(|it| it.value), Some(7));
		assert!(seg.core().normal_position(probing_idx, &k).is_some());
		assert_eq!(seg.core().foreign[probing_idx], 1);

		for &n in &natives[..settings.bucket_size - 1] {
			seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(n, n));
		}
		assert!(seg.core().normal_is_full(probing_idx));
		let trigger = natives[settings.bucket_size - 1];
		promote_to_target(seg.core_mut(), Item::new(trigger, trigger), probing_idx, InsertPosition::Back);

		assert!(seg.core().normal_position(probing_idx, &k).is_none());
		assert!(seg.core().normal_position(target_idx, &k).is_some());
		assert_eq!(seg.core().foreign[probing_idx], 0);
		assert!(seg.core().check_foreign_counts());
		assert!(seg.core().check_borrowed_counts());
	}

	#[test]
	fn stash_hit_on_full_target_returns_item_at_front() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let target_idx = 0;
		let keys = find_keys_for_bucket(target_idx, seg_size, settings.bucket_size + 1);
		let k = keys[settings.bucket_size];
		for &f in &keys[..settings.bucket_size] {
			raw_put_normal(&mut seg, target_idx, Item::new(f, f * 10));
		}
		assert!(seg.core().normal_is_full(target_idx));

		seg.put(Item::new(k, 777));
		let got = seg.get(&k).map(|it| (it.key, it.value));
		assert_eq!(got, Some((k, 777)));
		assert_eq!(seg.core().normal_position(target_idx, &k), Some(0));
		assert!(seg.core().normal_is_full(target_idx));
		assert!(seg.core().check_slot_legality());
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
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
		let items = seg.core.normal_items(target_idx);
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
		assert!(seg.core.normal_position(probing_idx, &keys[1]).is_some());
		assert!(seg.core.normal_position(target_idx, &keys[1]).is_none());
		assert_eq!(seg.core.foreign[probing_idx], 1);
		assert!(seg.core.check_foreign_counts());
	}

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

		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(k, 99));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(filler, 50));
		assert!(seg.core().normal_is_full(probing_idx));

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 200), trigger_probing, InsertPosition::Back);

		assert!(seg.core().normal_position(probing_idx, &k).is_none());
		assert!(seg.core().normal_position(target_idx, &k).is_some());
		assert!(seg.core().check_foreign_counts());
	}

	fn cascade_to_stash_position(insert_pos: InsertPosition, expected_idx: usize) {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut seg = DashSegment::new(&settings);
		let seg_size = settings.num_of_normal_buckets;
		let stash_size = settings.num_of_stash_buckets;
		let target_idx = 0;
		let probing_idx = (target_idx + 1) % seg_size;
		let target_keys = find_keys_for_bucket(target_idx, seg_size, 4);
		let cascaded = target_keys[0];
		let target_filler_b = target_keys[1];
		let foreign = target_keys[2];
		let trigger = target_keys[3];
		let probing_filler = find_keys_for_bucket(probing_idx, seg_size, 1)[0];
		let stash_idx = get_index(&cascaded, stash_size);
		let stash_pre = find_keys_for_bucket(stash_idx, stash_size, 4)
			.into_iter()
			.find(|k| *k != cascaded)
			.expect("a distinct stash_pre must exist");

		seg.core_mut().put_stash_at_index(stash_idx, InsertPosition::Back, Item::new(stash_pre, 7));
		raw_put_normal(&mut seg, target_idx, Item::new(cascaded, 1));
		raw_put_normal(&mut seg, target_idx, Item::new(target_filler_b, 2));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(foreign, 3));
		seg.core_mut().put_normal_at_index(probing_idx, InsertPosition::Back, Item::new(probing_filler, 4));

		let trigger_probing = (get_index(&trigger, seg_size) + 1) % seg_size;
		promote_to_probing(seg.core_mut(), Item::new(trigger, 99), trigger_probing, insert_pos);

		let pos = seg
			.core()
			.stash_items(stash_idx)
			.iter()
			.position(|it| it.key == cascaded)
			.expect("cascaded target item should land in stash via target overflow");
		assert_eq!(pos, expected_idx);
		assert!(seg.core().check_borrowed_counts());
		assert!(seg.core().check_foreign_counts());
	}

	#[test]
	fn cascade_to_stash_at_back_when_cold() {
		cascade_to_stash_position(InsertPosition::Front, 1);
	}

	#[test]
	fn cascade_to_stash_at_back_when_hot() {
		cascade_to_stash_position(InsertPosition::Back, 1);
	}

	#[test]
	fn last_bucket_probing_wraps_to_zero() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		let seg_size = seg.core.segment_size;
		let target_idx = seg_size - 1;
		let keys = find_keys_for_bucket(target_idx, seg_size, 2);
		seg.put(Item::new(keys[0], 10));
		seg.get(&keys[0]);
		seg.put(Item::new(keys[1], 20));
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		seg.get(&keys[1]);
		assert!(seg.core.normal_position(0, &keys[1]).is_some());
		assert_eq!(seg.core.foreign[0], 1);
		assert!(seg.core.check_foreign_counts());
	}

	#[test]
	fn miss_returns_none() {
		let mut seg = new_segment(EvictionPolicyKind::ClassicLRU);
		assert!(seg.get(&999).is_none());
	}
}
