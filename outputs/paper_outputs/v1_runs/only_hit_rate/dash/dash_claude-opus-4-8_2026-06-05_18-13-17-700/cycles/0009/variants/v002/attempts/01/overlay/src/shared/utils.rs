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

/// Golden-ratio mixing constant used to derive a second, independent stash bucket
/// index for a key (cuckoo-style relocation). Mixing this salt into the hash gives
/// a placement that is uncorrelated with the primary `get_index` placement.
pub const STASH_ALT_SALT: u64 = 0x9E37_79B9_7F4A_7C15;

/// Salted variant of [`get_index`]: folds `salt` into the hash before reducing
/// modulo `size`, yielding a bucket index independent of the unsalted one.
pub fn get_index_with_salt<K>(key: &K, size: usize, salt: u64) -> usize
where
	K: Hash + Debug,
{
	let mut hasher = DefaultHasher::new();
	salt.hash(&mut hasher);
	key.hash(&mut hasher);
	(hasher.finish() as usize) % size
}

/// Alternate ("second home") stash bucket index for `key`.
///
/// With at least two stash buckets the alternate is a salted, independent
/// placement (it may still collide with the primary for some keys; callers must
/// guard the `alt == primary` degenerate case themselves). With a single stash
/// bucket there is no second home, so the alternate collapses onto the primary.
pub fn stash_alt_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	if size < 2 {
		return get_index(key, size);
	}
	get_index_with_salt(key, size, STASH_ALT_SALT)
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
	fn stash_alt_index_within_bounds() {
		for size in [1, 2, 4, 16, 100] {
			for key in 0..200 {
				assert!(stash_alt_index(&key, size) < size);
			}
		}
	}

	#[test]
	fn stash_alt_index_single_bucket_collapses_to_primary() {
		for key in 0..200 {
			assert_eq!(stash_alt_index(&key, 1), get_index(&key, 1));
		}
	}

	#[test]
	fn stash_alt_index_deterministic() {
		assert_eq!(stash_alt_index(&42, 8), stash_alt_index(&42, 8));
	}
}
