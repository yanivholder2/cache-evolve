// O(1) Segmented LRU (strict 80/20 SLRU) on the baseline arena-list.
//
// Two segments — probationary (LRU end = head) and protected (LRU end = head) —
// share a single `nodes: Vec<Node<K, V>>` arena and a single `map: HashMap<K, usize>`.
// Capacity split is fixed: prot_capacity = (capacity * 4) / 5, prob_capacity = the
// remainder. Both segments hold at least one slot, except when capacity == 1 the
// cache falls back to a single probationary slot with prot_capacity = 0.
//
// Slots are still reused in place on eviction (no free-list management): the
// arena is monotonically grown until full, then the probationary head's slot is
// reused for the new entry — the same trick the baseline used, just confined to
// the probationary segment so promotions never lose their slot.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const PROB: u8 = 0;
const PROT: u8 = 1;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena

	prob_head: Option<usize>, // probationary LRU (front)
	prob_tail: Option<usize>, // probationary MRU (back)
	prot_head: Option<usize>, // protected LRU (front)
	prot_tail: Option<usize>, // protected MRU (back)

	prob_len: usize,
	prot_len: usize,

	prob_capacity: usize,
	prot_capacity: usize,

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
		// Strict 80/20 split; both segments hold at least one slot for capacity >= 2.
		// At capacity == 1 we degenerate to a single probationary slot.
		let (prob_capacity, prot_capacity) = if capacity == 1 {
			(1usize, 0usize)
		} else {
			let prot = ((capacity * 4) / 5).max(1);
			let prob = (capacity - prot).max(1);
			(prob, prot)
		};
		log::info!(
			"LruCache.try_new: capacity={capacity} prob_capacity={prob_capacity} prot_capacity={prot_capacity}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prot_head: None,
			prot_tail: None,
			prob_len: 0,
			prot_len: 0,
			prob_capacity,
			prot_capacity,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, applying SLRU touch semantics:
	/// probationary hit -> promote to protected (possibly demoting the protected
	/// head back into probationary); protected hit -> re-pin at protected MRU.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.touch_resident(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. Eviction (when needed) always comes from the
	/// probationary segment — protected entries are scan-resistant.
	pub fn put(&mut self, key: K, value: V) {
		// Key already present: update value in place, then apply the same touch
		// logic a `get` hit would have run for whichever segment it lives in.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch_resident(idx);
			return;
		}

		// New key — need an arena slot. Eviction (if any) must come from
		// probationary; only allocate a fresh slot when the arena is not full.
		let idx = if self.prob_capacity > 0 && self.prob_len >= self.prob_capacity {
			self.evict_prob_head_and_reuse(key.clone(), value)
		} else if self.nodes.len() >= self.capacity {
			// Arena full but probationary has room.
			//
			// With the SLRU invariant prot_len <= prot_capacity, this branch is
			// only reachable in degenerate configurations (e.g. capacity == 1
			// where prot_capacity == 0 is impossible to overflow into); keep it
			// defensive so the cache never returns from `put` without inserting.
			if self.prob_head.is_some() {
				self.evict_prob_head_and_reuse(key.clone(), value)
			} else {
				self.evict_prot_head_and_reuse(key.clone(), value)
			}
		} else {
			// Arena has room: allocate a new slot at the probationary tail.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				segment: PROB,
			});
			idx
		};

		// Link the new/reused slot at the probationary tail (MRU) and register.
		self.push_back(idx, PROB);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert!(self.prob_len <= self.prob_capacity.max(1));
		debug_assert!(self.prot_len <= self.prot_capacity);
	}

	/// SLRU hit handling for a resident node already in `self.map`.
	fn touch_resident(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment;
		if seg == PROB {
			// Promote prob -> prot tail.
			self.unlink(idx);
			self.push_back(idx, PROT);
			// If protected overflows, demote its LRU end back into probationary.
			if self.prot_len > self.prot_capacity {
				let demote = self
					.prot_head
					.expect("prot_len > 0 implies prot_head is Some");
				self.unlink(demote);
				self.push_back(demote, PROB);
			}
		} else {
			// Already protected: refresh MRU position within the protected list.
			self.unlink(idx);
			self.push_back(idx, PROT);
		}
	}

	/// Evict the probationary head, removing it from `map`, and return its
	/// arena slot index for in-place reuse by the new entry's key/value.
	/// The returned slot is detached from any list (segment field untouched);
	/// the caller links it back into a segment via `push_back`.
	fn evict_prob_head_and_reuse(&mut self, new_key: K, new_value: V) -> usize {
		let head_idx = self
			.prob_head
			.expect("evict_prob_head_and_reuse: prob list must be non-empty");
		let evicted_key = self.nodes[head_idx].key.clone();
		self.map.remove(&evicted_key);
		self.unlink(head_idx);
		self.nodes[head_idx].key = new_key;
		self.nodes[head_idx].value = new_value;
		head_idx
	}

	/// Defensive fallback: evict the protected head when probationary is empty
	/// yet the arena is full. Under the SLRU invariants this is unreachable for
	/// capacity >= 2, but the implementation must still terminate gracefully.
	fn evict_prot_head_and_reuse(&mut self, new_key: K, new_value: V) -> usize {
		let head_idx = self
			.prot_head
			.expect("evict_prot_head_and_reuse: prot list must be non-empty");
		let evicted_key = self.nodes[head_idx].key.clone();
		self.map.remove(&evicted_key);
		self.unlink(head_idx);
		self.nodes[head_idx].key = new_key;
		self.nodes[head_idx].value = new_value;
		head_idx
	}

	/// Detach a node from its current segment's list, updating the corresponding
	/// head/tail/len for that segment. The node's segment field is left as-is
	/// (the next `push_back` will overwrite it).
	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;

		if let Some(p) = prev {
			self.nodes[p].next = next;
		} else {
			match seg {
				PROB => self.prob_head = next,
				_ => self.prot_head = next,
			}
		}
		if let Some(n) = next {
			self.nodes[n].prev = prev;
		} else {
			match seg {
				PROB => self.prob_tail = prev,
				_ => self.prot_tail = prev,
			}
		}

		match seg {
			PROB => self.prob_len -= 1,
			_ => self.prot_len -= 1,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append a node at the MRU tail of `seg`, setting its segment field and
	/// updating that segment's head/tail/len. The node must currently be
	/// detached (prev/next == None) — i.e. fresh, just-unlinked, or just-evicted.
	fn push_back(&mut self, idx: usize, seg: u8) {
		self.nodes[idx].segment = seg;
		let tail = match seg {
			PROB => self.prob_tail,
			_ => self.prot_tail,
		};
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		if let Some(t) = tail {
			self.nodes[t].next = Some(idx);
		} else {
			match seg {
				PROB => self.prob_head = Some(idx),
				_ => self.prot_head = Some(idx),
			}
		}
		match seg {
			PROB => self.prob_tail = Some(idx),
			_ => self.prot_tail = Some(idx),
		}
		match seg {
			PROB => self.prob_len += 1,
			_ => self.prot_len += 1,
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
	// Most of the baseline's unit tests assert pure-LRU eviction order, which
	// does not hold under SLRU. The gates that decide pass/fail are the
	// integration tests under `tests/` (contract_gate, universal_invariant_gate,
	// invariant_gate). These local tests cover only properties that survive the
	// LRU -> SLRU change.
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
	fn capacity_one_round_trips() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1 (only one slot, which is probationary)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn slru_protected_survives_probationary_pressure() {
		// capacity=8 -> prob_cap=2, prot_cap=6. Promote key 1 into protected,
		// then flood probationary with scan-only keys; key 1 must survive.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11); // promote 1 to protected
		for k in 100..200 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		// Key 1 lives in protected; the scan only churns probationary.
		assert_eq!(*cache.get(&1).unwrap(), 11);
	}
}
