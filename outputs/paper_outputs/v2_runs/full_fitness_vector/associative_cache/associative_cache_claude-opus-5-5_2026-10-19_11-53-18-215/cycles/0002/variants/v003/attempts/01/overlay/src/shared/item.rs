//! This module defines a key-value data object stored in the associative cache.
//!
//! H03 (cycle 2): the per-item recency stamp is a 32-bit logical tick rather
//! than a 16-byte `Instant`. The LFU counter is a 32-bit counter. So
//! `Item<u64, u64>` is `8 + 8 + 4 + 4 = 24` bytes with no padding. A
//! thread-local logical clock replaces every `Instant::now()` call. Recency
//! comparisons are wrap-safe: two stamps compare correctly as long as they are
//! less than 2^31 ticks apart.
//!
//! # Examples
//!
//! ```ignore
//! use associative_cache::shared::item::Item;
//!
//! let item = Item::new(1, 2);
//! assert_eq!(item.key, 1);
//! assert_eq!(item.value, 2);
//! assert_eq!(item.lfu_counter, 0);
//! ```

use std::{
	cell::Cell,
	fmt::{Display, Formatter},
	hash::Hash,
};

thread_local! {
	static CLOCK: Cell<u32> = const { Cell::new(0) };
}

/// Advances the thread-local logical clock by one (wrapping) and returns the
/// new value. This replaces `Instant::now()` for recency stamps.
#[inline]
pub fn tick() -> u32 {
	CLOCK.with(|c| {
		let t = c.get().wrapping_add(1);
		c.set(t);
		t
	})
}

/// Current value of the thread-local logical clock, without advancing it.
#[inline]
pub fn now() -> u32 {
	CLOCK.with(|c| c.get())
}

/// Wrap-safe "a is strictly newer than b" for logical stamps less than 2^31
/// ticks apart.
#[inline]
pub fn is_newer(a: u32, b: u32) -> bool {
	(a.wrapping_sub(b) as i32) > 0
}

#[derive(Debug, Clone)]
pub struct Item<K, V>
where
	K: Hash + Eq + Clone,
	V: Eq + Clone,
{
	pub key: K,
	pub value: V,
	pub timestamp: u32,
	pub lfu_counter: u32,
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
			timestamp: tick(),
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
	fn tick_is_monotonic_and_stamps_new_items() {
		let a = Item::new(1u64, 1u64);
		let b = Item::new(2u64, 2u64);
		assert!(is_newer(b.timestamp, a.timestamp));
		assert_eq!(now(), b.timestamp);
		let t = tick();
		assert!(is_newer(t, b.timestamp));
	}

	#[test]
	fn is_newer_is_wrap_safe() {
		assert!(is_newer(1, u32::MAX));
		assert!(!is_newer(u32::MAX, 1));
		assert!(!is_newer(5, 5));
	}
}
