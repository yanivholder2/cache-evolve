use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Remove bit `p` from `mask`, closing the gap so every bit above `p` shifts
/// down one — the bitmask twin of `Vec::remove(p)`. Safe for `p <= 63` (the two
/// `>>` never reach a shift-by-64: `mask >> p` then `>> 1`).
#[inline]
fn mask_remove(mask: u64, p: usize) -> u64 {
	let low = mask & ((1u64 << p) - 1);
	let high = (mask >> p) >> 1; // bits strictly above `p`, dropped down to start at `p`
	low | (high << p)
}

/// Open a gap at `i` (every bit at/above `i` shifts up one) and set bit `i` to
/// 1 — the bitmask twin of `Vec::insert(i, true)`. Callers hold `mask`'s top
/// live bit at `< 63`, so `high << 1` never overflows off bit 63.
#[inline]
fn mask_insert_set(mask: u64, i: usize) -> u64 {
	let low = mask & ((1u64 << i) - 1);
	let high = mask & !((1u64 << i) - 1); // bits at/above `i`, in place
	let opened = low | (high << 1); // gap opened at `i`
	opened | (1u64 << i) // accessed slot is now "reused"
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-slot reuse-protection bits, packed into a scalar and kept
	/// index-aligned with `items`: bit `i` is set iff the resident item in slot
	/// `i` has been hit at least once since it was inserted. Bits at positions
	/// `>= items.len()` are held at 0.
	///
	/// This is the same scan-resistance state v002 carried in a parallel
	/// `reuse: Vec<bool>`, folded into a single `u64` so per-hit maintenance and
	/// eviction-victim selection become register bit-ops instead of heap-vector
	/// shifts/scans. Requires `bucket_size <= 64` (small associativity). The
	/// bucket struct is not measured by the metadata-size metric (only `Item` and
	/// the top-level `AssociativeCache` are), so this state is free on that
	/// dimension — as it was in v002.
	reuse_mask: u64,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		debug_assert!(
			settings.bucket_size <= 64,
			"reuse_mask packs one bit per slot into a u64; bucket_size {} exceeds 64",
			settings.bucket_size
		);
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			reuse_mask: 0,
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
		}
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Insert `item` at the back, maintaining the packed reuse mask. A freshly
	/// inserted miss item is unprotected (bit clear); it only earns protection
	/// once it is hit. Updates to an existing key route through
	/// `get_from_position`, which counts as an access and sets the bit.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		let evicted = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		let idx = self.items.len() - 1;
		// New top slot is unprotected. `mask_remove`/`mask_insert_set` already keep
		// bits `>= len` at 0, so this is a belt-and-suspenders clear of bit `idx`.
		self.reuse_mask &= !(1u64 << idx);
		let pushed = self.items.last().expect("bucket invariant: just pushed");
		(pushed, evicted)
	}

	/// Access the item at `position`, applying the policy's reordering side effect
	/// and marking the item as reused. The policy relocates the accessed item from
	/// slot `position` to slot `index` (a single-element move). We mirror that
	/// exact move onto `reuse_mask` — remove the bit at `position` (closing the
	/// gap), then insert a set bit at `index` (opening a gap) — in a few masked
	/// shifts, no per-slot loop and no second heap-vector shift. This is the
	/// bit-for-bit equal of v002's `reuse.remove(position)` + `insert(index, bit)`
	/// + `reuse[index] = true`, whose net effect is exactly "remove at position,
	/// insert `true` at index".
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let closed = mask_remove(self.reuse_mask, position);
		self.reuse_mask = mask_insert_set(closed, index);
		&self.items[index]
	}

	/// Scan-resistant eviction candidate selection (v001's mechanism), now O(1):
	/// evict the first slot in LRU-front order whose reuse bit is clear — a
	/// one-hit-wonder that was inserted but never touched again — found via
	/// `trailing_zeros` on the complement of the live reuse bits. If every
	/// resident item is protected, fall back to the LRU front (index 0). Driven
	/// from the bucket because the `EvictionPolicy` trait cannot see the
	/// bucket-side reuse set.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		// `len` in `1..=64`, so `64 - len` in `0..=63`: never a shift-by-64.
		let live = u64::MAX >> (64 - len);
		let unreused = !self.reuse_mask & live;
		if unreused != 0 {
			Some(unreused.trailing_zeros() as usize)
		} else {
			Some(0)
		}
	}

	/// Remove the item at `index`, keeping the packed reuse mask index-aligned by
	/// closing the mask gap at `index`.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.reuse_mask = mask_remove(self.reuse_mask, index);
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
