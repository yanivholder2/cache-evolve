use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

/// Maps a key to a bucket index in `0..size`.
///
/// The key is first reduced to a 64-bit hash (generic `K` cannot be multiplied
/// directly), then spread with a Fibonacci multiplicative hash: the multiply by
/// the golden-ratio constant is followed by taking the top `log2(size)` bits when
/// `size` is a power of two (a shift, not a modulo). For non-power-of-two sizes
/// the mixed value is reduced modulo `size` so the index stays in range.
pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	if size <= 1 {
		return 0;
	}
	let mixed = (hash(key) as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15);
	if size.is_power_of_two() {
		let bits = size.trailing_zeros();
		(mixed >> (64 - bits)) as usize
	} else {
		(mixed % size as u64) as usize
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
