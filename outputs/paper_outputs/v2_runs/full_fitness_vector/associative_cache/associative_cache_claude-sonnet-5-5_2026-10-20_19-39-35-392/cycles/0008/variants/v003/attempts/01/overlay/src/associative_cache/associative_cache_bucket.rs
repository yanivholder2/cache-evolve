use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::sync::atomic::{AtomicU8, Ordering};

/// Single global saturating hit/miss counter in [0, 15]: a miss increments,
/// a hit decrements. >= 8 means scan-heavy (insert at len/4), else len/2.
static MISS_COUNTER: AtomicU8 = AtomicU8::new(0);

#[inline(always)]
fn bump(miss: bool) {
	let c = MISS_COUNTER.load(Ordering::Relaxed);
	let n = if miss {
		c + (c < 15) as u8
	} else {
		c - (c > 0) as u8
	};
	MISS_COUNTER.store(n, Ordering::Relaxed);
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

	/// Transpose promotion: hot end is the front, one step toward it per hit.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let idx = if position > 0 {
			self.items.swap(position, position - 1);
			position - 1
		} else {
			position
		};
		&self.items[idx]
	}

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let pos = self.get_position(key);
		bump(pos.is_none());
		Some(self.get_from_position(pos?))
	}

	/// Eviction takes the last slot; the new item is inserted at an adaptive
	/// index (len/4 or len/2) by swap, displacing that occupant to the tail.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let p = self.get_from_position(position);
			return (p, None);
		}
		let evicted = if self.items.len() >= self.max_size {
			self.items.pop()
		} else {
			None
		};
		let len = self.items.len();
		let div_shift = 2 - (MISS_COUNTER.load(Ordering::Relaxed) < 8) as usize; // 1 -> /2, 2 -> /4
		let idx = len >> div_shift;
		self.items.push(item);
		self.items.swap(idx, len);
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
