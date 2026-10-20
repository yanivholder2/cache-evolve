use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// A resident item becomes scan-resistant ("protected") only once its hit count
/// reaches this threshold. v002 protected after a single hit (`reuse` bit);
/// H03 raises the bar to two hits so one-hit-wonders dragged in by a scan stay
/// evictable. The counter saturates here — no value above `PROTECT_THRESHOLD`
/// is ever stored, so a single `u8` per slot suffices.
const PROTECT_THRESHOLD: u8 = 2;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-slot saturating hit counter, kept index-aligned with `items`
	/// (`hits.len() == items.len()` at all times). `hits[i]` counts how many
	/// times the resident item in slot `i` has been hit since it was inserted,
	/// saturating at [`PROTECT_THRESHOLD`]. A slot is "protected" (scan-resistant)
	/// iff `hits[i] >= PROTECT_THRESHOLD`.
	///
	/// This generalises v002's binary `reuse: Vec<bool>` to a two-level count so
	/// that a single hit no longer earns protection — the admission bar for
	/// scan-resistance is now two hits. Like `reuse`, this state lives off the
	/// measured `Item`, so `Item` stays at the 16-byte metadata floor. The bucket
	/// struct is not measured by the metadata-size metric (only `Item` and the
	/// top-level `AssociativeCache` are), so a `u8` per slot is free on that
	/// dimension.
	hits: Vec<u8>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			hits: Vec::with_capacity(settings.bucket_size),
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

	/// Insert `item` at the back, maintaining the parallel hit-count set. A freshly
	/// inserted miss item starts unprotected with `hits = 0`; it only earns
	/// protection once its count reaches [`PROTECT_THRESHOLD`]. Updates to an
	/// existing key route through `get_from_position`, which counts as an access
	/// and increments the counter.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		let evicted = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		self.hits.push(0);
		let pushed = self.items.last().expect("bucket invariant: just pushed");
		(pushed, evicted)
	}

	/// Access the item at `position`, applying the policy's reordering side effect
	/// and incrementing the item's saturating hit count. The policy relocates the
	/// accessed item from slot `position` to slot `index` (a single-element move:
	/// `remove(position)` + `push` for ClassicLRU, adjacent swap for Swap, or
	/// identity for the insertion-order policies). We mirror that exact move onto
	/// `hits` with an index-preserving `remove(position)` + `insert(index, count)`
	/// — no key search, no per-item rebuild — so `hits[i]` ↔ `items[i]` is
	/// preserved in O(bucket) shifts. The accessed item's count is then bumped by
	/// one, saturating at [`PROTECT_THRESHOLD`]: two hits are required before a
	/// slot is treated as protected by the eviction path.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let count = self.hits.remove(position);
		self.hits.insert(index, count);
		if self.hits[index] < PROTECT_THRESHOLD {
			self.hits[index] += 1;
		}
		&self.items[index]
	}

	/// Scan-resistant eviction candidate selection with a two-hit protection
	/// threshold: evict the first slot in LRU-front order (index 0 = least-recently
	/// used under ClassicLRU) whose hit count is below [`PROTECT_THRESHOLD`] — a
	/// never-hit item or a one-hit-wonder that a scan dragged in. Because the front
	/// of the bucket holds the coldest items, never-hit / lower-count slots are
	/// naturally preferred within the unprotected set. If every resident item is
	/// protected (count `>= PROTECT_THRESHOLD`), fall back to the LRU front
	/// (index 0), exactly as v002 falls back today. Driven from the bucket because
	/// the `EvictionPolicy` trait cannot see the bucket-side hit-count set.
	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		for (idx, &count) in self.hits.iter().enumerate() {
			if count < PROTECT_THRESHOLD {
				return Some(idx);
			}
		}
		Some(0)
	}

	/// Remove the item at `index`, keeping the hit-count set index-aligned.
	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.hits.remove(index);
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
