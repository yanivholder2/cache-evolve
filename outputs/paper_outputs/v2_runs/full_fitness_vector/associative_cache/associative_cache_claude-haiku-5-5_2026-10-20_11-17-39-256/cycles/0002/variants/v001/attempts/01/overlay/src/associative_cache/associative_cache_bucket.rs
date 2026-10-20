use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// One-byte key fingerprint: the top byte of the key's hash.
#[inline]
fn fingerprint<K: Hash>(key: &K) -> u8 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	(hasher.finish() >> 56) as u8
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// `fingerprints[i]` is the fingerprint of `items[i].key`; kept the same length
	/// as `items` and updated wherever `items` is written, cleared, or reordered.
	fingerprints: Vec<u8>,
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
			fingerprints: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Recompute every fingerprint from the items. Only used for policies whose
	/// reordering is not known to this bucket.
	fn rebuild_fingerprints(&mut self) {
		self.fingerprints.clear();
		self.fingerprints.extend(self.items.iter().map(|item| fingerprint(&item.key)));
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
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.fingerprints.push(fingerprint(&item.key));
		self.items.push(item);
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	fn get_position(&self, key: &K) -> Option<usize> {
		let fp = fingerprint(key);
		for (i, stored) in self.fingerprints.iter().enumerate() {
			if *stored == fp && self.items[i].key == *key {
				return Some(i);
			}
		}
		None
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// Mirror the policy's reordering of `items` into `fingerprints`.
		match self.eviction_policy.name() {
			// remove(position) + push: move the fingerprint to the back, no rehash.
			"ClassicLRU" => {
				let fp = self.fingerprints.remove(position);
				self.fingerprints.push(fp);
			}
			// swap(position, position - 1) when position > 0.
			"Swap" => {
				if position > 0 {
					self.fingerprints.swap(position, position - 1);
				}
			}
			// These policies never reorder items.
			"TimestampLRU" | "Lifo" | "Lfu" | "Fifo" => {}
			_ => self.rebuild_fingerprints(),
		}
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		self.fingerprints.remove(index);
		self.items.remove(index)
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
