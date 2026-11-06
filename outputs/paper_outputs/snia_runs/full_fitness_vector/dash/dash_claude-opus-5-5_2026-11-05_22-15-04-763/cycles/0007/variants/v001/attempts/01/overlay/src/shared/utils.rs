use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

/// Bucket index for `key` among `size` buckets. Equivalent to
/// `index_from_hash(hash_key(key), size)`; callers that need several indices
/// for the same key should hash once via `hash_key` and reduce each index
/// with `index_from_hash`.
#[inline]
pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	index_from_hash(hash_key(key), size)
}

/// The exact hash `get_index` reduces: `hash(key)` widened to `u64`.
#[inline]
pub fn hash_key<K>(key: &K) -> u64
where
	K: Hash + Debug,
{
	hash(key) as u64
}

/// The exact reduction `get_index` applies to a key's hash: `hash % size`.
#[inline]
pub fn index_from_hash(h: u64, size: usize) -> usize {
	(h as usize) % size
}

/// Segment index for `key` among `size` segments, taken from the hash's high
/// 32 bits. Bucket and stash indexes inside a segment use `hash % n` (the low
/// bits); if the segment index also used `hash % size`, then whenever the
/// segment count shares factors with the bucket count (e.g. 8 or 64 segments
/// of 8 normal buckets) every key in a segment would map to the same few
/// buckets, leaving most of the segment unreachable. With one segment this is
/// always 0, identical to `get_index(key, 1)`.
pub fn segment_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	((hash(key) as u64 >> 32) as usize) % size
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
	fn split_hash_matches_get_index() {
		for size in [1, 2, 3, 8, 24, 100] {
			for key in -500..500_i64 {
				assert_eq!(index_from_hash(hash_key(&key), size), hash(&key) % size);
				assert_eq!(get_index(&key, size), hash(&key) % size);
			}
		}
	}

	#[test]
	fn segment_index_single_segment_is_zero() {
		for key in 0..200_i64 {
			assert_eq!(segment_index(&key, 1), 0);
		}
	}

	#[test]
	fn segment_index_independent_of_bucket_index() {
		// Within one of 8 segments, keys must still spread over all 8 normal
		// buckets and all 24 stash buckets.
		for segments in [8, 64] {
			let mut buckets = std::collections::HashSet::new();
			let mut stash = std::collections::HashSet::new();
			for key in 0..100_000_i64 {
				if segment_index(&key, segments) == 0 {
					buckets.insert(get_index(&key, 8));
					stash.insert(get_index(&key, 24));
				}
			}
			assert_eq!(buckets.len(), 8, "segments={segments}");
			assert_eq!(stash.len(), 24, "segments={segments}");
		}
	}

	#[test]
	fn get_index_deterministic() {
		assert_eq!(get_index(&42, 10), get_index(&42, 10));
	}
}
