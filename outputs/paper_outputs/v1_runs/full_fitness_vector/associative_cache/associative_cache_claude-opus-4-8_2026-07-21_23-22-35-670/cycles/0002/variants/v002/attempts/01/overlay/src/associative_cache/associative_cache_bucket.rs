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

	/// CLOCK second-chance sweep for the ClassicLRU policy.
	///
	/// Walks the bucket front-to-back (LRU order). The first item whose
	/// reference bit (`lfu_counter`) is already `0` is the victim. Every item
	/// with the bit set to `1` that is passed over on the way is given its
	/// second chance by clearing its bit to `0` — a side effect that lets a
	/// long-resident-but-cold entry become evictable on a later sweep instead
	/// of enjoying permanent immunity. If a full sweep finds no already-clear
	/// bit (every item was referenced), all bits are now cleared and we wrap to
	/// slot `0`. Precondition: `self.items` is non-empty.
	fn clock_victim_index(&mut self) -> usize {
		let items = &mut self.items;
		for i in 0..items.len() {
			if items[i].lfu_counter == 0 {
				return i;
			}
			// Referenced item: consume its second chance and keep sweeping.
			items[i].lfu_counter = 0;
		}
		// Full sweep cleared every bit; slot 0 is now the victim.
		0
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
		&self.items[index]
	}

	/// Evicts exactly one resident item per the bucket's policy.
	///
	/// For `ClassicLRU` this runs the mutating CLOCK second-chance sweep, which
	/// the immutable `EvictionPolicy::eviction_candidate_index` cannot express
	/// (it only borrows `&[Item]`). For every other policy this preserves the
	/// default behaviour exactly: pick `eviction_candidate_index` and remove it.
	/// Never touches an empty bucket.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		let idx = if self.eviction_policy.name() == "ClassicLRU" {
			self.clock_victim_index()
		} else {
			self.eviction_policy.eviction_candidate_index(&self.items)
		};
		Some(self.items.remove(idx))
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
