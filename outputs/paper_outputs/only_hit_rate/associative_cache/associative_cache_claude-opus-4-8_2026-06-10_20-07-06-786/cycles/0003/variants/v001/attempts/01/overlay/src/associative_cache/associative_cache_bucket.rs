use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

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

	/// Frequency-first access side effect, recency-aware but reorder-free.
	///
	/// On every hit we (a) bump the item's `lfu_counter`, turning it into a live
	/// per-item hit count, and (b) refresh its `timestamp` to record this access
	/// as its most-recent use. Unlike the parent variant we do **not** move the
	/// item to the back of `items`: it keeps its slot, so there is no O(bucket)
	/// shuffle per hit. The item never leaves this bucket (P2) and neither the
	/// counter nor the timestamp changes the resident set, so P1/P3 are
	/// untouched. Returns a reference to the item at its unchanged index.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.items[position].lfu_counter += 1;
		self.items[position].timestamp = Instant::now();
		&self.items[position]
	}

	/// Eviction victim selection: recency-aware least-frequently-used.
	///
	/// Returns the index of the item minimising the composite key
	/// `(lfu_counter, timestamp)`: fewest hits first, and among equal-frequency
	/// items the one with the **oldest last-access timestamp** (true recency,
	/// not vector position). `Instant` is totally ordered, so the smallest
	/// timestamp is the least-recently-accessed item. Returns `None` on an empty
	/// bucket.
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| (item.lfu_counter, item.timestamp))
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
