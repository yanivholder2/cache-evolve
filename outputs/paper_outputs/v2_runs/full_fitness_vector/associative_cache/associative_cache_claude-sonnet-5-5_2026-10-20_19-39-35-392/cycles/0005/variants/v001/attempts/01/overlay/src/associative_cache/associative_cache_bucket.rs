use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Cheap multiplicative hasher used only to derive 8-bit tags.
#[derive(Default)]
struct TagHasher(u64);

impl Hasher for TagHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		}
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0.rotate_left(5) ^ i).wrapping_mul(0x9E37_79B9_7F4A_7C15);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}
}

#[inline]
fn tag_of<K: Hash>(key: &K) -> u8 {
	let mut h = TagHasher::default();
	key.hash(&mut h);
	(h.finish() >> 56) as u8
}

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Contiguous per-slot key tags, parallel to `items`.
	tags: Vec<u8>,
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
			tags: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
		}
	}

	/// Recompute tags for the inclusive slot range touched by a policy move.
	#[inline]
	fn resync(&mut self, a: usize, b: usize) {
		let (lo, hi) = if a <= b { (a, b) } else { (b, a) };
		for i in lo..=hi {
			self.tags[i] = tag_of(&self.items[i].key);
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

	fn get_position(&self, key: &K) -> Option<usize> {
		if self.tags.len() != self.items.len() {
			// Items were mutated externally; fall back to a plain scan.
			return self.items.iter().position(|d| d.key == *key);
		}
		let t = tag_of(key);
		let items = &self.items;
		for (i, &tg) in self.tags.iter().enumerate() {
			if tg == t && items[i].key == *key {
				return Some(i);
			}
		}
		None
	}

	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		if self.tags.len() == self.items.len() && index != position {
			self.resync(position, index);
		}
		&self.items[index]
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		if self.tags.len() == self.items.len() {
			self.tags.remove(index);
		}
		self.items.remove(index)
	}

	/// New items enter at index len/4 (near the LRU end) for ClassicLRU,
	/// instead of the MRU end. Hits still promote to MRU.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let idx = self.get_from_position(position).key;
			let _ = idx;
			let pos = self.get_position(&item.key).expect("key present after update");
			return (&self.items[pos], None);
		}
		let evicted = if self.is_full() { self.evict_item() } else { None };
		let at = if self.eviction_policy.name() == "ClassicLRU" {
			self.items.len() / 4
		} else {
			self.items.len()
		};
		let t = tag_of(&item.key);
		self.items.insert(at, item);
		if self.tags.len() + 1 == self.items.len() {
			self.tags.insert(at, t);
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
