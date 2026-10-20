use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Number of recently evicted keys remembered per bucket ("ghosts").
const GHOST_CAPACITY: usize = 4;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Ring of recently evicted keys.
	ghosts: Vec<K>,
	/// Round-robin write index into `ghosts` once it is full.
	ghost_next: usize,
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
			ghosts: Vec::with_capacity(GHOST_CAPACITY),
			ghost_next: 0,
		}
	}

	/// Records an evicted key in the ghost ring.
	fn remember_ghost(&mut self, key: K) {
		if self.ghosts.len() < GHOST_CAPACITY {
			self.ghosts.push(key);
			self.ghost_next = self.ghosts.len() % GHOST_CAPACITY;
		} else {
			self.ghosts[self.ghost_next] = key;
			self.ghost_next = (self.ghost_next + 1) % GHOST_CAPACITY;
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

	/// Ghost-aware put: a key that was recently evicted from this bucket is
	/// reinserted at the bucket midpoint rather than at the back.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.get_items_mut()[position].value = item.value;
			let updated = self.get_from_position(position);
			return (updated, None);
		}

		let evicted = if self.is_full() { self.evict_item() } else { None };
		if let Some(ref ev) = evicted {
			let key = ev.key;
			self.remember_ghost(key);
		}

		let ghost_pos = self.ghosts.iter().position(|g| *g == item.key);
		let idx = if let Some(gpos) = ghost_pos {
			self.ghosts.remove(gpos);
			let mid = self.items.len() / 2;
			self.items.insert(mid, item);
			mid
		} else {
			self.items.push(item);
			self.items.len() - 1
		};

		(&self.items[idx], evicted)
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
