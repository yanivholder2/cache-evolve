use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Items with `lfu_counter == 0` are probationary; `1` means protected.
/// Probation is about a sixteenth of the bucket; the rest is protected.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	protected_cap: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let probation = (settings.bucket_size / 16).max(1);
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			protected_cap: settings.bucket_size.saturating_sub(probation),
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

	/// Single pass with early exit on match.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.items.iter().position(|d| d.key == *key)?;
		Some(self.get_from_position(position))
	}

	/// One scan: finds an existing key (update) or, on miss, the victim
	/// (first probationary item, else the front) in the same pass.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		let mut found = None;
		let mut first_prob = None;
		for (i, it) in self.items.iter().enumerate() {
			if it.key == item.key {
				found = Some(i);
				break;
			}
			if first_prob.is_none() && it.lfu_counter == 0 {
				first_prob = Some(i);
			}
		}
		if let Some(position) = found {
			self.items[position].value = item.value;
			let pushed = self.get_from_position(position);
			return (pushed, None);
		}
		let evicted = if self.items.len() == self.max_size && !self.items.is_empty() {
			Some(self.items.remove(first_prob.unwrap_or(0)))
		} else {
			None
		};
		self.items.push(item);
		let pushed = self.items.last().expect("bucket invariant: just pushed");
		(pushed, evicted)
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.items[position].lfu_counter == 0 {
			// Promotion from probation; keep protected size bounded by demoting
			// the protected LRU entry (first flagged item in order).
			let mut protected = 0usize;
			let mut first_flagged = None;
			for (i, it) in self.items.iter().enumerate() {
				if it.lfu_counter != 0 {
					if first_flagged.is_none() {
						first_flagged = Some(i);
					}
					protected += 1;
				}
			}
			if protected >= self.protected_cap {
				if let Some(d) = first_flagged {
					self.items[d].lfu_counter = 0;
				}
			}
			self.items[position].lfu_counter = 1;
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn eviction_candidate_index(&self) -> Option<usize> {
		if self.items.is_empty() {
			return None;
		}
		Some(self.items.iter().position(|i| i.lfu_counter == 0).unwrap_or(0))
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
