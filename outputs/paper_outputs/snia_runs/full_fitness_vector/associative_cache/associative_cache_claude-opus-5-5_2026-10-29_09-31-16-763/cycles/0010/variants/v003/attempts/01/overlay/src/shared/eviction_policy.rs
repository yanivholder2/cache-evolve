//! In-crate eviction policy implementations and the selection enum.
//!
//! Behaviour lives in one zero-sized struct per policy, each implementing
//! [`EvictionPolicy`]. `EvictionPolicyKind` is the lightweight `Copy` selector
//! carried by settings (and crossed over JNI as an ordinal/name); `build`
//! turns a kind into the boxed trait object a bucket owns.
//!
//! Per-item state lives in the bucket's side buffer `meta` (see the trait
//! docs), not in `Item`. TimestampLRU keeps a stamp there and Lfu keeps a
//! counter. The other policies never read it, but the ones that reorder items
//! (ClassicLRU, Swap) reorder a non-empty `meta` in lockstep.

use crate::shared::item::{self, meta_get, meta_set, Item, META_STRIDE};
use crate::shared::traits::eviction_policy::{EvictionPolicy, MetaUse};
use std::fmt::Debug;
use std::hash::Hash;

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

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, meta: &mut [u8], position: usize) -> usize {
		let item = items.remove(position);
		items.push(item);
		if !meta.is_empty() {
			// Same remove-then-push on the side buffer.
			meta[position * META_STRIDE..].rotate_left(META_STRIDE);
		}
		items.len() - 1
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>], _meta: &[u8]) -> usize {
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>], _meta: &[u8]) -> usize {
		items.len() - 1
	}
}

/// Signed position of `ts` relative to `reference` on the wrapping u32 clock.
/// Correct as long as every compared stamp lies within 2^31 ticks of the
/// reference, which holds for the stamps of one bucket.
#[inline]
fn rel_age(ts: u32, reference: u32) -> i32 {
	ts.wrapping_sub(reference) as i32
}

impl<K, V> EvictionPolicy<K, V> for TimestampLRU
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn name(&self) -> &'static str {
		"TimestampLRU"
	}

	fn meta_use(&self) -> MetaUse {
		MetaUse::Stamp
	}

	/// Stamps from the shared logical clock so a bucket that does not provide
	/// its own stamps still gets LRU order. `AssociativeCacheBucket` overwrites
	/// this stamp with its per-bucket clock right after this call. TimestampLRU
	/// is not a benchmarked policy, so this atomic stays on its hit path only.
	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, meta: &mut [u8], position: usize) -> usize {
		meta_set(meta, position, item::tick());
		position
	}

	/// Oldest = smallest stamp in wrapping order. Ordering is taken relative
	/// to the slice's own first stamp, not a global "now", so it works both
	/// for per-bucket clocks and for the shared logical clock, and survives a
	/// u32 wrap. Ties resolve to the first item.
	fn eviction_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize {
		assert!(!items.is_empty(), "eviction_candidate_index precondition: non-empty");
		let reference = meta_get(meta, 0);
		(0..items.len())
			.min_by_key(|&i| rel_age(meta_get(meta, i), reference))
			.expect("eviction_candidate_index precondition: non-empty")
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize {
		assert!(!items.is_empty(), "promotion_candidate_index precondition: non-empty");
		let reference = meta_get(meta, 0);
		(0..items.len())
			.max_by_key(|&i| rel_age(meta_get(meta, i), reference))
			.expect("promotion_candidate_index precondition: non-empty")
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

	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, _meta: &mut [u8], position: usize) -> usize {
		position
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>], _meta: &[u8]) -> usize {
		items.len() - 1
	}

	fn promotion_candidate_index(&self, _items: &[Item<K, V>], _meta: &[u8]) -> usize {
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

	fn meta_use(&self) -> MetaUse {
		MetaUse::Counter
	}

	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, meta: &mut [u8], position: usize) -> usize {
		let c = meta_get(meta, position).saturating_add(1);
		meta_set(meta, position, c);
		position
	}

	fn eviction_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize {
		(0..items.len())
			.min_by_key(|&i| meta_get(meta, i))
			.expect("eviction_candidate_index precondition: non-empty")
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize {
		(0..items.len())
			.max_by_key(|&i| meta_get(meta, i))
			.expect("promotion_candidate_index precondition: non-empty")
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

	fn get_from_position(&self, _items: &mut Vec<Item<K, V>>, _meta: &mut [u8], position: usize) -> usize {
		position
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>], _meta: &[u8]) -> usize {
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>], _meta: &[u8]) -> usize {
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

	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, meta: &mut [u8], position: usize) -> usize {
		if position > 0 {
			items.swap(position, position - 1);
			if !meta.is_empty() {
				let a = (position - 1) * META_STRIDE;
				let (lo, hi) = meta[a..a + 2 * META_STRIDE].split_at_mut(META_STRIDE);
				lo.swap_with_slice(hi);
			}
			position - 1
		} else {
			position
		}
	}

	fn eviction_candidate_index(&self, _items: &[Item<K, V>], _meta: &[u8]) -> usize {
		0
	}

	fn promotion_candidate_index(&self, items: &[Item<K, V>], _meta: &[u8]) -> usize {
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
	fn timestamp_lru_orders_across_u32_wrap() {
		let p = TimestampLRU;
		let stamps = [u32::MAX, u32::MAX - 1, 2, 0];
		let items: Vec<Item<i64, i64>> = (1..=4).map(|k| Item::new(k, k)).collect();
		let mut meta = vec![0u8; stamps.len() * META_STRIDE];
		for (i, &ts) in stamps.iter().enumerate() {
			meta_set(&mut meta, i, ts);
		}
		// Stamps straddle the wrap: u32::MAX - 1 is oldest, 2 newest.
		assert_eq!(EvictionPolicy::<i64, i64>::eviction_candidate_index(&p, &items, &meta), 1);
		assert_eq!(EvictionPolicy::<i64, i64>::promotion_candidate_index(&p, &items, &meta), 2);
	}

	#[test]
	fn reordering_policies_keep_meta_in_lockstep() {
		let mk = || -> (Vec<Item<i64, i64>>, Vec<u8>) {
			let items: Vec<Item<i64, i64>> = (0..4).map(|k| Item::new(k, k)).collect();
			let mut meta = vec![0u8; 4 * META_STRIDE];
			for i in 0..4 {
				meta_set(&mut meta, i, 100 + i as u32);
			}
			(items, meta)
		};
		let words = |items: &[Item<i64, i64>], meta: &[u8]| -> Vec<(i64, u32)> {
			items.iter().enumerate().map(|(i, it)| (it.key, meta_get(meta, i))).collect()
		};
		let (mut items, mut meta) = mk();
		EvictionPolicy::<i64, i64>::get_from_position(&ClassicLRU, &mut items, &mut meta, 1);
		assert_eq!(words(&items[..], &meta[..]), vec![(0, 100), (2, 102), (3, 103), (1, 101)]);
		let (mut items, mut meta) = mk();
		EvictionPolicy::<i64, i64>::get_from_position(&Swap, &mut items, &mut meta, 2);
		assert_eq!(words(&items[..], &meta[..]), vec![(0, 100), (2, 102), (1, 101), (3, 103)]);
	}
}
