// Adaptive Replacement Cache (ARC) over a HashMap + arena-based doubly-linked-list.
//
// ARC maintains two resident LRU lists and two ghost LRU lists, plus a
// self-tuning target `p` for the size of T1:
//   T1 — resident keys seen once (recency).
//   T2 — resident keys seen 2+ times (frequency).
//   B1 — ghost keys recently evicted from T1 (key only, no value payload).
//   B2 — ghost keys recently evicted from T2 (key only, no value payload).
// Residents live in the `nodes` arena (key + value); ghosts live in a separate
// `ghosts` arena (key only) so they carry no value. Both arenas reuse slots
// in place via small free lists. No unsafe, no external crates.
//
// H02 tuning: the adaptation target `p` is clamped to a small ceiling
// (`capacity / 20`, i.e. ~5% of capacity) so the recency list T1 can never
// grow large enough to shrink the frequency list T2 below ~95% of capacity.
// This imports the winning 95/5 SLRU split into ARC's structure: `replace()`
// will almost always demote the T1 (recency) LRU rather than T2, so
// re-accessed keys accumulate in a large, protected T2 while ARC's
// ghost-driven promotion is retained.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// Resident list ids (index into the per-list head/tail/len arrays).
const T1: usize = 0;
const T2: usize = 1;
// Ghost list ids.
const B1: usize = 0;
const B2: usize = 1;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	list: usize, // which resident list (T1 or T2) this node is linked into
}

#[derive(Debug)]
struct Ghost<K> {
	key: K,
	prev: Option<usize>,
	next: Option<usize>,
	list: usize, // which ghost list (B1 or B2)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	// Residents: key -> arena index.
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	res_free: Vec<usize>,
	res_head: [Option<usize>; 2], // LRU end of each resident list
	res_tail: [Option<usize>; 2], // MRU end of each resident list
	res_len: [usize; 2],

	// Ghosts: key -> ghost-arena index.
	ghost_map: HashMap<K, usize>,
	ghosts: Vec<Ghost<K>>,
	ghost_free: Vec<usize>,
	ghost_head: [Option<usize>; 2],
	ghost_tail: [Option<usize>; 2],
	ghost_len: [usize; 2],

