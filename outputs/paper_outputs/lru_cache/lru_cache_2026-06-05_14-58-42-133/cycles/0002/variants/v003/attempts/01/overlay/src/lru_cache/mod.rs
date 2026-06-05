// Adaptive Replacement Cache (ARC) over an arena + HashMap, keeping the
// baseline's public surface (`LruCache<K, V>` with try_new/new/get/put) so the
// protected JNI shim and invariant suite are untouched.
//
// ARC self-tunes a recency/frequency target `p` using two resident lists
// (T1 = seen-once, T2 = seen-twice-or-more) and two value-less ghost lists
// (B1, B2) that remember recently evicted keys. Residents live in the `nodes`
// arena and reuse slots in place via a small free-list; ghosts live in a
// separate arena and only store keys. No unsafe, no external crates, O(1) ops.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// Resident list ids (index into the res_* arrays).
const T1: usize = 0;
const T2: usize = 1;
// Ghost list ids (index into the ghost_* arrays).
const B1: usize = 0;
const B2: usize = 1;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	list: usize, // T1 or T2
}

#[derive(Debug)]
struct Ghost<K> {
	key: K,
	prev: Option<usize>,
	next: Option<usize>,
	list: usize, // B1 or B2
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	// Resident store: key -> arena index.
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>, // freed resident arena slots, reused in place

	// Ghost store: key -> ghost arena index (value-less).
	ghost_map: HashMap<K, usize>,
	ghosts: Vec<Ghost<K>>,
	ghost_free: Vec<usize>,

	// head = LRU end, tail = MRU end. Indexed by list id.
	res_head: [Option<usize>; 2],
	res_tail: [Option<usize>; 2],
	res_len: [usize; 2],
	ghost_head: [Option<usize>; 2],
	ghost_tail: [Option<usize>; 2],
	ghost_len: [usize; 2],

