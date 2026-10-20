use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// 8-bit fingerprint of a key, taken from the high byte of its hash.
///
/// The low bits of the hash already select the bucket (`get_index` is
/// `hash % num_buckets`), so within one bucket the low byte carries almost no
/// information. The high byte is independent of the bucket index.
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
	/// Parallel to `items`: `fingerprints[i]` is the fingerprint of `items[i].key`.
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

	/// Scans the fingerprint array first and compares full keys only where the
	/// fingerprint matches.
	fn get_position(&self, key: &K) -> Option<usize> {
		let fp = fingerprint(key);
		self.fingerprints
			.iter()
			.enumerate()
			.find(|&(i, f)| *f == fp && self.items[i].key == *key)
			.map(|(i, _)| i)
	}

	/// Same as the trait default, but keeps the fingerprint array in sync on
	/// insert, update and eviction.
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

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		// The policy reorders the bucket only when it returns a different index
		// (ClassicLRU move-to-back, Swap). Re-derive the fingerprints in that case.
		if index != position {
			self.fingerprints.clear();
			self.fingerprints.extend(self.items.iter().map(|item| fingerprint(&item.key)));
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
