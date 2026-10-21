use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Miss-pressure counter lives in the high bits of `max_size` (no new field,
/// so the struct size is unchanged). Low 32 bits are the real max size.
const SIZE_MASK: usize = 0xFFFF_FFFF;
const CTR_SHIFT: u32 = 32;
const CTR_MAX: usize = 15;
const CTR_THRESHOLD: usize = 7;

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
			max_size: settings.bucket_size & SIZE_MASK,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	#[inline]
	fn counter(&self) -> usize {
		self.max_size >> CTR_SHIFT
	}

	#[inline]
	fn set_counter(&mut self, c: usize) {
		self.max_size = (self.max_size & SIZE_MASK) | (c << CTR_SHIFT);
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
		self.max_size & SIZE_MASK
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// New items enter at len/2 or len/4 depending on a miss-pressure counter
	/// updated only on the put-miss path. Hits are untouched.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.eviction_policy.get_from_position(&mut self.items, position);
			return (&self.items[idx], None);
		}
		let lru = self.eviction_policy.name() == "ClassicLRU";
		let mut evicted = None;
		if self.is_full() {
			let bottom = self.items.first().map(|i| i.key);
			evicted = self.evict_item();
			if lru {
				let c = self.counter();
				// Victim sitting at the bottom (<= insertion slot) means it was
				// never promoted past it: pressure up; otherwise down.
				let at_bottom = match (&evicted, bottom) {
					(Some(e), Some(b)) => e.key == b,
					_ => false,
				};
				if at_bottom {
					if c < CTR_MAX {
						self.set_counter(c + 1);
					}
				} else if c > 0 {
					self.set_counter(c - 1);
				}
			}
		}
		let at = if lru {
			if self.counter() > CTR_THRESHOLD {
				self.items.len() / 4
			} else {
				self.items.len() / 2
			}
		} else {
			self.items.len()
		};
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
