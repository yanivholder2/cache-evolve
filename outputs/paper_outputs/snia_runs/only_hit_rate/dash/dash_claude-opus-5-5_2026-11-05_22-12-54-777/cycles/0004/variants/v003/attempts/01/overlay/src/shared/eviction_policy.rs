//! In-crate eviction policy implementations and the selection enum.
//!
//! Behaviour lives in one zero-sized struct per policy, each implementing
//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector
//! carried by settings (and crossed over JNI as an ordinal/name); `build`
//! turns a kind into the boxed trait object a bucket owns.
//!
//! Variant cycle/0002/v002 (H02): `ClassicLRU` becomes a frequency-aware LRU.
//! Recency order is still kept by move-to-back on hit, but the victim is the
//! least-frequently-hit item among the `LRU_TAIL_WINDOW` least-recent slots
//! (ties broken toward the older slot). Every hit bumps `lfu_counter`,
//! saturating at `LFU_COUNTER_CAP`.
//!
//! Variant cycle/0004/v003 (H03): ageing is driven by eviction pressure
//! instead of halve-on-cap. `record_hit` is a plain saturating increment; after
//! each eviction from a `ClassicLRU` bucket the segment calls
//! [`age_tail_window`], decrementing the counters of the window's surviving
//! slots (the victim's competitors) by one, saturating at zero.

use crate::shared::item::Item;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Instant;

/// Number of least-recent slots examined when choosing a victim.
pub const LRU_TAIL_WINDOW: usize = 8;
/// Saturation value of `lfu_counter`.
pub const LFU_COUNTER_CAP: usize = 15;

/// Count one hit on `items[position]`, saturating at `LFU_COUNTER_CAP`.
pub fn record_hit<K, V>(items: &mut [Item<K, V>], position: usize)
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items[position].lfu_counter = (items[position].lfu_counter + 1).min(LFU_COUNTER_CAP);
}

/// Eviction-pressure ageing: decrement `lfu_counter` by one (saturating at 0)
/// for each of the first `window.min(items.len())` (least-recent) slots.
pub fn age_tail_window<K, V>(items: &mut [Item<K, V>], window: usize)
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	let n = window.min(items.len());
	for item in items[..n].iter_mut() {
		item.lfu_counter = item.lfu_counter.saturating_sub(1);
	}
}

/// Frequency-filtered LRU victim: among the first (least-recent)
/// `LRU_TAIL_WINDOW` slots pick the lowest `lfu_counter`; ties go to the older
/// (lower-index) slot. Precondition: `items` is non-empty.
pub fn lru_tail_lfu_victim<K, V>(items: &[Item<K, V>]) -> usize
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	let window = items.len().min(LRU_TAIL_WINDOW);
	let mut best = 0;
	for i in 1..window {
		if items[i].lfu_counter < items[best].lfu_counter {
			best = i;
		}
	}
	best
}

/// Every cache hit moves the item to the back of the bucket and counts the hit;
/// the victim is the least-frequent item in the LRU tail.
#[derive(Debug)]
pub struct ClassicLRU;
/// LRU by last-hit timestamp; items keep their position.
#[derive(Debug)]
pub struct TimestampLRU;
#[derive(Debug)]
pub struct Lifo;
#[derive(Debug)]
pub struct Lfu;
#[derive(Debug)]
pub struct Fifo;
#[derive(Debug)]
pub struct Swap;

impl<K, V> EvictionPolicy<K, V> for ClassicLRU
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"ClassicLRU"
	}

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		record_hit(items, position);
		let item = items.remove(position);
		items.push(item);
		items.len() - 1
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		lru_tail_lfu_victim(items)
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items.len() - 1
	}
}

impl<K, V> EvictionPolicy<K, V> for TimestampLRU
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"TimestampLRU"
	}

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		items[position].timestamp = Instant::now();
		position
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.timestamp)
			.expect("eviction_candidate_index precondition: non-empty")
			.0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items
			.iter()
			.enumerate()
			.max_by_key(|(_, item)| item.timestamp)
			.expect("promotion_candidate_index precondition: non-empty")
			.0
	}
}

