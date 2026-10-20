use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

// Sampled global hit/miss counters (1-in-16 buckets) and adaptive insertion depth.
static HITS: AtomicU32 = AtomicU32::new(0);
static MISSES: AtomicU32 = AtomicU32::new(0);
static PUTS: AtomicU32 = AtomicU32::new(0);
static DEEP_INSERT: AtomicBool = AtomicBool::new(false);

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

	/// Sampled bucket: derived from the items buffer address (no extra field).
	#[inline]
	fn is_sampled(&self) -> bool {
		((self.items.as_ptr() as usize) >> 6) & 15 == 0
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

	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let position = self.get_position(key);
		if self.is_sampled() {
			if position.is_some() {
				HITS.fetch_add(1, Ordering::Relaxed);
			} else {
				MISSES.fetch_add(1, Ordering::Relaxed);
			}
		}
		let position = position?;
		Some(self.get_from_position(position))
	}

	/// New items enter at index len/4 (or len/2 when the sampled hit ratio is
	/// high) for ClassicLRU. Hits still promote to MRU.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.eviction_policy.get_from_position(&mut self.items, position);
			return (&self.items[idx], None);
		}
		let n = PUTS.fetch_add(1, Ordering::Relaxed).wrapping_add(1);
		if n & 1023 == 0 {
			let h = HITS.load(Ordering::Relaxed);
			let m = MISSES.load(Ordering::Relaxed);
			DEEP_INSERT.store(h > m, Ordering::Relaxed);
			HITS.store(h >> 1, Ordering::Relaxed);
			MISSES.store(m >> 1, Ordering::Relaxed);
		}
		let evicted = if self.is_full() { self.evict_item() } else { None };
		let at = if self.eviction_policy.name() == "ClassicLRU" {
			if DEEP_INSERT.load(Ordering::Relaxed) {
				self.items.len() / 2
			} else {
				self.items.len() / 4
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
