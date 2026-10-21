use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Bucket with a probation/protected split. `items[..probation_len]` is the
/// probation segment (front = LRU), `items[probation_len..]` is protected
/// (front = LRU). Probation capacity is max(1, max_size / 16).
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	probation_len: usize,
	probation_cap: usize,
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
			probation_len: 0,
			probation_cap: std::cmp::max(1, settings.bucket_size / 16),
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

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		let evicted = if self.is_full() { self.evict_item() } else { None };
		let at = self.probation_len;
		self.items.insert(at, item);
		self.probation_len += 1;
		(&self.items[at], evicted)
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		if self.probation_len >= self.probation_cap || self.probation_len == self.items.len() {
			Some(0)
		} else {
			Some(self.probation_len)
		}
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		if index < self.probation_len {
			self.probation_len -= 1;
		}
		self.items.remove(index)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if position < self.probation_len {
			let item = self.items.remove(position);
			self.probation_len -= 1;
			let prot_cap = self.max_size.saturating_sub(self.probation_cap);
			let prot_len = self.items.len() - self.probation_len;
			if prot_len >= prot_cap && self.probation_len < self.items.len() {
				// Demote LRU protected to back of probation.
				self.probation_len += 1;
			}
			self.items.push(item);
			let last = self.items.len() - 1;
			if prot_cap == 0 {
				// Degenerate: no protected space; keep it in probation.
				self.probation_len = self.items.len();
			}
			&self.items[last]
		} else {
			let item = self.items.remove(position);
			self.items.push(item);
			let last = self.items.len() - 1;
			&self.items[last]
		}
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
