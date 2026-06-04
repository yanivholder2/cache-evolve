// CLOCK-Pro-lite cache with H01 bypass-on-first-sight admission gating.
//
// Inherits the parent v001's CLOCK-Pro-lite skeleton:
// * Each slot carries a `(state, referenced)` pair (Cold/Hot, ref bit).
// * `get` promotes a Cold hit to Hot and sets the ref bit; Hot hits set the
//   ref bit.
// * `put` of a ghost-ring hit inserts as Hot; otherwise as Cold. Ref bit
//   starts clear.
// * Eviction sweep prefers Cold-unreferenced victims (pass 1); pass 2 demotes
//   Hot ref-set slots and evicts the first Hot ref-clear.
// * Cold evictions push the evicted key onto the ghost ring (length =
//   capacity, the parent v001's sizing). Hot evictions skip the ghost.
//
// H01 additions on top of the parent (cycle/0006/v001 fix):
// * A `seen: Vec<bool>` first-seen sketch of length
//   `capacity.saturating_mul(4).max(4)`, indexed by a `DefaultHasher` fold of
//   the key.
// * Only `put` marks `seen[fp] = true`. `get` does NOT touch the sketch — neither
//   read nor write — so the gate's precondition (`was_seen == false` on a
//   brand-new key's first `put`) survives the framework's standard
//   `get_miss → put` pattern.
// * `put` of a brand-new key (not resident, not in the ghost ring), when the
//   cache is at capacity and `seen[fp]` was `false` *prior* to this put, is
//   treated as a no-op: the sketch is marked, but no slot is allocated, no
//   eviction is triggered, and the ghost ring is not touched. The key is
//   admitted only on its second `put` (when `seen[fp]` is already true), at
//   which point the existing CLOCK-Pro-lite admission path runs unchanged.
// * Ghost-promoted inserts (key present in `ghost_set`) bypass the gate
//   unconditionally — proven worthy on prior residence.
// * Resident updates (key already in `map`) also bypass the gate unchanged.
//
// The public type is still `LruCache<K, V>` so the JNI ABI in
// `lru_cache_jni.rs` (which is `#[contract] protected-files`) is untouched.

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
	seen: Vec<bool>,                 // H01 first-seen sketch (4x capacity, min 4)
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
		// Parent v001 sizing: ghost ring length = capacity.
		let ghost_len = capacity;
		let mut ghost = Vec::with_capacity(ghost_len);
		for _ in 0..ghost_len {
			ghost.push(None);
		}
		// H01 sketch sizing: capacity * 4, with a floor of 4 so tiny caches
		// still have a non-degenerate sketch.
		let sketch_len = capacity.saturating_mul(4).max(4);
		let seen = vec![false; sketch_len];
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

	/// `DefaultHasher` fold of `key`, reduced modulo the sketch length.
	fn fingerprint(&self, key: &K) -> usize {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		(hasher.finish() as usize) % self.seen.len()
	}

	/// Returns the value for `key` if resident. A Cold hit is promoted to Hot
	/// (the second-touch promotion the parent skeleton cites as scan-resistant);
	/// the reference bit is set in either case.
	///
	/// `get` does NOT touch the first-seen sketch. Marking the sketch here would
	/// collapse the H01 admission gate's precondition under the framework's
	/// standard `get_miss → put` pattern (every put would observe `was_seen ==
	/// true`). The sketch is the responsibility of `put` alone.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let entry = self
			.slots[idx]
			.as_mut()
			.expect("map index points to occupied slot");
		if entry.state == SlotState::Cold {
			entry.state = SlotState::Hot;
		}
		entry.referenced = true;
		Some(&entry.value)
	}

	/// Inserts or updates `key`. Brand-new keys are subject to the H01
	/// admission gate before any allocation or eviction:
	/// * Resident updates bypass the gate (in-place value swap).
	/// * Ghost-promoted inserts bypass the gate (proven worthy).
	/// * Brand-new keys at full capacity whose sketch bit was clear prior to
	///   this put are dropped — the sketch is marked, but no slot is taken
	///   and no eviction is triggered.
	/// * Brand-new keys with the sketch bit already set (a re-touched key from
	///   the same fingerprint class) proceed through the unchanged CLOCK-Pro-
	///   lite admission path.
	pub fn put(&mut self, key: K, value: V) {
		// Capture the sketch bit *before* this put writes it. The check below
		// reads `was_seen`, the write reflects "every put marks `seen[fp]`".
		let fp = self.fingerprint(&key);
		let was_seen = self.seen[fp];
		self.seen[fp] = true;

		// Key already resident: update value in place and set its reference bit.
		// (Bypasses the gate — resident updates are not "brand-new".)
		if let Some(&idx) = self.map.get(&key) {
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.value = value;
			entry.referenced = true;
			return;
		}

		// Brand-new key. Check the ghost ring before the gate: a ghost hit
		// bypasses admission gating unconditionally.
		let from_ghost = self.ghost_set.contains(&key);

		// H01 admission gate: drop brand-new, never-before-sketched keys when
		// the cache is at capacity. The sketch has already been marked, so the
		// *next* put of any key sharing this fingerprint will be admitted.
		if self.len >= self.capacity && !from_ghost && !was_seen {
			return;
		}

		if from_ghost {
			self.ghost_remove(&key);
		}
		let initial_state = if from_ghost {
			SlotState::Hot
		} else {
			SlotState::Cold
		};

		if self.len < self.capacity {
			// Below capacity: an empty slot exists; scan forward from `hand`
			// for the first one. Terminates because len < capacity.
			loop {
				if self.slots[self.hand].is_none() {
					let idx = self.hand;
					self.slots[idx] = Some(Entry {
						key: key.clone(),
						value,
						state: initial_state,
						referenced: false,
					});
					self.map.insert(key, idx);
					self.len += 1;
					self.hand = (self.hand + 1) % self.capacity;
					return;
				}
				self.hand = (self.hand + 1) % self.capacity;
			}
		} else {
			// At capacity: evict one slot via the two-pass sweep, then refill.
			let idx = self.evict_one();
			self.slots[idx] = Some(Entry {
				key: key.clone(),
				value,
				state: initial_state,
				referenced: false,
			});
			self.map.insert(key, idx);
			self.len += 1;
			debug_assert!(self.map.len() <= self.capacity);
		}
	}

	/// Selects a victim per CLOCK-Pro-lite policy and empties its slot.
	///
	/// Pass 1 only touches Cold slots: ref-set → clear and advance, ref-clear →
	/// evict (and push to ghost). Hot slots are skipped untouched.
	///
	/// If pass 1 finds no Cold-ref-clear victim, pass 2 re-sweeps. Hot ref-set
	/// slots are cleared and demoted to Cold; the first Hot ref-clear slot is
	/// evicted (Hot evictions skip the ghost). Cold slots in pass 2 are now all
	/// ref-clear (pass 1 cleared them) and are skipped in favour of a Hot
	/// victim.
	///
	/// Fallback: if pass 2 also finds nothing (every Hot slot was ref-set when
	/// pass 2 started, so all got demoted), evict the slot at the current hand
	/// position — by that point it is a Cold ref-clear slot.
	///
	/// Returns the index of the now-empty slot. Map, ghost ring, and `len` have
	/// been updated.
	fn evict_one(&mut self) -> usize {
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

		// Fallback (rare): pass 2 demoted every Hot slot to Cold without finding
		// a ref-clear Hot victim. The slot at `hand` is now Cold ref-clear; evict
		// it and treat it as a cold eviction for ghost purposes.
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
	/// `capacity` slots long, so the linear scan is O(capacity) in the worst
	/// case.
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
	// CLOCK-Pro-lite is not LRU, so strict-LRU eviction-order tests would fail;
	// only structural / contract-style checks are kept here. The four gates
	// (build / contract / universal-invariant / invariant) do not depend on
	// these inline tests.

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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
