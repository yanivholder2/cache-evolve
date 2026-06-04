// CLOCK-Pro-lite with Hot-pool budgeting: v002 extends cycle/0003/v001 by
// capping the Hot pool at `capacity / 2` (floor of 1) and force-demoting the
// longest-resident Hot slot when a Cold→Hot promotion or a ghost-hit Hot
// insert would exceed the cap.
//
// Why: at cap=512 on lirs:multi1, cycle 3's insight observed pass 2 of
// `evict_one` rarely fires, so the Hot pool grows monotonically and squeezes
// the Cold-side recency window that the trace's reuses depend on. Reserving
// at least half of `capacity` for Cold circulation — regardless of how many
// second-touches occur — protects that recency window.
//
// State changes vs the parent:
// * `hot_cap: usize` — fixed in `try_new` as `(capacity / 2).max(1)`.
// * `hot_len: usize` — count of `state == Hot` resident slots, maintained on
//   every transition (Cold→Hot promotion, Hot insert, Hot→Cold demotion, Hot
//   eviction).
// * `force_demote_one_hot()` — walks the slot ring starting at the position
//   opposite the CLOCK hand and demotes the first Hot slot it finds to Cold
//   with `referenced = false`, so the demoted slot becomes immediately
//   pass-1-evictable.
// * The two-pass `evict_one` is otherwise untouched: pass 1 still prefers
//   Cold ref-clear; pass 2 still demotes Hot ref-set and evicts the first Hot
//   ref-clear; fallback unchanged.
//
// The public type is still `LruCache<K, V>` so the JNI ABI in
// `lru_cache_jni.rs` (which is `#[contract] protected-files`) is untouched.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

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
	hot_cap: usize,                  // H02: ceiling on resident Hot slots, fixed at construction
	hot_len: usize,                  // H02: count of resident slots with state == Hot
	ghost: Vec<Option<K>>,           // recent-cold-evictions ring
	ghost_head: usize,               // ghost ring write head
	ghost_set: HashSet<K>,           // O(1) membership mirror of `ghost`
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
		// Ghost-ring length stays at full `capacity` (cycle/0003/v001's H01 sweep).
		let ghost_len = capacity;
		let mut ghost = Vec::with_capacity(ghost_len);
		for _ in 0..ghost_len {
			ghost.push(None);
		}
		// H02: cap the Hot pool at half of capacity (floor of 1 so a cap=1
		// cache can still hold one Hot slot). The Cold pool is therefore
		// guaranteed at least `capacity - hot_cap` slots.
		let hot_cap = (capacity / 2).max(1);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			slots,
			hand: 0,
			len: 0,
			capacity,
			hot_cap,
			hot_len: 0,
			ghost,
			ghost_head: 0,
			ghost_set: HashSet::with_capacity(ghost_len),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident. A Cold hit is promoted to Hot
	/// (the second-touch promotion the parent cites as scan-resistant); the
	/// reference bit is set in either case. If the promotion pushes
	/// `hot_len > hot_cap`, `force_demote_one_hot` immediately demotes the
	/// longest-resident Hot slot back to Cold.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let promoted = {
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.referenced = true;
			if entry.state == SlotState::Cold {
				entry.state = SlotState::Hot;
				true
			} else {
				false
			}
		};
		if promoted {
			self.hot_len += 1;
			if self.hot_len > self.hot_cap {
				self.force_demote_one_hot();
			}
		}
		debug_assert!(self.hot_len <= self.hot_cap);
		self.debug_check_hot_count();
		// Re-borrow to return the value. The slot at `idx` is still occupied:
		// `force_demote_one_hot` only flips a slot's Cold/Hot bit (possibly
		// this one), it never removes a slot.
		let entry = self
			.slots[idx]
			.as_ref()
			.expect("map index points to occupied slot");
		Some(&entry.value)
	}

	/// Inserts or updates `key`. Brand-new keys land in an empty slot if one
	/// exists, otherwise the CLOCK-Pro-lite sweep selects a victim. A ghost-hit
	/// installs the new key as Hot; otherwise as Cold. After any path that may
	/// have raised `hot_len` above `hot_cap`, the helper demotes one Hot slot.
	pub fn put(&mut self, key: K, value: V) {
		// Key already resident: update value in place and set its reference bit.
		// (The state is preserved — promotion only happens via `get` hits. The
		// framework gates only put on get-miss, so this path is dead under the
		// gates but still supports the integration tests.)
		if let Some(&idx) = self.map.get(&key) {
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.value = value;
			entry.referenced = true;
			return;
		}

		// Brand-new key: a ghost hit signals a "recently evicted, now reused"
		// pattern — install as Hot to mirror the CLOCK-Pro evicted-key memory.
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
					if initial_state == SlotState::Hot {
						self.hot_len += 1;
					}
					self.hand = (self.hand + 1) % self.capacity;
					if self.hot_len > self.hot_cap {
						self.force_demote_one_hot();
					}
					debug_assert!(self.hot_len <= self.hot_cap);
					self.debug_check_hot_count();
					return;
				}
				self.hand = (self.hand + 1) % self.capacity;
			}
		} else {
			// At capacity: evict one slot via the two-pass sweep, then refill.
			// `evict_one` maintains `hot_len` for any Hot demotions/evictions it
			// performs; we only need to account for the new entry here.
			let idx = self.evict_one();
			self.slots[idx] = Some(Entry {
				key: key.clone(),
				value,
				state: initial_state,
				referenced: false,
			});
			self.map.insert(key, idx);
			self.len += 1;
			if initial_state == SlotState::Hot {
				self.hot_len += 1;
			}
			if self.hot_len > self.hot_cap {
				self.force_demote_one_hot();
			}
			debug_assert!(self.map.len() <= self.capacity);
			debug_assert!(self.hot_len <= self.hot_cap);
			self.debug_check_hot_count();
		}
	}

	/// Selects a victim per CLOCK-Pro-lite policy and empties its slot.
	///
	/// Pass 1 only touches Cold slots: ref-set → clear and advance, ref-clear →
	/// evict (and push to ghost). Hot slots are skipped untouched.
	///
	/// If pass 1 finds no Cold-ref-clear victim, pass 2 re-sweeps. Hot ref-set
	/// slots are cleared and demoted to Cold (decrementing `hot_len`); the
	/// first Hot ref-clear slot is evicted (Hot evictions skip the ghost).
	/// Cold slots in pass 2 are now all ref-clear and are skipped in favour of
	/// a Hot victim.
	///
	/// Fallback: if pass 2 also finds nothing (every Hot slot was ref-set when
	/// pass 2 started, so all got demoted), evict the slot at the current hand
	/// position — by that point it is a Cold ref-clear slot.
	///
	/// Returns the index of the now-empty slot. Map, ghost ring, `len`, and
	/// `hot_len` have all been updated.
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
					debug_assert!(self.hot_len > 0);
					self.hot_len -= 1;
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
	/// evicted key onto the ghost ring; Hot evictions do not enter the ghost
	/// and decrement `hot_len` instead.
	fn evict_at(&mut self, idx: usize, was_cold: bool) {
		let entry = self
			.slots[idx]
			.take()
			.expect("evicted slot must be occupied");
		debug_assert_eq!(
			entry.state == SlotState::Cold,
			was_cold,
			"evict_at: was_cold={was_cold} contradicts slot state {:?}",
			entry.state
		);
		if !was_cold {
			debug_assert!(self.hot_len > 0);
			self.hot_len -= 1;
		}
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
	/// case — inherited from cycle/0003/v001's H01 sizing.
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

	/// H02: forced Hot→Cold demotion to keep `hot_len <= hot_cap`. Walks the
	/// slot ring starting at `(hand + capacity - 1) % capacity` — the position
	/// opposite the CLOCK hand, where the hypothesis expects the longest-
	/// resident Hot slot to live — and demotes the first slot whose
	/// `state == Hot` to `Cold` with `referenced = false`, so the demoted
	/// slot becomes immediately pass-1-evictable.
	///
	/// Walks at most `capacity` slots; if every occupied slot is already
	/// non-Hot (hot_len == 0) the helper is a no-op.
	fn force_demote_one_hot(&mut self) {
		if self.hot_len == 0 {
			return;
		}
		let mut cursor = (self.hand + self.capacity - 1) % self.capacity;
		for _ in 0..self.capacity {
			if let Some(entry) = self.slots[cursor].as_mut() {
				if entry.state == SlotState::Hot {
					entry.state = SlotState::Cold;
					entry.referenced = false;
					self.hot_len -= 1;
					return;
				}
			}
			cursor = (cursor + 1) % self.capacity;
		}
		// Unreachable: hot_len > 0 implies at least one slot has state == Hot,
		// and the scan covers every slot.
		debug_assert!(false, "force_demote_one_hot: hot_len={} but no Hot slot found", self.hot_len);
	}

	/// Debug-only invariant: `hot_len` equals the number of resident slots
	/// whose `state == SlotState::Hot`. Compiled out in release.
	#[inline]
	fn debug_check_hot_count(&self) {
		#[cfg(debug_assertions)]
		{
			let counted = self
				.slots
				.iter()
				.filter_map(|s| s.as_ref())
				.filter(|e| e.state == SlotState::Hot)
				.count();
			debug_assert_eq!(
				counted, self.hot_len,
				"hot_len drift: tracked={} counted={}",
				self.hot_len, counted
			);
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
	fn hot_pool_stays_within_cap() {
		// capacity=8, hot_cap=4. Trigger many Cold→Hot promotions and verify
		// hot_len never exceeds hot_cap after any get.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..8i64 {
			cache.put(k, k);
		}
		for k in 0..8i64 {
			let _ = cache.get(&k);
			assert!(
				cache.hot_len <= cache.hot_cap,
				"hot_len={} exceeded hot_cap={}",
				cache.hot_len,
				cache.hot_cap
			);
		}
		assert_eq!(cache.hot_cap, 4);
	}

	#[test]
	fn hot_cap_floor_at_capacity_one() {
		// capacity=1 must still produce hot_cap=1 (not zero); otherwise the
		// promotion path would infinitely re-demote.
		let cache: LruCache<i64, i64> = LruCache::new(1);
		assert_eq!(cache.hot_cap, 1);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
