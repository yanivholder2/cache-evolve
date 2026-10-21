use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Bucket using CLOCK (second-chance) eviction. The per-slot reference bit is
/// stored in `Item::lfu_counter` (0 or 1); hits only set the bit, never move items.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	hand: u8,
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
			hand: 0,
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

	/// CLOCK hit: set the reference bit only, no movement.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let item = &mut self.items[position];
		item.lfu_counter = 1;
		&self.items[position]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let it = self.get_from_position(position);
			return (it, None);
		}
		let mut item = item;
		item.lfu_counter = 0;
		let len = self.items.len();
		if len < self.max_size || len == 0 {
			self.items.push(item);
			return (self.items.last().expect("just pushed"), None);
		}
		let mut h = self.hand as usize % len;
		loop {
			if self.items[h].lfu_counter == 0 {
				break;
			}
			self.items[h].lfu_counter = 0;
			h += 1;
			if h == len {
				h = 0;
			}
		}
		let evicted = std::mem::replace(&mut self.items[h], item);
		self.hand = ((h + 1) % len) as u8;
		(&self.items[h], Some(evicted))
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
