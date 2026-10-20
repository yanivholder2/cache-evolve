//! In-crate eviction policy implementations and the selection enum.
//!
//! Behaviour lives in one zero-sized struct per policy, each implementing
//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector
//! carried by settings (and crossed over JNI as an ordinal/name); `build`
//! turns a kind into the boxed trait object a bucket owns.
//!
//! Every policy implements the slice-based access hook so a bucket can apply
//! it to a segment in place.
//!
//! H03 (cycle 0005): `Item` no longer carries a timestamp or an LFU counter.
//! - `TimestampLRU` is purely positional: a hit moves the item to the back and
//!   index 0 is evicted. Items are pushed at the back on insert, so this gives
//!   the same eviction order as "oldest last-touch tick, first on ties".
//! - `Lfu` keeps its counters in the bucket's `meta` side array, parallel to
//!   `items`, through the `*_meta` trait methods. Through the items-only
//!   methods every counter reads as equal (all 0).

use crate::shared::item::Item;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use std::fmt::Debug;
use std::hash::Hash;

/// Every cache hit moves the item to the back of the bucket.
#[derive(Debug)]
pub struct ClassicLRU;
/// LRU by recency of last touch. Positional: hit moves to back, evict front.
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
		self.get_from_position_in_slice(items.as_mut_slice(), position)
	}

	fn get_from_position_in_slice(&self, items: &mut [Item<K, V>], position: usize) -> usize {
		items[position..].rotate_left(1);
		items.len() - 1
	}

	fn get_from_position_meta(&self, items: &mut [Item<K, V>], meta: &mut [u32], position: usize) -> usize {
		items[position..].rotate_left(1);
		meta[position..].rotate_left(1);
		items.len() - 1
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>]) -> usize {
		0
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
		self.get_from_position_in_slice(items.as_mut_slice(), position)
	}

	/// Refresh recency: move to the back (newest).
	fn get_from_position_in_slice(&self, items: &mut [Item<K, V>], position: usize) -> usize {
		items[position..].rotate_left(1);
		items.len() - 1
	}

	fn get_from_position_meta(&self, items: &mut [Item<K, V>], meta: &mut [u32], position: usize) -> usize {
		items[position..].rotate_left(1);
		meta[position..].rotate_left(1);
		items.len() - 1
	}

	/// Oldest touch is at the front.
	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		assert!(!items.is_empty(), "eviction_candidate_index precondition: non-empty");
		0
	}

	/// Newest touch is at the back.
	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		assert!(!items.is_empty(), "promotion_candidate_index precondition: non-empty");
		items.len() - 1
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

	fn get_from_position_in_slice(&self, _items: &mut [Item<K, V>], position: usize) -> usize {
		position
	}

	fn get_from_position_meta(&self, _items: &mut [Item<K, V>], _meta: &mut [u32], position: usize) -> usize {
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

	/// Without side storage there is nowhere to count: no-op.
	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		position
	}

	fn get_from_position_in_slice(&self, _items: &mut [Item<K, V>], position: usize) -> usize {
		position
	}

	/// Bump the side-array counter (saturating). Items keep their position.
	fn get_from_position_meta(&self, _items: &mut [Item<K, V>], meta: &mut [u32], position: usize) -> usize {
		meta[position] = meta[position].saturating_add(1);
		position
	}

	/// All counters equal: first one (matches `min_by_key` tie-breaking).
	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		assert!(!items.is_empty(), "eviction_candidate_index precondition: non-empty");
		0
	}

	/// Lowest counter; first one on ties (matches `min_by_key`).
	fn eviction_candidate_index_meta(&self, items: &[Item<K, V>], meta: &[u32]) -> usize {
		assert!(!items.is_empty(), "eviction_candidate_index precondition: non-empty");
		debug_assert_eq!(items.len(), meta.len(), "meta must be parallel to items");
		let mut best = 0;
		for i in 1..meta.len() {
			if meta[i] < meta[best] {
				best = i;
			}
		}
		best
	}

	/// All counters equal: last one (matches `max_by_key` tie-breaking).
	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		assert!(!items.is_empty(), "promotion_candidate_index precondition: non-empty");
		items.len() - 1
	}

	/// Highest counter; last one on ties (matches `max_by_key`).
	fn promotion_candidate_index_meta(&self, items: &[Item<K, V>], meta: &[u32]) -> usize {
		assert!(!items.is_empty(), "promotion_candidate_index precondition: non-empty");
		debug_assert_eq!(items.len(), meta.len(), "meta must be parallel to items");
		let mut best = 0;
		for i in 1..meta.len() {
			if meta[i] >= meta[best] {
				best = i;
			}
		}
		best
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

	fn get_from_position_in_slice(&self, _items: &mut [Item<K, V>], position: usize) -> usize {
		position
	}

	fn get_from_position_meta(&self, _items: &mut [Item<K, V>], _meta: &mut [u32], position: usize) -> usize {
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
		self.get_from_position_in_slice(items.as_mut_slice(), position)
	}

	fn get_from_position_in_slice(&self, items: &mut [Item<K, V>], position: usize) -> usize {
		if position > 0 {
			items.swap(position, position - 1);
			position - 1
		} else {
			position
		}
	}

	fn get_from_position_meta(&self, items: &mut [Item<K, V>], meta: &mut [u32], position: usize) -> usize {
		if position > 0 {
			items.swap(position, position - 1);
			meta.swap(position, position - 1);
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
	fn classic_lru_slice_touch_moves_to_back() {
		let mut items = vec![Item::new(1i64, 10i64), Item::new(2, 20), Item::new(3, 30)];
		let idx = EvictionPolicy::<i64, i64>::get_from_position_in_slice(&ClassicLRU, &mut items[..], 0);
		assert_eq!(idx, 2);
		assert_eq!(items.iter().map(|i| i.key).collect::<Vec<_>>(), vec![2, 3, 1]);
	}

	#[test]
	fn timestamp_lru_touch_moves_to_back_and_evicts_front() {
		let mut items = vec![Item::new(1i64, 10i64), Item::new(2, 20), Item::new(3, 30)];
		let mut meta = vec![0u32; 3];
		let idx = EvictionPolicy::<i64, i64>::get_from_position_meta(&TimestampLRU, &mut items[..], &mut meta, 0);
		assert_eq!(idx, 2);
		assert_eq!(items.iter().map(|i| i.key).collect::<Vec<_>>(), vec![2, 3, 1]);
		assert_eq!(EvictionPolicy::<i64, i64>::eviction_candidate_index(&TimestampLRU, &items), 0);
		assert_eq!(EvictionPolicy::<i64, i64>::promotion_candidate_index(&TimestampLRU, &items), 2);
	}

	#[test]
	fn lfu_meta_counter_saturates() {
		let mut items = vec![Item::new(1i64, 10i64)];
		let mut meta = vec![u32::MAX];
		EvictionPolicy::<i64, i64>::get_from_position_meta(&Lfu, &mut items[..], &mut meta, 0);
		assert_eq!(meta[0], u32::MAX);
	}

	#[test]
	fn lfu_meta_eviction_and_promotion_tie_breaking() {
		let items = vec![Item::new(1i64, 10i64), Item::new(2, 20), Item::new(3, 30), Item::new(4, 40)];
		let meta = vec![2u32, 1, 1, 2];
		assert_eq!(EvictionPolicy::<i64, i64>::eviction_candidate_index_meta(&Lfu, &items, &meta), 1);
		assert_eq!(EvictionPolicy::<i64, i64>::promotion_candidate_index_meta(&Lfu, &items, &meta), 3);
	}

	#[test]
	fn swap_meta_stays_parallel() {
		let mut items = vec![Item::new(1i64, 10i64), Item::new(2, 20), Item::new(3, 30)];
		let mut meta = vec![10u32, 20, 30];
		let idx = EvictionPolicy::<i64, i64>::get_from_position_meta(&Swap, &mut items[..], &mut meta, 2);
		assert_eq!(idx, 1);
		assert_eq!(items.iter().map(|i| i.key).collect::<Vec<_>>(), vec![1, 3, 2]);
		assert_eq!(meta, vec![10, 30, 20]);
	}
}
