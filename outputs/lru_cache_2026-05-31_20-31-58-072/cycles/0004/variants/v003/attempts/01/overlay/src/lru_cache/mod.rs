// CLOCK-Pro-lite cache with a Cold→Warm→Hot promotion ladder.
//
// Refinement on v001's CLOCK-Pro-lite (H03): instead of promoting on the
// first re-hit (Cold → Hot in one step), this variant inserts a `Warm`
// sub-state between Cold and Hot, requiring two Cold-side touches before a
// slot reaches the protected Hot pool.
//
// * Each slot carries a `(state, referenced)` pair where `state` is one of
//   `Cold`, `Warm`, `Hot`.
// * A `get` hit:
//     - on a Cold slot promotes it to Warm and sets the ref bit;
//     - on a Warm slot promotes it to Hot and sets the ref bit;
//     - on a Hot slot only sets the ref bit (unchanged from v001).
// * A `put` of a brand-new key whose key is in the ghost ring inserts as
//   Hot (an evicted-then-re-referenced key has already proved itself);
//   otherwise as Cold. The reference bit starts clear.
// * The eviction sweep has two stages:
//     - Pass 1 walks the ring twice. First revolution: only Cold ref-clear
//       slots are victims (Warm and Hot skipped; Cold ref-set is cleared
//       and the hand advances). Second revolution: only Warm ref-clear
//       slots are victims (Cold all ref-clear by now and skipped; Warm
//       ref-set is cleared and demoted back to Cold; Hot still skipped).
//     - Pass 2 (unchanged from v001) demotes Hot ref-set slots to Cold
//       and evicts the first Hot ref-clear slot. Hot evictions skip the
//       ghost.
//     - Fallback (unchanged): evict the slot at the hand, treated as a
//       Cold eviction for ghost purposes.
// * On eviction of a Cold or Warm slot, the evicted key joins the ghost
//   ring (length `capacity`). Hot evictions skip the ghost.
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
	Warm,
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
	ghost: Vec<Option<K>>,           // recent cold/warm-eviction ring
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
		// Ghost-ring length stays at full `capacity` (v001 sizing — H01).
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

	/// Returns the value for `key` if resident. A Cold hit is promoted to
	/// Warm (first re-touch); a Warm hit is promoted to Hot (second
	/// re-touch); a Hot hit is unchanged. The reference bit is set in all
	/// three cases.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let entry = self
			.slots[idx]
			.as_mut()
			.expect("map index points to occupied slot");
		match entry.state {
			SlotState::Cold => entry.state = SlotState::Warm,
			SlotState::Warm => entry.state = SlotState::Hot,
			SlotState::Hot => {}
		}
		entry.referenced = true;
		Some(&entry.value)
	}

	/// Inserts or updates `key`. Brand-new keys land in an empty slot if one
	/// exists, otherwise the CLOCK-Pro-lite sweep selects a victim.
	pub fn put(&mut self, key: K, value: V) {
		// Key already resident: update value in place and set its reference bit.
		// (State is preserved — the hypothesis only describes promotion via
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

		// Brand-new key. Consult the ghost ring before deciding the initial
		// state: a key recently evicted (as Cold or Warm) but now re-referenced
		// is treated as Hot, mirroring the CLOCK-Pro idea of an evicted-key
		// memory.
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
					self.hand = (self.hand + 1) % self.capacity;
					return;
				}
				self.hand = (self.hand + 1) % self.capacity;
			}
		} else {
			// At capacity: evict one slot via the multi-pass sweep, then refill.
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

	/// Selects a victim per the H03 Cold→Warm→Hot ladder.
	///
	/// Pass 1 walks the ring up to twice:
	///   * Revolution 1: only Cold slots are inspected as victims. Cold
	///     ref-set → clear bit, advance; Cold ref-clear → evict (push to
	///     ghost). Warm and Hot are skipped untouched.
	///   * Revolution 2 (only if revolution 1 found no victim): only Warm
	///     slots are inspected as victims. Warm ref-set → clear bit, demote
	///     to Cold, advance; Warm ref-clear → evict (push to ghost). Cold
	///     (now all ref-clear) and Hot are skipped untouched.
	///
	/// Pass 2 (unchanged from v001) demotes Hot ref-set slots to Cold and
	/// evicts the first Hot ref-clear slot. Hot evictions skip the ghost.
	///
	/// Fallback (unchanged): if pass 2 also finds nothing (every Hot slot
	/// was ref-set when pass 2 started, so all got demoted), evict the slot
	/// at the current hand position — by that point it is a Cold ref-clear
	/// slot.
	///
	/// Returns the index of the now-empty slot. Map, ghost ring, and `len`
	/// have been updated.
	fn evict_one(&mut self) -> usize {
		// Pass 1, revolution 1: prefer Cold ref-clear victims.
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
						self.evict_at(idx, /*push_to_ghost=*/ true);
						self.hand = (idx + 1) % self.capacity;
						return idx;
					}
				}
				SlotState::Warm | SlotState::Hot => {
					self.hand = (self.hand + 1) % self.capacity;
				}
			}
		}

		// Pass 1, revolution 2: no Cold ref-clear victim found. All Cold
		// slots are now ref-clear (revolution 1 cleared every Cold ref bit)
		// but we defer to Warm. Warm ref-set → clear and demote to Cold;
		// Warm ref-clear → evict (push to ghost). Hot still skipped.
		for _ in 0..self.capacity {
			let entry = self
				.slots[self.hand]
				.as_mut()
				.expect("slot must be occupied at full capacity");
			match entry.state {
				SlotState::Warm => {
					if entry.referenced {
						entry.referenced = false;
						entry.state = SlotState::Cold;
						self.hand = (self.hand + 1) % self.capacity;
					} else {
						let idx = self.hand;
						self.evict_at(idx, /*push_to_ghost=*/ true);
						self.hand = (idx + 1) % self.capacity;
						return idx;
					}
				}
				SlotState::Cold | SlotState::Hot => {
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
					self.evict_at(idx, /*push_to_ghost=*/ false);
					self.hand = (idx + 1) % self.capacity;
					return idx;
				}
				(SlotState::Cold, _) | (SlotState::Warm, _) => {
					// Pass 1 already drained the Cold/Warm pools; defer to the
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
		self.evict_at(idx, /*push_to_ghost=*/ true);
		self.hand = (idx + 1) % self.capacity;
		idx
	}

	/// Removes the entry at `idx` from the cache. If `push_to_ghost`, push
	/// the evicted key onto the ghost ring; Hot evictions do not enter the
	/// ghost.
	fn evict_at(&mut self, idx: usize, push_to_ghost: bool) {
		let entry = self
			.slots[idx]
			.take()
			.expect("evicted slot must be occupied");
		self.map.remove(&entry.key);
		debug_assert!(self.len > 0);
		self.len -= 1;
		if push_to_ghost {
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
		// `put`, so a freshly-evicted key never collides with an existing
		// ring entry — but `HashSet::insert` is idempotent regardless.
		self.ghost_set.insert(key.clone());
		self.ghost[self.ghost_head] = Some(key);
		self.ghost_head = (self.ghost_head + 1) % ring_len;
	}

	/// Removes `key` from both the ghost set and its ring slot.
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
	// CLOCK-Pro-lite is not LRU, so strict-LRU eviction-order tests would
	// fail; only structural / contract-style checks are kept here. The four
	// gates (build / contract / universal-invariant / invariant) do not
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
