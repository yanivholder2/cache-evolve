use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Saturation ceiling for the GCLOCK frequency counter (fits in 2 bits).
const CLOCK_MAX: u32 = 3;

/// Bits per packed GCLOCK counter.
const CTR_BITS: usize = 2;

/// Mask for a single packed counter.
const CTR_MASK: u64 = (1 << CTR_BITS) - 1;

/// Max slots whose counters fit in the packed `u64` word (64 / 2).
const PACKED_SLOTS: usize = 64 / CTR_BITS;

/// Number of recently evicted key fingerprints remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Small local FNV-1a hasher used only for ghost fingerprints (cheaper than
/// `DefaultHasher` on the miss path).
struct FpHasher(u64);

impl FpHasher {
	const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
	const PRIME: u64 = 0x0000_0100_0000_01b3;

	#[inline]
	fn new() -> Self {
		FpHasher(Self::OFFSET)
	}
}

impl Hasher for FpHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut h = self.0;
		for &b in bytes {
			h ^= b as u64;
			h = h.wrapping_mul(Self::PRIME);
		}
		self.0 = h;
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0 ^ i).wrapping_mul(Self::PRIME);
	}

	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}
}

/// 16-bit non-zero fingerprint of a key. Uses the TOP 16 bits of a mixed
/// 64-bit hash so it does not correlate with the low bits used for bucket
/// indexing; 0 is reserved for "empty slot".
#[inline]
fn ghost_fp<K: Hash>(key: &K) -> u16 {
	let mut h = FpHasher::new();
	key.hash(&mut h);
	let mut x = h.finish();
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	let fp = (x >> 48) as u16;
	if fp == 0 {
		1
	} else {
		fp
	}
}

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs as a GCLOCK ring instead: a hit saturating-increments an in-place
/// frequency counter (range 0..=CLOCK_MAX) and a miss on a full bucket sweeps a
/// per-bucket hand, decrementing non-zero counters and replacing the first
/// zero-count slot in place. After the replacement the hand is left resting ON
/// the newcomer (LIP-style filter): unless the newcomer is hit before the
/// bucket's next miss, it is the next victim, evicted in O(1).
///
/// The GCLOCK counters are NOT stored in `Item`: slot `i`'s 2-bit counter lives
/// in bits `2i..2i+2` of the per-bucket `clock_bits` word (buckets of up to 32
/// slots); larger buckets fall back to a per-slot `Vec<u8>` chosen once in
/// `new`. Counters are kept positionally in lockstep with `items`.
///
/// A small per-bucket ghost ring remembers 16-bit fingerprints of the last
/// `GHOST_LEN` evicted keys (0 = empty). A newcomer whose fingerprint is
/// found in the ring is admitted with counter 1 (instead of 0), surviving
/// exactly one extra sweep pass; the ghost entry is consumed.
///
/// Every other policy keeps the baseline behaviour, with its per-item state
/// (LFU counter / logical stamp) held in the parallel `meta` vector.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	/// Per-item policy metadata for non-clock policies (empty in clock mode).
	meta: Vec<u32>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	clock_mode: bool,
	/// Clock mode with more than `PACKED_SLOTS` slots: counters in `clock_wide`.
	wide: bool,
	hand: usize,
	/// Packed 2-bit GCLOCK counters, slot `i` at bits `2i..2i+2`.
	clock_bits: u64,
	/// Fallback per-slot counters (length `max_size`) when `wide`.
	clock_wide: Vec<u8>,
	ghosts: [u16; GHOST_LEN],
	ghost_pos: u8,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let clock_mode = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let wide = clock_mode && settings.bucket_size > PACKED_SLOTS;
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			meta: if clock_mode {
				Vec::new()
			} else {
				Vec::with_capacity(settings.bucket_size)
			},
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			clock_mode,
			wide,
			hand: 0,
			clock_bits: 0,
			clock_wide: if wide { vec![0u8; settings.bucket_size] } else { Vec::new() },
			ghosts: [0; GHOST_LEN],
			ghost_pos: 0,
		}
	}

	/// GCLOCK counter of slot `i`.
	#[inline]
	fn ctr(&self, i: usize) -> u32 {
		if self.wide {
			self.clock_wide[i] as u32
		} else {
			((self.clock_bits >> (CTR_BITS * i)) & CTR_MASK) as u32
		}
	}

	/// Set the GCLOCK counter of slot `i` to `c` (0..=CLOCK_MAX).
	#[inline]
	fn set_ctr(&mut self, i: usize, c: u32) {
		debug_assert!(c <= CLOCK_MAX);
		if self.wide {
			self.clock_wide[i] = c as u8;
		} else {
			let s = CTR_BITS * i;
			self.clock_bits = (self.clock_bits & !(CTR_MASK << s)) | ((c as u64) << s);
		}
	}

	/// Remove slot `idx`'s counter, shifting later counters down one slot to
	/// mirror `Vec::remove` on `items`.
	#[inline]
	fn ctr_remove(&mut self, idx: usize) {
		if self.wide {
			self.clock_wide.remove(idx);
			self.clock_wide.push(0);
		} else {
			let s = CTR_BITS * idx;
			let low = if s == 0 { 0 } else { self.clock_bits & ((1u64 << s) - 1) };
			let high = if s + CTR_BITS >= 64 {
				0
			} else {
				(self.clock_bits >> (s + CTR_BITS)) << s
			};
			self.clock_bits = low | high;
		}
	}

	/// GCLOCK sweep: advance the hand, decrementing non-zero counters, until it
	/// rests on a zero-count slot. Each full pass lowers every counter by 1, so
	/// this terminates within `CLOCK_MAX * len + 1` steps. Precondition:
	/// `items` is non-empty.
	#[inline]
	fn clock_find_victim(&mut self) -> usize {
		let len = self.items.len();
		debug_assert!(len > 0, "clock_find_victim precondition: non-empty");
		if self.hand >= len {
			self.hand = 0;
		}
		loop {
			let c = self.ctr(self.hand);
			if c > 0 {
				self.set_ctr(self.hand, c - 1);
				self.hand += 1;
				if self.hand == len {
					self.hand = 0;
				}
			} else {
				return self.hand;
			}
		}
	}

	/// Record an evicted key's fingerprint in the ghost ring, overwriting the
	/// oldest entry.
	#[inline]
	fn record_ghost(&mut self, key: K) {
		self.ghosts[self.ghost_pos as usize] = ghost_fp(&key);
		self.ghost_pos = ((self.ghost_pos as usize + 1) % GHOST_LEN) as u8;
	}

	/// Admission counter for a newcomer: 1 if its fingerprint is in the ghost
	/// ring (the ghost entry is consumed), 0 otherwise.
	#[inline]
	fn admission_counter(&mut self, key: &K) -> u32 {
		let fp = ghost_fp(key);
		for g in self.ghosts.iter_mut() {
			if *g == fp {
				*g = 0;
				return 1;
			}
		}
		0
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_meta(&self) -> &Vec<u32> {
		&self.meta
	}

	fn get_meta_mut(&mut self) -> &mut Vec<u32> {
		&mut self.meta
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock_mode {
			let c = self.ctr(position);
			if c < CLOCK_MAX {
				self.set_ctr(position, c + 1);
			}
			return &self.items[position];
		}
		let index = self
			.eviction_policy
			.get_from_position(&mut self.items, &mut self.meta, position);
		&self.items[index]
	}

	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if self.clock_mode && self.is_full() && !self.items.is_empty() {
			// Ghost check happens before the eviction so the newcomer cannot
			// match the key it is about to displace.
			let admission = self.admission_counter(&item.key);
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.set_ctr(idx, admission);
			self.record_ghost(evicted.key);
			// LIP-style: leave the hand on the newcomer so it is examined first
			// on the next miss; it survives only if hit before then (or if it
			// was ghost-admitted with counter 1, for exactly one extra pass).
			self.hand = idx;
			debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
			return (&self.items[idx], Some(evicted));
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if self.clock_mode {
			let admission = self.admission_counter(&item.key);
			let slot = self.items.len();
			self.set_ctr(slot, admission);
		} else {
			let m = self.eviction_policy.initial_meta();
			self.meta.push(m);
		}
		self.items.push(item);
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	fn evict_item(&mut self) -> Option<Item<K, V>> {
		if self.items.is_empty() {
			return None;
		}
		if self.clock_mode {
			let idx = self.clock_find_victim();
			let evicted = self.remove_at(idx);
			self.hand %= self.items.len().max(1);
			self.record_ghost(evicted.key);
			debug!("Bucket.evict_item: policy=GCLOCK evicted_key={:?}", evicted.key);
			return Some(evicted);
		}
		let idx = self.eviction_candidate_index()?;
		let evicted = self.remove_at(idx);
		debug!(
			"Bucket.evict_item: policy={} evicted_key={:?}",
			self.get_eviction_policy().name(),
			evicted.key
		);
		Some(evicted)
	}

	fn remove_at(&mut self, index: usize) -> Item<K, V> {
		debug_assert!(
			index < self.items.len(),
			"remove_at: index {index} out of bounds (len={})",
			self.items.len()
		);
		if self.clock_mode {
			self.ctr_remove(index);
		} else {
			self.meta.remove(index);
		}
		self.items.remove(index)
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn settings(bucket_size: usize) -> AssociativeCacheSettings {
		AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		}
	}

	#[test]
	fn packed_counter_remove_shifts() {
		let mut b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::new(&settings(16));
		for i in 0..4 {
			b.set_ctr(i, (i as u32) % 4);
		}
		b.ctr_remove(1);
		assert_eq!(b.ctr(0), 0);
		assert_eq!(b.ctr(1), 2);
		assert_eq!(b.ctr(2), 3);
		b.ctr_remove(0);
		assert_eq!(b.ctr(0), 2);
		assert_eq!(b.ctr(1), 3);
	}

	#[test]
	fn packed_counter_top_slot() {
		let mut b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::new(&settings(32));
		b.set_ctr(31, 3);
		b.set_ctr(30, 1);
		assert_eq!(b.ctr(31), 3);
		b.ctr_remove(31);
		assert_eq!(b.ctr(30), 1);
		b.ctr_remove(30);
		assert_eq!(b.clock_bits, 0);
	}

	#[test]
	fn wide_bucket_works() {
		let mut b: AssociativeCacheBucket<i64, i64> = AssociativeCacheBucket::new(&settings(40));
		assert!(b.wide);
		for k in 0..200 {
			if b.get(&(k % 60)).is_none() {
				b.put(Item::new(k % 60, k));
			}
			assert!(b.size() <= 40);
		}
	}
}
