//! Eviction policy: the per-bucket algorithm deciding how an item moves on
//! access and which item is evicted or most protected. Dispatched as a trait
//! object owned by the bucket, so a variant can plug a new policy without
//! touching the bucket or its trait.
//!
//! H03 (cycle 0005): `Item` carries no per-item metadata. Policies that need
//! state (LFU counters) read it from a per-bucket `meta: &mut [u32]` slice kept
//! parallel to `items` by the bucket, through the `*_meta` methods. The
//! items-only methods remain for buckets without side storage. For a stateful
//! policy they behave as if every counter were equal.

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
	/// resulting index. May reorder `items` per the policy (move to back, swap
	/// toward front).
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

	/// Access hook with per-item side metadata. `meta` is parallel to `items`
	/// (`meta[i]` belongs to `items[i]`), and the policy must keep it parallel.
	///
	/// The default applies the slice hook and assumes the accessed item moved
	/// from `position` to the returned index, with the items in between shifted
	/// by one. That covers move-to-back, adjacent swap and no-op. It applies the
	/// same rotation to `meta`. Stateful or otherwise-reordering policies override it.
	fn get_from_position_meta(&self, items: &mut [Item<K, V>], meta: &mut [u32], position: usize) -> usize {
		let idx = self.get_from_position_in_slice(items, position);
		if idx > position {
			meta[position..=idx].rotate_left(1);
		} else if idx < position {
			meta[idx..=position].rotate_right(1);
		}
		idx
	}

	/// Index of the item that would be evicted. Precondition: `items` is non-empty.
	fn eviction_candidate_index(&self, items: &[Item<K, V>]) -> usize;

	/// Eviction candidate with per-item side metadata parallel to `items`.
	/// The default ignores `meta`.
	fn eviction_candidate_index_meta(&self, items: &[Item<K, V>], _meta: &[u32]) -> usize {
		self.eviction_candidate_index(items)
	}

	/// Index of the hottest item, the one most protected from eviction. Symmetric
	/// inverse of `eviction_candidate_index`. Precondition: `items` is non-empty.
	fn promotion_candidate_index(&self, items: &[Item<K, V>]) -> usize;

	/// Promotion candidate with per-item side metadata parallel to `items`.
	/// The default ignores `meta`.
	fn promotion_candidate_index_meta(&self, items: &[Item<K, V>], _meta: &[u32]) -> usize {
		self.promotion_candidate_index(items)
	}
}
