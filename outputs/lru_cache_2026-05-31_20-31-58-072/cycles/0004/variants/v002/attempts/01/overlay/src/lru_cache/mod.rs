// CLOCK-Pro-lite cache: extends the parent v001 CLOCK ring with per-slot
// recency tracking so pass 1 evicts the *least-recently-touched* Cold
// ref-clear slot instead of the first one the hand happens to hit.
//
// * Each slot carries a `(state, referenced, last_touch)` triple. A monotonic
//   `tick` counter on the cache stamps `last_touch` on every `get` hit and
//   every `put` (insert and resident update).
// * A `get` hit promotes a Cold slot to Hot (second-touch) and sets the ref
//   bit; a Hot hit only sets the ref bit. Both refresh `last_touch`.
// * A `put` of a brand-new key whose key is in the ghost ring inserts as Hot;
//   otherwise as Cold. The reference bit starts clear; `last_touch` is the
//   current tick.
// * Pass 1 of the eviction sweep performs a single full ring traversal: every
//   Cold ref-set slot has its ref bit cleared (CLOCK aging still progresses),
//   while every Cold ref-clear slot is considered as a candidate, and we
//   remember the one with the smallest `last_touch`. After the sweep, if any
//   Cold ref-clear candidate was found, evict the least-recently-touched one
//   and advance the hand past it.
// * If pass 1 finds no Cold ref-clear candidate, pass 2 (Hot demotion) and
//   the fallback path remain identical to v001.
// * On eviction of a Cold slot, the evicted key joins a ghost ring of length
//   `capacity` (inherited from v001's H01 sweep). Hot evictions skip the
//   ghost.
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
	last_touch: u64,
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
	tick: u64,                       // monotonic touch counter
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
		// H01 (inherited from v001): ghost-ring length scaled to full
		// `capacity` to capture re-references at longer eviction-to-reference
		// distances on lirs:multi1.
		let ghost_len = capacity;
		let mut ghost = Vec::with_capacity(ghost_len);
		for _ in 0..ghost_len {
			ghost.push(None);
		}
		Some(Self {
			map: HashMap::with_capacity(capacity),
			slots,
			hand: 0,
			len: 0,
			capacity,
			ghost,
			ghost_head: 0,
			ghost_set: HashSet::with_capacity(ghost_len),
			tick: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident. A Cold hit is promoted to Hot
	/// (the second-touch promotion the parent insight cites as scan-resistant);
	/// the reference bit and `last_touch` tick are refreshed in either case.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let stamp = self.tick;
		self.tick = self.tick.wrapping_add(1);
		let entry = self
			.slots[idx]
			.as_mut()
			.expect("map index points to occupied slot");
		if entry.state == SlotState::Cold {
			entry.state = SlotState::Hot;
		}
		entry.referenced = true;
		entry.last_touch = stamp;
		Some(&entry.value)
	}

	/// Inserts or updates `key`. Brand-new keys land in an empty slot if one
	/// exists, otherwise the CLOCK-Pro-lite sweep selects a victim.
	pub fn put(&mut self, key: K, value: V) {
		// Key already resident: update value in place, set its reference bit,
		// and refresh `last_touch`. (The state is preserved — only `get` hits
		// promote Cold→Hot. The framework gates only put on get-miss, so this
		// path is dead under the gates but still supports the integration
		// tests.)
		if let Some(&idx) = self.map.get(&key) {
			let stamp = self.tick;
			self.tick = self.tick.wrapping_add(1);
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.value = value;
			entry.referenced = true;
			entry.last_touch = stamp;
			return;
		}

		// Brand-new key. Consult the ghost ring before deciding the initial
		// state: a key recently evicted (as Cold) but now re-referenced is
		// treated as hot, mirroring the CLOCK-Pro idea of an evicted-key memory.
		let from_ghost = self.ghost_set.contains(&key);
		if from_ghost {
			self.ghost_remove(&key);
		}
		let initial_state = if from_ghost {
			SlotState::Hot
		} else {
			SlotState::Cold
		};

		let stamp = self.tick;
		self.tick = self.tick.wrapping_add(1);

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
						last_touch: stamp,
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
				last_touch: stamp,
			});
			self.map.insert(key, idx);
			self.len += 1;
			debug_assert!(self.map.len() <= self.capacity);
		}
	}

	/// Selects a victim per CLOCK-Pro-lite policy and empties its slot.
	///
	/// Pass 1 (H02): a single full ring sweep over all `capacity` slots
	/// starting from `hand`. For each Cold ref-set slot, clear its ref bit and
	/// continue (CLOCK aging still progresses). For each Cold ref-clear slot,
	/// track the one with the smallest `last_touch` seen so far. Hot slots are
	/// skipped untouched. After the sweep, if any Cold ref-clear candidate was
	/// found, evict the least-recently-touched one, advance `hand` to
	/// `(idx + 1) % capacity`, and return.
	///
	/// If pass 1 finds no Cold-ref-clear candidate, pass 2 re-sweeps. Hot
	/// ref-set slots are cleared and demoted to Cold; the first Hot ref-clear
	/// slot is evicted (Hot evictions skip the ghost). Cold slots in pass 2
	/// are now all ref-clear (pass 1 cleared them) and are skipped in favour
	/// of a Hot victim.
	///
	/// Fallback: if pass 2 also finds nothing (every Hot slot was ref-set when
	/// pass 2 started, so all got demoted), evict the slot at the current
	/// hand position — by that point it is a Cold ref-clear slot.
	///
	/// Returns the index of the now-empty slot. Map, ghost ring, and `len`
	/// have been updated.
	fn evict_one(&mut self) -> usize {
		// Pass 1: single full ring sweep. Clear ref bits on Cold ref-set
		// slots, and track the Cold ref-clear slot with the minimum
		// `last_touch`.
		let mut victim: Option<(usize, u64)> = None;
		for offset in 0..self.capacity {
			let idx = (self.hand + offset) % self.capacity;
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("slot must be occupied at full capacity");
			match entry.state {
				SlotState::Cold => {
					if entry.referenced {
						entry.referenced = false;
					} else {
						let stamp = entry.last_touch;
						match victim {
							Some((_, best)) if best <= stamp => {}
							_ => victim = Some((idx, stamp)),
						}
					}
				}
				SlotState::Hot => {
					// Untouched in pass 1.
				}
			}
		}
		if let Some((idx, _)) = victim {
			self.evict_at(idx, /*was_cold=*/ true);
			self.hand = (idx + 1) % self.capacity;
			return idx;
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
					// Pass 1 already cleared every Cold ref bit and did not
					// pick a victim (otherwise we returned above); defer to
					// the fallback rather than competing with the Hot pool.
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
	/// `capacity` slots long (H01 sizing), so the linear scan is O(capacity)
	/// in the worst case.
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
	// CLOCK-Pro-lite is not strict LRU, so strict-LRU eviction-order tests
	// would fail; only structural / contract-style checks are kept here. The
	// four gates (build / contract / universal-invariant / invariant) do not
	// depend on these inline tests.

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
	fn least_recently_touched_cold_is_preferred() {
		// capacity=3: all three insertions land Cold, ref-clear. Touch slot 1
		// and slot 2 via get hits so slot 0 has the smallest `last_touch`.
		// The next get-miss + put should evict slot 0's key.
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(10, 100);
		cache.put(20, 200);
		cache.put(30, 300);
		// Touch 20 and 30 — both become Hot with refreshed last_touch.
		// 10 stays Cold ref-clear with the smallest last_touch.
		assert_eq!(*cache.get(&20).unwrap(), 200);
		assert_eq!(*cache.get(&30).unwrap(), 300);
		// Insert a brand-new key. Pass 1 finds only one Cold ref-clear
		// candidate (slot for key 10), so it must be the victim.
		cache.put(40, 400);
		assert!(cache.get(&10).is_none(), "key 10 should have been evicted as LRT Cold");
		assert_eq!(*cache.get(&40).unwrap(), 400);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
