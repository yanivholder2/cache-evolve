// Four-level segmented LRU (S4LRU) over an arena-backed doubly-linked list.
//
// Each of L0..L3 is its own doubly-linked list with `head` = MRU and
// `tail` = LRU, threaded through one shared `Vec<Option<Entry>>` arena.
// Brand-new keys enter at L0's head; a `get` hit on Li (i < 3) promotes the
// entry to L(i+1)'s head, and — if L(i+1) overflows its per-level cap —
// demotes L(i+1)'s tail back to Li's head. Eviction (a key actually leaving
// the cache) only ever happens at L0's tail when L0 overflows. Because
// every overflowing promotion is paired with a demotion to the source
// level, total residence is conserved across promote/demote, so the cache
// satisfies sum(l_len) <= sum(cap_per_level) = capacity at all times.
//
// Slot bookkeeping mirrors the baseline's arena-slot-reuse pattern: an L0
// eviction sets the freed slot to `None`, and the very next brand-new
// insert re-uses that slot. Because alloc+evict are paired inside one
// `put`-on-miss call, at any quiescent moment the free chain holds at most
// one slot — so a single-`usize` `free_head` (sentinel `NIL`) is enough.

use std::collections::HashMap;
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
	map: HashMap<K, usize>,            // key -> arena index
	arena: Vec<Option<Entry<K, V>>>,   // slots; `None` = free
	l_head: [usize; NUM_LEVELS],       // per-level MRU end (NIL when empty)
	l_tail: [usize; NUM_LEVELS],       // per-level LRU end (NIL when empty)
	l_len: [usize; NUM_LEVELS],        // per-level residency
	capacity: usize,                   // total residence budget
	free_head: usize,                  // free slot (NIL when none)
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			arena: Vec::with_capacity(capacity),
			l_head: [NIL; NUM_LEVELS],
			l_tail: [NIL; NUM_LEVELS],
			l_len: [0; NUM_LEVELS],
			capacity,
			free_head: NIL,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Per-level capacity: L0..L(N-2) get `capacity / N`, L(N-1) absorbs the
	/// remainder so the four caps sum to `capacity` exactly.
	fn cap_for(&self, level: usize) -> usize {
		let base = self.capacity / NUM_LEVELS;
		if level + 1 == NUM_LEVELS {
			self.capacity - base * (NUM_LEVELS - 1)
		} else {
			base
		}
	}

	/// Returns the value for `key`, promoting one step up the level ladder
	/// on a hit (or moving to head within the top level).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.entry(idx).value)
	}

	/// Inserts `key` at the head of L0 on a miss; on a hit, updates the
	/// value and runs the same promotion path as `get`.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			// Resident key: refresh value in place, then promote one step.
			self.entry_mut(idx).value = value;
			self.touch(idx);
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
	/// `src` so total residence is conserved.
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
	/// freeing the arena slot for the next brand-new insert.
	fn evict_l0_tail(&mut self) {
		let tail_idx = self.l_tail[0];
		debug_assert!(tail_idx != NIL, "overflowing L0 must have a tail");
		let evicted_key = self.entry(tail_idx).key.clone();
		self.unlink(tail_idx, 0);
		self.map.remove(&evicted_key);
		self.arena[tail_idx] = None;
		// alloc_slot drained free_head just above, so the chain is empty.
		debug_assert!(self.free_head == NIL, "free chain expected empty before evict");
		self.free_head = tail_idx;
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
