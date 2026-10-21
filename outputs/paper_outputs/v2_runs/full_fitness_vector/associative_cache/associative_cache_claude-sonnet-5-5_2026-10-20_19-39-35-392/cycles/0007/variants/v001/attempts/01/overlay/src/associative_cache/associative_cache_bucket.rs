use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// 8-bit fingerprint of a key.
fn fingerprint<K: Hash>(key: &K) -> u16 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	((h.finish() >> 32) & 0xff) as u16
}

#[derive(Debug)]
#[repr(align(64))]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Ghost: 0 = none, else 0x100 | fingerprint of the last evicted key.
	ghost: u16,
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
			ghost: 0,
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

	/// Hit promotion for ClassicLRU: instead of a full move-to-MRU, swap with the
	/// slot halfway (at least one step) between the hit and the MRU end (last
	/// index). Other policies are unchanged.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.eviction_policy.name() == "ClassicLRU" {
			let last = self.items.len() - 1;
			let dist = last - position;
			if dist == 0 {
				return &self.items[position];
			}
			let target = position + std::cmp::max(1, dist / 2);
			self.items.swap(position, target);
			return &self.items[target];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Parent: new items enter at len/4 for ClassicLRU. Ghost filter: when the
	/// bucket is full, a key whose fingerprint doesn't match the last evicted
	/// key goes to the LRU end (index 0, next victim); a match enters at len/4.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.eviction_policy.get_from_position(&mut self.items, position);
			return (&self.items[idx], None);
		}
		let was_full = self.is_full();
		let evicted = if was_full { self.evict_item() } else { None };
		let fp = 0x100 | fingerprint(&item.key);
		let at = if self.eviction_policy.name() == "ClassicLRU" {
			if was_full && self.ghost != fp {
				0
			} else {
				self.items.len() / 4
			}
		} else {
			self.items.len()
		};
		if let Some(e) = &evicted {
			self.ghost = 0x100 | fingerprint(&e.key);
		}
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
