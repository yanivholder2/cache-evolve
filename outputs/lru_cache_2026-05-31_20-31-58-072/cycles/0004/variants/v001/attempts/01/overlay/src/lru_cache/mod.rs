// CLOCK-Pro-lite cache + H01 scan-taint filter.
//
// Inherits the parent v001 (cycle 3) skeleton:
//   * Per-slot (state, referenced) pair with Cold/Hot states.
//   * `get` hit promotes Cold->Hot and sets the ref bit; Hot just sets ref.
//   * `put` of a brand-new key whose key is in the ghost ring inserts as Hot;
//     otherwise as Cold. Ghost ring is full `capacity` long.
//   * Two-pass eviction sweep: pass 1 prefers Cold ref-clear; pass 2 demotes
//     Hot ref-set then evicts the first Hot ref-clear; fallback evicts the
//     slot at hand (now Cold ref-clear).
//
// H01 additions:
//   * A `seen: Vec<bool>` first-seen sketch of length `4 * capacity` (>=4),
//     indexed by `hash(key) % seen.len()` using `DefaultHasher`. Every `get`
//     and every `put` marks the slot, regardless of resident/ghost outcome.
//   * A `scan_taint: bool` on each Entry, set only when admitting a brand-new
//     Cold key (not from the ghost ring, not when the cache is still below
//     capacity) whose fingerprint had not been seen prior to this access.
//     Resident updates and ghost-promoted Hot admissions leave the taint
//     untouched at insert.
//   * A new pre-pass in `evict_one` sweeps the ring once (bounded at
//     `capacity` steps) for Cold ref-clear slots whose `scan_taint` is true,
//     evicting the first one found as a Cold eviction (ghost-pushed). If
//     none is found, the existing pass 1 / pass 2 / fallback chain runs
//     unchanged.
//   * On a `get` hit, the taint clears at the same point the ref bit is set
//     so a touched scan insert is no longer treated as scan-only.
//
// The JNI ABI in `lru_cache_jni.rs` is unchanged.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SlotState {
	Cold,
	Hot,
}

