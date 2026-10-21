use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::{debug, trace};

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;
use std::sync::atomic::{AtomicI32, AtomicUsize, Ordering};

/// Saturation ceiling for the GCLOCK frequency counter.
const CLOCK_MAX: usize = 3;

/// Number of recently evicted keys remembered per bucket (ghost ring).
const GHOST_LEN: usize = 4;

/// Set-dueling roles (DIP-style). 1/16 of buckets are fixed leaders.
const ROLE_FOLLOWER: u8 = 0;
const ROLE_STRICT_LEADER: u8 = 1;
const ROLE_LENIENT_LEADER: u8 = 2;

/// Leader-assignment period: sequence value 0 → strict leader, 1 → lenient
/// leader, everything else → follower.
const LEADER_PERIOD: usize = 32;

/// Saturation bound for the global policy selector.
const PSEL_MAX: i32 = 1023;

// NOTE: these statics are process-global, i.e. shared across every cache
// instance in the process. That is acceptable for the single-cache simulator
// run; with several caches the leader assignment and selector would simply be
// pooled across them. Only ClassicLRU (clock_mode) buckets ever touch them.

/// Sequence used to assign leader roles to ClassicLRU buckets at construction.
static BUCKET_SEQ: AtomicUsize = AtomicUsize::new(0);

/// Global policy selector. Strict-leader misses increment it, lenient-leader
/// misses decrement it; followers go lenient when it is positive (i.e. strict
/// leaders are missing more).
static PSEL: AtomicI32 = AtomicI32::new(0);

#[inline]
fn psel_add(delta: i32) {
	let _ = PSEL.fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
		let n = (v + delta).clamp(-PSEL_MAX, PSEL_MAX);
		if n == v {
			None
		} else {
			Some(n)
		}
	});
}

/// Bucket container. When the configured policy is `ClassicLRU`, the bucket
/// runs as a GCLOCK ring instead: a hit saturating-increments an in-place
/// frequency counter (stored in `Item::lfu_counter`, range 0..=CLOCK_MAX) and a
/// miss on a full bucket sweeps a per-bucket hand, decrementing non-zero
/// counters and replacing the first zero-count slot in place.
///
/// Placement of the hand after replacement is chosen by DIP-style set dueling:
/// - strict (LIP-style): the hand rests ON the newcomer, so it is the next
///   victim unless hit before the bucket's next miss;
/// - lenient: the hand moves past the newcomer, so it is examined last in the
///   next sweep (one-sweep grace).
///
/// Strict leaders always use strict, lenient leaders always use lenient; each
/// leader's insertion misses vote into a global saturating selector (PSEL),
/// and followers adopt lenient only while PSEL > 0.
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
	role: u8,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let clock_mode = settings.eviction_policy == EvictionPolicyKind::ClassicLRU;
		let role = if clock_mode {
			match BUCKET_SEQ.fetch_add(1, Ordering::Relaxed) % LEADER_PERIOD {
				0 => ROLE_STRICT_LEADER,
				1 => ROLE_LENIENT_LEADER,
				_ => ROLE_FOLLOWER,
			}
		} else {
			ROLE_FOLLOWER
		};
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			clock_mode,
			hand: 0,
			ghosts: [None; GHOST_LEN],
			ghost_pos: 0,
			role,
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

	/// Leader vote on an insertion miss (ClassicLRU only). Followers abstain.
	#[inline]
	fn vote_miss(&self) {
		match self.role {
			ROLE_STRICT_LEADER => psel_add(1),
			ROLE_LENIENT_LEADER => psel_add(-1),
			_ => {}
		}
	}

	/// Whether this bucket uses the lenient (one-sweep grace) placement.
	#[inline]
	fn use_lenient(&self) -> bool {
		match self.role {
			ROLE_LENIENT_LEADER => true,
			ROLE_STRICT_LEADER => false,
			_ => PSEL.load(Ordering::Relaxed) > 0,
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
			self.vote_miss();
			// Ghost check happens before the eviction so the newcomer cannot
			// match the key it is about to displace.
			item.lfu_counter = self.admission_counter(&item.key);
			let idx = self.clock_find_victim();
			let evicted = std::mem::replace(&mut self.items[idx], item);
			self.record_ghost(evicted.key);
			if self.use_lenient() {
				// Lenient: move past the newcomer so it is examined last in
				// the next sweep (one-sweep grace).
				self.hand = (idx + 1) % self.items.len();
			} else {
				// Strict (LIP-style): leave the hand on the newcomer so it is
				// examined first on the next miss.
				self.hand = idx;
			}
			debug!("Bucket.put: policy=GCLOCK evicted_key={:?}", evicted.key);
			return (&self.items[idx], Some(evicted));
		}

		if self.clock_mode {
			self.vote_miss();
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
