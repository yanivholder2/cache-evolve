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
//! # Layout (variant c0004/v001)
//!
//! Metadata fields, beyond key/value:
//! - `timestamp: u32`: a logical access tick drawn from a process-wide
//!   monotonically increasing counter ([`next_tick`]). It replaces the 16-byte
//!   `Instant`. Smaller tick = older access, the same ordering `Instant` gave
//!   `TimestampLRU`. The counter wraps after 2^32 ticks, which is acceptable for
//!   the benchmark traces.
//! - `lfu_counter: u8` (from c0003/v002). Callers saturate at 255.
//!
//! For `Item<u64, u64>`: key(8) + value(8) + tick(4) + counter(1) = 21 bytes,
//! which rounds up to 24 at 8-byte alignment (it was 40).

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
	sync::atomic::{AtomicU32, Ordering},
};

/// Process-wide logical clock. Only relative order matters, so `Relaxed` is enough.
static TICK: AtomicU32 = AtomicU32::new(0);

/// Returns the next logical access tick (strictly increasing until wraparound).
#[inline]
pub fn next_tick() -> u32 {
	TICK.fetch_add(1, Ordering::Relaxed)
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
	pub lfu_counter: u8,
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
			timestamp: next_tick(),
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
	use std::mem::size_of;

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
	fn equality_ignores_timestamp() {
		let a = Item::new(1, 10);
		let mut b = Item::new(1, 10);
		b.timestamp = a.timestamp.wrapping_add(1000);
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
	fn lfu_counter_is_one_byte() {
		let item = Item::new(1_u64, 1_u64);
		assert_eq!(std::mem::size_of_val(&item.lfu_counter), 1);
	}

	#[test]
	fn timestamp_is_four_bytes() {
		let item = Item::new(1_u64, 1_u64);
		assert_eq!(std::mem::size_of_val(&item.timestamp), 4);
	}

	#[test]
	fn later_items_get_larger_ticks() {
		let a = Item::new(1_u64, 1_u64);
		let b = Item::new(2_u64, 2_u64);
		assert!(b.timestamp > a.timestamp);
	}

	#[test]
	fn item_not_larger_than_before() {
		assert!(size_of::<Item<u64, u64>>() <= 40);
	}

	#[test]
	fn item_u64_u64_smaller_than_40_bytes() {
		assert!(size_of::<Item<u64, u64>>() < 40);
		assert_eq!(size_of::<Item<u64, u64>>(), 24);
	}
}
