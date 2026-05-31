use super::{SegmentCore, Slot};
use crate::shared::item::Item;
use crate::shared::traits::bucket::InsertPosition;
use log::debug;
use std::fmt::Debug;
use std::hash::Hash;

/// A Segment owns one [`SegmentCore`] (the shared normal + stash buckets) and
/// decides *where* items go on `get` (promotion, peer-placement, touch-in-place)
/// while deferring per-bucket ordering to [`EvictionPolicy`].
///
/// # Contract
///
/// **Mutation scope of `get`:** implementations may freely move items between
/// slots (stash → target, target → probing) during a `get`. Callers must not
/// assume the item's slot is unchanged after a hit.
///
/// **`put` precondition:** the key must not already be resident anywhere in the
/// segment. The debug assertion in the default `put` enforces this. A caller that
/// re-inserts a resident key violates the uniqueness invariant (ADR 0004) and
/// will produce a phantom duplicate that can never be evicted through the normal
/// path.
///
///
/// **Stash overflow:** when the stash bucket is full, the weakest stash resident
/// is evicted to make room for the arriving item. Do not retry; stash overflow
/// is the defined end of the line.
pub trait Segment<K, V>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
{
	/// Shared reference to the segment's normal and stash buckets.
	fn core(&self) -> &SegmentCore<K, V>;

	/// Exclusive reference to the segment's normal and stash buckets.
	fn core_mut(&mut self) -> &mut SegmentCore<K, V>;

	/// Look up `key` and return a reference to its [`Item`] if present.
	///
	/// **Postcondition:** returns `Some(&item)` where `item.key == *key` if the
	/// key is resident in any slot (stash, target, or probing); `None` on a miss.
	/// Implementations must search all three slots.
	///
	/// **Mutation:** implementations may reorder items within a bucket (the common
	/// case) or move them between slots (e.g., stash → target on promotion).
	/// The returned reference reflects the item's location after any such move.
	/// Depends on the uniqueness invariant (ADR 0004) holding.
	fn get<'a>(&'a mut self, key: &K) -> Option<&'a Item<K, V>>;

	/// Insert a new `item` into the segment.
	///
	/// **Precondition:** `item.key` must not already be resident anywhere in the
	/// segment. Violated in debug builds via a `debug_assert` that panics; in
	/// release builds, a duplicate insertion silently creates a phantom item
	/// that can never be retrieved or evicted.
	///
	/// **Default behaviour:** routes the item to its stash bucket
	/// (`Slot::Stash`, `InsertPosition::Back`). This is the recommended default
	/// for new Segment implementations. Override only if the Variant requires
	/// fundamentally different initial placement (rare).
	fn put(&mut self, item: Item<K, V>) {
		let core = self.core_mut();
		debug_assert!(
			!core.contains_key(&item.key),
			"put: key={:?} already present — caller must not re-insert a resident key",
			item.key,
		);
		debug!("Segment.put: key={:?}", item.key);
		core.put_at(Slot::Stash, InsertPosition::Back, item);
	}
}
