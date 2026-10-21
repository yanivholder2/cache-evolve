use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Saturation ceiling for the GCLOCK frequency counter.
const CLOCK_MAX: usize = 3;

/// Number of recently evicted keys remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Saturation ceiling for the per-bucket adaptive grace score.
const GRACE_MAX: u8 = 15;

/// Grace score at or above which the bucket switches to the lenient filter
/// (hand parked one slot past the newcomer).
const GRACE_ON: u8 = 8;

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs as a GCLOCK ring instead: a hit saturating-increments an in-place
/// frequency counter (stored in `Item::lfu_counter`, range 0..=CLOCK_MAX) and a
/// miss on a full bucket sweeps a per-bucket hand, decrementing non-zero
/// counters and replacing the first zero-count slot in place.
///
/// Newcomer placement is adaptive per bucket. A saturating `grace` score
/// (0..=GRACE_MAX) rises on every ghost hit (a recently evicted key returned)
/// and falls whenever the previous newcomer is evicted without ever being
/// hit. While `grace < GRACE_ON` the hand is left resting ON the newcomer
/// (strict LIP-style one-miss filter, identical to the parent). Once
/// `grace >= GRACE_ON` the hand is parked one slot past the newcomer, so it is
/// examined last on the next sweep and gets roughly one full sweep of grace.
///
/// A small per-bucket ghost ring remembers the last `GHOST_LEN` evicted keys.
/// A newcomer whose key is found in the ring is admitted with counter 1
/// (instead of 0), surviving exactly one extra sweep pass; the ghost entry is
/// consumed. The ring is never consulted on `get`. Every other policy keeps
/// the baseline behaviour exactly.
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
	grace: u8,
	rest: Option<usize>,
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
			grace: 0,
			rest: None,
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
	/// (the ghost entry is consumed, and the bucket's grace score rises),
	/// 0 otherwise.
	#[inline]
	fn admission_counter(&mut self, key: &K) -> usize {
		for g in self.ghosts.iter_mut() {
			if g.as_ref() == Some(key) {
				*g = None;
				self.grace = (self.grace + 1).min(GRACE_MAX);
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
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		if self.clock_mode {
			// The resting newcomer was hit: it can no longer count as a
			// never-hit eviction.
			if self.rest == Some(position) {
				self.rest = None;
			}
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
			// Ghost check happens before the eviction so the newcomer cannot
			// match the key it is about to displace.
			item.lfu_counter = self.admission_counter(&item.key);
			let idx = self.clock_find_victim();
			// Negative signal: the previous newcomer is being evicted without
			// ever having been hit.
			if Some(idx) == self.rest && self.items[idx].lfu_counter == 0 {
				self.grace = self.grace.saturating_sub(1);
			}
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.record_ghost(evicted.key);
			self.rest = Some(idx);
			if self.grace >= GRACE_ON {
				// Lenient: park the hand one past the newcomer, so it is
				// examined last on the next sweep.
				self.hand = (idx + 1) % self.items.len();
			} else {
				// Strict LIP-style: leave the hand on the newcomer so it is
				// examined first on the next miss.
				self.hand = idx;
			}
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
			// Indices shift after removal; the resting slot is no longer valid.
			self.rest = None;
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
