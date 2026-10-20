//! Eviction policy: the per-bucket algorithm deciding how an item moves on
//! access and which item is evicted or most protected. Dispatched as a trait
//! object owned by the bucket, so a variant can plug a new policy without
//! touching the bucket or its trait.

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

	/// Applies the access side effect for the item at `position` and returns its
	/// resulting index. May reorder `items` or mutate item state (move to back,
	/// swap toward front, refresh timestamp, bump counter) per the policy.
	fn get_from_position(&self, items: &mut Vec<Item<K, V>>, position: usize) -> usize;

	/// Same as [`get_from_position`](Self::get_from_position) but on a slice, so a
	/// bucket can apply the policy to one segment of a bucket without
	/// allocating. Length is preserved. The default falls back to the `Vec`
	/// version through a temporary copy; in-crate policies override it.
	fn get_from_position_in_slice(&self, items: &mut [Item<K, V>], position: usize) -> usize {
		let mut tmp = items.to_vec();
		let idx = self.get_from_position(&mut tmp, position);
		debug_assert_eq!(tmp.len(), items.len(), "get_from_position must preserve length");
		items.clone_from_slice(&tmp);
		idx
	}

	/// Index of the item that would be evicted. Precondition: `items` is non-empty.
	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize;

	/// Index of the hottest item, the one most protected from eviction. Symmetric
	/// inverse of `eviction_candidate_index`. Precondition: `items` is non-empty.
	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize;
}
