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
	/// True when the positional segmented rule applies (ClassicLRU policy).
	positional: bool,
	/// Size of the probation zone (positions from the eviction end).
	probation: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let positional = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			positional,
			probation: (settings.bucket_size / 16).max(1),
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if !self.positional {
			let index = self.eviction_policy.get_from_position(&mut self.items, position);
			return &self.items[index];
		}
		let n = self.items.len();
		// Probation zone hit: move to MRU. Elsewhere: skip if already in the
		// most-recent quarter, otherwise move to MRU.
		let recent_quarter = (n / 4).max(1);
		if position >= self.probation && position + recent_quarter >= n {
			return &self.items[position];
		}
		let item = self.items.remove(position);
		self.items.push(item);
		&self.items[n - 1]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if !self.positional {
			if let Some(position) = self.get_position(&item.key) {
				self.items[position].value = item.value;
				let pushed = self.get_from_position(position);
				return (pushed, None);
			}
			let evicted = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			return (self.items.last().expect("just pushed"), evicted);
		}
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		if self.is_full() {
			let evicted = self.items.remove(0);
			// Insert just above the probation boundary rather than at MRU.
			let at = self.probation.min(self.items.len());
			self.items.insert(at, item);
			return (&self.items[at], Some(evicted));
		}
		self.items.push(item);
		(self.items.last().expect("just pushed"), None)
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
