use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	// H01: bias-free multiplicative (Lemire) reduction. Using the *high* bits of
	// the avalanche-mixed hash avoids the low-bit modulo bias that `hash % size`
	// suffers when `size` is not a power of two. For any `x < 2^64`,
	// `(x * size) >> 64 < size`, so the result is always a valid bucket index.
	let x = hash(key) as u64;
	((x as u128 * size as u128) >> 64) as usize
}

pub fn hash<K>(key: &K) -> usize
where
	K: Hash + Debug,
{
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	// H01: splitmix64-style finalizer. `DefaultHasher` (SipHash) output can
	// correlate in its low bits for the sequential/structured integer keys this
	// trace uses; this avalanche step makes adjacent keys scatter across buckets.
	// It is a bijection on u64, so distinct keys keep distinct hashes.
	let mut x = hasher.finish();
	x ^= x >> 30;
	x = x.wrapping_mul(0xbf58476d1ce4e5b9);
	x ^= x >> 27;
	x = x.wrapping_mul(0x94d049bb133111eb);
	x ^= x >> 31;
	let hash = x as usize;
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