	capacity: usize,
	p: usize,     // adaptation target for |T1|, in [0, p_cap]
	p_cap: usize, // ceiling on `p`: ~5% of capacity (the incumbent's probationary budget)
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
		log::info!("LruCache.try_new: capacity={capacity} (ARC)");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			res_free: Vec::new(),
			res_head: [None, None],
			res_tail: [None, None],
			res_len: [0, 0],
			ghost_map: HashMap::with_capacity(capacity),
			ghosts: Vec::with_capacity(capacity),
			ghost_free: Vec::new(),
			ghost_head: [None, None],
			ghost_tail: [None, None],
			ghost_len: [0, 0],
			capacity,
			p: 0,
			p_cap: capacity / 20,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if it is resident, promoting it to T2 (the
	/// frequency list) most-recently-used. A ghost hit is still a miss here —
	/// ghosts carry no value; their adaptation effect happens on the next `put`.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.res_unlink(idx);
		self.res_push_mru(idx, T2);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key` following the canonical ARC cases.
	pub fn put(&mut self, key: K, value: V) {
		// Case I — already resident: update value, promote to T2 MRU.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.res_unlink(idx);
			self.res_push_mru(idx, T2);
			return;
		}

		// Case II/III — key is a ghost: adapt `p`, replace, admit into T2.
		if let Some(&gidx) = self.ghost_map.get(&key) {
			let glist = self.ghosts[gidx].list;
			if glist == B1 {
				// p = min(p + max(|B2|/|B1|, 1), p_cap)
				// H02: clamp to p_cap (~5% of capacity) instead of capacity so
				// the recency list T1 stays small and T2 (frequency) stays near
				// 95% of capacity, mirroring the 95/5 SLRU split.
				let b1 = self.ghost_len[B1].max(1);
				let delta = (self.ghost_len[B2] / b1).max(1);
				self.p = (self.p + delta).min(self.p_cap);
				self.remove_ghost(gidx);
				self.replace(false);
			} else {
				// p = max(p - max(|B1|/|B2|, 1), 0)
				let b2 = self.ghost_len[B2].max(1);
				let delta = (self.ghost_len[B1] / b2).max(1);
				self.p = self.p.saturating_sub(delta);
				self.remove_ghost(gidx);
				self.replace(true);
			}
			self.admit(key, value, T2);
			debug_assert!(self.res_len[T1] + self.res_len[T2] <= self.capacity);
			return;
		}

		// Case IV — pure miss on a wholly new key.
		let l1 = self.res_len[T1] + self.ghost_len[B1];
		if l1 == self.capacity {
			if self.res_len[T1] < self.capacity {
				// B1 non-empty: drop its LRU ghost, then replace.
				self.delete_lru_ghost(B1);
				self.replace(false);
			} else {
				// B1 empty and T1 saturated: evict T1 LRU from the cache (no ghost).
				self.delete_lru_resident(T1);
			}
		} else {
			let total =
				self.res_len[T1] + self.res_len[T2] + self.ghost_len[B1] + self.ghost_len[B2];
			if total >= self.capacity {
				if total == 2 * self.capacity {
					self.delete_lru_ghost(B2);
				}
				self.replace(false);
			}
		}
		self.admit(key, value, T1);
		debug_assert!(self.res_len[T1] + self.res_len[T2] <= self.capacity);
	}

	/// ARC REPLACE: make room for one admission by demoting a resident LRU to a
	/// ghost list. `in_b2` is true when the triggering key was found in B2.
	fn replace(&mut self, in_b2: bool) {
		let t1 = self.res_len[T1];
		if t1 >= 1 && (t1 > self.p || (in_b2 && t1 == self.p)) {
			self.demote_lru(T1, B1);
		} else if self.res_len[T2] >= 1 {
			self.demote_lru(T2, B2);
		} else if t1 >= 1 {
			// Degenerate fallback: only T1 has residents.
			self.demote_lru(T1, B1);
		}
	}

	/// Admit a fresh resident `(key, value)` into list `l` at the MRU end.
	fn admit(&mut self, key: K, value: V, l: usize) {
		let idx = self.alloc_res(key.clone(), value);
		self.res_push_mru(idx, l);
		self.map.insert(key, idx);
	}

	// ── Resident arena / list machinery ─────────────────────────────────────

	fn alloc_res(&mut self, key: K, value: V) -> usize {
		if let Some(idx) = self.res_free.pop() {
			self.nodes[idx].key = key;
			self.nodes[idx].value = value;
			self.nodes[idx].prev = None;
			self.nodes[idx].next = None;
			idx
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				prev: None,
				next: None,
				list: T1,
			});
			idx
		}
	}

	fn res_unlink(&mut self, idx: usize) {
		let l = self.nodes[idx].list;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.res_head[l] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.res_tail[l] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.res_len[l] -= 1;
	}

	fn res_push_mru(&mut self, idx: usize, l: usize) {
		self.nodes[idx].list = l;
		self.nodes[idx].prev = self.res_tail[l];
		self.nodes[idx].next = None;
		match self.res_tail[l] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.res_head[l] = Some(idx),
		}
		self.res_tail[l] = Some(idx);
		self.res_len[l] += 1;
	}

	/// Detach and return the LRU (head) resident of list `l`, if any.
	fn res_pop_lru(&mut self, l: usize) -> Option<usize> {
		let idx = self.res_head[l]?;
		self.res_unlink(idx);
		Some(idx)
	}

	/// Move the LRU resident of `res_list` into `ghost_list` (drops its value).
	fn demote_lru(&mut self, res_list: usize, ghost_list: usize) {
		let Some(idx) = self.res_pop_lru(res_list) else {
			return;
		};
		let key = self.nodes[idx].key.clone();
		self.map.remove(&key);
		self.res_free.push(idx);
		let gidx = self.alloc_ghost(key.clone());
		self.ghost_push_mru(gidx, ghost_list);
		self.ghost_map.insert(key, gidx);
	}

	/// Evict the LRU resident of `l` from the cache entirely (no ghost kept).
	fn delete_lru_resident(&mut self, l: usize) {
		if let Some(idx) = self.res_pop_lru(l) {
			let key = self.nodes[idx].key.clone();
			self.map.remove(&key);
			self.res_free.push(idx);
		}
	}

	// ── Ghost arena / list machinery ────────────────────────────────────────

	fn alloc_ghost(&mut self, key: K) -> usize {
		if let Some(idx) = self.ghost_free.pop() {
			self.ghosts[idx].key = key;
			self.ghosts[idx].prev = None;
			self.ghosts[idx].next = None;
			idx
		} else {
			let idx = self.ghosts.len();
			self.ghosts.push(Ghost {
				key,
				prev: None,
				next: None,
				list: B1,
			});
			idx
		}
	}

	fn ghost_unlink(&mut self, idx: usize) {
		let l = self.ghosts[idx].list;
		let prev = self.ghosts[idx].prev;
		let next = self.ghosts[idx].next;
		match prev {
			Some(p) => self.ghosts[p].next = next,
			None => self.ghost_head[l] = next,
		}
		match next {
			Some(n) => self.ghosts[n].prev = prev,
			None => self.ghost_tail[l] = prev,
		}
		self.ghosts[idx].prev = None;
		self.ghosts[idx].next = None;
		self.ghost_len[l] -= 1;
	}

	fn ghost_push_mru(&mut self, idx: usize, l: usize) {
		self.ghosts[idx].list = l;
		self.ghosts[idx].prev = self.ghost_tail[l];
		self.ghosts[idx].next = None;
		match self.ghost_tail[l] {
			Some(t) => self.ghosts[t].next = Some(idx),
			None => self.ghost_head[l] = Some(idx),
		}
		self.ghost_tail[l] = Some(idx);
		self.ghost_len[l] += 1;
	}

	/// Drop a specific ghost (e.g. one being promoted to a resident).
	fn remove_ghost(&mut self, gidx: usize) {
		self.ghost_unlink(gidx);
		let key = self.ghosts[gidx].key.clone();
		self.ghost_map.remove(&key);
		self.ghost_free.push(gidx);
	}

	/// Drop the LRU (head) ghost of list `l`, if any.
	fn delete_lru_ghost(&mut self, l: usize) {
		if let Some(gidx) = self.ghost_head[l] {
			self.ghost_unlink(gidx);
			let key = self.ghosts[gidx].key.clone();
			self.ghost_map.remove(&key);
			self.ghost_free.push(gidx);
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
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		// Should still have room for a second key without eviction
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promotes key 1 into T2 (frequency)
		cache.put(3, 30); // evicts key 2 (the demotable T1 LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // overwrite promotes key 1 into T2
		cache.put(3, 30); // evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // cache full
		cache.put(1, 11); // overwrite, must NOT evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1); // key 1 is now frequent (T2), protected from eviction
		}
		cache.put(4, 40); // should evict key 2 (T1 LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // T1 LRU
		cache.put(2, 20); // T1 middle
		cache.put(3, 30); // T1 MRU
		cache.get(&2); // promote key 2 into T2
		cache.put(4, 40); // evicts key 1 (T1 LRU)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn frequency_protection_after_promotion() {
		// ARC-specific: a re-accessed key migrates to T2 (frequency) and is
		// protected from the recency-driven eviction that strict LRU would apply.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // T1 saturated, key 1 evicted (no ghost yet)
		cache.get(&3); // key 3 promoted into T2 (frequency)
		cache.put(4, 40); // demotes T1 LRU (key 2), key 3 stays resident in T2
		cache.put(5, 50); // demotes T1 LRU (key 4); key 3 still protected in T2
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected by frequency list
		assert!(cache.get(&4).is_none()); // demoted to a ghost
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// One resident; get unlinks then re-pushes it into T2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify the cache is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite promotes key 3 into T2
		cache.put(4, 40); // demotes T1 LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
