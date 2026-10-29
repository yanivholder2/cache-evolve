//! This module defines a key-value data object stored in the associative cache.
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
//!
//! Layout: just `key` and `value`, so `Item<u64,u64>` is 16 bytes (was 24
//! with two inline `u32` bookkeeping fields, 40 with `usize` + `Instant`).
//!
//! Per-item policy bookkeeping (TimestampLRU stamp, LFU counter) no longer
//! lives in the item. It sits in a per-bucket side byte buffer kept in
//! lockstep with the items vector, `META_STRIDE` bytes (one little-endian
//! `u32`) per item. Policies that keep no state (ClassicLRU, FIFO, ...) are
//! handed an empty buffer, or ignore it. [`meta_get`]/[`meta_set`] read and
//! write slot `i` of such a buffer.

use std::{
	fmt::{Display, Formatter},
	hash::Hash,
	sync::atomic::{AtomicU32, Ordering},
};

/// Logical clock kept for callers outside the bucket (e.g. a policy driven by
/// a bucket that does not stamp items itself). Strictly increasing per tick
/// (wraps after 2^32 ticks).
static LOGICAL_CLOCK: AtomicU32 = AtomicU32::new(0);

/// Advances the logical clock and returns the new tick.
#[inline]
pub fn tick() -> u32 {
	LOGICAL_CLOCK.fetch_add(1, Ordering::Relaxed).wrapping_add(1)
}

/// Current logical-clock value (no advance).
#[inline]
pub fn now() -> u32 {
	LOGICAL_CLOCK.load(Ordering::Relaxed)
}

/// Bytes of per-item policy metadata in a side buffer (one `u32`).
pub const META_STRIDE: usize = 4;

/// Reads the `u32` metadata word of item `i` from a side buffer.
/// Panics if the buffer has no slot `i`.
#[inline]
pub fn meta_get(meta: &[u8], i: usize) -> u32 {
	let o = i * META_STRIDE;
	let mut b = [0u8; META_STRIDE];
	b.copy_from_slice(&meta[o..o + META_STRIDE]);
	u32::from_le_bytes(b)
}

/// Writes the `u32` metadata word of item `i` into a side buffer.
/// Panics if the buffer has no slot `i`.
#[inline]
pub fn meta_set(meta: &mut [u8], i: usize, v: u32) {
	let o = i * META_STRIDE;
	meta[o..o + META_STRIDE].copy_from_slice(&v.to_le_bytes());
}

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

	#[test]
	fn meta_round_trips() {
		let mut buf = vec![0u8; 3 * META_STRIDE];
		meta_set(&mut buf, 0, 7);
		meta_set(&mut buf, 2, u32::MAX);
		assert_eq!(meta_get(&buf, 0), 7);
		assert_eq!(meta_get(&buf, 1), 0);
		assert_eq!(meta_get(&buf, 2), u32::MAX);
	}
}