#[derive(Debug)]
struct Entry<K, V> {
	key: K,
	value: V,
	state: SlotState,
	referenced: bool,
	// H01: true when this slot was admitted as a brand-new Cold key whose
	// fingerprint had never been seen before. Cleared on any subsequent `get`
	// hit; cleared implicitly when the slot is evicted.
	scan_taint: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,          // key -> slot index
	slots: Vec<Option<Entry<K, V>>>, // fixed-length ring (Option lets eviction leave a hole)
	hand: usize,                     // CLOCK hand, modulo capacity
	len: usize,                      // current resident count
	capacity: usize,
	ghost: Vec<Option<K>>,           // recent-cold-evictions ring
	ghost_head: usize,               // ghost ring write head
	ghost_set: HashSet<K>,           // O(1) membership mirror of `ghost`
	seen: Vec<bool>,                 // H01: first-seen sketch over fingerprints
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("LruCache.try_new: capacity must be > 0");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		let mut slots = Vec::with_capacity(capacity);
		for _ in 0..capacity {
			slots.push(None);
		}
		// Full-capacity ghost ring inherited from parent v001 (cycle 3).
		let ghost_len = capacity;
		let mut ghost = Vec::with_capacity(ghost_len);
		for _ in 0..ghost_len {
			ghost.push(None);
		}
		// H01 sketch: 4 * capacity bits, floored at 4 so the modulo is always
		// defined even at capacity=1. Sized to comfortably exceed the working
		// set; collisions only make the taint slightly more permissive (rarer
		// taints, not spurious ones).
		let seen_len = capacity.saturating_mul(4).max(4);
		let seen = vec![false; seen_len];
		Some(Self {
			map: HashMap::with_capacity(capacity),
			slots,
			hand: 0,
			len: 0,
			capacity,
			ghost,
			ghost_head: 0,
			ghost_set: HashSet::with_capacity(ghost_len),
			seen,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Hash `key` with `DefaultHasher` and fold to a sketch index.
	fn fingerprint(&self, key: &K) -> usize {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		(hasher.finish() as usize) % self.seen.len()
	}

	/// Returns the value for `key` if resident. A Cold hit is promoted to Hot
	/// (the second-touch promotion the parent cites as scan-resistant); the
	/// reference bit is set in either case. The first-seen sketch is updated
	/// regardless of hit/miss. A hit also clears `scan_taint` — a touched
	/// scan insert is no longer scan-only.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let fp = self.fingerprint(key);
		// Mark seen unconditionally: per H01, seen[fp] tracks whether we've
		// ever observed this fingerprint, independent of hit/miss/ghost.
		self.seen[fp] = true;
		let &idx = self.map.get(key)?;
		let entry = self
			.slots[idx]
			.as_mut()
			.expect("map index points to occupied slot");
		if entry.state == SlotState::Cold {
			entry.state = SlotState::Hot;
		}
		entry.referenced = true;
		entry.scan_taint = false;
		Some(&entry.value)
	}

	/// Inserts or updates `key`. Brand-new keys land in an empty slot if one
	/// exists, otherwise the CLOCK-Pro-lite sweep (with the H01 pre-pass)
	/// selects a victim.
	pub fn put(&mut self, key: K, value: V) {
		let fp = self.fingerprint(&key);
		// Capture seen[fp] *before* marking, so the taint decision below sees
		// the prior bit. Then mark the fingerprint as seen — this happens for
		// every put outcome (resident update, ghost promotion, brand-new).
		let prior_seen = self.seen[fp];
		self.seen[fp] = true;

		// Key already resident: update value in place, set the ref bit.
		// Taint is left untouched (H01: "resident updates ... must not be
		// tainted" — i.e. we do not newly mark them; we also do not clear
		// here, per the spec which limits clearing to `get` hits).
		if let Some(&idx) = self.map.get(&key) {
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.value = value;
			entry.referenced = true;
			return;
		}

		// Brand-new key. Ghost promotion takes precedence over scan-taint:
		// a key that was recently Cold-evicted and is now re-referenced is
		// treated as Hot and is explicitly excluded from tainting.
		let from_ghost = self.ghost_set.contains(&key);
		if from_ghost {
			self.ghost_remove(&key);
		}
		let initial_state = if from_ghost {
			SlotState::Hot
		} else {
			SlotState::Cold
		};

		if self.len < self.capacity {
			// Below capacity: per H01, do not taint. Scan forward from `hand`
			// for the first empty slot (terminates because len < capacity).
			loop {
				if self.slots[self.hand].is_none() {
					let idx = self.hand;
					self.slots[idx] = Some(Entry {
						key: key.clone(),
						value,
						state: initial_state,
						referenced: false,
						scan_taint: false,
					});
					self.map.insert(key, idx);
					self.len += 1;
					self.hand = (self.hand + 1) % self.capacity;
					return;
				}
				self.hand = (self.hand + 1) % self.capacity;
			}
		} else {
			// At capacity: evict one slot via pre-pass + two-pass sweep, then
			// refill. Taint only fires on brand-new non-ghost admissions
			// whose fingerprint hadn't been seen before this put.
			let idx = self.evict_one();
			let scan_taint = !from_ghost && !prior_seen;
			self.slots[idx] = Some(Entry {
				key: key.clone(),
				value,
				state: initial_state,
				referenced: false,
				scan_taint,
			});
			self.map.insert(key, idx);
			self.len += 1;
			debug_assert!(self.map.len() <= self.capacity);
		}
	}

	/// Selects a victim per H01 pre-pass + CLOCK-Pro-lite policy and empties
	/// its slot.
	///
	/// Pre-pass (H01): sweep the ring once starting at `hand`, evicting the
	/// first Cold ref-clear slot whose `scan_taint` is true. Treated as a
	/// Cold eviction for ghost purposes. The sweep is read-only on the slots
	/// it visits (no ref-bit clearing, no demotion), so failing the pre-pass
	/// leaves the ring untouched and pass 1 starts from `hand` unchanged.
	///
	/// Pass 1 only touches Cold slots: ref-set -> clear and advance, ref-clear
	/// -> evict (and push to ghost). Hot slots are skipped untouched.
	///
	/// Pass 2: Hot ref-set slots are cleared and demoted to Cold; the first
	/// Hot ref-clear slot is evicted (Hot evictions skip the ghost). Cold
	/// slots in pass 2 are now all ref-clear (pass 1 cleared them) and are
	/// skipped in favour of a Hot victim.
	///
	/// Fallback: if pass 2 also finds nothing (every Hot slot was ref-set
	/// when pass 2 started, so all got demoted), evict the slot at the
	/// current hand position — by that point it is Cold ref-clear.
	///
	/// Returns the index of the now-empty slot. Map, ghost ring, and `len`
	/// have been updated. Total work is O(capacity) across all phases.
	fn evict_one(&mut self) -> usize {
		// H01 pre-pass: bounded O(capacity) scan for tainted Cold ref-clear
		// victims. We index by offset off `hand` rather than mutating `hand`
		// during the scan, so a no-hit pre-pass leaves pass 1's starting
		// position identical to the parent's behaviour.
		for offset in 0..self.capacity {
			let i = (self.hand + offset) % self.capacity;
			let tainted = self
				.slots[i]
				.as_ref()
				.map(|e| e.state == SlotState::Cold && !e.referenced && e.scan_taint)
				.unwrap_or(false);
			if tainted {
				self.evict_at(i, /*was_cold=*/ true);
				self.hand = (i + 1) % self.capacity;
				return i;
			}
		}

		// Pass 1: prefer cold-unreferenced victims.
		for _ in 0..self.capacity {
			let entry = self
				.slots[self.hand]
				.as_mut()
				.expect("slot must be occupied at full capacity");
			match entry.state {
				SlotState::Cold => {
					if entry.referenced {
						entry.referenced = false;
						self.hand = (self.hand + 1) % self.capacity;
					} else {
						let idx = self.hand;
						self.evict_at(idx, /*was_cold=*/ true);
						self.hand = (idx + 1) % self.capacity;
						return idx;
					}
				}
				SlotState::Hot => {
					self.hand = (self.hand + 1) % self.capacity;
				}
			}
		}

		// Pass 2: demote ref-set Hot slots and evict the first ref-clear Hot.
		for _ in 0..self.capacity {
			let entry = self
				.slots[self.hand]
				.as_mut()
				.expect("slot must be occupied at full capacity");
			match (entry.state, entry.referenced) {
				(SlotState::Hot, true) => {
					entry.referenced = false;
					entry.state = SlotState::Cold;
					self.hand = (self.hand + 1) % self.capacity;
				}
				(SlotState::Hot, false) => {
					let idx = self.hand;
					self.evict_at(idx, /*was_cold=*/ false);
					self.hand = (idx + 1) % self.capacity;
					return idx;
				}
				(SlotState::Cold, _) => {
					// Pass 1 already cleared every Cold ref bit; defer to the
					// fallback rather than competing with the Hot pool.
					self.hand = (self.hand + 1) % self.capacity;
				}
			}
		}

		// Fallback (rare): pass 2 demoted every Hot slot to Cold without
		// finding a ref-clear Hot victim. The slot at `hand` is now Cold
		// ref-clear; evict it and treat it as a cold eviction for ghost
		// purposes.
		let idx = self.hand;
		self.evict_at(idx, /*was_cold=*/ true);
		self.hand = (idx + 1) % self.capacity;
		idx
	}

	/// Removes the entry at `idx` from the cache. If `was_cold`, push the
	/// evicted key onto the ghost ring; Hot evictions do not enter the ghost.
	fn evict_at(&mut self, idx: usize, was_cold: bool) {
		let entry = self
			.slots[idx]
			.take()
			.expect("evicted slot must be occupied");
		self.map.remove(&entry.key);
		debug_assert!(self.len > 0);
		self.len -= 1;
		if was_cold {
			self.push_ghost(entry.key);
		}
	}

	/// Writes `key` into the ghost ring at `ghost_head`, displacing any prior
	/// occupant. Keeps `ghost_set` in lockstep with the ring contents.
	fn push_ghost(&mut self, key: K) {
		let ring_len = self.ghost.len();
		if let Some(old) = self.ghost[self.ghost_head].take() {
			self.ghost_set.remove(&old);
		}
		// A re-inserted key is removed from the ghost via `ghost_remove` in
		// `put`, so a freshly-evicted Cold key never collides with an existing
		// ring entry — but `HashSet::insert` is idempotent regardless.
		self.ghost_set.insert(key.clone());
		self.ghost[self.ghost_head] = Some(key);
		self.ghost_head = (self.ghost_head + 1) % ring_len;
	}

	/// Removes `key` from both the ghost set and its ring slot. The ring is
	/// `capacity` slots long, so the linear scan is O(capacity) worst case.
	fn ghost_remove(&mut self, key: &K) {
		if !self.ghost_set.remove(key) {
			return;
		}
		for slot in self.ghost.iter_mut() {
			if slot.as_ref().map_or(false, |k| k == key) {
				*slot = None;
				return;
			}
		}
	}
}

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		LruCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		LruCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	// CLOCK-Pro-lite + scan-taint is not LRU, so strict-LRU eviction-order
	// tests would fail; only structural / contract-style checks are kept
	// here. The four gates (build / contract / universal-invariant /
	// invariant) do not depend on these inline tests.

	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_round_trips() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_bound_under_overfill() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
	}

	#[test]
	fn never_put_keys_read_none() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..16i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for phantom in [1000i64, 2000, -1, -50] {
			assert!(cache.get(&phantom).is_none());
		}
	}

	#[test]
	fn resident_keys_read_back_their_value() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		for k in 0..50i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	fn ghost_promotion_recovers_evicted_key_as_hot() {
		// capacity=2, ghost length = 2 (parent v001 sizing).
		let mut cache: LruCache<i64, i64> = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts a Cold slot; key joins ghost ring
		let evicted: Vec<i64> = [1i64, 2]
			.into_iter()
			.filter(|k| cache.get(k).is_none())
			.collect();
		assert!(
			!evicted.is_empty(),
			"expected at least one of {{1,2}} to be evicted by capacity=2"
		);
		let k = evicted[0];
		cache.put(k, k * 100);
		assert_eq!(*cache.get(&k).unwrap(), k * 100);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
