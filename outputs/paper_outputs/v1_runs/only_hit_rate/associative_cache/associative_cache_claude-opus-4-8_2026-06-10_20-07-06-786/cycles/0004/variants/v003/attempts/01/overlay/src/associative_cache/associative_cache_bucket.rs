use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use log::debug;
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

	/// CLOCK (second-chance) hit path: set the accessed item's reference bit
	/// (`lfu_counter = 1`) and return it **in place**, without reordering the
	/// vector. This is the deliberate contrast with the SLRU/ClassicLRU lineage,
	/// which removes-and-re-pushes (or otherwise promotes) on hit.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		self.items[position].lfu_counter = 1;
		&self.items[position]
	}

	/// CLOCK hand: scan items from the front (index 0) upward. An item with its
	/// reference bit set (`lfu_counter == 1`) gets a reprieve — clear the bit and
	/// continue. The first item with a clear bit (`lfu_counter == 0`) is removed
	/// and returned as the eviction victim.
	///
	/// If a full pass finds no clear bit (all were 1), every bit has now been
	/// cleared, so the front item (index 0) is the victim. This guarantees
	/// termination in at most two logical passes and that exactly one item is
	/// removed.
	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}

		let len = self.items.len();
		let mut victim = None;
		for i in 0..len {
			if self.items[i].lfu_counter == 0 {
				victim = Some(i);
				break;
			}
			// Consume the second chance.
			self.items[i].lfu_counter = 0;
		}

		// All reference bits were set: they are now cleared, so evict the front.
		let index = victim.unwrap_or(0);
		let evicted = self.items.remove(index);
		debug!(
			"Bucket.evict_item: policy=CLOCK evicted_key={:?}",
			evicted.key
		);
		Some(evicted)
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
