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

/// Number of recently evicted keys remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Number of counters in the per-bucket miss-frequency sketch.
const SKETCH_LEN: usize = 32;

/// Saturation ceiling for a sketch counter.
const SKETCH_MAX: u8 = 15;

/// Minimal FNV-1a 64-bit hasher, local to the sketch (no external crates).
struct Fnv1a(u64);

impl Fnv1a {
	#[inline]
	fn new() -> Self {
		Fnv1a(0xcbf2_9ce4_8422_2325)
	}
}

impl Hasher for Fnv1a {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut h = self.0;
		for &b in bytes {
			h ^= b as u64;
			h = h.wrapping_mul(0x0000_0100_0000_01b3);
		}
		self.0 = h;
	}
}

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs as a GCLOCK ring instead: a hit saturating-increments an in-place
/// frequency counter (stored in `Item::lfu_counter`, range 0..=CLOCK_MAX) and a
/// miss on a full bucket sweeps a per-bucket hand, decrementing non-zero
/// counters and replacing the first zero-count slot in place. After the
/// replacement the hand is left resting ON the newcomer (LIP-style filter):
/// unless the newcomer is hit before the bucket's next miss, it is the next
/// victim, evicted in O(1). No `Vec` shifting happens on either path.
///
/// A small per-bucket ghost ring remembers the last `GHOST_LEN` evicted keys.
/// A newcomer whose key is found in the ring is admitted with counter 1
/// (instead of 0), surviving exactly one extra sweep pass; the ghost entry is
/// consumed. The ring is never consulted on `get`.
///
/// In addition, a 32-counter count-min-style sketch (two indices per key)
/// counts clock-mode misses per key. A newcomer whose pre-increment estimate
/// is >= 2 (it has missed at least twice within the ageing window) is also
/// admitted with counter 1. The sketch is halved every `8 * max_size` misses.
/// Hits never touch it. Every other policy keeps the baseline behaviour.
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
	ghosts: [Option<K>; GHOST_LEN],
	ghost_pos: u8,
	sketch: [u8; SKETCH_LEN],
	sketch_misses: u16,
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
			ghosts: [None; GHOST_LEN],
			ghost_pos: 0,
			sketch: [0; SKETCH_LEN],
			sketch_misses: 0,
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

	/// Record an evicted key in the ghost ring, overwriting the oldest entry.
	#[inline]
	fn record_ghost(&mut self, key: K) {
		self.ghosts[self.ghost_pos as usize] = Some(key);
		self.ghost_pos = ((self.ghost_pos as usize + 1) % GHOST_LEN) as u8;
	}

	/// Admission counter for a newcomer: 1 if its key is in the ghost ring
	/// (the ghost entry is consumed), 0 otherwise.
	#[inline]
	fn admission_counter(&mut self, key: &K) -> usize {
		for g in self.ghosts.iter_mut() {
			if g.as_ref() == Some(key) {
				*g = None;
				return 1;
			}
		}
		0
	}

	/// Two sketch indices taken from high bit ranges of a mixed FNV-1a hash,
	/// so they do not reuse the low bits that select the bucket.
	#[inline]
	fn sketch_idx(key: &K) -> (usize, usize) {
		let mut h = Fnv1a::new();
		key.hash(&mut h);
		let mut x = h.finish();
		x ^= x >> 33;
		x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
		x ^= x >> 33;
		(((x >> 40) as usize) % SKETCH_LEN, ((x >> 52) as usize) % SKETCH_LEN)
	}

	/// Record a clock-mode miss for `key` in the sketch and return the
	/// pre-increment estimate `min(sketch[a], sketch[b])`. Ages the sketch by
	/// halving every counter once `8 * max_size` misses have been recorded.
	#[inline]
	fn sketch_record_miss(&mut self, key: &K) -> u8 {
		let (a, b) = Self::sketch_idx(key);
		let est = self.sketch[a].min(self.sketch[b]);
		if self.sketch[a] < SKETCH_MAX {
			self.sketch[a] += 1;
		}
		if b != a && self.sketch[b] < SKETCH_MAX {
			self.sketch[b] += 1;
		}
		self.sketch_misses = self.sketch_misses.saturating_add(1);
		let reset = (8 * self.max_size).clamp(1, u16::MAX as usize);
		if self.sketch_misses as usize >= reset {
			for c in self.sketch.iter_mut() {
				*c >>= 1;
			}
			self.sketch_misses = 0;
		}
		est
	}

	/// Admission decision for a clock-mode newcomer: the ghost ring is always
	/// consulted first (consuming a matching entry); counter 1 is granted if
	/// the ghost matched or the sketch shows >= 2 prior misses.
	#[inline]
	fn admit(&mut self, key: &K) -> usize {
		let est = self.sketch_record_miss(key);
		let ghost = self.admission_counter(key);
		if ghost == 1 || est >= 2 {
			1
		} else {
			0
		}
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
			// Sketch + ghost check happen before the eviction so the newcomer
			// cannot match the key it is about to displace.
			item.lfu_counter = self.admit(&item.key);
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.record_ghost(evicted.key);
			// LIP-style: leave the hand on the newcomer so it is examined first
			// on the next miss; it survives only if hit before then (or if it
			// was admitted with counter 1, for exactly one extra pass).
			self.hand = idx;
			debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
			return (&self.items[idx], Some(evicted));
		}

		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		if self.clock_mode {
			item.lfu_counter = self.admit(&item.key);
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
