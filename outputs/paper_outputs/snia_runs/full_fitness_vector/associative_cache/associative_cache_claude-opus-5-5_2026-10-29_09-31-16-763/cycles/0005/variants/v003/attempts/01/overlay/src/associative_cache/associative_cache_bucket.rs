use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::cmp::{max, min};
use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// ClassicLRU insertion depth: a newly admitted item is inserted at
/// `len / INSERT_DEPTH_DIVISOR` (measured from the eviction end, index 0)
/// instead of at the MRU end. A hit still promotes it to the back.
/// Used during warm-up and as the initial adaptive depth.
const INSERT_DEPTH_DIVISOR: usize = 4;

/// Bimodal insertion (BIP): every `BIP_MRU_PERIOD`-th new-key put in a bucket
/// is inserted at the MRU end instead of at the adaptive depth.
const BIP_MRU_PERIOD: u32 = 32;

/// Adaptive depth: the minimum depth is `max_size / DEPTH_MIN_DIVISOR` (>= 1).
const DEPTH_MIN_DIVISOR: usize = 16;
/// Adaptive depth: the maximum depth is `max_size / DEPTH_MAX_DIVISOR`.
const DEPTH_MAX_DIVISOR: usize = 2;
/// Every `DECAY_PERIOD`-th eviction lowers the depth by one.
const DECAY_PERIOD: u32 = 8;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// True when the policy is ClassicLRU (decided once at construction).
	lip: bool,
	/// Per-bucket miss counter driving the bimodal MRU insertion.
	bip_tick: u32,
	/// Adaptive LIP insertion depth (from the LRU end), in [depth_min, depth_max].
	depth: u32,
	/// Eviction counter driving the depth decay.
	miss_ctr: u32,
	depth_min: u32,
	depth_max: u32,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let lip = eviction_policy.name() == "ClassicLRU";
		let max_size = settings.bucket_size;
		let depth_min = max(max_size / DEPTH_MIN_DIVISOR, 1);
		let depth_max = max(max_size / DEPTH_MAX_DIVISOR, depth_min);
		let depth = (max_size / INSERT_DEPTH_DIVISOR).clamp(depth_min, depth_max);
		Self {
			items: Vec::with_capacity(max_size),
			max_size,
			eviction_policy,
			lip,
			bip_tick: 0,
			depth: depth as u32,
			miss_ctr: 0,
			depth_min: depth_min as u32,
			depth_max: depth_max as u32,
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

	/// Under ClassicLRU the items are ordered LRU (index 0) -> MRU (len-1), so
	/// hot keys sit at the back: scan from the MRU end. Keys are unique within a
	/// bucket, so the scan direction does not change the returned index.
	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		if self.lip {
			self.items.iter().rposition(|it| it.key == *key)
		} else {
			self.items.iter().position(|it| it.key == *key)
		}
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		// Hit signal: a hit below the insertion point means a recently admitted
		// (probationary) item was reused before promotion -> admit deeper.
		if self.lip && position < self.depth as usize {
			self.depth = min(self.depth + 1, self.depth_max);
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		// Existing key: identical to the trait default.
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		let was_full = self.is_full();
		let evicted_item = if was_full { self.evict_item() } else { None };

		if self.lip {
			if was_full {
				// Decay signal: evictions without probationary hits drift depth down.
				self.miss_ctr = self.miss_ctr.wrapping_add(1);
				if self.miss_ctr % DECAY_PERIOD == 0 {
					self.depth = max(self.depth.saturating_sub(1), self.depth_min);
				}
			}
			self.bip_tick = self.bip_tick.wrapping_add(1);
			if self.bip_tick % BIP_MRU_PERIOD == 0 {
				// Bimodal MRU insertion: occasionally admit at the MRU end so a
				// changing working set can enter the protected region.
				self.items.push(item);
				let pushed_item = self.items.last().expect("bucket invariant: just pushed");
				return (pushed_item, evicted_item);
			}
			// LIP-style insertion at the adaptive depth once the bucket is full;
			// during warm-up keep the parent's quarter-of-len position.
			let insert_pos = if was_full {
				min(self.depth as usize, self.items.len())
			} else {
				self.items.len() / INSERT_DEPTH_DIVISOR
			};
			self.items.insert(insert_pos, item);
			return (&self.items[insert_pos], evicted_item);
		}

		// All other policies: default behaviour (append at the back).
		self.items.push(item);
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
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
