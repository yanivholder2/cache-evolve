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
	/// True when the policy is ClassicLRU: use CLOCK-style reference bits
	/// (stored in the otherwise unused `lfu_counter` of each item).
	clock: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let clock = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			clock,
		}
	}

	/// Second-chance scan from the LRU end (index 0).
	fn clock_victim(&mut self) -> usize {
		let n = self.items.len();
		for i in 0..n {
			if self.items[i].lfu_counter != 0 {
				self.items[i].lfu_counter = 0;
			} else {
				return i;
			}
		}
		n - 1
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
		if self.clock {
			self.items[position].lfu_counter = 1;
			return &self.items[position];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// New items enter at index len/4 with the reference bit clear. Hits set
	/// the reference bit rather than reordering; eviction gives second chances.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let r = self.get_from_position(position);
			let _ = r;
			return (&self.items[position], None);
		}
		if !self.clock {
			let evicted = if self.is_full() { self.evict_item() } else { None };
			let at = self.items.len();
			self.items.insert(at, item);
			return (&self.items[at], evicted);
		}
		let evicted = if self.is_full() && !self.items.is_empty() {
			let v = self.clock_victim();
			Some(self.items.remove(v))
		} else {
			None
		};
		let mut item = item;
		item.lfu_counter = 0;
		let at = self.items.len() / 4;
		self.items.insert(at, item);
		(&self.items[at], evicted)
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
