//! This module defines a key-value data object stored in the associative cache.
//!
//! H03 (cycle 0005): 16-byte item. `Item` is just `{ key, value }`, so
//! `Item<u64, u64>` packs into exactly 16 bytes. The SLRU bucket orders items
//! purely by position and never needed per-item metadata. The non-SLRU fallback
//! policies are either positional (LRU / FIFO / LIFO / Swap) or keep their state
//! (LFU counters) in a per-bucket side array parallel to `items`
//! (see `AssociativeCacheBucket::meta`).
//!
//! # Examples
//!
//! ```ignore
//! use associative_cache::shared::item::Item;
//!
//! let item = Item::new(1, 2);
//! assert_eq!(item.key, 1);
//! assert_eq!(item.value, 2);
//! ```

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
};

#[derive(Debug, Clone)]
pub struct Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
}

impl<K, V> Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	#[inline]
	pub fn new(key: K, value: V) -> Self {
		Self { key, value }
	}
}

impl<K, V> PartialEq for Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	fn eq(&self, other: &Item<K, V>) -> bool {
		self.key == other.key && self.value == other.value
	}
}

impl<K, V> Eq for Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
}

impl<K, V> Display for Item<K, V>
where
	K: Hash + Eq + Clone + Display,
	V: Eq + Clone + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Bucket Item {{ key: {}, value: {} }}", self.key, self.value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn new_initializes_fields() {
		let item = Item::new(42, "hello");
		assert_eq!(item.key, 42);
		assert_eq!(item.value, "hello");
	}

	#[test]
	fn equality_compares_key_and_value() {
		let a = Item::new(1, 10);
		let b = Item::new(1, 10);
		let c = Item::new(1, 20);
		let d = Item::new(2, 10);

		assert_eq!(a, b);
		assert_ne!(a, c); // different value
		assert_ne!(a, d); // different key
	}

	#[test]
	fn clone_produces_independent_copy() {
		let mut a = Item::new(1, 10);
		let b = a.clone();
		a.value = 20;
		assert_eq!(a.value, 20);
		assert_eq!(b.value, 10);
	}

	#[test]
	fn item_u64_u64_is_16_bytes() {
		assert_eq!(std::mem::size_of::<Item<u64, u64>>(), 16);
	}
}