impl<K, V> EvictionPolicy<K, V> for Lifo
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"Lifo"
	}

	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		position
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items.len() - 1
	}

	fn promotion_candidate_index(&self, _items: &[Item<K, V>]) -> usize {
		0
	}
}

impl<K, V> EvictionPolicy<K, V> for Lfu
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"Lfu"
	}

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		items[position].lfu_counter += 1;
		position
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.expect("eviction_candidate_index precondition: non-empty")
			.0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items
			.iter()
			.enumerate()
			.max_by_key(|(_, item)| item.lfu_counter)
			.expect("promotion_candidate_index precondition: non-empty")
			.0
	}
}

impl<K, V> EvictionPolicy<K, V> for Fifo
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"Fifo"
	}

	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		position
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items.len() - 1
	}
}

impl<K, V> EvictionPolicy<K, V> for Swap
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"Swap"
	}

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		if position > 0 {
			items.swap(position, position - 1);
			position - 1
		} else {
			position
		}
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items.len() - 1
	}
}

/// Lightweight `Copy` selector for an eviction policy. Carried by settings and
/// crossed over JNI; `build` materialises the boxed trait object.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvictionPolicyKind {
	ClassicLRU,
	TimestampLRU,
	Lifo,
	Lfu,
	Fifo,
	Swap,
}

impl EvictionPolicyKind {
	/// Returns all variants. The exhaustive match ensures a compile error if a variant is added.
	pub fn all() -> [Self; 6] {
		use EvictionPolicyKind::*;
		let _exhaustive = |p: EvictionPolicyKind| match p {
			ClassicLRU | TimestampLRU | Lifo | Lfu | Fifo | Swap => {}
		};
		[ClassicLRU, TimestampLRU, Lifo, Lfu, Fifo, Swap]
	}

