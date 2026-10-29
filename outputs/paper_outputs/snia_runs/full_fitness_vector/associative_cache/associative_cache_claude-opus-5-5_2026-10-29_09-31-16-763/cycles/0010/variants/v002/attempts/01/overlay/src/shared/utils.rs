use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

use crate::associative_cache::associative_cache_bucket::tag_hash;

/// Key -> bucket index. Derived from the same single multiplicative hash the
/// bucket uses for its 1-byte tag (`tag_hash`), so the cache computes one hash
/// per op. This is the one source of truth for bucket placement: the cache's
/// get/put and the invariant suite's slot-legality check both go through
/// `bucket_index_from_hash`.
pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	bucket_index_from_hash(tag_hash(key), size)
}

/// Bucket index from a full `tag_hash` value. Uses bits 24..=55: disjoint from
/// the tag byte (bits 56..=63) and clear of the weak low bits of a
/// multiplicative hash. Power-of-two bucket counts take a mask; other counts
/// fall back to `%`.
#[inline]
pub fn bucket_index_from_hash(h: u64, size: usize) -> usize {
	let x = ((h >> 24) as u32) as usize;
	if size.is_power_of_two() {
		x & (size - 1)
	} else {
		x % size
	}
}

/// SipHash of a key. No longer used for bucket placement; kept for callers
/// outside the bucket-mapping path.
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
		for size in [1, 2, 3, 4, 16, 100, 128] {
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
	fn mask_matches_modulo_for_power_of_two() {
		for &h in &[0u64, 1, 0xFFFF_FFFF_FFFF_FFFF, 0x9E37_79B9_7F4A_7C15, 0x0123_4567_89AB_CDEF] {
			for &n in &[1usize, 2, 64, 128, 1024] {
				assert_eq!(bucket_index_from_hash(h, n), (((h >> 24) as u32) as usize) % n);
			}
		}
	}
}
