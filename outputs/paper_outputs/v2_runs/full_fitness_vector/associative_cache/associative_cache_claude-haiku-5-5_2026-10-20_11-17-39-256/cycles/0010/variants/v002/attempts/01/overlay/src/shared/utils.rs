use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

/// 2^64 / golden ratio, the Fibonacci multiplicative hashing constant.
const FIBONACCI_MULTIPLIER: u64 = 0x9E37_79B9_7F4A_7C15;

/// Map a key to a bucket index in `0..size`.
///
/// For a power-of-two `size`, the bucket is the top `log2(size)` bits of
/// `hash64(key) * FIBONACCI_MULTIPLIER` (Fibonacci multiplicative hashing).
/// Other sizes fall back to plain modulo so that utility callers with arbitrary
/// sizes keep working; the cache constructor itself requires a power of two.
pub fn get_index<K>(key: &K, size: usize) -> usize
where
    K: Hash + Debug,
{
    let hash = hash(key);
    if size.is_power_of_two() {
        if size == 1 {
            return 0;
        }
        let log2_buckets = size.trailing_zeros();
        ((hash as u64).wrapping_mul(FIBONACCI_MULTIPLIER) >> (64 - log2_buckets)) as usize
    } else {
        hash % size
    }
}

pub fn hash<K>(key: &K) -> usize
where
    K: Hash + Debug,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    let hash = hasher.finish() as usize;
    trace!("hash: key={key:?} result={hash}");
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_is_deterministic() {
        assert_eq!(hash(&42), hash(&42));
        assert_eq!(hash(&"hello"), hash(&"hello"));
    }

    #[test]
    fn hash_differs_for_different_keys() {
        assert_ne!(hash(&1), hash(&2));
    }

    #[test]
    fn get_index_within_bounds() {
        for size in [1, 2, 4, 16, 100] {
            for key in 0..200 {
                assert!(get_index(&key, size) < size);
            }
        }
    }

    #[test]
    fn get_index_deterministic() {
        assert_eq!(get_index(&42, 10), get_index(&42, 10));
    }
}
