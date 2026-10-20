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

	/// Probationary front admission layered over frequency-first selection.
	///
	/// Existing-key updates keep the trait-default behaviour: overwrite the value
	/// in place and run `get_from_position` so the key is promoted per the
	/// frequency rule below. A *new* key is instead admitted at the **front**
	/// (`insert(0, item)`) rather than pushed to the back. The fresh item carries
	/// `lfu_counter = 0`, so it is simultaneously the recency-oldest and the
	/// minimum-frequency entry — i.e. the immediate next eviction candidate until
	/// it earns a hit. This kills one-hit-wonders and scan traffic at the front
	/// before they can accumulate any frequency, while genuinely reused keys both
	/// climb toward the back (via `get_from_position`) and build a protective
	/// `lfu_counter`. We preserve the evict-exactly-one-when-full invariant (P1)
	/// by calling `evict_item()` before the insert whenever the bucket is full.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.insert(0, item);
		let pushed_item = self.items.first().expect("bucket invariant: just inserted");
		(pushed_item, evicted_item)
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
