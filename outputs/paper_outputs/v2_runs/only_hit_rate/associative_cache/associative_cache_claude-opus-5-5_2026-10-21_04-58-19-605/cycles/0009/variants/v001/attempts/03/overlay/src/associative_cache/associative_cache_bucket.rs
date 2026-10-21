use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Saturation ceiling for the GCLOCK frequency counter.
const CLOCK_MAX: usize = 3;

/// Number of recently evicted key fingerprints remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Small local FNV-1a hasher used for ghost fingerprints and the two-choice
/// alternate bucket index (cheaper than `DefaultHasher` on the miss path).
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

/// Full 64-bit mixed hash of a key: FNV-1a followed by a murmur-style
/// finalizer step. The top 16 bits feed the ghost fingerprint; bits 16..32
/// feed the two-choice alternate bucket index.
#[inline]
pub(crate) fn mixed_hash<K: Hash>(key: &K) -> u64 {
	let mut h = FpHasher::new();
	key.hash(&mut h);
	let mut x = h.finish();
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x
}

/// 16-bit non-zero fingerprint from a precomputed mixed hash (TOP 16 bits);
/// 0 is reserved for "empty slot".
#[inline]
pub(crate) fn fp_from_mixed(x: u64) -> u16 {
	let fp = (x >> 48) as u16;
	if fp == 0 {
		1
	} else {
		fp
	}
}

/// 16-bit non-zero fingerprint of a key.
#[inline]
pub(crate) fn ghost_fp<K: Hash>(key: &K) -> u16 {
	fp_from_mixed(mixed_hash(key))
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
/// In ClassicLRU mode the owning cache may use two-choice placement and drive
/// the bucket through `victim_score`, `take_ghost` and `put_with_counter`.
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
	hand: usize,
	ghosts: [u16; GHOST_LEN],
	ghost_pos: u8,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			clock_mode: settings.eviction_policy == EvictionPolicyKind::ClassicLRU,
			hand: 0,
			ghosts: [0; GHOST_LEN],
			ghost_pos: 0,
		}
	}

	/// Whether this bucket runs in GCLOCK (ClassicLRU) mode.
	#[inline]
	pub(crate) fn is_clock_mode(&self) -> bool {
		self.clock_mode
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
		self.ghosts[self.ghost_pos as usize] = ghost_fp(&key);
		self.ghost_pos = ((self.ghost_pos as usize + 1) % GHOST_LEN) as u8;
	}

	/// Consume a matching fingerprint from the ghost ring. Returns whether a
	/// match was found.
	#[inline]
	pub(crate) fn take_ghost(&mut self, fp: u16) -> bool {
		for g in self.ghosts.iter_mut() {
			if *g == fp {
				*g = 0;
				return true;
			}
		}
		false
	}

	/// Admission counter for a newcomer: 1 if its fingerprint is in the ghost
	/// ring (the ghost entry is consumed), 0 otherwise.
	#[inline]
	fn admission_counter(&mut self, key: &K) -> usize {
		if self.take_ghost(ghost_fp(key)) {
			1
		} else {
			0
		}
	}

	/// Read-only coldness score of this bucket's next victim: 0 if the item
	/// under the (wrapped) hand is an unproven newcomer (counter 0), otherwise
	/// `1 + sum of all counters`. Lower is colder.
	#[inline]
	pub(crate) fn victim_score(&self) -> usize {
		let len = self.items.len();
		if len == 0 {
			return 0;
		}
		let h = if self.hand >= len { 0 } else { self.hand };
		if self.items[h].lfu_counter == 0 {
			return 0;
		}
		1 + self.items.iter().map(|i| i.lfu_counter).sum::<usize>()
	}

	/// Insert a non-resident item with a precomputed admission counter
	/// (two-choice mode; the cache already checked both candidates' ghost
	/// rings). Returns the evicted item, if any. The evicted key is recorded
	/// in this bucket's ghost ring.
	pub(crate) fn put_with_counter(&mut self, mut item: Item<K, V>, counter: usize) -> Option<Item<K, V>> {
		debug_assert!(self.clock_mode, "put_with_counter is GCLOCK-only");
		item.lfu_counter = counter;
		if self.is_full() && !self.items.is_empty() {
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.record_ghost(evicted.key);
			self.hand = idx;
			debug!("Bucket.put_with_counter: policy=GCLOCK evicted_key={:?}", evicted.key);
			return Some(evicted);
		}
		if self.is_full() {
			// max_size == 0: nothing can be stored.
			return None;
		}
		self.items.push(item);
		None
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
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock_mode {
			let c = self.items[position].lfu_counter;
			if c < CLOCK_MAX {
				self.items[position].lfu_counter = c + 1;
			}
			return &self.items[position];
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	fn put(&mut self, mut item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}

		if self.clock_mode && self.is_full() && !self.items.is_empty() {
			item.lfu_counter = self.admission_counter(&item.key);
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.record_ghost(evicted.key);
			self.hand = idx;
			debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
			return (&self.items[idx], Some(evicted));
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if self.clock_mode {
			item.lfu_counter = self.admission_counter(&item.key);
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
