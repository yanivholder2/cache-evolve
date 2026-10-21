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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Ghost filter (parent) kept. For ClassicLRU the new item is written into the
	/// victim slot (index 0 when full, a fresh tail slot otherwise) with no shift.
	/// If the ghost matches (or the bucket is not full) it is promoted to len/4 by
	/// a three-way rotation: new -> len/4, old len/4 occupant -> len/2, old len/2
	/// occupant -> victim slot.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.eviction_policy.get_from_position(&mut self.items, position);
			return (&self.items[idx], None);
		}
		let was_full = self.is_full();
		if self.eviction_policy.name() != "ClassicLRU" {
			let evicted = if was_full { self.evict_item() } else { None };
			if let Some(e) = &evicted {
				self.ghost = 0x100 | fingerprint(&e.key);
			}
			let at = self.items.len();
			self.items.push(item);
			return (&self.items[at], evicted);
		}
		let fp = 0x100 | fingerprint(&item.key);
		let promote = !was_full || self.ghost == fp;
		let (vi, evicted) = if was_full {
			let old = std::mem::replace(&mut self.items[0], item);
			(0usize, Some(old))
		} else {
			self.items.push(item);
			(self.items.len() - 1, None)
		};
		if let Some(e) = &evicted {
			self.ghost = 0x100 | fingerprint(&e.key);
		}
		let n = self.items.len();
		let mut at = vi;
		if promote {
			let q = n / 4;
			let h = (n / 2).min(n - 1);
			if q != vi {
				self.items.swap(vi, q); // new -> q, old q -> victim slot
				at = q;
				if h != q && h != vi {
					// old q -> h, old h -> victim slot
					self.items.swap(vi, h);
				}
			}
		}
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
