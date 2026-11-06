//! In-crate eviction policy implementations and the selection enum.
//!
//! Behaviour lives in one zero-sized struct per policy, each implementing
//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector
//! carried by settings (and crossed over JNI as an ordinal/name); `build`
//! turns a kind into the boxed trait object a bucket owns.

use crate::shared::item::Item;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::fmt::Debug;
use std::hash::Hash;
use std::time::Instant;

/// Saturation ceiling of ClassicLRU's 4-bit per-item hit counter.
const CLASSIC_LRU_COUNTER_MAX: usize = 15;
/// Number of least-recently-used positions ClassicLRU considers for eviction.
const CLASSIC_LRU_EVICTION_WINDOW: usize = 8;

/// Frequency-aware LRU: every cache hit bumps a saturating 4-bit counter and
/// moves the item to the back of the bucket. Eviction picks the lowest-count
/// item among the `CLASSIC_LRU_EVICTION_WINDOW` least-recent positions (ties
/// resolve to the older item). When a counter saturates, every counter in the
/// bucket is halved (ageing).
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
		let bumped = (items[position].lfu_counter + 1).min(CLASSIC_LRU_COUNTER_MAX);
		items[position].lfu_counter = bumped;
		if bumped == CLASSIC_LRU_COUNTER_MAX {
			for it in items.iter_mut() {
				it.lfu_counter /= 2;
			}
		}
		let item = items.remove(position);
		items.push(item);
		items.len() - 1
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		let window = items.len().min(CLASSIC_LRU_EVICTION_WINDOW);
		items[..window]
			.iter()
			.enumerate()
			.min_by_key(|(i, it)| (it.lfu_counter, *i))
			.map(|(i, _)| i)
			.unwrap_or(0)
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
	fn classic_lru_evicts_lowest_count_in_lru_window() {
		let policy = ClassicLRU;
		let mut items: Vec<Item<i64, i64>> = (0..4).map(|k| Item::new(k, k)).collect();
		// Hit key 0 (the LRU) once: it moves to the back with count 1.
		EvictionPolicy::<i64, i64>::get_from_position(&policy, &mut items, 0);
		assert_eq!(items.last().unwrap().key, 0);
		assert_eq!(items.last().unwrap().lfu_counter, 1);
		// Oldest zero-count item (key 1, now at index 0) is the victim.
		assert_eq!(EvictionPolicy::<i64, i64>::eviction_candidate_index(&policy, &items), 0);
		// Hit keys 1 and 2; key 3 is the only zero-count item left.
		let p1 = items.iter().position(|it| it.key == 1).unwrap();
		EvictionPolicy::<i64, i64>::get_from_position(&policy, &mut items, p1);
		let p2 = items.iter().position(|it| it.key == 2).unwrap();
		EvictionPolicy::<i64, i64>::get_from_position(&policy, &mut items, p2);
		let victim = EvictionPolicy::<i64, i64>::eviction_candidate_index(&policy, &items);
		assert_eq!(items[victim].key, 3);
	}

	#[test]
	fn classic_lru_counter_saturates_and_ages() {
		let policy = ClassicLRU;
		let mut items: Vec<Item<i64, i64>> = vec![Item::new(1, 1), Item::new(2, 2)];
		items[0].lfu_counter = 6;
		items[1].lfu_counter = 14;
		let idx = EvictionPolicy::<i64, i64>::get_from_position(&policy, &mut items, 1);
		assert_eq!(items[idx].key, 2);
		assert_eq!(items[idx].lfu_counter, 7);
		assert_eq!(items[0].lfu_counter, 3);
	}
}
