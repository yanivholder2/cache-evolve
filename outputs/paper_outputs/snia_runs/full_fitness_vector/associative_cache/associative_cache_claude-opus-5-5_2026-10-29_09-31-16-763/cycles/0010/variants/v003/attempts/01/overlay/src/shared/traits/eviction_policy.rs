//! Eviction policy: the per-bucket algorithm deciding how an item moves on
//! access and which item is evicted or most protected. Dispatched as a trait
//! object owned by the bucket, so a variant can plug a new policy without
//! touching the bucket or its trait.
//!
//! Per-item policy state (timestamps, counters) is not stored in `Item`. It
//! lives in a side byte buffer `meta` owned by the bucket. `meta` is either
//! empty (the bucket keeps no per-item state for this policy) or holds exactly
//! `items.len() * META_STRIDE` bytes in lockstep with `items`: slot `i`
//! (see `item::meta_get`/`meta_set`) belongs to `items[i]`. A policy that
//! reorders `items` must apply the same reorder to a non-empty `meta`.

use crate::shared::item::Item;
use std::fmt::Debug;
use std::hash::Hash;

/// What a policy keeps in its per-item metadata slot. The bucket uses this to
/// initialise a new item's slot and to decide whether to stamp on hit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetaUse {
	/// No per-item state: the slot is never read.
	None,
	/// Last-access stamp: the bucket stamps it from its clock on insert and hit.
	Stamp,
	/// Access counter: starts at 0, maintained by the policy.
	Counter,
}

pub trait EvictionPolicy<K, V>: Debug
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Canonical CamelCase name of the policy.
	fn name(&self) -> &'static str;

	/// How this policy uses its per-item metadata slot.
	fn meta_use(&self) -> MetaUse {
		MetaUse::None
	}

	/// Applies the access side effect for the item at `position` and returns its
	/// resulting index. May reorder `items` (and `meta` in lockstep) or mutate
	/// per-item state in `meta` (refresh timestamp, bump counter) per the policy.
	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, meta: &mut [u8], position: usize) -> usize;

	/// Index of the item that would be evicted. Precondition: `items` is non-empty.
	fn eviction_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize;

	/// Index of the hottest item, the one most protected from eviction. Symmetric
	/// inverse of `eviction_candidate_index`. Precondition: `items` is non-empty.
	fn promotion_candidate_index(&self, items: &[Item<K, V>], meta: &[u8]) -> usize;
}