	p: usize, // adaptive target size for T1, in 0..=capacity
	capacity: usize,
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
		log::info!("LruCache.try_new: capacity={capacity} policy=arc");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			ghost_map: HashMap::with_capacity(capacity),
			ghosts: Vec::with_capacity(capacity),
			ghost_free: Vec::new(),
			res_head: [None, None],
			res_tail: [None, None],
			res_len: [0, 0],
			ghost_head: [None, None],
			ghost_tail: [None, None],
			ghost_len: [0, 0],
			p: 0,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident, promoting it to the MRU end of
	/// T2 (a referenced page becomes "frequent"). Ghost-only keys read as miss.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		// Promote: detach from whichever resident list it is in, push to T2 MRU.
		self.res_detach(idx);
		self.res_push_back(idx, T2);
		Some(&self.nodes[idx].value)
	}

	/// Inserts `key`. Per the get-miss-then-put contract `key` is non-resident;
	/// it may, however, be a ghost (B1/B2) which drives ARC's adaptation.
	pub fn put(&mut self, key: K, value: V) {
		let c = self.capacity;

		// Defensive: if somehow already resident, update value + promote to T2.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.res_detach(idx);
			self.res_push_back(idx, T2);
			return;
		}

		match self.ghost_map.get(&key).map(|&g| self.ghosts[g].list) {
			Some(list) if list == B1 => {
				// Case II: ghost hit in B1 -> favour recency, grow p.
				let b1 = self.ghost_len[B1].max(1);
				let b2 = self.ghost_len[B2];
				let delta = (b2 / b1).max(1);
				self.p = (self.p + delta).min(c);
				self.replace(false);
				self.remove_ghost(&key);
				self.alloc_resident(key, value, T2);
			}
			Some(_) => {
				// Case III: ghost hit in B2 -> favour frequency, shrink p.
				let b1 = self.ghost_len[B1];
				let b2 = self.ghost_len[B2].max(1);
				let delta = (b1 / b2).max(1);
				self.p = self.p.saturating_sub(delta);
				self.replace(true);
				self.remove_ghost(&key);
				self.alloc_resident(key, value, T2);
			}
			None => {
				// Case IV: full miss. Enforce L1/L2 size bounds, then REPLACE.
				let l1 = self.res_len[T1] + self.ghost_len[B1];
				if l1 == c {
					if self.res_len[T1] < c {
						// B1 non-empty: drop its LRU ghost, then replace.
						self.evict_ghost_lru(B1);
						self.replace(false);
					} else if let Some(idx) = self.res_head[T1] {
						// T1 already full (B1 empty): evict T1 LRU resident outright.
						let evicted = self.nodes[idx].key.clone();
						self.res_detach(idx);
						self.map.remove(&evicted);
						self.free.push(idx);
					}
				} else {
					let total = self.res_len[T1]
						+ self.res_len[T2]
						+ self.ghost_len[B1]
						+ self.ghost_len[B2];
					if total >= c {
						if total == 2 * c {
							self.evict_ghost_lru(B2);
						}
						self.replace(false);
					}
				}
				self.alloc_resident(key, value, T1);
			}
		}

		debug_assert!(self.map.len() <= self.capacity);
	}

	/// ARC REPLACE(p): move one resident's slot to a ghost list, choosing T1 vs
	/// T2 by the adaptive target. `in_b2` is true when the triggering request
	/// was a B2 ghost hit (breaks the tie toward evicting T1).
	fn replace(&mut self, in_b2: bool) {
		let p = self.p;
		let t1 = self.res_len[T1];
		if t1 > 0 && (t1 > p || (in_b2 && t1 == p)) {
			let idx = self.res_head[T1].expect("non-empty T1 has a head");
			self.evict_resident_to_ghost(idx, B1);
		} else if self.res_len[T2] > 0 {
			let idx = self.res_head[T2].expect("non-empty T2 has a head");
			self.evict_resident_to_ghost(idx, B2);
		} else if t1 > 0 {
			// Degenerate fallback (T2 empty): still free a resident from T1.
			let idx = self.res_head[T1].expect("non-empty T1 has a head");
			self.evict_resident_to_ghost(idx, B1);
		}
		// else: no residents to evict (e.g. all keys currently ghosts).
	}

	/// Move a resident node to the MRU end of a ghost list, freeing its slot.
	fn evict_resident_to_ghost(&mut self, idx: usize, ghost_list: usize) {
		let key = self.nodes[idx].key.clone();
		self.res_detach(idx);
		self.map.remove(&key);
		self.free.push(idx);
		self.add_ghost(key, ghost_list);
	}

	/// Allocate a resident slot (reusing a freed one in place when available)
	/// and link it at the MRU end of `list`.
	fn alloc_resident(&mut self, key: K, value: V, list: usize) {
		let idx = if let Some(i) = self.free.pop() {
			self.nodes[i].key = key.clone();
			self.nodes[i].value = value;
			i
		} else {
			let i = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				list,
			});
			i
		};
		self.res_push_back(idx, list);
		self.map.insert(key, idx);
	}

	/// Detach a resident node from its current list (reading its `list` tag).
	fn res_detach(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let l = self.nodes[idx].list;
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

	/// Append a resident node to the MRU (tail) end of `list`, tagging it.
	fn res_push_back(&mut self, idx: usize, list: usize) {
		self.nodes[idx].list = list;
		self.nodes[idx].prev = self.res_tail[list];
		self.nodes[idx].next = None;
		match self.res_tail[list] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.res_head[list] = Some(idx),
		}
		self.res_tail[list] = Some(idx);
		self.res_len[list] += 1;
	}

	/// Insert a fresh ghost key at the MRU end of `list`.
	fn add_ghost(&mut self, key: K, list: usize) {
		let gidx = if let Some(g) = self.ghost_free.pop() {
			self.ghosts[g].key = key.clone();
			g
		} else {
			let g = self.ghosts.len();
			self.ghosts.push(Ghost {
				key: key.clone(),
				prev: None,
				next: None,
				list,
			});
			g
		};
		self.ghost_push_back(gidx, list);
		self.ghost_map.insert(key, gidx);
	}

	/// Remove a specific ghost key (e.g. when it is promoted to resident).
	fn remove_ghost(&mut self, key: &K) {
		if let Some(gidx) = self.ghost_map.remove(key) {
			self.ghost_detach(gidx);
			self.ghost_free.push(gidx);
		}
	}

	/// Drop the LRU ghost of `list` to keep ARC's size bounds.
	fn evict_ghost_lru(&mut self, list: usize) {
		if let Some(gidx) = self.ghost_head[list] {
			let key = self.ghosts[gidx].key.clone();
			self.ghost_detach(gidx);
			self.ghost_map.remove(&key);
			self.ghost_free.push(gidx);
		}
	}

	fn ghost_detach(&mut self, gidx: usize) {
		let prev = self.ghosts[gidx].prev;
		let next = self.ghosts[gidx].next;
		let l = self.ghosts[gidx].list;
		match prev {
			Some(p) => self.ghosts[p].next = next,
			None => self.ghost_head[l] = next,
		}
		match next {
			Some(n) => self.ghosts[n].prev = prev,
			None => self.ghost_tail[l] = prev,
		}
		self.ghosts[gidx].prev = None;
		self.ghosts[gidx].next = None;
		self.ghost_len[l] -= 1;
	}

	fn ghost_push_back(&mut self, gidx: usize, list: usize) {
		self.ghosts[gidx].list = list;
		self.ghosts[gidx].prev = self.ghost_tail[list];
		self.ghosts[gidx].next = None;
		match self.ghost_tail[list] {
			Some(t) => self.ghosts[t].next = Some(gidx),
			None => self.ghost_head[list] = Some(gidx),
		}
		self.ghost_tail[list] = Some(gidx);
		self.ghost_len[list] += 1;
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
	fn capacity_bound_holds_under_churn() {
		let cap = 4;
		let mut cache = LruCache::new(cap);
		for k in 0i64..100 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0i64..100).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= cap, "{resident} residents > capacity {cap}");
	}

	#[test]
	fn frequent_keys_survive_recency_scans() {
		// A repeatedly reused key should out-survive a one-shot scan stream.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		for _ in 0..3 {
			cache.get(&1); // promote 1 into the frequent list T2
		}
		// Stream of cold misses; ARC should protect the frequent key.
		for k in 100i64..110 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
