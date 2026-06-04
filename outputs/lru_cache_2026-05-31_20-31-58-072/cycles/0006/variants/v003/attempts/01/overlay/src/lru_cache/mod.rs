// Four-level segmented LRU (S4LRU) over an arena-backed doubly-linked list,
// extended with two cycle-5-prescribed fixes:
//
//   (1) Asymmetric, top-heavy per-level split. Instead of `capacity/4` for
//       every level, the top tier L3 takes half the cache and the remaining
//       half is split three ways across L0/L1/L2 (any remainder lands in L0).
//       A key that earns its way to L3 now has 2x the retention room of the
//       parent's symmetric split.
//
//   (2) Ghost ring on L3. A FIFO ring of length `L3_cap` tracks keys evicted
//       *directly from L3* (i.e. by the new direct-admit path below, not by
//       the demotion path that conserves residency). On a `put` of a
//       non-resident key found in the ghost ring, the key is installed
//       straight into L3's MRU end, skipping the L0->L1->L2->L3 climb. If
//       L3 then overflows its per-level cap, its LRU-end key is true-evicted
//       and recorded back into the ghost ring. Membership is mirrored in a
//       `HashSet` so the lookup is O(1).
//
// Conservation in the L0/L1/L2 sub-stack is unchanged: every overflowing
// promotion is paired with a demotion back to the source level, so
// `sum(l_len[0..3])` does not change on `get`-induced promotions there. L3
// differs in one direction only: in-ladder promotion into L3 still demotes
// L3's tail to L2 (conservation, exactly as before), but ghost-hit direct
// admission into L3 true-evicts L3's tail to the ghost ring.
//
// Slot bookkeeping mirrors the baseline: at any quiescent moment the free
// chain holds at most one slot, so a single-`usize` `free_head` (sentinel
// `NIL`) is enough. Both the L0-tail eviction and the new L3-tail eviction
// pair their `arena[..] = None`/`free_head = idx` with a fresh `alloc_slot`
// drain inside the same `put`-on-miss call.

use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const NUM_LEVELS: usize = 4;
const NIL: usize = usize::MAX;

#[derive(Debug)]
struct Entry<K, V> {
	key: K,
	value: V,
	prev: usize, // NIL when this entry is the head of its level
	next: usize, // NIL when this entry is the tail of its level
	level: u8,   // 0..NUM_LEVELS
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,          // key -> arena index
	arena: Vec<Option<Entry<K, V>>>, // slots; `None` = free
	l_head: [usize; NUM_LEVELS],     // per-level MRU end (NIL when empty)
	l_tail: [usize; NUM_LEVELS],     // per-level LRU end (NIL when empty)
	l_len: [usize; NUM_LEVELS],      // per-level residency
	l_cap: [usize; NUM_LEVELS],      // precomputed asymmetric per-level caps
	capacity: usize,                 // total residence budget
	free_head: usize,                // free slot (NIL when none)
	ghost: Vec<Option<K>>,           // FIFO ring of L3-evicted keys (empty when ghost_cap == 0)
	ghost_head: usize,               // next write index into `ghost` (mod ghost.len())
	ghost_set: HashSet<K>,           // O(1) mirror of the keys present in `ghost`
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
		// Asymmetric top-heavy split:
		//   L3 = capacity / 2
		//   L0/L1/L2 each take (capacity - L3_cap) / 3, with the remainder
		//   landing in L0 (the bottom level).
		let l3_cap = capacity / 2;
		let remaining = capacity - l3_cap;
		let base = remaining / 3;
		let l1_cap = base;
		let l2_cap = base;
		let l0_cap = remaining - l1_cap - l2_cap; // absorbs the 0..2 remainder
		let l_cap = [l0_cap, l1_cap, l2_cap, l3_cap];
		debug_assert_eq!(l_cap.iter().sum::<usize>(), capacity);
		let ghost_cap = l3_cap;
		log::info!(
			"LruCache.try_new: capacity={capacity}, l_cap={l_cap:?}, ghost_cap={ghost_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			arena: Vec::with_capacity(capacity),
			l_head: [NIL; NUM_LEVELS],
			l_tail: [NIL; NUM_LEVELS],
			l_len: [0; NUM_LEVELS],
			l_cap,
			capacity,
			free_head: NIL,
			ghost: vec![None; ghost_cap],
			ghost_head: 0,
			ghost_set: HashSet::new(),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Per-level capacity from the precomputed asymmetric split.
	fn cap_for(&self, level: usize) -> usize {
		self.l_cap[level]
	}

	/// Returns the value for `key`, promoting one step up the level ladder
	/// on a hit (or moving to head within the top level).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.entry(idx).value)
	}

