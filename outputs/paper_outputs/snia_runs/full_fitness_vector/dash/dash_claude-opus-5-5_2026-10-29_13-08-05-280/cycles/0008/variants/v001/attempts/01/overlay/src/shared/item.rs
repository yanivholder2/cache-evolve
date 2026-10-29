//! This module defines a key-value data object stored in the Dash cache.
//!
//! # Layout
//!
//! `timestamp` is a 32-bit logical tick (not a wall-clock `Instant`) and
//! `lfu_counter` is a `u8`, so `Item<u64, u64>` is 8 + 8 + 4 + 1 = 21 bytes,
//! padded to 24. Ticks are assigned by the owning segment on insert (from a
//! segment-local counter, no global atomic) and compared with wrapping-aware
//! subtraction (see [`tick_offset`]).
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
	/// Logical tick (segment-local). Compare only via wrapping-aware offsets.
	pub timestamp: u32,
	pub lfu_counter: u8,
}

/// Wrapping-aware signed distance from `reference` to `tick`: positive when
/// `tick` is newer than `reference`. Correct as long as live ticks being
/// compared lie within 2^31 of each other.
#[inline]
pub fn tick_offset(tick: u32, reference: u32) -> i32 {
	tick.wrapping_sub(reference) as i32
}

impl<K, V> Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	/// Builds an item with tick 0; the owning segment stamps the real tick on insert.
	pub fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			timestamp: 0,
			lfu_counter: 0,
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
			"Bucket Item {{ key: {}, value: {}, lfu_counter: {}, timestamp: {} }}",
			self.key, self.value, self.lfu_counter, self.timestamp
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
	fn item_u64_u64_is_24_bytes() {
		assert_eq!(std::mem::size_of::<Item<u64, u64>>(), 24);
	}

	#[test]
	fn tick_offset_is_wrapping_aware() {
		assert_eq!(tick_offset(5, 3), 2);
		assert_eq!(tick_offset(3, 5), -2);
		assert_eq!(tick_offset(1, u32::MAX), 2);
		assert_eq!(tick_offset(u32::MAX, 1), -2);
	}
}
