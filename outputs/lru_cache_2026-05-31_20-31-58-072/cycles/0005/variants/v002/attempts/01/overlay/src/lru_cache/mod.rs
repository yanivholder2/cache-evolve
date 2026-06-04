// CLOCK-Pro-lite cache: extends the parent v001 (cycle/0003) CLOCK ring with a
// per-slot Cold/Hot state and a ghost ring of recently-evicted cold keys.
//
// * Each slot carries a `(state, referenced)` pair.
// * A `get` hit promotes a Cold slot to Hot (second-touch) and sets the ref bit;
//   a Hot hit only sets the ref bit.
// * A `put` of a brand-new key whose key is in the ghost ring inserts as Hot;
//   otherwise as Cold. The reference bit starts clear, except for the H02
//   ghost-freshness branch noted below.
// * The eviction sweep prefers cold-unreferenced victims (pass 1). If no Cold
//   slot is evictable, a second pass demotes Hot ref-set slots to Cold and
//   evicts the first Hot ref-clear slot.
// * On eviction of a Cold slot, the evicted key joins a ghost ring of length
//   `capacity` (parent v001 sizing). Hot evictions skip the ghost.
//
// H02 differentiator (this variant):
//   The strength of a ghost-hit promotion now depends on *where* in the ghost
//   ring the matched entry sat. `ghost_remove` returns the matched ring index;
//   the position's "freshness offset" against `ghost_head` is computed as
//   `offset = (ghost_head + ring_len - idx) % ring_len`. If the offset falls in
//   the freshest quarter (`offset < ring_len / 4`), the re-admitted key is
//   installed as Hot *with `referenced: true`*, buying it one extra pass-2
//   ref-clear sweep before becoming eligible for eviction. Older three-quarters
//   keep the parent's behaviour (Hot + `referenced: false`). Non-ghost inserts,
//   resident updates, pass 1, pass 2, the fallback, and the ghost-ring write
//   path are untouched, and the four cache invariants (`len <= capacity`,
//   `ghost.len() == capacity`, ghost-set/ring lockstep, single occupancy per
//   key) all hold.
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
		// Ghost-ring length scaled to full `capacity` (parent v001 sizing).
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
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident. A Cold hit is promoted to Hot
	/// (the second-touch promotion the hypothesis cites as scan-resistant); the
	/// reference bit is set in either case.
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

	/// Inserts or updates `key`. Brand-new keys land in an empty slot if one
	/// exists, otherwise the CLOCK-Pro-lite sweep selects a victim.
	pub fn put(&mut self, key: K, value: V) {
		// Key already resident: update value in place and set its reference bit.
		// (The state is preserved — the hypothesis only describes promotion via
		// `get` hits. The framework gates only put on get-miss, so this path is
		// dead under the gates but still supports the integration tests.)
		if let Some(&idx) = self.map.get(&key) {
			let entry = self
				.slots[idx]
				.as_mut()
				.expect("map index points to occupied slot");
			entry.value = value;
			entry.referenced = true;
			return;
		}

		// Brand-new key. Consult the ghost ring: a key recently evicted (as
		// Cold) but now re-referenced is treated as Hot, mirroring the
		// CLOCK-Pro idea of an evicted-key memory. H02 additionally scales
		// the promotion's *strength* by ghost-ring freshness.
		let ghost_hit_idx = self.ghost_remove(&key);
		let (initial_state, initial_referenced) = match ghost_hit_idx {
			Some(idx) => {
				// Freshness offset against the ghost write head. Per H02 the
				// freshest quarter (`offset < ring_len / 4`) earns the extra
				// `referenced: true` bit; older three-quarters keep the parent
				// v001 behaviour of Hot + referenced=false.
				let ring_len = self.ghost.len();
				let offset = (self.ghost_head + ring_len - idx) % ring_len;
				let freshest_quarter = offset < ring_len / 4;
				(SlotState::Hot, freshest_quarter)
			}
			None => (SlotState::Cold, false),
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
						referenced: initial_referenced,
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
				referenced: initial_referenced,
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

	/// Removes `key` from both the ghost set and its ring slot. Returns the
	/// ring index of the matched entry when `key` was present, `None`
	/// otherwise. The ring is `capacity` slots long (v001 sizing), so the
	/// linear scan is O(capacity) in the worst case — H02 uses the returned
	/// index to gauge ghost-ring freshness when seeding the re-admitted slot.
	fn ghost_remove(&mut self, key: &K) -> Option<usize> {
		if !self.ghost_set.remove(key) {
			return None;
		}
		for (idx, slot) in self.ghost.iter_mut().enumerate() {
			if slot.as_ref().map_or(false, |k| k == key) {
				*slot = None;
				return Some(idx);
			}
		}
		// `ghost_set` mirrored a present key but no matching slot was found —
		// impossible if push_ghost / ghost_remove are the only mutators. Treat
		// as a miss rather than panic, so the variant still satisfies the
		// universal invariants under any future refactor.
		None
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
	fn ghost_promotion_recovers_evicted_key_as_hot() {
		// capacity=2, ghost length = 2 (full-capacity ghost ring).
		let mut cache: LruCache<i64, i64> = LruCache::new(2);
		cache.put(1, 10); // slot 0, Cold
		cache.put(2, 20); // slot 1, Cold
		cache.put(3, 30); // evicts a Cold slot, pushes its key to ghost
		// Re-insert one of {1, 2} — whichever is in the ghost ring will land as Hot.
		// At least one of the two evicted-then-re-put keys must round-trip.
		let evicted: Vec<i64> = [1i64, 2].into_iter().filter(|k| cache.get(k).is_none()).collect();
		assert!(!evicted.is_empty(), "expected at least one of {{1,2}} to be evicted by capacity=2");
		// Re-put the evicted one and verify the value round-trips.
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
