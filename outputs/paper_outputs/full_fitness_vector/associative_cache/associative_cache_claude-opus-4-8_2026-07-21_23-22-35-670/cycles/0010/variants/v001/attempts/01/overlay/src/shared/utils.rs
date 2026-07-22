use std::fmt::Debug;
use std::hash::{Hash, Hasher};

use log::trace;

/// Fibonacci / golden-ratio odd multiplier used by both the streaming fold and
/// the final avalanche. Chosen for good bit diffusion under wrapping multiply.
const MIX: u64 = 0x9E37_79B9_7F4A_7C15;

/// Allocation-free, keyless hasher that replaces the SipHash `DefaultHasher`
/// on the cache hot path. `get_index` runs on *every* `put` and
/// `get_and_update_item`, and SipHash's keyed-permutation setup was tens of
/// nanoseconds of fixed per-op cost independent of any bucket work.
///
/// This hasher folds each input with a wrapping-multiply/xorshift mixer
/// (FxHash-style) and finishes with a Fibonacci multiply + xorshift avalanche.
/// It carries no per-instance key material, so construction is free and the
/// whole hash of an 8-byte key is a handful of integer ops (~1ns) rather than
/// a SipHash permutation. It is deterministic (`hash(k) == hash(k)`), total
/// over the key space, and diffuses well enough to keep bucket distribution
/// approximately uniform.
#[derive(Default)]
struct FastHasher {
	state: u64,
}

impl Hasher for FastHasher {
	fn finish(&self) -> u64 {
		// Final avalanche: two rounds of Fibonacci multiply + xorshift so that
		// low-entropy integer keys still spread across all output bits before
		// the caller takes `% size`.
		let mut h = self.state;
		h = h.wrapping_mul(MIX);
		h ^= h >> 29;
		h = h.wrapping_mul(MIX);
		h ^= h >> 32;
		h
	}

	fn write(&mut self, bytes: &[u8]) {
		let mut s = self.state;
		for &b in bytes {
			s = (s.rotate_left(5) ^ (b as u64)).wrapping_mul(MIX);
		}
		self.state = s;
	}

	// Fast paths for the integer key types the eval workload actually uses
	// (i64/usize keys). These skip the byte loop entirely: one xor + one
	// wrapping multiply folds the whole word into the running state.
	fn write_u64(&mut self, i: u64) {
		self.state = (self.state ^ i).wrapping_mul(MIX);
	}

	fn write_u32(&mut self, i: u32) {
		self.write_u64(i as u64);
	}

	fn write_usize(&mut self, i: usize) {
		self.write_u64(i as u64);
	}

	fn write_i32(&mut self, i: i32) {
		self.write_u64(i as u64);
	}

	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}

	fn write_isize(&mut self, i: isize) {
		self.write_u64(i as u64);
	}
}

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
	let mut hasher = FastHasher::default();
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
