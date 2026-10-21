//! Eviction policy: the per-bucket algorithm deciding how an item moves on
//! access and which item is evicted or most protected. Dispatched as a trait
//! object owned by the bucket, so a variant can plug a new policy without
//! touching the bucket or its trait.
//!
//! Per-item policy state (frequency counter, logical stamp) is NOT stored in
//! `Item`; the bucket keeps a parallel `meta: Vec<u32>` in lockstep with its
//! items (`meta[i]` belongs to `items[i]`) and hands it to every policy call.
//! Policies that need no state ignore it, but any reordering of `items` must be
//! mirrored on `meta`.

use crate::shared::item::Item;
use std::fmt::Debug;
use std::hash::Hash;

pub trait EvictionPolicy<K, V>: Debug
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Canonical CamelCase name of the policy.
	fn name(&self) -> &'static str;

	/// Metadata word assigned to a freshly inserted item (default 0).
	fn initial_meta(&self) -> u32 {
		0
	}

	/// Applies the access side effect for the item at `position` and returns its
	/// resulting index. May reorder `items` (mirroring the reorder on `meta`) or
	/// mutate per-item metadata per the policy.
	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, meta: &mut [u32], position: usize) -> usize;

	/// Index of the item that would be evicted. Precondition: `items` is non-empty.
	fn eviction_candidate_index(&self, items: &[Item<K, V>], meta: &[u32]) -> usize;

	/// Index of the hottest item, the one most protected from eviction. Symmetric
	/// inverse of `eviction_candidate_index`. Precondition: `items` is non-empty.
	fn promotion_candidate_index(&self, items: &[Item<K, V>], meta: &[u32]) -> usize;
}
