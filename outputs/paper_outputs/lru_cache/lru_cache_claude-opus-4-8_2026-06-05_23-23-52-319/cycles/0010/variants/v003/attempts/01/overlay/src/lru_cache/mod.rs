// 2Q-admission-front hybrid over the converged 95/5 SLRU incumbent.
//
// Variant H03 grafts 2Q's one-hit-wonder admission filter onto the proven 95/5
// incumbent (cycle/0008/v001). The protected segment and its single-hit
// promotion are kept exactly as in the incumbent; only the probationary floor is
// replaced by a 2Q admission front:
//
//   * A1in — a FIFO *admission queue* sized `a1in_cap = capacity - protected_cap`
//     (~5% of capacity). Brand-new keys enter at its MRU/tail. A1in is never
//     reordered on a get-hit: on a hit an A1in key is promoted straight to the
//     protected segment MRU (single-hit promotion, unchanged). It ages strictly
//     by insertion order.
//   * A1out — a key-only *ghost* FIFO (nodes whose `value` is semantically empty
//     and are never returned by `get`), target size `kout = capacity / 2`,
//     allocated beyond `capacity` and tracked in a separate `ghost_map`. When a
//     `put` admits a new key and A1in is over budget, A1in's FIFO head is *paged*
//     into A1out instead of being evicted outright; A1out is then trimmed to
//     `kout` by dropping its oldest ghost.
//   * On a `put` miss whose key is an A1out ghost, the key is promoted directly
//     into the protected segment MRU (proven reuse across a full admission-queue
//     lifetime), then `enforce_protected_budget` runs.
//   * `enforce_protected_budget` still demotes the over-budget protected LRU node,
//     but now demotes it to the A1in tail (FIFO) rather than to a probationary
//     LRU; the demotion may in turn page an A1in head into A1out. No reprieve flag
//     is needed because A1in does not reorder.
//
// Invariants preserved: resident count `a1in_len + prot_len <= capacity`,
// `prot_len <= protected_cap`, `a1in_len <= a1in_cap`, `ghost_len <= kout`, ghost
// nodes hold no live value and are never returned by `get`, slot reuse via an
// explicit free list, and the `framework::Cache` / JNI surface unchanged. No
// unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Which structure a node currently belongs to. A node is in exactly one of
/// these at a time; `Free` marks a slot sitting on the free list.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
	A1in,
	Protected,
	Ghost,
	Free,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	// Live for resident nodes (A1in / Protected); semantically empty for Ghost
	// and Free slots, where it is never read.
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	loc: Loc,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	// Resident key -> node index (A1in + Protected only).
	map: HashMap<K, usize>,
	// Ghost key -> node index (A1out only). Disjoint from `map`.
	ghost_map: HashMap<K, usize>,
	// Shared arena holding residents and ghosts.
	nodes: Vec<Node<K, V>>,
	// Indices of slots not currently in any list, available for reuse.
	free_list: Vec<usize>,

	// A1in admission queue (FIFO): head = oldest (next to page out), tail = newest.
	a1in_head: Option<usize>,
	a1in_tail: Option<usize>,
	a1in_len: usize,

	// Protected segment (LRU): head = LRU, tail = MRU.
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	prot_len: usize,

	// A1out ghost queue (FIFO): head = oldest ghost, tail = newest ghost.
	ghost_head: Option<usize>,
	ghost_tail: Option<usize>,
	ghost_len: usize,

	capacity: usize,
	// ~95% of capacity (floored).
	protected_cap: usize,
	// Remainder of capacity, so the two resident budgets sum to exactly
	// `capacity`. Always >= 1 for capacity >= 1.
	a1in_cap: usize,
	// Ghost budget (capacity / 2); 0 disables ghosts entirely.
	kout: usize,
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
		// floor(0.95 * capacity); A1in budget is the remainder, so the two
		// resident budgets sum to exactly `capacity`.
		let protected_cap = (capacity * 19) / 20;
		let a1in_cap = capacity - protected_cap;
		let kout = capacity / 2;
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} a1in_cap={a1in_cap} kout={kout}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			ghost_map: HashMap::with_capacity(kout),
			nodes: Vec::with_capacity(capacity + kout),
			free_list: Vec::new(),
			a1in_head: None,
			a1in_tail: None,
			a1in_len: 0,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			ghost_head: None,
			ghost_tail: None,
			ghost_len: 0,
			capacity,
			protected_cap,
			a1in_cap,
			kout,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if it is resident, promoting it to the
	/// protected segment (MRU end). Ghost keys are not resident and read as a
	/// miss; A1in is never reordered — a hit promotes straight to protected.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		// A hit earns protection (single-hit promotion): detach from its current
		// resident segment and re-attach at the protected MRU end.
		self.unlink(idx);
		self.push_back_protected(idx);
		// Keep the protected segment within budget; any demotion lands at the
		// A1in tail (and may page an A1in head into A1out). Net resident count is
		// unchanged, so `idx` stays resident and its value is valid.
		self.enforce_protected_budget();
		Some(&self.nodes[idx].value)
	}

	/// Inserts `key`. Per the call contract `put` is only invoked for a
	/// non-resident key (after a get-miss); the resident branch is kept only so
	/// the standalone unit/integration suites that overwrite keys still behave.
	pub fn put(&mut self, key: K, value: V) {
		// Resident key: update value in place and treat as an access (promote to
		// protected MRU). No new slot, no eviction.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back_protected(idx);
			self.enforce_protected_budget();
			return;
		}
		// Ghost hit: the key was admitted, aged out of A1in, and is now reused
		// within its ghost lifetime — promote it directly into protected.
		if let Some(&gidx) = self.ghost_map.get(&key) {
			self.unlink(gidx);
			self.ghost_map.remove(&key);
			self.nodes[gidx].value = value;
			self.push_back_protected(gidx);
			self.map.insert(key, gidx);
			// Adding a protected resident may exceed the protected budget (and,
			// transitively, the A1in budget); enforcement restores both and pages
			// the displaced A1in head into A1out, keeping residents <= capacity.
			self.enforce_protected_budget();
			debug_assert!(self.a1in_len + self.prot_len <= self.capacity);
			return;
		}
		// Brand-new key: admit at the A1in (FIFO) tail.
		let idx = self.alloc_slot(key.clone(), value);
		self.push_back_a1in(idx);
		self.map.insert(key, idx);
		// If A1in is now over budget, page its FIFO head into A1out.
		self.enforce_a1in_budget();
		debug_assert!(self.a1in_len + self.prot_len <= self.capacity);
		debug_assert!(self.prot_len <= self.protected_cap);
		debug_assert!(self.a1in_len <= self.a1in_cap);
		debug_assert!(self.ghost_len <= self.kout);
		debug_assert_eq!(self.a1in_len + self.prot_len, self.map.len());
	}

	/// Acquire an arena slot for a new resident, reusing a freed slot when one is
	/// available. The returned slot is detached (`loc == Free`); the caller links
	/// it into a segment.
	fn alloc_slot(&mut self, key: K, value: V) -> usize {
		if let Some(idx) = self.free_list.pop() {
			self.nodes[idx].key = key;
			self.nodes[idx].value = value;
			self.nodes[idx].prev = None;
			self.nodes[idx].next = None;
			self.nodes[idx].loc = Loc::Free;
			idx
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				prev: None,
				next: None,
				loc: Loc::Free,
			});
			idx
		}
	}

	/// Return a now-unused slot to the free list. The node must already be
	/// unlinked from every segment.
	fn free_slot(&mut self, idx: usize) {
		self.nodes[idx].loc = Loc::Free;
		self.free_list.push(idx);
	}

	/// Detach a node from whichever list it currently occupies, fixing that
	/// list's head/tail and length.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let loc = self.nodes[idx].loc;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => match loc {
				Loc::A1in => self.a1in_head = next,
				Loc::Protected => self.prot_head = next,
				Loc::Ghost => self.ghost_head = next,
				Loc::Free => unreachable!("unlink of a free slot"),
			},
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => match loc {
				Loc::A1in => self.a1in_tail = prev,
				Loc::Protected => self.prot_tail = prev,
				Loc::Ghost => self.ghost_tail = prev,
				Loc::Free => unreachable!("unlink of a free slot"),
			},
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		match loc {
			Loc::A1in => self.a1in_len -= 1,
			Loc::Protected => self.prot_len -= 1,
			Loc::Ghost => self.ghost_len -= 1,
			Loc::Free => unreachable!("unlink of a free slot"),
		}
	}

	/// Append a node at the A1in FIFO tail (MRU end).
	fn push_back_a1in(&mut self, idx: usize) {
		self.nodes[idx].loc = Loc::A1in;
		self.nodes[idx].prev = self.a1in_tail;
		self.nodes[idx].next = None;
		match self.a1in_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.a1in_head = Some(idx),
		}
		self.a1in_tail = Some(idx);
		self.a1in_len += 1;
	}

	/// Append a node at the protected MRU (tail) end.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].loc = Loc::Protected;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
		self.prot_len += 1;
	}

	/// Append a node at the A1out ghost FIFO tail (newest ghost).
	fn push_back_ghost(&mut self, idx: usize) {
		self.nodes[idx].loc = Loc::Ghost;
		self.nodes[idx].prev = self.ghost_tail;
		self.nodes[idx].next = None;
		match self.ghost_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.ghost_head = Some(idx),
		}
		self.ghost_tail = Some(idx);
		self.ghost_len += 1;
	}

	/// While A1in is over budget, page its FIFO head into A1out (or evict it
	/// outright when ghosts are disabled), then trim A1out to `kout`.
	fn enforce_a1in_budget(&mut self) {
		while self.a1in_len > self.a1in_cap {
			let head = self
				.a1in_head
				.expect("A1in non-empty while over budget");
			self.unlink(head);
			let evicted_key = self.nodes[head].key.clone();
			self.map.remove(&evicted_key);
			if self.kout > 0 {
				// Page into the ghost FIFO (key-only; value left in place but never
				// read), then trim to the ghost budget.
				self.push_back_ghost(head);
				self.ghost_map.insert(evicted_key, head);
				self.trim_ghosts();
			} else {
				// Ghosts disabled: evict outright and reclaim the slot.
				self.free_slot(head);
			}
		}
	}

	/// Drop oldest ghosts until A1out is within `kout`, reclaiming their slots.
	fn trim_ghosts(&mut self) {
		while self.ghost_len > self.kout {
			let head = self
				.ghost_head
				.expect("A1out non-empty while over budget");
			self.unlink(head);
			let ghost_key = self.nodes[head].key.clone();
			self.ghost_map.remove(&ghost_key);
			self.free_slot(head);
		}
	}

	/// If the protected segment exceeds its budget, demote its LRU node to the
	/// A1in tail (FIFO re-entry). The demotion may push A1in over budget, which
	/// pages an A1in head into A1out — keeping the resident count bounded.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.protected_cap {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_back_a1in(lru);
				self.enforce_a1in_budget();
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
	fn capacity_one_evicts_old() {
		// capacity 1: protected_cap=0, a1in_cap=1, kout=0 (no ghosts).
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // displaces key 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn ghost_reuse_promotes_to_protected_and_survives_churn() {
		// capacity 4: protected_cap=3, a1in_cap=1, kout=2.
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // A1in={4}; ghosts={2,3} (1 trimmed)
		// Key 3 is a ghost: a get reads a miss, then the put promotes it into the
		// protected segment.
		assert!(cache.get(&3).is_none());
		cache.put(3, 33);
		assert_eq!(*cache.get(&3).unwrap(), 33);
		// Churn brand-new keys through the tiny admission window; protected 3
		// survives.
		cache.put(5, 50);
		cache.put(6, 60);
		cache.put(7, 70);
		assert_eq!(*cache.get(&3).unwrap(), 33);
	}

	#[test]
	fn never_put_key_reads_none_after_workload() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		for phantom in [1000i64, 2000, -1, -50] {
			assert!(cache.get(&phantom).is_none());
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
