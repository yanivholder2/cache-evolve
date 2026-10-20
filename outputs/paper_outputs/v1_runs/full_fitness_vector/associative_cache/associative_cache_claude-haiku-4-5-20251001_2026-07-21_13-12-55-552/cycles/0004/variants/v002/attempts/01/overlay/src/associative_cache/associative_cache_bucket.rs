use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::trace;
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::time::Instant;

/// Segment-based bucket organization.
/// Divides each bucket into N_SEGMENTS segments to bound scan depth during eviction.
/// Each segment maintains its own LRU order and oldest-access timestamp.
const N_SEGMENTS: usize = 8;

/// Per-segment metadata for LRU tracking.
#[derive(Debug, Clone, Copy)]
struct SegmentMeta {
	/// The oldest (minimum) timestamp of any item in this segment.
	/// Used to quickly identify which segment contains the oldest-accessed item.
	oldest_timestamp: Instant,
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	segment_metas: [SegmentMeta; N_SEGMENTS],
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let now = Instant::now();
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			segment_metas: [SegmentMeta {
				oldest_timestamp: now,
			}; N_SEGMENTS],
		}
	}

	/// Compute which segment an item at index belongs to.
	fn segment_of(&self, index: usize) -> usize {
		if self.items.is_empty() {
			return 0;
		}
		// Distribute items across segments as they're added sequentially.
		// Items 0..4 -> segment 0, items 4..8 -> segment 1, etc.
		let segment_size = (self.max_size + N_SEGMENTS - 1) / N_SEGMENTS;
		(index / segment_size).min(N_SEGMENTS - 1)
	}

	/// Update segment metadata after a change to items.
	/// Sets each segment's oldest_timestamp to the minimum timestamp in that segment,
	/// or to a far-future time if the segment is empty.
	fn update_segment_metas(&mut self) {
		let segment_size = (self.max_size + N_SEGMENTS - 1) / N_SEGMENTS;

		for segment_idx in 0..N_SEGMENTS {
			let start_idx = segment_idx * segment_size;
			let end_idx = ((segment_idx + 1) * segment_size).min(self.items.len());

			if start_idx >= self.items.len() {
				// Segment is empty; set to far future so it won't be selected for eviction
				self.segment_metas[segment_idx].oldest_timestamp = Instant::now() + std::time::Duration::from_secs(1000);
			} else {
				// Find the minimum timestamp in this segment
				let segment_items = &self.items[start_idx..end_idx];
				let oldest = segment_items
					.iter()
					.min_by_key(|item| item.timestamp)
					.expect("segment is non-empty");
				self.segment_metas[segment_idx].oldest_timestamp = oldest.timestamp;
			}
		}
	}

	/// Find the segment containing the oldest-accessed item.
	fn find_oldest_segment(&self) -> usize {
		let mut oldest_segment = 0;
		let mut oldest_time = self.segment_metas[0].oldest_timestamp;

		for (idx, meta) in self.segment_metas.iter().enumerate() {
			if meta.oldest_timestamp < oldest_time {
				oldest_time = meta.oldest_timestamp;
				oldest_segment = idx;
			}
		}

		oldest_segment
	}

	/// Find the LRU item within a specific segment and return its global index.
	fn find_lru_in_segment(&self, segment_idx: usize) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}

		let segment_size = (self.max_size + N_SEGMENTS - 1) / N_SEGMENTS;
		let start_idx = segment_idx * segment_size;
		let end_idx = ((segment_idx + 1) * segment_size).min(self.items.len());

		if start_idx >= self.items.len() {
			return None;
		}

		let segment_items = &self.items[start_idx..end_idx];
		let (local_idx, _) = segment_items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.timestamp)
			.map(|(idx, item)| (idx, item))?;

		Some(start_idx + local_idx)
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// Update segment metadata after the access
		self.update_segment_metas();
		&self.items[index]
	}

	/// Override put to update segment metadata after insertion.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.get_items_mut()[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.get_items_mut().push(item);
		let pushed_item = self.get_items().last().expect("bucket invariant: just pushed");
		// Update segment metadata after adding a new item
		self.update_segment_metas();
		(pushed_item, evicted_item)
	}

	/// Override evict_item to use segment-based selection.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		// Find the segment with the oldest-accessed item
		let oldest_segment = self.find_oldest_segment();

		// Find the LRU item within that segment
		if let Some(idx) = self.find_lru_in_segment(oldest_segment) {
			let evicted = self.items.remove(idx);
			// Update segment metadata after removal
			self.update_segment_metas();
			log::debug!(
				"Bucket.evict_item: policy={} segment={} evicted_key={:?}",
				self.get_eviction_policy().name(),
				oldest_segment,
				evicted.key
			);
			Some(evicted)
		} else {
			None
		}
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
