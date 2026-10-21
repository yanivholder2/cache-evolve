//! This module defines a key-value data object stored in the Dash cache.

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
	time::Instant,
};

/// Top bit of `lfu_counter` holds the "hit once in stash" flag.
const HIT_BIT: usize = 1 << (usize::BITS - 1);

#[derive(Debug, Clone)]
pub struct Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
	/// Frequency counter; the top bit is reserved for the stash-hit flag (see `item_hit`).
	pub lfu_counter: usize,
	pub timestamp: Instant,
}

/// True if the item has already been hit once in the stash.
#[inline(always)]
pub fn item_hit<K, V>(item: &Item<K, V>) -> bool
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	item.lfu_counter & HIT_BIT != 0
}

/// Marks the item as hit once in the stash.
#[inline(always)]
pub fn set_item_hit<K, V>(item: &mut Item<K, V>)
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	item.lfu_counter |= HIT_BIT;
}

impl<K, V> Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			lfu_counter: 0,
			timestamp: Instant::now(),
		}
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
		write!(
			f,
			"Bucket Item {{ key: {}, value: {}, lfu_counter: {}, timestamp: {:?} }}",
			self.key,
			self.value,
			self.lfu_counter & !HIT_BIT,
			self.timestamp
		)
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
		assert_eq!(item.lfu_counter, 0);
		assert!(!item_hit(&item));
	}

	#[test]
	fn hit_flag_roundtrip() {
		let mut item = Item::new(1, 2);
		set_item_hit(&mut item);
		assert!(item_hit(&item));
	}

	#[test]
	fn equality_compares_key_and_value() {
		let a = Item::new(1, 10);
		let b = Item::new(1, 10);
		let c = Item::new(1, 20);
		let d = Item::new(2, 10);

		assert_eq!(a, b);
		assert_ne!(a, c);
		assert_ne!(a, d);
	}

	#[test]
	fn equality_ignores_lfu_counter() {
		let mut a = Item::new(1, 10);
		let b = Item::new(1, 10);
		a.lfu_counter = 99;
		assert_eq!(a, b);
	}

	#[test]
	fn clone_produces_independent_copy() {
		let mut a = Item::new(1, 10);
		let b = a.clone();
		a.value = 20;
		assert_eq!(a.value, 20);
		assert_eq!(b.value, 10);
	}
}
