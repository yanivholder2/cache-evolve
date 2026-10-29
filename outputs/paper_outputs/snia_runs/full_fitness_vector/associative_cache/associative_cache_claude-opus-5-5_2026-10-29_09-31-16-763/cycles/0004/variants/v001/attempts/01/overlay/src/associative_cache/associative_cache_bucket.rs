use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// ClassicLRU insertion depth: a newly admitted item is inserted at
/// `len / INSERT_DEPTH_DIVISOR` (measured from the eviction end, index 0)
/// instead of at the MRU end. A hit still promotes it to the back.
const INSERT_DEPTH_DIVISOR: usize = 4;

/// Bimodal insertion (BIP): every `BIP_MRU_PERIOD`-th new-key put in a bucket
/// is inserted at the MRU end instead of at quarter depth.
const BIP_MRU_PERIOD: u32 = 32;

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
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let lip = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			lip,
			bip_tick: 0,
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

		if self.lip {
			// Fused full-bucket path: evict index 0 and insert in a single
			// rotation, producing exactly the parent's layout
			// (remove(0) followed by insert / push) with one memmove.
			if self.is_full()
				&& !self.items.is_empty()
				&& self.eviction_policy.eviction_candidate_index(&self.items) == 0
			{
				// Same increment point relative to the BIP decision as the parent.
				self.bip_tick = self.bip_tick.wrapping_add(1);
				let len = self.items.len();
				let pos = if self.bip_tick % BIP_MRU_PERIOD == 0 {
					// Bimodal MRU insertion: remove(0) + push == rotate whole
					// bucket left by one, then overwrite the last slot.
					len - 1
				} else {
					// LIP-style quarter-depth insertion: the parent's insert_pos is
					// computed after removal, i.e. (len - 1) / divisor.
					(len - 1) / INSERT_DEPTH_DIVISOR
				};
				self.items[0..=pos].rotate_left(1);
				let old = std::mem::replace(&mut self.items[pos], item);
				return (&self.items[pos], Some(old));
			}

			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.bip_tick = self.bip_tick.wrapping_add(1);
			if self.bip_tick % BIP_MRU_PERIOD == 0 {
				// Bimodal MRU insertion: occasionally admit at the MRU end so a
				// changing working set can enter the protected region.
				self.items.push(item);
				let pushed_item = self.items.last().expect("bucket invariant: just pushed");
				return (pushed_item, evicted_item);
			}
			// LIP-style insertion: place the new item a quarter of the way from
			// the LRU end (index 0) so one-hit scan items are evicted early.
			let insert_pos = self.items.len() / INSERT_DEPTH_DIVISOR;
			self.items.insert(insert_pos, item);
			return (&self.items[insert_pos], evicted_item);
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };

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
