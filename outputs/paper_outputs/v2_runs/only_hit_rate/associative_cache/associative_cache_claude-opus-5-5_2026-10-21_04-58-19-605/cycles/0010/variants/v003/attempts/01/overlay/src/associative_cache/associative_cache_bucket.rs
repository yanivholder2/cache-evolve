use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Saturation ceiling for the GCLOCK frequency counter.
const CLOCK_MAX: u32 = 3;

/// Number of recently evicted key fingerprints remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Number of slots covered by the per-bucket tag array (two u64 SWAR words).
const TAG_SLOTS: usize = 16;

const LO_BYTES: u64 = 0x0101_0101_0101_0101;
const HI_BITS: u64 = 0x8080_8080_8080_8080;

/// Small local FNV-1a hasher used for the per-op key hash (cheaper than
/// `DefaultHasher`).
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

/// One mixed 64-bit hash per op. Feeds both the slot tag and the ghost
/// fingerprint.
#[inline]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut h = FpHasher::new();
	key.hash(&mut h);
	let mut x = h.finish();
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x
}

/// Non-zero 8-bit slot tag (0 marks an empty slot).
#[inline]
fn tag_of(x: u64) -> u8 {
	((x >> 40) as u8) | 1
}

/// 16-bit non-zero fingerprint from a precomputed key hash. Uses the TOP 16
/// bits so it does not correlate with the low bits used for bucket indexing;
/// 0 is reserved for "empty slot".
#[inline]
fn ghost_fp(x: u64) -> u16 {
	let fp = (x >> 48) as u16;
	if fp == 0 {
		1
	} else {
		fp
	}
}

