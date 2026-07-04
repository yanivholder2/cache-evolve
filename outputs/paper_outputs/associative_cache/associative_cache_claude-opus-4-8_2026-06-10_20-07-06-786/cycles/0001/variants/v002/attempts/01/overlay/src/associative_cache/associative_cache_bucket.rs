use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
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

	/// Recency+frequency-aware access side effect.
	///
	/// On every hit we (a) bump the item's `lfu_counter`, turning it into a live
	/// per-item hit count, then (b) perform the same MRU promotion the baseline
	/// ClassicLRU path does (move to the back of `items`). The item never leaves
	/// this bucket (P2) and the count never changes the resident set, so P1/P3
	/// are untouched. Returns a reference to the item at its new index.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.items[position].lfu_counter += 1;
		let item = self.items.remove(position);
		self.items.push(item);
		self.items.last().expect("bucket invariant: just pushed")
	}

	/// Eviction victim selection by minimum hit-frequency.
	///
	/// Returns the index of the item with the smallest `lfu_counter`. Because
	/// `min_by_key` keeps the first element on ties, equal-count (e.g. never
	/// re-hit) items break toward the front — the least-recently-inserted, which
	/// preserves the baseline recency behaviour as the tie-break. Returns `None`
	/// on an empty bucket.
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(idx, _)| idx)
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
