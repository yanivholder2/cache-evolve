// O(1) SIEVE cache (Zhang et al., NSDI 2024) using HashMap + arena-based singly-linked list,
// plus a direct-mapped ghost fingerprint table of recently evicted keys.
//
// Single FIFO queue: new keys are appended at the tail (newest end). Each node carries a
// `visited` bit set on hit; hits never relink. On eviction a "hand" walks from the oldest
// end towards the newest, clearing visited bits, and evicts the first unvisited node.
// Surviving (visited) nodes stay in place (lazy promotion); new keys are behind the hand's
// direction of travel, so one-touch keys get demoted quickly (quick demotion).
//
// Ghost table: every evicted key (always unvisited at eviction) leaves a 32-bit fingerprint
// in a power-of-two, direct-mapped `Vec<u32>` (0 = empty). A newly inserted key whose
// fingerprint is found there was misclassified as one-hit, so it is admitted pre-marked
// visited (one extra pass of protection) and the ghost slot is cleared. The table lives on
// `LruCache`, not `Node`, so nodes stay 24 bytes. Hits never touch it.
//
// Nodes are only ever removed at the hand and appended at the tail, so no back-links are
// needed: the hand carries its own predecessor (`hand_prev`) as it walks forward. Links are
// `u32` with `NIL = u32::MAX`, so `Node<u64, u64>` fits in 24 bytes. Evicted slots are reused
// in place, so no free-list management is needed. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	next: u32,
	visited: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: u32,              // oldest (front)
	tail: u32,              // newest (back)
	hand: u32,              // SIEVE hand: NIL means "start at head"
	hand_prev: u32,         // predecessor of `hand`; NIL when hand is at head or NIL
	capacity: usize,
	ghost: Vec<u32>,   // direct-mapped fingerprints of evicted keys; 0 = empty
	ghost_mask: usize, // ghost.len() - 1 (len is a power of two)
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
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity={capacity} exceeds u32 link range");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		let ghost_len = capacity.next_power_of_two();
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			hand: NIL,
			hand_prev: NIL,
			capacity,
			ghost: vec![0u32; ghost_len],
			ghost_mask: ghost_len - 1,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, marking it visited (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx];
		node.visited = true;
		Some(&node.value)
	}

	/// Hash of `key` under the map's own hasher (no extra crates).
	#[inline]
	fn ghost_hash(&self, key: &K) -> u64 {
		self.map.hasher().hash_one(key)
	}

	/// (slot, fingerprint) for a hash; fingerprint is never 0.
	#[inline]
	fn ghost_slot_fp(&self, h: u64) -> (usize, u32) {
		((h as usize) & self.ghost_mask, ((h >> 32) as u32) | 1)
	}

	/// If `key` is in the ghost table, consume the entry and return true.
	#[inline]
	fn ghost_take(&mut self, key: &K) -> bool {
		let (slot, fp) = self.ghost_slot_fp(self.ghost_hash(key));
		if self.ghost[slot] == fp {
			self.ghost[slot] = 0;
			true
		} else {
			false
		}
	}

	/// Inserts or updates `key`. If at capacity, evicts via the SIEVE hand.
	pub fn put(&mut self, key: K, value: V) {
		// Existing key: update in place and mark visited; no relinking.
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.visited = true;
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let (victim, victim_prev) = self.find_victim();
			let v = victim as u32;
			let nxt = self.nodes[victim].next;
			// Unlink the victim using the predecessor the hand carried.
			if victim_prev == NIL {
				self.head = nxt;
			} else {
				self.nodes[victim_prev as usize].next = nxt;
			}
			if v == self.tail {
				self.tail = victim_prev;
			}
			// The hand resumes after the victim; NIL means restart at head next time.
			self.hand = nxt;
			self.hand_prev = if nxt == NIL { NIL } else { victim_prev };
			let evicted_key = self.nodes[victim].key.clone();
			// Record the (unvisited) victim in the ghost table, overwriting the slot.
			let (gslot, gfp) = self.ghost_slot_fp(self.ghost_hash(&evicted_key));
			self.ghost[gslot] = gfp;
			self.map.remove(&evicted_key);
			let visited = self.ghost_take(&key);
			let node = &mut self.nodes[victim];
			node.key = key.clone();
			node.value = value;
			node.next = NIL;
			node.visited = visited;
			victim
		} else {
			let visited = self.ghost_take(&key);
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				next: NIL,
				visited,
			});
			idx
		};
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Walk the hand from its position (or head) towards the newest end, clearing visited
	/// bits and wrapping to head, until an unvisited node is found. Returns the victim and
	/// its predecessor (NIL if the victim is head). Terminates within `capacity + 1` steps
	/// since one full pass clears every bit.
	fn find_victim(&mut self) -> (usize, u32) {
		debug_assert!(self.head != NIL, "head must be linked when arena is full");
		let (mut h, mut p) = if self.hand == NIL {
			(self.head, NIL)
		} else {
			(self.hand, self.hand_prev)
		};
		loop {
			let node = &mut self.nodes[h as usize];
			if !node.visited {
				return (h as usize, p);
			}
			node.visited = false;
			p = h;
			h = node.next;
			if h == NIL {
				h = self.head;
				p = NIL;
			}
		}
	}

	/// Append node to the tail (newest end) of the list.
	fn push_back(&mut self, idx: usize) {
		let i = idx as u32;
		self.nodes[idx].next = NIL;
		if self.tail == NIL {
			self.head = i;
		} else {
			self.nodes[self.tail as usize].next = i;
		}
		self.tail = i;
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

	/// Walk from head via `next`, verifying links, map agreement, tail, and that the hand and
	/// its predecessor are consistent.
	fn check_structure(c: &LruCache<i64, i64>) {
		assert!(c.map.len() <= c.capacity);
		assert_eq!(c.map.len(), c.nodes.len());
		assert!(c.ghost.len().is_power_of_two());
		assert_eq!(c.ghost_mask, c.ghost.len() - 1);
		let mut seen = vec![false; c.nodes.len()];
		let mut prev = NIL;
		let mut cur = c.head;
		let mut count = 0usize;
		let mut hand_linked = c.hand == NIL;
		while cur != NIL {
			let i = cur as usize;
			assert!(i < c.nodes.len(), "link out of range");
			assert!(!seen[i], "cycle in list");
			seen[i] = true;
			let n = &c.nodes[i];
			assert_eq!(c.map.get(&n.key), Some(&i), "map disagrees for slot {i}");
			if cur == c.hand {
				hand_linked = true;
				assert_eq!(c.hand_prev, prev, "hand_prev is not the hand's predecessor");
			}
			prev = cur;
			cur = n.next;
			count += 1;
		}
		assert_eq!(c.tail, prev, "tail mismatch");
		assert_eq!(count, c.map.len(), "list length != map length");
		assert!(hand_linked, "hand points at an unlinked slot");
		if c.hand != NIL {
			if c.hand_prev == NIL {
				assert_eq!(c.hand, c.head, "hand_prev NIL but hand is not head");
			} else {
				assert!(seen[c.hand_prev as usize], "hand_prev not linked");
				assert_eq!(c.nodes[c.hand_prev as usize].next, c.hand);
			}
		} else {
			assert_eq!(c.hand_prev, NIL, "hand NIL but hand_prev set");
		}
	}

	fn keys_in_order(c: &LruCache<i64, i64>) -> Vec<i64> {
		let mut out = Vec::new();
		let mut cur = c.head;
		while cur != NIL {
			out.push(c.nodes[cur as usize].key);
			cur = c.nodes[cur as usize].next;
		}
		out
	}

	#[test]
	fn node_is_24_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 24);
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
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn unvisited_oldest_is_evicted_fifo() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // nothing visited: evicts oldest (1)
		assert!(!cache.map.contains_key(&1));
		assert_eq!(keys_in_order(&cache), vec![2, 3, 4]);
		check_structure(&cache);
	}

	#[test]
	fn visited_key_survives_eviction_without_moving() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1); // mark visited, no relink
		assert_eq!(keys_in_order(&cache), vec![1, 2, 3]);
		cache.put(4, 40); // hand clears 1, evicts 2
		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&2));
		// 1 stayed at the oldest position; new key appended at newest end.
		assert_eq!(keys_in_order(&cache), vec![1, 3, 4]);
		assert!(!cache.nodes[cache.map[&1]].visited, "hand must clear visited bit");
		check_structure(&cache);
	}

	#[test]
	fn hand_resumes_after_victim() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.put(4, 40); // evicts 2; hand now at 3
		assert_eq!(cache.hand, cache.map[&3] as u32);
		assert_eq!(cache.hand_prev, cache.map[&1] as u32);
		cache.put(5, 50); // hand starts at 3 (unvisited): evicts 3, not 1
		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&3));
		assert_eq!(keys_in_order(&cache), vec![1, 4, 5]);
		assert_eq!(cache.hand, cache.map[&4] as u32);
		check_structure(&cache);
	}

	#[test]
	fn hand_wraps_to_head() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // evicts 1 (unvisited head); hand -> 2
		cache.get(&4);
		// hand at 2 (visited) -> clear, 3 (visited) -> clear, 4 (visited) -> clear, wrap
		// to head 2 (now unvisited) -> evict 2.
		cache.put(5, 50);
		assert!(!cache.map.contains_key(&2));
		assert_eq!(keys_in_order(&cache), vec![3, 4, 5]);
		check_structure(&cache);
	}

	#[test]
	fn scan_of_unique_keys_does_not_evict_visited_key() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		cache.put(1, 10);
		cache.get(&1);
		for k in 100..1100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert_eq!(cache.get(&1).copied(), Some(10), "visited key evicted by scan at {k}");
		}
		check_structure(&cache);
	}

	#[test]
	fn capacity_one_hand_resets() {
		let mut cache: LruCache<i64, i64> = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10); // visited
		cache.put(2, 20); // hand clears 1, wraps to head (1), evicts it
		assert_eq!(cache.hand, NIL, "victim.next is NIL so hand resets");
		assert_eq!(cache.hand_prev, NIL);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
		cache.put(3, 30);
		assert_eq!(cache.hand, NIL);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		check_structure(&cache);
	}

	#[test]
	fn ghost_hit_readmits_visited_and_survives_next_eviction() {
		let mut cache: LruCache<i64, i64> = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts 1 into the ghost table
		assert!(!cache.map.contains_key(&1));
		let (slot, fp) = cache.ghost_slot_fp(cache.ghost_hash(&1));
		assert_eq!(cache.ghost[slot], fp, "evicted key must leave a fingerprint");
		cache.put(1, 11); // evicts 2 (unvisited); 1 is a ghost hit
		assert!(cache.nodes[cache.map[&1]].visited, "ghost hit must pre-mark visited");
		assert_ne!(cache.ghost[slot], fp, "ghost entry must be consumed");
		assert_eq!(keys_in_order(&cache), vec![3, 1]);
		check_structure(&cache);
		cache.put(4, 40); // hand at 3 (unvisited): evicts 3
		assert!(cache.map.contains_key(&1));
		check_structure(&cache);
		cache.put(5, 50); // hand at 1 (visited): skipped and cleared, evicts 4
		assert!(cache.map.contains_key(&1), "pre-visited key must survive the sweep");
		assert!(!cache.map.contains_key(&4));
		assert!(!cache.nodes[cache.map[&1]].visited);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		check_structure(&cache);
	}

	#[test]
	fn randomized_workload_preserves_structure() {
		for &cap in &[1usize, 2, 3, 7, 16] {
			let mut cache: LruCache<i64, i64> = LruCache::new(cap);
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ cap as u64;
			for _ in 0..20_000 {
				state ^= state << 13;
				state ^= state >> 7;
				state ^= state << 17;
				let key = (state % 48) as i64;
				match cache.get(&key) {
					Some(&v) => assert_eq!(v, key * 3),
					None => cache.put(key, key * 3),
				}
				check_structure(&cache);
			}
			for phantom in [1000i64, -1] {
				assert!(cache.get(&phantom).is_none());
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}

	#[test]
	fn capacity_too_large_rejected() {
		assert!(LruCache::<i64, i64>::try_new(NIL as usize).is_none());
	}
}
