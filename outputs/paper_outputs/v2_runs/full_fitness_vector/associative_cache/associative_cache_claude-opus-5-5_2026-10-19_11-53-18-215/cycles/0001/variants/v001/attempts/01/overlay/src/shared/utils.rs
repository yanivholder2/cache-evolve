use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	let hash = hash(key);
	hash % size
}

/// Returns the two candidate bucket indices for `key` (power-of-two-choices).
///
/// Hashes once. `i1 = h % n`, `i2 = ((h >> 32) ^ h.rotate_left(17)) % n`; if
/// they collide, `i2 = (i1 + 1) % n` so the candidates always differ when
/// `n > 1`. With `n == 1` returns `(0, 0)` (single-bucket behaviour).
pub fn get_two_indices<K>(key: &K, n: usize) -> (usize, usize)
where
	K: Hash + Debug,
{
	if n <= 1 {
		return (0, 0);
	}
	let h = hash(key);
	let i1 = h % n;
	let mut i2 = ((h >> 32) ^ h.rotate_left(17)) % n;
	if i2 == i1 {
		i2 = (i1 + 1) % n;
	}
	(i1, i2)
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

	#[test]
	fn get_two_indices_distinct_and_in_bounds() {
		for n in [2, 3, 4, 16, 128] {
			for key in 0..200 {
				let (a, b) = get_two_indices(&key, n);
				assert!(a < n && b < n);
				assert_ne!(a, b);
				assert_eq!(a, get_index(&key, n));
			}
		}
	}

	#[test]
	fn get_two_indices_single_bucket() {
		assert_eq!(get_two_indices(&7, 1), (0, 0));
	}
}
