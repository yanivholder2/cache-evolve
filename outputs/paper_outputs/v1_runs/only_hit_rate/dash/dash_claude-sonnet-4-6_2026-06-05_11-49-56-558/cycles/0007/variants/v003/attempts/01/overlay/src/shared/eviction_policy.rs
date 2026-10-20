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

/// Every cache hit moves the item to the back of the bucket.
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
		let item = items.remove(position);
		items.push(item);
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

/// ClassicLRU variant that protects items with at least two in-tier accesses
/// from eviction when a less-accessed item exists as an alternative victim.
///
/// Two-stage guard:
///   1. Prefer items with `lfu_counter == 0` (never touched in this tier).
///   2. Next prefer items with `lfu_counter == 1` (single-touch, not yet
///      confirmed hot).
///   3. Fall back to position 0 (LRU front) when all residents have ≥ 2
///      accesses.
#[derive(Debug)]
pub struct ClassicLRUFreqProtect;

impl<K, V> EvictionPolicy<K, V> for ClassicLRUFreqProtect
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str { "ClassicLRUFreqProtect" }

	/// Move to MRU (same as ClassicLRU) AND increment lfu_counter to record
	/// that this item has been accessed at least once since entering its
	/// current tier.
	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize {
		items[position].lfu_counter += 1;
		let item = items.remove(position);
		items.push(item);
		items.len() - 1
	}

	/// Two-stage priority eviction guard:
	///
	/// Stage 1: prefer items that have never been accessed in this tier
	/// (`lfu_counter == 0`).
	/// Stage 2: next prefer items accessed exactly once (`lfu_counter == 1`),
	/// which are single-touch and not yet confirmed to recur.
	/// Fallback: evict the LRU-position item (index 0) when all residents have
	/// two or more accesses.
	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		// Stage 1: prefer items that have never been accessed in this tier
		if let Some((i, _)) = items.iter().enumerate().find(|(_, it)| it.lfu_counter == 0) {
			return i;
		}
		// Stage 2: next prefer items accessed exactly once (single-touch, not yet confirmed hot)
		if let Some((i, _)) = items.iter().enumerate().find(|(_, it)| it.lfu_counter == 1) {
			return i;
		}
		// Fallback: evict the LRU-position item (index 0)
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize {
		items.len() - 1  // same as ClassicLRU: MRU end
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
}