	/// On a miss:
	///   - if `key` is in the ghost ring, install it directly at L3's MRU
	///     end (skipping the L0->L1->L2->L3 ladder) and evict L3's tail to
	///     the ghost ring if L3 then overflows;
	///   - otherwise, allocate at L0's MRU end and evict L0's tail if L0
	///     overflows (the eviction is *not* recorded in the ghost ring).
	/// On a hit, refresh the value and run the same promotion path as `get`.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			// Resident key: refresh value in place, then promote one step.
			self.entry_mut(idx).value = value;
			self.touch(idx);
			return;
		}
		// Non-resident: check the ghost ring before falling through to L0.
		if !self.ghost.is_empty() && self.ghost_set.contains(&key) {
			self.ghost_remove(&key);
			let new_idx = self.alloc_slot(Entry {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				level: 3,
			});
			self.push_front(new_idx, 3);
			self.map.insert(key, new_idx);
			if self.l_len[3] > self.cap_for(3) {
				self.evict_l3_tail_to_ghost();
			}
			debug_assert!(self.map.len() <= self.capacity);
			return;
		}
		// Brand-new key: allocate a slot, insert at L0's head, evict L0's
		// tail if L0 now overflows its per-level cap.
		let new_idx = self.alloc_slot(Entry {
			key: key.clone(),
			value,
			prev: NIL,
			next: NIL,
			level: 0,
		});
		self.push_front(new_idx, 0);
		self.map.insert(key, new_idx);
		if self.l_len[0] > self.cap_for(0) {
			self.evict_l0_tail();
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Common promotion/move-to-MRU step for a resident entry. Used by both
	/// `get` and resident-key `put`.
	fn touch(&mut self, idx: usize) {
		let level = self.entry(idx).level as usize;
		if level + 1 < NUM_LEVELS {
			self.promote(idx, level);
		} else {
			// Already at the top level: pure LRU within L(N-1).
			self.unlink(idx, level);
			self.push_front(idx, level);
		}
	}

	/// Move `idx` from level `src` to the head of `src+1`. If `src+1`
	/// overflows its per-level cap, demote its tail back to the head of
	/// `src` so total residence is conserved. (Note: L3's overflow on
	/// in-ladder promotion still demotes to L2 — the ghost ring is only
	/// fed by the *direct-admit* eviction path in `put`.)
	fn promote(&mut self, idx: usize, src: usize) {
		debug_assert!(src + 1 < NUM_LEVELS);
		let dst = src + 1;
		self.unlink(idx, src);
		self.push_front(idx, dst);
		self.entry_mut(idx).level = dst as u8;
		if self.l_len[dst] > self.cap_for(dst) {
			let demoted = self.l_tail[dst];
			debug_assert!(demoted != NIL, "overflowing level must have a tail");
			self.unlink(demoted, dst);
			self.push_front(demoted, src);
			self.entry_mut(demoted).level = src as u8;
		}
	}

	/// True eviction: drop the L0 tail from both the map and its level,
	/// freeing the arena slot for the next brand-new insert. L0 evictions
	/// are *not* recorded in the ghost ring (per the hypothesis: the ring
	/// tracks only direct-from-L3 evictions).
	fn evict_l0_tail(&mut self) {
		let tail_idx = self.l_tail[0];
		debug_assert!(tail_idx != NIL, "overflowing L0 must have a tail");
		let evicted_key = self.entry(tail_idx).key.clone();
		self.unlink(tail_idx, 0);
		self.map.remove(&evicted_key);
		self.arena[tail_idx] = None;
		debug_assert!(self.free_head == NIL, "free chain expected empty before evict");
		self.free_head = tail_idx;
	}

	/// True eviction from L3 caused by a ghost-hit direct admission: drop
	/// L3's tail from both the map and the level, free the arena slot, and
	/// record the evicted key in the ghost ring so a near-future re-reference
	/// can shortcut back to L3.
	fn evict_l3_tail_to_ghost(&mut self) {
		let tail_idx = self.l_tail[3];
		debug_assert!(tail_idx != NIL, "overflowing L3 must have a tail");
		let evicted_key = self.entry(tail_idx).key.clone();
		self.unlink(tail_idx, 3);
		self.map.remove(&evicted_key);
		self.arena[tail_idx] = None;
		debug_assert!(self.free_head == NIL, "free chain expected empty before evict");
		self.free_head = tail_idx;
		self.ghost_insert(evicted_key);
	}

	/// Insert `key` at the ghost ring's write head, overwriting the oldest
	/// ghost entry there (if any). No-ops when the ghost ring has length 0
	/// (e.g. tiny capacities where `L3_cap == 0`). Keeps `ghost_set` in
	/// lockstep with the ring slots.
	fn ghost_insert(&mut self, key: K) {
		if self.ghost.is_empty() {
			return;
		}
		// `key` was just resident, and ghost-residency is mutually exclusive
		// with map-residency (a ghost hit removes from the ghost before
		// reinstating), so this defensive check should never fire — but
		// keeping it preserves the set/ring lockstep if logic upstream ever
		// changes.
		if self.ghost_set.contains(&key) {
			return;
		}
		let slot = self.ghost_head;
		if let Some(old_key) = self.ghost[slot].take() {
			self.ghost_set.remove(&old_key);
		}
		self.ghost[slot] = Some(key.clone());
		self.ghost_set.insert(key);
		self.ghost_head = (self.ghost_head + 1) % self.ghost.len();
	}

	/// Remove `key` from both the ghost set and its slot in the ghost ring.
	/// O(ghost_cap), but only invoked on ghost-hit `put`s (rare) and the
	/// ring is bounded by `capacity / 2`.
	fn ghost_remove(&mut self, key: &K) {
		if self.ghost.is_empty() {
			return;
		}
		if !self.ghost_set.remove(key) {
			return;
		}
		for slot in self.ghost.iter_mut() {
			if let Some(k) = slot {
				if k == key {
					*slot = None;
					return;
				}
			}
		}
	}

	/// Grab a slot for a new entry: reuse `free_head` if any, else grow.
	fn alloc_slot(&mut self, entry: Entry<K, V>) -> usize {
		if self.free_head != NIL {
			let idx = self.free_head;
			self.free_head = NIL;
			debug_assert!(self.arena[idx].is_none(), "free slot must be None");
			self.arena[idx] = Some(entry);
			idx
		} else {
			let idx = self.arena.len();
			self.arena.push(Some(entry));
			idx
		}
	}

	fn entry(&self, idx: usize) -> &Entry<K, V> {
		self.arena[idx].as_ref().expect("slot occupied")
	}

	fn entry_mut(&mut self, idx: usize) -> &mut Entry<K, V> {
		self.arena[idx].as_mut().expect("slot occupied")
	}

	/// Detach `idx` from its level's DLL. Leaves the arena slot occupied.
	fn unlink(&mut self, idx: usize, level: usize) {
		let prev = self.entry(idx).prev;
		let next = self.entry(idx).next;
		if prev != NIL {
			self.arena[prev].as_mut().expect("prev occupied").next = next;
		} else {
			self.l_head[level] = next;
		}
		if next != NIL {
			self.arena[next].as_mut().expect("next occupied").prev = prev;
		} else {
			self.l_tail[level] = prev;
		}
		let e = self.entry_mut(idx);
		e.prev = NIL;
		e.next = NIL;
		debug_assert!(self.l_len[level] > 0, "unlink underflow");
		self.l_len[level] -= 1;
	}

	/// Splice `idx` at the head (MRU end) of `level`'s DLL.
	fn push_front(&mut self, idx: usize, level: usize) {
		let old_head = self.l_head[level];
		{
			let e = self.entry_mut(idx);
			e.prev = NIL;
			e.next = old_head;
		}
		if old_head != NIL {
			self.arena[old_head].as_mut().expect("old head occupied").prev = idx;
		} else {
			// Empty list: this node is also the tail.
			self.l_tail[level] = idx;
		}
		self.l_head[level] = idx;
		self.l_len[level] += 1;
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
