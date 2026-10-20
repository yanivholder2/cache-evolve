//! This module defines a key-value data object stored in the Dash cache.
//!
//! # Examples
//!
//! ```ignore
//! use dash::shared::item::Item;
//!
//! let item = Item::new(1, 2);
//! assert_eq!(item.key, 1);
//! assert_eq!(item.value, 2);
//! assert_eq!(item.lfu_count(), 0);
//! ```

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
	time::Instant,
};

/// Top bit of `lfu_counter`, reserved for the "seen in stash" admission marker.
///
/// The counter grows by one per access, so no realistic count reaches this bit.
/// Keeping the marker here, rather than in a new field, leaves the entry's size unchanged.
const SEEN_IN_STASH_BIT: usize = 1 << (usize::BITS - 1);

#[derive(Debug, Clone)]
pub struct Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
	/// Access count in the low bits; the top bit is the stash marker. Read the count
	/// through [`Item::lfu_count`] and the marker through [`Item::is_seen_in_stash`].
	pub lfu_counter: usize,
	pub timestamp: Instant,
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

	/// The access count, with the stash marker bit masked off.
	pub fn lfu_count(&self) -> usize {
		self.lfu_counter & !SEEN_IN_STASH_BIT
	}

	/// Whether the item has been served from the stash once without promotion.
	pub fn is_seen_in_stash(&self) -> bool {
		(self.lfu_counter & SEEN_IN_STASH_BIT) != 0
	}

	/// Sets or clears the stash marker, leaving the access count untouched.
	pub fn set_seen_in_stash(&mut self, seen: bool) {
		if seen {
			self.lfu_counter |= SEEN_IN_STASH_BIT;
		} else {
			self.lfu_counter &= !SEEN_IN_STASH_BIT;
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
			self.lfu_count(),
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
		assert_eq!(item.lfu_count(), 0);
		assert!(!item.is_seen_in_stash());
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

	#[test]
	fn stash_marker_is_independent_of_lfu_count() {
		let mut item = Item::new(1, 10);
		item.lfu_counter = 5;
		item.set_seen_in_stash(true);
		assert!(item.is_seen_in_stash());
		assert_eq!(item.lfu_count(), 5);
		item.set_seen_in_stash(false);
		assert!(!item.is_seen_in_stash());
		assert_eq!(item.lfu_count(), 5);
	}
}
