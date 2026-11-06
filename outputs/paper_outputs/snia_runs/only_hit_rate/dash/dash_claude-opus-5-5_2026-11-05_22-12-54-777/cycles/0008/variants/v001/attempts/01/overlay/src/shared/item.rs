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
//! assert_eq!(item.lfu_counter, 0);
//! ```
//!
//! Variant cycle/0008/v001 (H01): the top bit of `lfu_counter` is the one-bit
//! `ghost_admitted` (probation) flag. The frequency counter is capped at
//! `LFU_COUNTER_CAP` (15), far below that bit, so the flag adds no metadata:
//! `Item` keeps its size. Frequency reads go through [`Item::freq`], which
//! masks the flag off.

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
	time::Instant,
};

/// Top bit of `lfu_counter`: set while an item is a ghost-admitted arrival that
/// has not yet used its one-time exemption from the tail-scan victim choice.
pub const GHOST_ADMITTED_BIT: usize = 1 << (usize::BITS - 1);

#[derive(Debug, Clone)]
pub struct Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
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

	/// Frequency counter with the `ghost_admitted` flag masked off.
	#[inline]
	pub fn freq(&self) -> usize {
		self.lfu_counter & !GHOST_ADMITTED_BIT
	}

	/// Whether the item still holds its one-time ghost-admission exemption.
	#[inline]
	pub fn is_ghost_admitted(&self) -> bool {
		self.lfu_counter & GHOST_ADMITTED_BIT != 0
	}

	#[inline]
	pub fn set_ghost_admitted(&mut self) {
		self.lfu_counter |= GHOST_ADMITTED_BIT;
	}

	#[inline]
	pub fn clear_ghost_admitted(&mut self) {
		self.lfu_counter &= !GHOST_ADMITTED_BIT;
	}

	/// Halve the frequency counter, preserving the `ghost_admitted` flag.
	#[inline]
	pub fn halve_freq(&mut self) {
		let flag = self.lfu_counter & GHOST_ADMITTED_BIT;
		self.lfu_counter = (self.freq() / 2) | flag;
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
			"Bucket Item {{ key: {}, value: {}, lfu_counter: {}, ghost_admitted: {}, timestamp: {:?} }}",
			self.key,
			self.value,
			self.freq(),
			self.is_ghost_admitted(),
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
		assert!(!item.is_ghost_admitted());
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
	fn ghost_flag_is_independent_of_freq() {
		let mut a = Item::new(1, 10);
		a.lfu_counter = 7;
		a.set_ghost_admitted();
		assert!(a.is_ghost_admitted());
		assert_eq!(a.freq(), 7);
		a.halve_freq();
		assert!(a.is_ghost_admitted());
		assert_eq!(a.freq(), 3);
		a.clear_ghost_admitted();
		assert!(!a.is_ghost_admitted());
		assert_eq!(a.lfu_counter, 3);
	}
}
