// LIRS cache (H02): classifies residents by inter-reference recency.
//
// Substrate is the original HashMap<K, usize> + arena Vec<Node>, but the
// eviction policy is LIRS rather than strict LRU:
//   - Stack S (recency-ordered) holds LIR blocks plus HIR blocks (resident or
//     non-resident); it measures inter-reference recency.
//   - List Q holds resident-HIR blocks in recency order; its front is the
//     eviction victim.
//   - A non-resident HIR block keeps only metadata (a node in S, no value).
// Slots are managed with a small free list because LIRS adds and removes
// metadata-only entries dynamically. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
	Lir,
	HirResident,
	HirNonResident,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: Option<V>, // Some when resident, None for non-resident HIR metadata
	status: Status,
	// Stack S links: s_head is the top (MRU), s_tail the bottom (LRU).
	s_prev: Option<usize>, // toward head (newer)
	s_next: Option<usize>, // toward tail (older)
	in_s: bool,
	// List Q links: q_head is the front (LRU victim), q_tail the back (MRU).
	q_prev: Option<usize>,
	q_next: Option<usize>,
	in_q: bool,
	// Non-resident ordering list (front = oldest) for O(1) bounded reclaim.
	nr_prev: Option<usize>,
	nr_next: Option<usize>,
	in_nr: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index (resident and non-resident)
	nodes: Vec<Node<K, V>>, // arena
	free_slots: Vec<usize>, // reclaimed arena slots
	// Stack S
	s_head: Option<usize>,
	s_tail: Option<usize>,
	// List Q (resident HIR)
	q_head: Option<usize>,
	q_tail: Option<usize>,
	// Non-resident ordering list
	nr_head: Option<usize>,
	nr_tail: Option<usize>,
	capacity: usize,
	l_lir: usize, // LIR budget
	resident_count: usize,
	lir_count: usize,
	nonresident_count: usize,
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
		// Reserve a ~5% resident-HIR budget (at least one block), mirroring the
		// proven SLRU 95/5 probationary floor; the rest is the LIR budget. A
		// wider HIR drain window catches genuine reuses before eviction.
		let l_hir = std::cmp::max(1, capacity / 20);
		let l_lir = capacity.saturating_sub(l_hir);
		log::info!("LruCache.try_new: capacity={capacity} l_lir={l_lir} l_hir={l_hir}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free_slots: Vec::new(),
			s_head: None,
			s_tail: None,
			q_head: None,
			q_tail: None,
			nr_head: None,
			nr_tail: None,
			capacity,
			l_lir,
			resident_count: 0,
			lir_count: 0,
			nonresident_count: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if it is resident, updating recency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		match self.nodes[idx].status {
			Status::HirNonResident => None, // metadata only: a miss
			_ => {
				self.touch_resident(idx);
				self.nodes[idx].value.as_ref()
			}
		}
	}

	/// Inserts or updates `key`. The framework only calls this for a
	/// non-resident key; the resident branch exists for direct unit/JNI use.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			match self.nodes[idx].status {
				Status::Lir | Status::HirResident => {
					// Resident update: refresh value, then touch for recency.
					self.nodes[idx].value = Some(value);
					self.touch_resident(idx);
					return;
				}
				Status::HirNonResident => {
					// Short reuse distance: re-admit (promote) this block.
					self.insert_reuse(idx, value);
					return;
				}
			}
		}
		self.insert_new(key, value);
	}

	// ── Access of a resident block ──────────────────────────────────────────

	fn touch_resident(&mut self, idx: usize) {
		match self.nodes[idx].status {
			Status::Lir => {
				let was_bottom = self.s_tail == Some(idx);
				self.s_move_top(idx);
				if was_bottom {
					self.prune_stack();
				}
			}
			Status::HirResident => {
				if self.l_lir > 0 && self.nodes[idx].in_s {
					// Promote HIR-resident to LIR; demote the bottom LIR.
					self.s_move_top(idx);
					self.nodes[idx].status = Status::Lir;
					self.lir_count += 1;
					if self.nodes[idx].in_q {
						self.q_unlink(idx);
					}
					self.enforce_lir_budget();
				} else {
					// Recency too large (or no LIR budget): stay HIR.
					self.s_move_top(idx);
					self.q_move_back(idx);
				}
			}
			Status::HirNonResident => {
				debug_assert!(false, "touch_resident on non-resident block");
			}
		}
	}

	// ── Insertion paths ─────────────────────────────────────────────────────

	fn insert_reuse(&mut self, idx: usize, value: V) {
		debug_assert_eq!(self.nodes[idx].status, Status::HirNonResident);
		debug_assert!(self.nodes[idx].in_s);
		// The block becomes resident again.
		self.nr_unlink(idx);
		self.nodes[idx].value = Some(value);
		self.nonresident_count -= 1;
		self.resident_count += 1;
		if self.l_lir > 0 {
			self.s_move_top(idx);
			self.nodes[idx].status = Status::Lir;
			self.lir_count += 1;
			self.enforce_lir_budget();
		} else {
			self.nodes[idx].status = Status::HirResident;
			self.s_move_top(idx);
			self.q_move_back(idx);
		}
		if self.resident_count > self.capacity {
			self.evict_one();
		}
		debug_assert!(self.resident_count <= self.capacity);
	}

	fn insert_new(&mut self, key: K, value: V) {
		let idx = self.alloc_node(key.clone(), Some(value));
		self.map.insert(key, idx);
		self.resident_count += 1;
		if self.l_lir > 0 && self.lir_count < self.l_lir {
			// Warm-up: fill the LIR set first.
			self.nodes[idx].status = Status::Lir;
			self.lir_count += 1;
			self.s_move_top(idx);
		} else {
			self.nodes[idx].status = Status::HirResident;
			self.s_move_top(idx);
			self.q_move_back(idx);
		}
		if self.resident_count > self.capacity {
			self.evict_one();
		}
		debug_assert!(self.resident_count <= self.capacity);
	}

	/// Evict the LRU resident-HIR block (front of Q).
	fn evict_one(&mut self) {
		let victim = self.q_head.expect("Q must be non-empty when evicting");
		self.q_unlink(victim);
		self.nodes[victim].value = None;
		self.resident_count -= 1;
		if self.nodes[victim].in_s {
			// Keep as non-resident metadata so a near-future reuse is detected.
			self.nodes[victim].status = Status::HirNonResident;
			self.nonresident_count += 1;
			self.nr_push_back(victim);
		} else {
			self.remove_node(victim);
		}
		// Bound non-resident metadata so memory stays O(capacity).
		while self.nonresident_count > self.capacity {
			self.reclaim_oldest_nonresident();
		}
	}

	/// Keep the LIR set within budget by demoting the bottom LIR block(s).
	fn enforce_lir_budget(&mut self) {
		self.prune_stack();
		while self.lir_count > self.l_lir {
			let bottom = self.s_tail.expect("LIR present implies non-empty S");
			debug_assert_eq!(self.nodes[bottom].status, Status::Lir);
			self.s_unlink(bottom);
			self.nodes[bottom].status = Status::HirResident;
			self.lir_count -= 1;
			self.q_move_back(bottom);
			self.prune_stack();
		}
	}

	/// Remove trailing non-LIR blocks from the bottom of S so its bottom is LIR.
	fn prune_stack(&mut self) {
		while let Some(bottom) = self.s_tail {
			match self.nodes[bottom].status {
				Status::Lir => break,
				Status::HirResident => {
					// Still resident in Q; just leaves the recency stack.
					self.s_unlink(bottom);
				}
				Status::HirNonResident => {
					self.s_unlink(bottom);
					self.nr_unlink(bottom);
					self.nonresident_count -= 1;
					self.remove_node(bottom);
				}
			}
		}
	}

	fn reclaim_oldest_nonresident(&mut self) {
		let oldest = self.nr_head.expect("non-resident present");
		debug_assert_eq!(self.nodes[oldest].status, Status::HirNonResident);
		self.nr_unlink(oldest);
		if self.nodes[oldest].in_s {
			self.s_unlink(oldest);
		}
		self.nonresident_count -= 1;
		self.remove_node(oldest);
	}

	// ── Arena management ────────────────────────────────────────────────────

	fn alloc_node(&mut self, key: K, value: Option<V>) -> usize {
		let node = Node {
			key,
			value,
			status: Status::HirResident,
			s_prev: None,
			s_next: None,
			in_s: false,
			q_prev: None,
			q_next: None,
			in_q: false,
			nr_prev: None,
			nr_next: None,
			in_nr: false,
		};
		if let Some(slot) = self.free_slots.pop() {
			self.nodes[slot] = node;
			slot
		} else {
			let idx = self.nodes.len();
			self.nodes.push(node);
			idx
		}
	}

	/// Forget a node entirely. Caller has already detached it from S, Q and NR
	/// and adjusted the relevant counts.
	fn remove_node(&mut self, idx: usize) {
		debug_assert!(!self.nodes[idx].in_s);
		debug_assert!(!self.nodes[idx].in_q);
		debug_assert!(!self.nodes[idx].in_nr);
		let key = self.nodes[idx].key.clone();
		self.map.remove(&key);
		self.nodes[idx].value = None;
		self.free_slots.push(idx);
	}

	// ── Stack S links ───────────────────────────────────────────────────────

	fn s_push_top(&mut self, idx: usize) {
		self.nodes[idx].s_prev = None;
		self.nodes[idx].s_next = self.s_head;
		match self.s_head {
			Some(h) => self.nodes[h].s_prev = Some(idx),
			None => self.s_tail = Some(idx),
		}
		self.s_head = Some(idx);
		self.nodes[idx].in_s = true;
	}

	fn s_unlink(&mut self, idx: usize) {
		if !self.nodes[idx].in_s {
			return;
		}
		let prev = self.nodes[idx].s_prev;
		let next = self.nodes[idx].s_next;
		match prev {
			Some(p) => self.nodes[p].s_next = next,
			None => self.s_head = next,
		}
		match next {
			Some(n) => self.nodes[n].s_prev = prev,
			None => self.s_tail = prev,
		}
		self.nodes[idx].s_prev = None;
		self.nodes[idx].s_next = None;
		self.nodes[idx].in_s = false;
	}

	fn s_move_top(&mut self, idx: usize) {
		self.s_unlink(idx);
		self.s_push_top(idx);
	}

	// ── List Q links ────────────────────────────────────────────────────────

	fn q_push_back(&mut self, idx: usize) {
		self.nodes[idx].q_next = None;
		self.nodes[idx].q_prev = self.q_tail;
		match self.q_tail {
			Some(t) => self.nodes[t].q_next = Some(idx),
			None => self.q_head = Some(idx),
		}
		self.q_tail = Some(idx);
		self.nodes[idx].in_q = true;
	}

	fn q_unlink(&mut self, idx: usize) {
		if !self.nodes[idx].in_q {
			return;
		}
		let prev = self.nodes[idx].q_prev;
		let next = self.nodes[idx].q_next;
		match prev {
			Some(p) => self.nodes[p].q_next = next,
			None => self.q_head = next,
		}
		match next {
			Some(n) => self.nodes[n].q_prev = prev,
			None => self.q_tail = prev,
		}
		self.nodes[idx].q_prev = None;
		self.nodes[idx].q_next = None;
		self.nodes[idx].in_q = false;
	}

	fn q_move_back(&mut self, idx: usize) {
		self.q_unlink(idx);
		self.q_push_back(idx);
	}

	// ── Non-resident ordering links ─────────────────────────────────────────

	fn nr_push_back(&mut self, idx: usize) {
		self.nodes[idx].nr_next = None;
		self.nodes[idx].nr_prev = self.nr_tail;
		match self.nr_tail {
			Some(t) => self.nodes[t].nr_next = Some(idx),
			None => self.nr_head = Some(idx),
		}
		self.nr_tail = Some(idx);
		self.nodes[idx].in_nr = true;
	}

	fn nr_unlink(&mut self, idx: usize) {
		if !self.nodes[idx].in_nr {
			return;
		}
		let prev = self.nodes[idx].nr_prev;
		let next = self.nodes[idx].nr_next;
		match prev {
			Some(p) => self.nodes[p].nr_next = next,
			None => self.nr_head = next,
		}
		match next {
			Some(n) => self.nodes[n].nr_prev = prev,
			None => self.nr_tail = prev,
		}
		self.nodes[idx].nr_prev = None;
		self.nodes[idx].nr_next = None;
		self.nodes[idx].in_nr = false;
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
	fn resident_keys_within_capacity() {
		// LIRS does not preserve strict-LRU eviction order, but residency must
		// never exceed capacity and resident values must read back faithfully.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..200i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		let resident = (0..200i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
		for k in 0..200i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10, "value mismatch for key {k}");
			}
		}
	}

	#[test]
	fn never_put_key_reads_none() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for phantom in [1000i64, 2000, -1] {
			assert!(cache.get(&phantom).is_none());
		}
	}

	#[test]
	fn capacity_one_holds_single_resident() {
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
	fn reuse_is_re_admitted() {
		// A block with a short reuse distance should be promoted on its second
		// reference and survive a flood of one-shot keys.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		cache.put(7, 70); // warm-up LIR
		assert_eq!(*cache.get(&7).unwrap(), 70);
		for k in 100..200i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			// keep touching 7 so it remains a low-IRR LIR block
			cache.get(&7);
		}
		assert_eq!(*cache.get(&7).unwrap(), 70, "frequently reused block should survive");
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
