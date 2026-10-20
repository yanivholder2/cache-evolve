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

/// Alternate (second) bucket index for a key, derived from the upper hash bits.
///
/// Guaranteed to differ from `get_index(key, size)` whenever `size > 1`; when
/// `size == 1` both indices are necessarily 0.
pub fn alt_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	let h = hash(key) as u64;
	let primary = (h % size as u64) as usize;
	let alt = (((h >> 32) ^ h.rotate_left(29)) % size as u64) as usize;
	if alt == primary {
		(primary + 1) % size
	} else {
		alt
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

	#[test]
	fn alt_index_within_bounds_and_distinct() {
		for size in [1, 2, 4, 16, 100] {
			for key in 0..200 {
				let a = alt_index(&key, size);
				assert!(a < size);
				if size > 1 {
					assert_ne!(a, get_index(&key, size));
				}
			}
		}
	}
}