	/// Canonical CamelCase name. Adding a variant forces a compile error here.
	pub fn name(&self) -> &'static str {
		match self {
			EvictionPolicyKind::ClassicLRU => "ClassicLRU",
			EvictionPolicyKind::TimestampLRU => "TimestampLRU",
			EvictionPolicyKind::Lifo => "Lifo",
			EvictionPolicyKind::Lfu => "Lfu",
			EvictionPolicyKind::Fifo => "Fifo",
			EvictionPolicyKind::Swap => "Swap",
		}
	}

	/// Stable integer index. Crossed over JNI as the policy ordinal; adding a
	/// variant forces a compile error here.
	pub fn index(&self) -> usize {
		match self {
			EvictionPolicyKind::ClassicLRU => 0,
			EvictionPolicyKind::TimestampLRU => 1,
			EvictionPolicyKind::Lifo => 2,
			EvictionPolicyKind::Lfu => 3,
			EvictionPolicyKind::Fifo => 4,
			EvictionPolicyKind::Swap => 5,
		}
	}

	/// CamelCase names, derived from `all()` + `name()`. Crossed over JNI so
	/// Java callers don't hard-code the list.
	pub fn all_names() -> [&'static str; 6] {
		Self::all().map(|p| p.name())
	}

	pub fn from_usize(index: usize) -> Option<Self> {
		Self::all().into_iter().find(|p| p.index() == index)
	}

	pub fn from_name(name: &str) -> Option<Self> {
		Self::all().into_iter().find(|p| p.name() == name)
	}

	/// Materialises the boxed trait object a bucket owns.
	pub fn build<K, V>(self) -> Box<dyn EvictionPolicy<K, V>>
	where
		K: Hash + Eq + Copy + Debug,
		V: Eq + Copy + Debug,
	{
		match self {
			EvictionPolicyKind::ClassicLRU => Box::new(ClassicLRU),
			EvictionPolicyKind::TimestampLRU => Box::new(TimestampLRU),
			EvictionPolicyKind::Lifo => Box::new(Lifo),
			EvictionPolicyKind::Lfu => Box::new(Lfu),
			EvictionPolicyKind::Fifo => Box::new(Fifo),
			EvictionPolicyKind::Swap => Box::new(Swap),
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn from_usize_valid_indices() {
		assert_eq!(EvictionPolicyKind::from_usize(0), Some(EvictionPolicyKind::ClassicLRU));
		assert_eq!(EvictionPolicyKind::from_usize(1), Some(EvictionPolicyKind::TimestampLRU));
		assert_eq!(EvictionPolicyKind::from_usize(2), Some(EvictionPolicyKind::Lifo));
		assert_eq!(EvictionPolicyKind::from_usize(3), Some(EvictionPolicyKind::Lfu));
		assert_eq!(EvictionPolicyKind::from_usize(4), Some(EvictionPolicyKind::Fifo));
		assert_eq!(EvictionPolicyKind::from_usize(5), Some(EvictionPolicyKind::Swap));
	}

	#[test]
	fn from_usize_out_of_range() {
		assert!(EvictionPolicyKind::from_usize(6).is_none());
		assert!(EvictionPolicyKind::from_usize(100).is_none());
		assert!(EvictionPolicyKind::from_usize(usize::MAX).is_none());
	}

	#[test]
	fn from_name_valid() {
		assert_eq!(EvictionPolicyKind::from_name("ClassicLRU"), Some(EvictionPolicyKind::ClassicLRU));
		assert_eq!(EvictionPolicyKind::from_name("TimestampLRU"), Some(EvictionPolicyKind::TimestampLRU));
		assert_eq!(EvictionPolicyKind::from_name("Lifo"), Some(EvictionPolicyKind::Lifo));
		assert_eq!(EvictionPolicyKind::from_name("Lfu"), Some(EvictionPolicyKind::Lfu));
		assert_eq!(EvictionPolicyKind::from_name("Fifo"), Some(EvictionPolicyKind::Fifo));
		assert_eq!(EvictionPolicyKind::from_name("Swap"), Some(EvictionPolicyKind::Swap));
	}

	#[test]
	fn from_name_unknown() {
		assert!(EvictionPolicyKind::from_name("classiclru").is_none());
		assert!(EvictionPolicyKind::from_name("NotAPolicy").is_none());
		assert!(EvictionPolicyKind::from_name("").is_none());
	}

	#[test]
	fn name_round_trip_for_every_variant() {
		for p in EvictionPolicyKind::all() {
			let resolved = EvictionPolicyKind::from_name(p.name())
				.unwrap_or_else(|| panic!("from_name({}) returned None", p.name()));
			assert_eq!(resolved, p, "{}.name() resolved to wrong variant", p.name());
		}
	}

	#[test]
	fn index_round_trip_for_every_variant() {
		for p in EvictionPolicyKind::all() {
			let resolved = EvictionPolicyKind::from_usize(p.index())
				.unwrap_or_else(|| panic!("from_usize({}) returned None", p.index()));
			assert_eq!(resolved, p, "index {} resolved to wrong variant", p.index());
		}
	}

	#[test]
	fn indices_are_dense_and_unique() {
		let indices: Vec<usize> = EvictionPolicyKind::all().iter().map(|p| p.index()).collect();
		let mut sorted = indices.clone();
		sorted.sort();
		let expected: Vec<usize> = (0..EvictionPolicyKind::all().len()).collect();
		assert_eq!(sorted, expected, "indices must be 0..N with no gaps or duplicates");
	}

	#[test]
	fn names_are_unique() {
		let names = EvictionPolicyKind::all_names();
		let mut seen = std::collections::HashSet::new();
		for name in names {
			assert!(seen.insert(name), "duplicate name: {name}");
		}
	}

	#[test]
	fn all_names_matches_per_variant_name() {
		let derived: Vec<&'static str> = EvictionPolicyKind::all().iter().map(|p| p.name()).collect();
		assert_eq!(derived, EvictionPolicyKind::all_names().to_vec());
	}

	#[test]
	fn build_name_matches_kind() {
		for kind in EvictionPolicyKind::all() {
			let policy: Box<dyn EvictionPolicy<i64, i64>> = kind.build();
			assert_eq!(policy.name(), kind.name());
		}
	}

	#[test]
	fn classic_lru_victim_prefers_low_count_in_tail() {
		let mut items: Vec<Item<i64, i64>> = (0..10).map(|k| Item::new(k, k)).collect();
		for it in items.iter_mut().take(3) {
			it.lfu_counter = 2;
		}
		items[5].lfu_counter = 1;
		items[3].lfu_counter = 1;
		items[4].lfu_counter = 1;
		items[6].lfu_counter = 1;
		items[7].lfu_counter = 1;
		// slots 8, 9 are outside the window even though their count is 0
		let p = ClassicLRU;
		assert_eq!(EvictionPolicy::<i64, i64>::eviction_candidate_index(&p, &items), 3);
	}

	#[test]
	fn classic_lru_counter_ages_at_cap() {
		// H03: no halving any more — the counter saturates at the cap and no
		// other slot changes.
		let mut items: Vec<Item<i64, i64>> = (0..3).map(|k| Item::new(k, k)).collect();
		items[0].lfu_counter = LFU_COUNTER_CAP;
		items[1].lfu_counter = 6;
		items[2].lfu_counter = 3;
		let p = ClassicLRU;
		let idx = EvictionPolicy::<i64, i64>::get_from_position(&p, &mut items, 0);
		assert_eq!(items[idx].key, 0);
		assert_eq!(items[idx].lfu_counter, LFU_COUNTER_CAP);
		assert_eq!(items[0].key, 1);
		assert_eq!(items[0].lfu_counter, 6);
		assert_eq!(items[1].key, 2);
		assert_eq!(items[1].lfu_counter, 3);
	}

	#[test]
	fn classic_lru_counter_increments_below_cap() {
		let mut items: Vec<Item<i64, i64>> = (0..2).map(|k| Item::new(k, k)).collect();
		items[0].lfu_counter = 4;
		let p = ClassicLRU;
		let idx = EvictionPolicy::<i64, i64>::get_from_position(&p, &mut items, 0);
		assert_eq!(items[idx].lfu_counter, 5);
		assert_eq!(items[0].lfu_counter, 0);
	}

	#[test]
	fn age_tail_window_decrements_first_window_and_saturates() {
		let mut items: Vec<Item<i64, i64>> = (0..12).map(|k| Item::new(k, k)).collect();
		for (i, it) in items.iter_mut().enumerate() {
			it.lfu_counter = i % 3; // 0,1,2,0,1,2,...
		}
		let before: Vec<usize> = items.iter().map(|i| i.lfu_counter).collect();
		age_tail_window(&mut items, LRU_TAIL_WINDOW);
		for i in 0..items.len() {
			let expected = if i < LRU_TAIL_WINDOW { before[i].saturating_sub(1) } else { before[i] };
			assert_eq!(items[i].lfu_counter, expected, "slot {i}");
		}
	}

	#[test]
	fn age_tail_window_handles_short_bucket() {
		let mut items: Vec<Item<i64, i64>> = (0..3).map(|k| Item::new(k, k)).collect();
		items[0].lfu_counter = 0;
		items[1].lfu_counter = 1;
		items[2].lfu_counter = 5;
		age_tail_window(&mut items, LRU_TAIL_WINDOW);
		assert_eq!(items.iter().map(|i| i.lfu_counter).collect::<Vec<_>>(), vec![0, 0, 4]);
		let mut empty: Vec<Item<i64, i64>> = Vec::new();
		age_tail_window(&mut empty, LRU_TAIL_WINDOW);
		assert!(empty.is_empty());
	}
}
