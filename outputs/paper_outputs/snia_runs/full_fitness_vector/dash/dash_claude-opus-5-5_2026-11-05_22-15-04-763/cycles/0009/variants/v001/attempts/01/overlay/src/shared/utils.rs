use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

/// murmur3 `fmix64` finalizer: a cheap, full-avalanche 64-bit mixer (cycle 8
/// H03). Replaces SipHash (`DefaultHasher`) on every key-hash call site.
#[inline(always)]
pub fn fast_hash(k: u64) -> u64 {
	let mut x = k;
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	x ^= x >> 33;
	x
}

/// Minimal `Hasher` whose `finish` applies [`fast_hash`]. Integer keys go
/// through `write_u64` (the default `write_i64` etc. forward to it) and cost a
/// single store plus the finalizer. Arbitrary byte input is folded 8 bytes at a
/// time so generic `K: Hash` still works.
#[derive(Default, Clone, Copy)]
pub struct FastHasher {
	state: u64,
}

impl Hasher for FastHasher {
	#[inline(always)]
	fn finish(&self) -> u64 {
		fast_hash(self.state)
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut w = [0u8; 8];
			w.copy_from_slice(c);
			self.state = fast_hash(self.state ^ u64::from_le_bytes(w)).wrapping_add(0x9E37_79B9_7F4A_7C15);
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut w = [0u8; 8];
			w[..rem.len()].copy_from_slice(rem);
			self.state = fast_hash(self.state ^ u64::from_le_bytes(w) ^ ((rem.len() as u64) << 56))
				.wrapping_add(0x9E37_79B9_7F4A_7C15);
		}
	}

	#[inline(always)]
	fn write_u64(&mut self, i: u64) {
		// Single-integer keys (the measured configuration): store directly so
		// `finish` is exactly `fast_hash(key)`. Multi-field keys chain through
		// the mixer so earlier fields still influence the result.
		self.state = if self.state == 0 {
			i
		} else {
			fast_hash(self.state ^ i).wrapping_add(0x9E37_79B9_7F4A_7C15)
		};
	}

	#[inline(always)]
	fn write_u32(&mut self, i: u32) {
		self.write_u64(i as u64);
	}

	#[inline(always)]
	fn write_usize(&mut self, i: usize) {
		self.write_u64(i as u64);
	}
}

pub type FastBuildHasher = BuildHasherDefault<FastHasher>;

/// Hash of `key` under [`FastHasher`]. Bucket indexes come from the high bits
/// (multiply-shift, see [`index_from_hash`]); the segment index comes from the
/// low 32 bits; the stash presence bit from the low 6 bits.
#[inline(always)]
pub fn hash_key<K>(key: &K) -> u64
where
	K: Hash + ?Sized,
{
	let mut hasher = FastBuildHasher::default().build_hasher();
	key.hash(&mut hasher);
	hasher.finish()
}

/// Back-compatible `usize` hash (same value as [`hash_key`]).
#[inline(always)]
pub fn hash<K>(key: &K) -> usize
where
	K: Hash + Debug,
{
	hash_key(key) as usize
}

/// Multiply-shift reduction of a precomputed hash into `[0, size)`. Depends on
/// the high bits of `h`.
#[inline(always)]
pub fn index_from_hash(h: usize, size: usize) -> usize {
	(((h as u64 as u128) * (size as u128)) >> 64) as usize
}

#[inline(always)]
pub fn get_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	index_from_hash(hash(key), size)
}

/// Segment index for a precomputed hash among `size` segments, taken from the
/// hash's low 32 bits (multiply-shift within that half). Bucket and stash
/// indexes inside a segment depend on the high bits, so the two stay
/// independent. With one segment this is always 0.
#[inline(always)]
pub fn segment_index_from_hash(h: usize, size: usize) -> usize {
	(((h as u64 & 0xFFFF_FFFF) * (size as u64)) >> 32) as usize
}

/// Segment index for `key` among `size` segments (see [`segment_index_from_hash`]).
#[inline(always)]
pub fn segment_index<K>(key: &K, size: usize) -> usize
where
	K: Hash + Debug,
{
	segment_index_from_hash(hash(key), size)
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
		assert_ne!(hash(&"a"), hash(&"b"));
	}

	#[test]
	fn i64_and_u64_hash_match_fast_hash() {
		assert_eq!(hash_key(&12345_u64), fast_hash(12345));
		assert_eq!(hash_key(&12345_i64), fast_hash(12345));
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
	fn get_index_spreads_evenly() {
		let n = 24;
		let mut counts = vec![0usize; n];
		for key in 0..240_000_i64 {
			counts[get_index(&key, n)] += 1;
		}
		for &c in &counts {
			assert!((9_000..11_000).contains(&c), "uneven spread: {counts:?}");
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
