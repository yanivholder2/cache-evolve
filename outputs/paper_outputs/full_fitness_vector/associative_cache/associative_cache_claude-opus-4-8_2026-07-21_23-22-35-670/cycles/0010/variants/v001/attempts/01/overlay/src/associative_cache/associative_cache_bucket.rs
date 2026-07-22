use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Close a one-bit gap at position `p`, mirroring `Vec::remove(p)` on the
/// index-aligned reuse set: bits below `p` are kept in place, bit `p` is
/// discarded, and every bit above `p` shifts down by one. Constant-time
/// register arithmetic in place of the `Vec<bool>` element shift.
#[inline]
fn mask_remove(mask: u64, p: usize) -> u64 {
	let low_mask = (1u64 << p) - 1;
	let low = mask & low_mask;
	// Bits strictly above `p`, shifted down by one. When `p == 63` there are no
	// higher bits, so avoid the width-64 shift (which is UB) and take `0`.
	let high = if p + 1 >= 64 { 0 } else { (mask >> (p + 1)) << p };
	low | high
}

/// Open a one-bit gap at position `i` and set that bit, mirroring
/// `Vec::insert(i, true)` on the index-aligned reuse set: bits below `i` are
/// kept in place, every bit at `i` and above shifts up by one, and the new
/// bit `i` is set to `1`.
#[inline]
fn mask_insert_set(mask: u64, i: usize) -> u64 {
	let low_mask = (1u64 << i) - 1;
	let low = mask & low_mask;
	let high = (mask & !low_mask) << 1;
	low | high | (1u64 << i)
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-slot reuse-protection bits, kept index-aligned with `items`
	/// (the low `items.len()` bits are meaningful). `bit i == 1` iff the resident
	/// item in slot `i` has been hit at least once since it was inserted.
	///
	/// This is the scan-resistance state v001 carried as a parallel
	/// `reuse: Vec<bool>`, now packed into a single `u64` register. The `Vec<bool>`
	/// forced a second heap-vector element shift on every hit and a linear scan on
	/// every eviction; the packed mask does the identical index-aligned splice with
	/// constant-time register arithmetic (`mask_remove`/`mask_insert_set`) and finds
	/// the eviction candidate with a single `trailing_zeros`. Requires
	/// `bucket_size <= 64`. The bucket struct is not measured by the metadata-size
	/// metric (only `Item` and the top-level `AssociativeCache` are), so this state
	/// is free on that dimension.
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

	/// Insert `item` at the back, maintaining the packed reuse set. A freshly
	/// inserted miss item is unprotected (its bit is cleared); it only earns
	/// protection once it is hit. Updates to an existing key route through
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
		self.reuse_mask &= !(1u64 << idx);
		let pushed = self.items.last().expect("bucket invariant: just pushed");
		(pushed, evicted)
	}

	/// Access the item at `position`, applying the policy's reordering side effect
	/// and marking the item as reused. The policy relocates the accessed item from
	/// slot `position` to slot `index` (a single-element move). We mirror that exact
	/// move onto the packed reuse set with an index-preserving remove-then-insert
	/// splice (`mask_insert_set(mask_remove(mask, position), index)`) — constant-time
	/// register arithmetic, no key search, no per-item rebuild — so `bit i` ↔
	/// `items[i]` is preserved. The accessed item's bit is set by `mask_insert_set` —
	/// the "hit at least once since insertion" signal the eviction path reads.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		self.reuse_mask = mask_insert_set(mask_remove(self.reuse_mask, position), index);
		&self.items[index]
	}

	/// Scan-resistant eviction candidate selection (v001's mechanism): evict the
	/// first slot in LRU-front order (index 0 = least-recently-used under
	/// ClassicLRU) whose reuse bit is `0` — a one-hit-wonder that was inserted but
	/// never touched again. If every resident item is protected, fall back to the
	/// LRU front (index 0). Driven from the bucket because the `EvictionPolicy`
	/// trait cannot see the bucket-side reuse set.
	fn eviction_candidate_index(&self) -> Option<usize> {
		let len = self.items.len();
		if len == 0 {
			return None;
		}
		let live = u64::MAX >> (64 - len);
		let unreused = !self.reuse_mask & live;
		if unreused != 0 {
			Some(unreused.trailing_zeros() as usize)
		} else {
			Some(0)
		}
	}

	/// Remove the item at `index`, keeping the packed reuse set index-aligned.
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