/// SWAR match mask: high bit of each byte set where `word`'s byte may equal
/// `pattern`'s byte. Never misses a true match; may report false positives
/// (resolved by a full key compare).
#[inline]
fn match_mask(word: u64, pattern: u64) -> u64 {
	let v = word ^ pattern;
	v.wrapping_sub(LO_BYTES) & !v & HI_BITS
}

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs as a GCLOCK ring instead: a hit saturating-increments an in-place
/// frequency counter (stored in `Item::lfu_counter`, range 0..=CLOCK_MAX) and a
/// miss on a full bucket sweeps a per-bucket hand, decrementing non-zero
/// counters and replacing the first zero-count slot in place. After the
/// replacement the hand is left resting ON the newcomer (LIP-style filter).
///
/// A small per-bucket ghost ring remembers 16-bit fingerprints of the last
/// `GHOST_LEN` evicted keys (0 = empty). A newcomer whose fingerprint is
/// found in the ring is admitted with counter 1 (instead of 0); the ghost
/// entry is consumed.
///
/// In clock mode with `max_size <= 16`, a per-slot 8-bit tag array (kept in
/// lockstep with `items`) filters slots by tag before full key compares. One
/// hash per op feeds both the tag probe and the ghost fingerprint.
#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	clock_mode: bool,
	use_tags: bool,
	hand: usize,
	tags: [u8; TAG_SLOTS],
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
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			clock_mode,
			use_tags: clock_mode && settings.bucket_size <= TAG_SLOTS,
			hand: 0,
			tags: [0; TAG_SLOTS],
			ghosts: [0; GHOST_LEN],
			ghost_pos: 0,
		}
	}

	#[inline]
	fn linear_position(&self, key: &K) -> Option<usize> {
		self.items.iter().position(|d| d.key == *key)
	}

	/// Tag-filtered lookup given a precomputed hash. Returns the same index as
	/// the linear scan.
	#[inline]
	fn tag_position(&self, key: &K, x: u64) -> Option<usize> {
		let len = self.items.len();
		let pattern = (tag_of(x) as u64).wrapping_mul(LO_BYTES);
		let mut lo = [0u8; 8];
		let mut hi = [0u8; 8];
		lo.copy_from_slice(&self.tags[0..8]);
		hi.copy_from_slice(&self.tags[8..16]);
		let words = [u64::from_le_bytes(lo), u64::from_le_bytes(hi)];
		for (w, &word) in words.iter().enumerate() {
			let base = w * 8;
			if base >= len {
				break;
			}
			let mut m = match_mask(word, pattern);
			while m != 0 {
				let i = base + (m.trailing_zeros() as usize >> 3);
				if i >= len {
					break;
				}
				if self.items[i].key == *key {
					return Some(i);
				}
				m &= m - 1;
			}
		}
		None
	}

	/// Lookup that also returns the key's hash, so `put` can reuse it for the
	/// ghost check on a miss. The hash is never stored as bucket state.
	#[inline]
	fn get_position_hashed(&self, key: &K) -> (Option<usize>, u64) {
		let x = key_hash(key);
		let pos = if self.use_tags {
			self.tag_position(key, x)
		} else {
			self.linear_position(key)
		};
		debug_assert_eq!(pos, self.linear_position(key), "tag lookup diverged from linear scan");
		(pos, x)
	}

	/// GCLOCK sweep: advance the hand, decrementing non-zero counters, until it
	/// rests on a zero-count slot. Precondition: `items` is non-empty.
	#[inline]
	fn clock_find_victim(&mut self) -> usize {
		let len = self.items.len();
		debug_assert!(len > 0, "clock_find_victim precondition: non-empty");
		if self.hand >= len {
			self.hand = 0;
		}
		loop {
			let slot = &mut self.items[self.hand];
			if slot.lfu_counter > 0 {
				slot.lfu_counter -= 1;
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
		self.ghosts[self.ghost_pos as usize] = ghost_fp(key_hash(&key));
		self.ghost_pos = ((self.ghost_pos as usize + 1) % GHOST_LEN) as u8;
	}

	/// Admission counter for a newcomer given its precomputed hash: 1 if its
	/// fingerprint is in the ghost ring (the ghost entry is consumed), 0
	/// otherwise.
	#[inline]
	fn admission_counter(&mut self, x: u64) -> u32 {
		let fp = ghost_fp(x);
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

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	#[inline]
	fn get_position(&self, key: &K) -> Option<usize> {
		if self.use_tags {
			self.get_position_hashed(key).0
		} else {
			self.linear_position(key)
		}
	}

	#[inline]
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock_mode {
			let c: u32 = self.items[position].lfu_counter;
			if c < CLOCK_MAX {
				self.items[position].lfu_counter = c + 1;
			}
			return &self.items[position];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if !self.clock_mode {
			if let Some(position) = self.linear_position(&item.key) {
				trace!("Bucket.put: key={:?} update_existing", item.key);
				self.items[position].value = item.value;
				let pushed_item = self.get_from_position(position);
				return (pushed_item, None);
			}
			let evicted_item = if self.is_full() { self.evict_item() } else { None };
			self.items.push(item);
			let pushed_item = self.items.last().expect("bucket invariant: just pushed");
			return (pushed_item, evicted_item);
		}

		let (found, x) = self.get_position_hashed(&item.key);
		if let Some(position) = found {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if self.is_full() && !self.items.is_empty() {
			// Ghost check happens before the eviction so the newcomer cannot
			// match the key it is about to displace.
			item.lfu_counter = self.admission_counter(x);
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			if self.use_tags {
				self.tags[idx] = tag_of(x);
			}
			self.record_ghost(evicted.key);
			// LIP-style: leave the hand on the newcomer.
			self.hand = idx;
			debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
			return (&self.items[idx], Some(evicted));
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		item.lfu_counter = self.admission_counter(x);
		if self.use_tags {
			self.tags[self.items.len()] = tag_of(x);
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
		if self.use_tags {
			let len = self.items.len();
			self.tags.copy_within(index + 1..len, index);
			self.tags[len - 1] = 0;
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
mod tag_tests {
	use super::*;

	fn bucket(size: usize) -> AssociativeCacheBucket<i64, i64> {
		AssociativeCacheBucket::new(&AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		})
	}

	#[test]
	fn tag_lookup_matches_linear_scan() {
		for size in [1usize, 3, 8, 9, 16] {
			let mut b = bucket(size);
			for k in 0i64..500 {
				if b.get(&k).is_none() {
					b.put(Item::new(k, k));
				}
				for probe in (k - 40)..(k + 5) {
					assert_eq!(b.get_position(&probe), b.linear_position(&probe));
				}
				assert!(b.items.len() <= size);
			}
		}
	}

	#[test]
	fn evict_item_keeps_tags_in_sync() {
		let mut b = bucket(8);
		for k in 0i64..8 {
			b.put(Item::new(k, k));
		}
		while b.evict_item().is_some() {
			for probe in 0i64..8 {
				assert_eq!(b.get_position(&probe), b.linear_position(&probe));
			}
		}
		assert!(b.tags.iter().all(|&t| t == 0));
	}
}
