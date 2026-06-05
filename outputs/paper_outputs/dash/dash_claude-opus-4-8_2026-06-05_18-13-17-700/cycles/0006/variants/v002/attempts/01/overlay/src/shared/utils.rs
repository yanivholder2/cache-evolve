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

/// Salt mixed into the secondary stash hash (golden-ratio constant), so the
/// alternate derivation does not simply reproduce `get_index`.
const STASH_ALT_SALT: u64 = 0x9E37_79B9;

/// Hash `key` with [`STASH_ALT_SALT`] mixed in, yielding an index in `[0, size)`.
/// A second, independent derivation from `get_index`.
pub fn get_index_with_salt<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	STASH_ALT_SALT.hash(&mut hasher);
	(hasher.finish() as usize) % size
}

/// Secondary ("alternate") stash bucket index for `key`.
///
/// This is the cuckoo-style second home a stash key may legally occupy. It is a
/// distinct derivation from [`get_index`] and is guaranteed different from the
/// primary index whenever `size >= 2`. For `size < 2` there is no alternate
/// bucket, so it returns the primary index — callers then fall back to
/// single-bucket behaviour exactly as the baseline does.
pub fn stash_alt_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	let primary = get_index(key, size);
	if size < 2 {
		return primary;
	}
	let alt = get_index_with_salt(key, size);
	if alt != primary {
		alt
	} else {
		(primary + 1) % size
	}
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
	fn stash_alt_index_distinct_when_size_ge_2() {
		for size in [2usize, 4, 8, 24] {
			for key in 0i64..200 {
				let alt = stash_alt_index(&key, size);
				assert!(alt < size);
				assert_ne!(alt, get_index(&key, size), "key={key} size={size}");
			}
		}
	}

	#[test]
	fn stash_alt_index_falls_back_for_single_bucket() {
		for key in 0i64..50 {
			assert_eq!(stash_alt_index(&key, 1), get_index(&key, 1));
		}
	}

	#[test]
	fn stash_alt_index_deterministic() {
		assert_eq!(stash_alt_index(&42, 8), stash_alt_index(&42, 8));
	}
}
