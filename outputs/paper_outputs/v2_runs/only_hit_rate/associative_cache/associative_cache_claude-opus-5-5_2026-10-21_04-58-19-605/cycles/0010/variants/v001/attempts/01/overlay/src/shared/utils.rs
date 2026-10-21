use std::collections::hash_map::DefaultHasher;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

/// splitmix64 / murmur3-style 64-bit finaliser: full avalanche so that every
/// output bit depends on every input bit.
#[inline]
pub fn mix64(mut h: u64) -> u64 {
	h ^= h >> 30;
	h = h.wrapping_mul(0xbf58_476d_1ce4_e5b9);
	h ^= h >> 27;
	h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
	h ^= h >> 31;
	h
}

/// Single home bucket of `key` among `size` buckets. Pure and deterministic:
/// the existing `DefaultHasher` hash, passed through a 64-bit finaliser, then
/// reduced with multiply-shift on the HIGH 32 bits (works for any `size`).
/// Every get/put path and the invariant checker route through this function.
#[inline]
pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	if size <= 1 {
		return 0;
	}
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	let h = mix64(hasher.finish());
	let idx = (((h >> 32) * size as u64) >> 32) as usize;
	trace!("get_index: key={key:?} size={size} result={idx}");
	idx
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
	fn get_index_spreads_sequential_keys() {
		let n = 128usize;
		let mut counts = vec![0usize; n];
		for key in 0i64..(n as i64 * 64) {
			counts[get_index(&key, n)] += 1;
		}
		assert!(counts.iter().all(|&c| c > 0));
	}
}
