// O(1)-amortised LIRS cache (Jiang & Zhang, 2002) behind the original `LruCache` surface.
//
// Storage: a `Vec<Node>` arena indexed by `u32` with intrusive links for two lists:
//   * stack S (recency stack; holds LIR, resident HIR and non-resident HIR entries),
//   * queue Q (resident HIR entries, front = next eviction victim).
// A `HashMap<K, u32>` indexes every node, resident or not. Reclaimed slots go to a
// free list. `u32::MAX` is the null link. No unsafe, no external crates.
//
// Sizing: hir_cap = max(1, capacity / 20), lir_cap = capacity - hir_cap. Non-resident
// HIR entries kept in S are bounded at 2 * capacity.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
	Lir,
	HirResident,
	HirNonResident,
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	/// `None` means non-resident HIR (or a freed slot).
	value: Option<V>,
	status: Status,
	in_s: bool,
	in_q: bool,
	/// Stack S links: `s_prev` points toward the top (newer), `s_next` toward the bottom (older).
	s_prev: u32,
	s_next: u32,
	/// Queue Q links: `q_prev` toward the front (older), `q_next` toward the tail (newer).
	q_prev: u32,
	q_next: u32,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index (resident and non-resident)
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<u32>,         // reclaimed arena slots
	s_top: u32,
	s_bottom: u32,
	q_head: u32, // front: eviction victim
	q_tail: u32,
	capacity: usize,
	lir_cap: usize,
	lir_count: usize,
	q_len: usize,
	nonres_count: usize,
	nonres_cap: usize,
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
		// All nodes (≤ capacity residents + ≤ 2*capacity non-residents + 1 transient) must be
		// addressable by u32 with NIL reserved.
		if capacity > (u32::MAX as usize - 2) / 4 {
			log::error!("LruCache.try_new: capacity {capacity} too large");
			return None;
		}
		// capacity == 1 keeps hir_cap = 1, lir_cap = 0 (as before); for capacity > 1,
		// capacity / 20 is always <= capacity - 1, so lir_cap stays >= 1.
		let hir_cap = std::cmp::max(1, capacity / 20);
		let lir_cap = capacity - hir_cap;
		let nonres_cap = 2 * capacity;
		log::info!("LruCache.try_new (LIRS): capacity={capacity} lir_cap={lir_cap} hir_cap={hir_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity + nonres_cap + 1),
			nodes: Vec::with_capacity(capacity + nonres_cap + 1),
			free: Vec::new(),
			s_top: NIL,
			s_bottom: NIL,
			q_head: NIL,
			q_tail: NIL,
			capacity,
			lir_cap,
			lir_count: 0,
			q_len: 0,
			nonres_count: 0,
			nonres_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident, applying the LIRS access logic.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)?;
		if self.nodes[idx as usize].status == Status::HirNonResident {
			return None;
		}
		self.access(idx);
		self.nodes[idx as usize].value.as_ref()
	}

	/// Inserts or updates `key`. On a miss with a full cache, evicts the front of Q.
	pub fn put(&mut self, key: K, value: V) {
		// Existing resident key: update value and treat as an access.
		if let Some(&idx) = self.map.get(&key) {
			if self.nodes[idx as usize].status != Status::HirNonResident {
				self.nodes[idx as usize].value = Some(value);
				self.access(idx);
				return;
			}
		}

		// Warm-up: LIR set not yet full -> admit as LIR.
		if self.lir_count < self.lir_cap && self.resident_count() < self.capacity {
			match self.map.get(&key).copied() {
				Some(idx) => {
					// Non-resident node still in S: revive as LIR.
					self.s_remove(idx);
					let n = &mut self.nodes[idx as usize];
					n.value = Some(value);
					n.status = Status::Lir;
					self.nonres_count -= 1;
					self.lir_count += 1;
					self.s_push_top(idx);
				}
				None => {
					let idx = self.alloc(key, value, Status::Lir);
					self.lir_count += 1;
					self.s_push_top(idx);
				}
			}
			self.enforce_nonres_bound();
			debug_assert!(self.resident_count() <= self.capacity);
			return;
		}

		// Make room: evict the front of Q.
		if self.resident_count() >= self.capacity {
			self.evict_q_front();
		}

		match self.map.get(&key).copied() {
			Some(idx) => {
				// Non-resident HIR in S: promote to LIR, demote the bottom LIR.
				debug_assert_eq!(self.nodes[idx as usize].status, Status::HirNonResident);
				self.s_remove(idx);
				let n = &mut self.nodes[idx as usize];
				n.value = Some(value);
				n.status = Status::Lir;
				self.nonres_count -= 1;
				self.lir_count += 1;
				self.s_push_top(idx);
				self.demote_bottom_lir();
			}
			None => {
				let idx = self.alloc(key, value, Status::HirResident);
				self.s_push_top(idx);
				self.q_push_tail(idx);
			}
		}
		self.enforce_nonres_bound();
		debug_assert!(self.resident_count() <= self.capacity);
	}

	#[inline]
	fn resident_count(&self) -> usize {
		self.lir_count + self.q_len
	}

	/// LIRS hit handling for a resident node.
	fn access(&mut self, idx: u32) {
		let status = self.nodes[idx as usize].status;
		match status {
			Status::Lir => {
				let was_bottom = self.s_bottom == idx;
				self.s_remove(idx);
				self.s_push_top(idx);
				if was_bottom {
					self.prune_stack();
				}
			}
			Status::HirResident => {
				if self.nodes[idx as usize].in_s {
					// Short IRR: promote to LIR.
					self.s_remove(idx);
					self.q_remove(idx);
					self.nodes[idx as usize].status = Status::Lir;
					self.lir_count += 1;
					self.s_push_top(idx);
					self.demote_bottom_lir();
				} else {
					self.s_push_top(idx);
					self.q_remove(idx);
					self.q_push_tail(idx);
				}
			}
			Status::HirNonResident => {}
		}
	}

	/// Move the bottom LIR of S to the tail of Q as a resident HIR, then prune S.
	fn demote_bottom_lir(&mut self) {
		self.prune_stack();
		let b = self.s_bottom;
		if b == NIL {
			return;
		}
		debug_assert_eq!(self.nodes[b as usize].status, Status::Lir);
		self.s_remove(b);
		self.nodes[b as usize].status = Status::HirResident;
		self.lir_count -= 1;
		self.q_push_tail(b);
		self.prune_stack();
	}

	/// Pop non-LIR nodes off the bottom of S. Non-resident ones are freed; resident
	/// HIRs stay in Q.
	fn prune_stack(&mut self) {
		while self.s_bottom != NIL {
			let b = self.s_bottom;
			let status = self.nodes[b as usize].status;
			match status {
				Status::Lir => break,
				Status::HirResident => self.s_remove(b),
				Status::HirNonResident => {
					self.s_remove(b);
					self.nonres_count -= 1;
					self.free_node(b);
				}
			}
		}
	}

	/// Evict the front of Q: becomes non-resident if still in S, otherwise freed.
	fn evict_q_front(&mut self) {
		let f = self.q_head;
		if f == NIL {
			return;
		}
		self.q_remove(f);
		let n = &mut self.nodes[f as usize];
		n.value = None;
		if n.in_s {
			n.status = Status::HirNonResident;
			self.nonres_count += 1;
		} else {
			self.free_node(f);
		}
	}

	/// Bound non-resident entries in S: drop the oldest ones (bottom-scan of S).
	fn enforce_nonres_bound(&mut self) {
		while self.nonres_count > self.nonres_cap {
			let mut cur = self.s_bottom;
			while cur != NIL && self.nodes[cur as usize].status != Status::HirNonResident {
				cur = self.nodes[cur as usize].s_prev;
			}
			if cur == NIL {
				break;
			}
			self.s_remove(cur);
			self.nonres_count -= 1;
			self.free_node(cur);
		}
	}

	fn alloc(&mut self, key: K, value: V, status: Status) -> u32 {
		let node = Node {
			key: key.clone(),
			value: Some(value),
			status,
			in_s: false,
			in_q: false,
			s_prev: NIL,
			s_next: NIL,
			q_prev: NIL,
			q_next: NIL,
		};
		let idx = match self.free.pop() {
			Some(i) => {
				self.nodes[i as usize] = node;
				i
			}
			None => {
				let i = self.nodes.len() as u32;
				self.nodes.push(node);
				i
			}
		};
		self.map.insert(key, idx);
		idx
	}

	/// Remove a node (already unlinked from S and Q) from the index and reclaim its slot.
	fn free_node(&mut self, idx: u32) {
		debug_assert!(!self.nodes[idx as usize].in_s && !self.nodes[idx as usize].in_q);
		let key = self.nodes[idx as usize].key.clone();
		self.map.remove(&key);
		self.nodes[idx as usize].value = None;
		self.free.push(idx);
	}

	// ── stack S ──────────────────────────────────────────────────────────────

	fn s_push_top(&mut self, idx: u32) {
		debug_assert!(!self.nodes[idx as usize].in_s);
		let old_top = self.s_top;
		{
			let n = &mut self.nodes[idx as usize];
			n.s_prev = NIL;
			n.s_next = old_top;
			n.in_s = true;
		}
		if old_top != NIL {
			self.nodes[old_top as usize].s_prev = idx;
		} else {
			self.s_bottom = idx;
		}
		self.s_top = idx;
	}

	fn s_remove(&mut self, idx: u32) {
		if !self.nodes[idx as usize].in_s {
			return;
		}
		let (p, n) = {
			let node = &self.nodes[idx as usize];
			(node.s_prev, node.s_next)
		};
		if p != NIL {
			self.nodes[p as usize].s_next = n;
		} else {
			self.s_top = n;
		}
		if n != NIL {
			self.nodes[n as usize].s_prev = p;
		} else {
			self.s_bottom = p;
		}
		let node = &mut self.nodes[idx as usize];
		node.s_prev = NIL;
		node.s_next = NIL;
		node.in_s = false;
	}

	// ── queue Q ──────────────────────────────────────────────────────────────

	fn q_push_tail(&mut self, idx: u32) {
		debug_assert!(!self.nodes[idx as usize].in_q);
		let old_tail = self.q_tail;
		{
			let n = &mut self.nodes[idx as usize];
			n.q_prev = old_tail;
			n.q_next = NIL;
			n.in_q = true;
		}
		if old_tail != NIL {
			self.nodes[old_tail as usize].q_next = idx;
		} else {
			self.q_head = idx;
		}
		self.q_tail = idx;
		self.q_len += 1;
	}

	fn q_remove(&mut self, idx: u32) {
		if !self.nodes[idx as usize].in_q {
			return;
		}
		let (p, n) = {
			let node = &self.nodes[idx as usize];
			(node.q_prev, node.q_next)
		};
		if p != NIL {
			self.nodes[p as usize].q_next = n;
		} else {
			self.q_head = n;
		}
		if n != NIL {
			self.nodes[n as usize].q_prev = p;
		} else {
			self.q_tail = p;
		}
		let node = &mut self.nodes[idx as usize];
		node.q_prev = NIL;
		node.q_next = NIL;
		node.in_q = false;
		self.q_len -= 1;
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

	fn get_or_put(cache: &mut LruCache<i64, i64>, k: i64) -> bool {
		if cache.get(&k).is_some() {
			true
		} else {
			cache.put(k, k);
			false
		}
	}

	fn resident_keys(cache: &mut LruCache<i64, i64>, range: std::ops::Range<i64>) -> usize {
		range.filter(|k| cache.get(k).is_some()).count()
	}

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
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
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
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn new_keys_evict_resident_hir_not_lir() {
		// capacity 3: lir_cap 2, hir_cap 1.
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LIR
		cache.put(2, 20); // LIR
		cache.put(3, 30); // resident HIR
		cache.put(4, 40); // evicts 3 (HIR), not LIR 1
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn non_resident_reuse_promotes_to_lir() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LIR
		cache.put(2, 20); // LIR
		cache.put(3, 30); // HIR
		cache.put(4, 40); // evict 3 -> non-resident in S
		assert!(cache.get(&3).is_none());
		cache.put(3, 31); // evict 4; 3 promoted to LIR, bottom LIR 1 demoted to HIR
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert!(cache.get(&4).is_none());
	}

	#[test]
	fn hir_cap_is_five_percent() {
		let cache = LruCache::<i64, i64>::new(2048);
		assert_eq!(cache.lir_cap, 2048 - 102);
		let cache = LruCache::<i64, i64>::new(1);
		assert_eq!(cache.lir_cap, 0);
		let cache = LruCache::<i64, i64>::new(10);
		assert_eq!(cache.lir_cap, 9);
	}

	#[test]
	fn loop_of_capacity_plus_one_mostly_hits() {
		// Strict LRU gets zero hits on a cyclic loop of capacity+1 keys.
		let cap = 100usize;
		let mut cache = LruCache::new(cap);
		let n = cap as i64 + 1;
		for k in 0..n {
			get_or_put(&mut cache, k);
		}
		let rounds = 20;
		let mut hits = 0usize;
		for _ in 0..rounds {
			for k in 0..n {
				if get_or_put(&mut cache, k) {
					hits += 1;
				}
			}
		}
		let total = rounds * n as usize;
		assert!(hits * 2 >= total, "loop hits {hits}/{total} too low for LIRS");
		assert!(resident_keys(&mut cache, 0..n) <= cap);
	}

	#[test]
	fn scan_does_not_evict_lir_entries() {
		let cap = 10usize; // hir_cap = max(1, 10 / 20) = 1, lir_cap 9
		let mut cache = LruCache::new(cap);
		for k in 0..cap as i64 {
			get_or_put(&mut cache, k);
		}
		for k in 0..9i64 {
			assert!(get_or_put(&mut cache, k), "warm key {k} should hit");
		}
		for k in 1000..1500i64 {
			get_or_put(&mut cache, k);
		}
		for k in 0..9i64 {
			assert!(cache.get(&k).is_some(), "LIR key {k} evicted by scan");
		}
	}

	#[test]
	fn residency_and_nonresident_bound_hold_under_random_workload() {
		for &cap in &[1usize, 2, 3, 8, 50, 250] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 0x9E3779B97F4A7C15;
			for _ in 0..20_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let k = (x % (cap as u64 * 4 + 3)) as i64;
				if let Some(&v) = cache.get(&k) {
					assert_eq!(v, k);
				} else {
					cache.put(k, k);
				}
				assert!(cache.resident_count() <= cap);
				assert!(cache.nonres_count <= cache.nonres_cap);
				assert!(cache.lir_count <= cache.lir_cap.max(1));
				if cache.s_bottom != NIL && cache.lir_count > 0 {
					assert_eq!(cache.nodes[cache.s_bottom as usize].status, Status::Lir);
				}
			}
			let resident = resident_keys(&mut cache, 0..(cap as i64 * 4 + 3));
			assert!(resident <= cap);
		}
	}
}
