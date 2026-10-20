use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Full 64-bit hash of a key.
fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

/// 8-bit fingerprint of a key.
fn fingerprint(h: u64) -> u16 {
	((h >> 32) & 0xff) as u16
}

/// 16-bit tag of a key (bits 16..32 of the hash).
fn tag16(h: u64) -> u16 {
	((h >> 16) & 0xffff) as u16
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
	/// Ring of the 4 most recently evicted 16-bit key tags.
	ring: [u16; 4],
	cursor: u8,
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
			ring: [0; 4],
			cursor: 0,
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

	/// Put-hit: overwrite value in place, no promotion. Miss: ghost-filtered
	/// insertion as in parent, except a key whose tag is in the 4-entry
	/// eviction ring is inserted deeper, at len/2.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			return (&self.items[position], None);
		}
		let was_full = self.is_full();
		let evicted = if was_full { self.evict_item() } else { None };
		let h = hash_of(&item.key);
		let fp = 0x100 | fingerprint(h);
		let at = if self.eviction_policy.name() == "ClassicLRU" {
			let t = tag16(h);
			let r = self.ring;
			let in_ring = was_full && ((r[0] == t) | (r[1] == t) | (r[2] == t) | (r[3] == t));
			if in_ring {
				self.items.len() / 2
			} else if was_full && self.ghost != fp {
				0
			} else {
				self.items.len() / 4
			}
		} else {
			self.items.len()
		};
		if let Some(e) = &evicted {
			let eh = hash_of(&e.key);
			self.ghost = 0x100 | fingerprint(eh);
			self.ring[(self.cursor & 3) as usize] = tag16(eh);
			self.cursor = (self.cursor + 1) & 3;
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
