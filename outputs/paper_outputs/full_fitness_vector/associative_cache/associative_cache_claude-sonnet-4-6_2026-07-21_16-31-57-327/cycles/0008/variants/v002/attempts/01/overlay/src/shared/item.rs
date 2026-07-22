//! This module defines a key-value data object stored in the associative cache.
//!
//! Compact 32-byte `#[repr(C)]` layout: key, value, lfu_counter, _pad.
//! The `timestamp` field present in the baseline has been removed; the
//! `lfu_counter` field serves as a write-once "accessed" flag for the
//! second-chance eviction policy implemented in ClassicLRU.
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
    fmt::{Display, Formatter},
    hash::Hash,
};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Item<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub key: K,
    pub value: V,
    pub lfu_counter: usize,
    // Padding to preserve 32-byte struct size on 64-bit targets (replaces the
    // removed `timestamp: Instant` field which was 16 bytes on the target).
    _pad: u64,
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
            _pad: 0,
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
            "Bucket Item {{ key: {}, value: {}, lfu_counter: {} }}",
            self.key, self.value, self.lfu_counter
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
}
