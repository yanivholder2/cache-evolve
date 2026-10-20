// O(1) S3-FIFO cache (Yang et al., SOSP 2023) with a SIEVE main queue (Zhang et al., NSDI 2024),
// using HashMap + arena-based singly-linked lists.
//
// Two queues share the arena's `next` links:
// - *small* (probationary) queue: a pure FIFO sized to ~10% of capacity. New keys enter here.
//   When popped from its head, a visited key is promoted to the main tail (no eviction yet); an
//   unvisited key is evicted and its hash fingerprint recorded in the ghost table.
// - *main* queue: v002's SIEVE. Hits set `visited` and never relink; on eviction a hand walks
//   from the oldest end towards the newest, clearing visited bits, and evicts the first unvisited
//   node. Main evictions are not recorded in the ghost.
// - *ghost*: a direct-mapped `Vec<u32>` of fingerprints (0 = empty). A missed key whose
//   fingerprint is in the ghost skips probation and is admitted straight to the main tail.
//
// Links are `u32` with `NIL = u32::MAX`, so `Node<u64, u64>` fits in 24 bytes. Evicted slots are
// reused in place, so no free-list management is needed. No unsafe, no external crates.
//
// The index map uses an in-file multiplicative (Fx-style) hasher instead of SipHash-1-3,
// and stores `u32` slot indices to match the link width.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Fx-style multiplicative hasher (as used in rustc). Not DoS-resistant; fast for integer keys.
#[derive(Default, Clone, Copy)]
pub struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline(always)]
	fn add_to_hash(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.add_to_hash(u64::from_le_bytes(buf));
		}
		for &b in chunks.remainder() {
			self.add_to_hash(b as u64);
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add_to_hash(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.add_to_hash(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.hash
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

/// Ghost fingerprint of a hash: the high 32 bits, forced odd so it is never 0 (empty).
#[inline(always)]
fn fingerprint(h: u64) -> u32 {
	((h >> 32) as u32) | 1
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	next: u32,
	visited: bool, // S3-FIFO 1-bit frequency / SIEVE visited bit
	small: bool,   // true while the node is in the small (probationary) queue
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	// Main queue (SIEVE).
	head: u32,      // oldest (front)
	tail: u32,      // newest (back)
	hand: u32,      // SIEVE hand: NIL means "start at head"
	hand_prev: u32, // predecessor of `hand`; NIL when hand is at head or NIL
	// Small queue (FIFO).
	s_head: u32,
	s_tail: u32,
	s_len: usize,
	small_target: usize,
	// Ghost fingerprints of keys evicted from small; direct-mapped, 0 = empty.
	ghost: Vec<u32>,
	gmask: usize,
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
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity={capacity} exceeds u32 link range");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		let gsize = capacity.next_power_of_two();
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			hand: NIL,
			hand_prev: NIL,
			s_head: NIL,
			s_tail: NIL,
			s_len: 0,
			small_target: (capacity / 10).max(1),
			ghost: vec![0u32; gsize],
			gmask: gsize - 1,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, marking it visited (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx as usize];
		node.visited = true;
		Some(&node.value)
	}

	/// Inserts or updates `key`. A new key is admitted to the small queue, or straight to the
	/// main queue if its fingerprint is in the ghost. At capacity, `evict` frees a slot first.
	pub fn put(&mut self, key: K, value: V) {
		// Existing key: update in place and mark visited; no relinking.
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			node.visited = true;
			return;
		}
		// Ghost lookup happens before eviction so a colliding fingerprint written by this
		// miss's own small eviction cannot erase the incoming key's ghost entry.
		let h = self.map.hasher().hash_one(&key);
		let gslot = (h as usize) & self.gmask;
		let to_main = self.ghost[gslot] == fingerprint(h);
		if to_main {
			self.ghost[gslot] = 0;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let x = self.evict();
			let node = &mut self.nodes[x];
			node.key = key.clone();
			node.value = value;
			node.next = NIL;
			node.visited = false;
			node.small = !to_main;
			x
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				next: NIL,
				visited: false,
				small: !to_main,
			});
			idx
		};
		if to_main {
			self.push_back_main(idx);
		} else {
			self.push_back_small(idx);
			self.s_len += 1;
		}
		self.map.insert(key, idx as u32);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Free one arena slot: unlink it from its queue, remove its key from the map, and return
	/// its index. Visited small-queue heads are promoted to main without evicting.
	fn evict(&mut self) -> usize {
		loop {
			if self.s_len > 0 && (self.s_len >= self.small_target || self.head == NIL) {
				// Pop the small head.
				let x = self.s_head as usize;
				let nxt = self.nodes[x].next;
				self.s_head = nxt;
				if nxt == NIL {
					self.s_tail = NIL;
				}
				self.s_len -= 1;
				if self.nodes[x].visited {
					// Promotion: move to main, no eviction yet.
					let node = &mut self.nodes[x];
					node.visited = false;
					node.small = false;
					self.push_back_main(x);
					continue;
				}
				let h = self.map.hasher().hash_one(&self.nodes[x].key);
				self.ghost[(h as usize) & self.gmask] = fingerprint(h);
				self.map.remove(&self.nodes[x].key);
				self.nodes[x].next = NIL;
				return x;
			}
			// Main eviction via the SIEVE hand (not recorded in the ghost).
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
			self.map.remove(&self.nodes[victim].key);
			self.nodes[victim].next = NIL;
			return victim;
		}
	}

	/// Walk the hand from its position (or head) towards the newest end of main, clearing
	/// visited bits and wrapping to head, until an unvisited node is found. Returns the victim
	/// and its predecessor (NIL if the victim is head). Terminates within `capacity + 1` steps
	/// since one full pass clears every bit.
	fn find_victim(&mut self) -> (usize, u32) {
		debug_assert!(self.head != NIL, "main must be non-empty for a main eviction");
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

	/// Append node to the tail (newest end) of the main queue.
	fn push_back_main(&mut self, idx: usize) {
		let i = idx as u32;
		self.nodes[idx].next = NIL;
		if self.tail == NIL {
			self.head = i;
		} else {
			self.nodes[self.tail as usize].next = i;
		}
		self.tail = i;
	}

	/// Append node to the tail of the small queue (caller maintains `s_len`).
	fn push_back_small(&mut self, idx: usize) {
		let i = idx as u32;
		self.nodes[idx].next = NIL;
		if self.s_tail == NIL {
			self.s_head = i;
		} else {
			self.nodes[self.s_tail as usize].next = i;
		}
		self.s_tail = i;
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

	/// Walk both queues, verifying links, map agreement, tails, `small` flags, `s_len`, and that
	/// the main hand and its predecessor are consistent.
	fn check_structure(c: &LruCache<i64, i64>) {
		assert!(c.map.len() <= c.capacity);
		assert_eq!(c.map.len(), c.nodes.len());
		let mut seen = vec![false; c.nodes.len()];

		// Main queue.
		let mut prev = NIL;
		let mut cur = c.head;
		let mut main_len = 0usize;
		let mut hand_linked = c.hand == NIL;
		while cur != NIL {
			let i = cur as usize;
			assert!(i < c.nodes.len(), "link out of range");
			assert!(!seen[i], "cycle or shared slot in main");
			seen[i] = true;
			let n = &c.nodes[i];
			assert!(!n.small, "main node {i} flagged small");
			assert_eq!(
				c.map.get(&n.key).map(|&v| v as usize),
				Some(i),
				"map disagrees for slot {i}"
			);
			if cur == c.hand {
				hand_linked = true;
				assert_eq!(c.hand_prev, prev, "hand_prev is not the hand's predecessor");
			}
			prev = cur;
			cur = n.next;
			main_len += 1;
		}
		assert_eq!(c.tail, prev, "main tail mismatch");
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

		// Small queue.
		let mut prev = NIL;
		let mut cur = c.s_head;
		let mut small_len = 0usize;
		while cur != NIL {
			let i = cur as usize;
			assert!(i < c.nodes.len(), "link out of range");
			assert!(!seen[i], "cycle or shared slot in small");
			seen[i] = true;
			let n = &c.nodes[i];
			assert!(n.small, "small node {i} not flagged small");
			assert_eq!(
				c.map.get(&n.key).map(|&v| v as usize),
				Some(i),
				"map disagrees for slot {i}"
			);
			prev = cur;
			cur = n.next;
			small_len += 1;
		}
		assert_eq!(c.s_tail, prev, "small tail mismatch");
		assert_eq!(small_len, c.s_len, "s_len mismatch");
		assert_eq!(main_len + small_len, c.map.len(), "queue lengths != map length");
	}

	fn main_keys(c: &LruCache<i64, i64>) -> Vec<i64> {
		let mut out = Vec::new();
		let mut cur = c.head;
		while cur != NIL {
			out.push(c.nodes[cur as usize].key);
			cur = c.nodes[cur as usize].next;
		}
		out
	}

	fn small_keys(c: &LruCache<i64, i64>) -> Vec<i64> {
		let mut out = Vec::new();
		let mut cur = c.s_head;
		while cur != NIL {
			out.push(c.nodes[cur as usize].key);
			cur = c.nodes[cur as usize].next;
		}
		out
	}

	fn in_ghost(c: &LruCache<i64, i64>, key: i64) -> bool {
		let h = c.map.hasher().hash_one(&key);
		c.ghost[(h as usize) & c.gmask] == fingerprint(h)
	}

	#[test]
	fn node_is_24_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 24);
	}

	#[test]
	fn fx_hasher_distinguishes_keys() {
		let h = |k: i64| {
			let mut s = FxHasher::default();
			k.hash(&mut s);
			s.finish()
		};
		assert_ne!(h(1), h(2));
		assert_ne!(h(0), h(-1));
		assert_eq!(h(42), h(42));
		let mut a = FxHasher::default();
		a.write(&[1, 2, 3, 4, 5, 6, 7, 8, 9]);
		let mut c = FxHasher::default();
		c.write(&[1, 2, 3, 4, 5, 6, 7, 8, 10]);
		assert_ne!(a.finish(), c.finish());
	}

	#[test]
	fn fingerprint_never_zero() {
		for h in [0u64, 1, u64::MAX, 0xFFFF_FFFF, 0xFFFF_FFFF_0000_0000] {
			assert_ne!(fingerprint(h), 0);
		}
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
	fn new_keys_enter_small_and_unvisited_head_is_evicted_fifo() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(small_keys(&cache), vec![1, 2, 3]);
		assert!(main_keys(&cache).is_empty());
		cache.put(4, 40); // small head 1 unvisited: evicted to ghost
		assert!(!cache.map.contains_key(&1));
		assert!(in_ghost(&cache, 1));
		assert_eq!(small_keys(&cache), vec![2, 3, 4]);
		check_structure(&cache);
	}

	#[test]
	fn visited_small_head_is_promoted_to_main() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.put(4, 40); // 1 promoted (bit cleared), then 2 evicted
		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&2));
		assert_eq!(main_keys(&cache), vec![1]);
		assert_eq!(small_keys(&cache), vec![3, 4]);
		let n = &cache.nodes[cache.map[&1] as usize];
		assert!(!n.small && !n.visited);
		check_structure(&cache);
	}

	#[test]
	fn main_sieve_eviction_when_small_drains() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		// All three promoted (bits cleared); small empty -> main SIEVE evicts head 1.
		cache.put(4, 40);
		assert!(!cache.map.contains_key(&1));
		assert!(!in_ghost(&cache, 1), "main evictions are not ghosted");
		assert_eq!(main_keys(&cache), vec![2, 3]);
		assert_eq!(small_keys(&cache), vec![4]);
		assert_eq!(cache.hand, cache.map[&2]);
		assert_eq!(cache.hand_prev, NIL);
		check_structure(&cache);
	}

	#[test]
	fn scan_of_one_touch_keys_stays_in_small() {
		let mut cache: LruCache<i64, i64> = LruCache::new(10);
		cache.put(1, 10);
		cache.get(&1);
		for k in 100..1100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert_eq!(cache.get(&1).copied(), Some(10), "visited key evicted by scan at {k}");
			check_structure(&cache);
		}
		// Key 1 was promoted once; the scan never reached main.
		assert_eq!(main_keys(&cache), vec![1]);
		assert!(small_keys(&cache).iter().all(|&k| k >= 100));
	}

	#[test]
	fn ghost_hit_admits_straight_to_main() {
		let mut cache: LruCache<i64, i64> = LruCache::new(4);
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		cache.put(5, 50); // evicts 1 from small into ghost
		assert!(!cache.map.contains_key(&1));
		assert!(in_ghost(&cache, 1));
		cache.put(1, 11); // ghost hit -> main
		let n = &cache.nodes[cache.map[&1] as usize];
		assert!(!n.small, "returning key must land in main");
		assert!(!n.visited);
		assert_eq!(main_keys(&cache), vec![1]);
		assert!(!in_ghost(&cache, 1), "ghost entry must be cleared on hit");
		assert_eq!(*cache.get(&1).unwrap(), 11);
		check_structure(&cache);
	}

	#[test]
	fn capacity_one() {
		let mut cache: LruCache<i64, i64> = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10); // visited
		cache.put(2, 20); // 1 promoted to main, then SIEVE evicts it
		assert_eq!(cache.hand, NIL, "victim.next is NIL so hand resets");
		assert_eq!(cache.hand_prev, NIL);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
		cache.put(3, 30);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
