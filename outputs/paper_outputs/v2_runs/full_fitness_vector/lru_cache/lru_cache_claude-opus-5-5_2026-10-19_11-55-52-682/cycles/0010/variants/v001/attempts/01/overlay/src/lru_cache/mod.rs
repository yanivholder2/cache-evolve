// O(1) S3-FIFO cache (Yang et al., SOSP 2023) with a SIEVE main queue (Zhang et al., NSDI 2024),
// using HashMap + arena-based singly-linked lists.
//
// Two queues share the arena's forward links:
// - *small* (probationary) queue: a pure FIFO sized to ~10% of capacity. New keys enter here.
//   When popped from its head, a visited key is promoted to the main tail (no eviction yet); an
//   unvisited key is evicted and its hash fingerprint recorded in the ghost table.
// - *main* queue: SIEVE. Hits set `visited` and never relink; on eviction a hand walks
//   from the oldest end towards the newest, clearing visited bits, and evicts the first unvisited
//   node. Main evictions are also recorded in the ghost, so a key SIEVE evicted from main that
//   returns while its fingerprint survives is re-admitted straight to main.
// - *ghost*: a half-size direct-mapped `Vec<u32>` of fingerprints (0 = empty), with
//   `next_power_of_two(capacity) / 2` slots (min 1; 1024 at capacity 2048), fed by both
//   small-head and main evictions. A missed key whose fingerprint is in the ghost skips probation
//   and is admitted straight to the main tail.
//
// Representation: `Node<K, V>` holds only `{ key, value }` (16 bytes for u64/u64). Per-node
// policy state lives outside the node:
//   - a parallel packed `links: Vec<u32>`: bit 30 = SMALL, bits 0..30 = forward link
//     (NIL = LINK_MASK);
//   - a dense `visited: Vec<u64>` bitset (1 bit per slot; 256 bytes at capacity 2048, so it
//     stays L1-resident). A `get` hit touches only the map, this bitset and `nodes[idx]`.
// Evicted slots are reused in place, and the victim's key is swapped out (then removed from the
// map) rather than removed-then-cloned. No unsafe, no external crates.
//
// The index map uses an in-file multiplicative (Fx-style) hasher instead of SipHash-1-3,
// and stores `u32` slot indices. `finish` applies a rustc-hash-2.x-style `rotate_left(26)` so
// the well-mixed high product bits feed hashbrown's bucket index (low bits) and its 7-bit tag
// (top bits). The ghost undoes that rotation (`ghost_hash`) so its slots and fingerprints are
// bit-for-bit identical to the un-rotated Fx product, keeping eviction order unchanged.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const SMALL: u32 = 1 << 30;
const LINK_MASK: u32 = (1 << 30) - 1;
const NIL: u32 = LINK_MASK;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Rotation applied by `FxHasher::finish` (rustc-hash 2.x finalizer).
const FX_ROTATE: u32 = 26;

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
		// rustc-hash 2.x finalizer: move the well-mixed high product bits down into the low
		// bits hashbrown uses for the bucket index.
		self.hash.rotate_left(FX_ROTATE)
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

/// Undo `finish`'s rotation, recovering the raw Fx product so ghost slots and fingerprints
/// are identical to the pre-rotation hasher.
#[inline(always)]
fn ghost_hash(h: u64) -> u64 {
	h.rotate_right(FX_ROTATE)
}

/// Ghost fingerprint of a hash: the high 32 bits, forced odd so it is never 0 (empty).
#[inline(always)]
fn fingerprint(h: u64) -> u32 {
	((h >> 32) as u32) | 1
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32, FxBuild>, // key -> node index
	nodes: Vec<Node<K, V>>,        // arena
	links: Vec<u32>,               // parallel to `nodes`: SMALL | next
	visited: Vec<u64>,             // dense VISITED bitset, one bit per slot
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
	// Ghost fingerprints of keys evicted from small or main; half-size direct-mapped, 0 = empty.
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
			log::error!("LruCache.try_new: capacity={capacity} exceeds 30-bit link range");
			return None;
		}
		log::info!("LruCache.try_new: capacity={capacity}");
		let gsize = (capacity.next_power_of_two() / 2).max(1);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			links: Vec::with_capacity(capacity),
			visited: vec![0u64; (capacity + 63) / 64],
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

	#[inline(always)]
	fn next(&self, i: usize) -> u32 {
		self.links[i] & LINK_MASK
	}

	#[inline(always)]
	fn set_next(&mut self, i: usize, n: u32) {
		let l = &mut self.links[i];
		*l = (*l & !LINK_MASK) | n;
	}

	#[inline(always)]
	fn vbit_set(&mut self, i: usize) {
		self.visited[i >> 6] |= 1u64 << (i & 63);
	}

	#[inline(always)]
	fn vbit_clear(&mut self, i: usize) {
		self.visited[i >> 6] &= !(1u64 << (i & 63));
	}

	#[inline(always)]
	fn vbit_test(&self, i: usize) -> bool {
		self.visited[i >> 6] & (1u64 << (i & 63)) != 0
	}

	/// Set VISITED only if clear, so repeated hits on hot keys do not dirty the word.
	#[inline(always)]
	fn set_visited(&mut self, i: usize) {
		if !self.vbit_test(i) {
			self.vbit_set(i);
		}
	}

	#[inline(always)]
	fn clear_visited(&mut self, i: usize) {
		self.vbit_clear(i);
	}

	#[inline(always)]
	fn is_visited(&self, i: usize) -> bool {
		self.vbit_test(i)
	}

	#[inline(always)]
	fn is_small(&self, i: usize) -> bool {
		self.links[i] & SMALL != 0
	}

	#[inline(always)]
	fn set_small(&mut self, i: usize, small: bool) {
		if small {
			self.links[i] |= SMALL;
		} else {
			self.links[i] &= !SMALL;
		}
	}

	/// Returns the value for `key` if present, marking it visited (no relinking). The hit path
	/// touches the map, the visited bitset and `nodes[idx]` only, never `links`.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)? as usize;
		self.set_visited(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. A new key is admitted to the small queue, or straight to the
	/// main queue if its fingerprint is in the ghost. At capacity, `evict` frees a slot first.
	pub fn put(&mut self, key: K, value: V) {
		// Existing key: update in place and mark visited; no relinking.
		if let Some(&idx) = self.map.get(&key) {
			let idx = idx as usize;
			self.set_visited(idx);
			self.nodes[idx].value = value;
			return;
		}
		// Ghost lookup happens before eviction so a colliding fingerprint written by this
		// miss's own eviction cannot erase the incoming key's ghost entry.
		let h = ghost_hash(self.map.hasher().hash_one(&key));
		let gslot = (h as usize) & self.gmask;
		let to_main = self.ghost[gslot] == fingerprint(h);
		if to_main {
			self.ghost[gslot] = 0;
		}
		let flags = if to_main { 0 } else { SMALL };
		let idx = if self.nodes.len() >= self.capacity {
			// `evict` unlinks the slot but leaves its key in the map; swap the key, then remove
			// the old one.
			let x = self.evict();
			let old = std::mem::replace(&mut self.nodes[x].key, key.clone());
			self.map.remove(&old);
			self.nodes[x].value = value;
			self.links[x] = NIL | flags;
			self.vbit_clear(x);
			x
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
			});
			self.links.push(NIL | flags);
			self.vbit_clear(idx);
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

	/// Free one arena slot: unlink it from its queue and return its index. The slot's key is
	/// still in the map; the caller swaps it out and removes it. Visited small-queue heads are
	/// promoted to main without evicting.
	fn evict(&mut self) -> usize {
		loop {
			if self.s_len > 0 && (self.s_len >= self.small_target || self.head == NIL) {
				// Pop the small head.
				let x = self.s_head as usize;
				let nxt = self.next(x);
				self.s_head = nxt;
				if nxt == NIL {
					self.s_tail = NIL;
				}
				self.s_len -= 1;
				if self.is_visited(x) {
					// Promotion: move to main, no eviction yet.
					self.clear_visited(x);
					self.set_small(x, false);
					self.push_back_main(x);
					continue;
				}
				let h = ghost_hash(self.map.hasher().hash_one(&self.nodes[x].key));
				self.ghost[(h as usize) & self.gmask] = fingerprint(h);
				self.set_next(x, NIL);
				return x;
			}
			// Main eviction via the SIEVE hand; the victim's fingerprint is recorded in the
			// same ghost as small-head evictions.
			let (victim, victim_prev) = self.find_victim();
			let h = ghost_hash(self.map.hasher().hash_one(&self.nodes[victim].key));
			self.ghost[(h as usize) & self.gmask] = fingerprint(h);
			let v = victim as u32;
			let nxt = self.next(victim);
			// Unlink the victim using the predecessor the hand carried.
			if victim_prev == NIL {
				self.head = nxt;
			} else {
				self.set_next(victim_prev as usize, nxt);
			}
			if v == self.tail {
				self.tail = victim_prev;
			}
			// The hand resumes after the victim; NIL means restart at head next time.
			self.hand = nxt;
			self.hand_prev = if nxt == NIL { NIL } else { victim_prev };
			self.set_next(victim, NIL);
			return victim;
		}
	}

	/// Walk the hand from its position (or head) towards the newest end of main, clearing
	/// visited bits and wrapping to head, until an unvisited node is found. Returns the victim
	/// and its predecessor (NIL if the victim is head). Terminates within `capacity + 1` steps
	/// since one full pass clears every bit.
	fn find_victim(&mut self) -> (usize, u32) {
		debug_assert!(self.head != NIL, "main must be non-empty for a main eviction");
		let head = self.head;
		let (mut h, mut p) = if self.hand == NIL {
			(head, NIL)
		} else {
			(self.hand, self.hand_prev)
		};
		let links = &self.links;
		let visited = &mut self.visited;
		loop {
			let i = h as usize;
			let w = &mut visited[i >> 6];
			let bit = 1u64 << (i & 63);
			if *w & bit == 0 {
				return (i, p);
			}
			*w &= !bit;
			p = h;
			h = links[i] & LINK_MASK;
			if h == NIL {
				h = head;
				p = NIL;
			}
		}
	}

	/// Append node to the tail (newest end) of the main queue.
	fn push_back_main(&mut self, idx: usize) {
		let i = idx as u32;
		self.set_next(idx, NIL);
		if self.tail == NIL {
			self.head = i;
		} else {
			self.set_next(self.tail as usize, i);
		}
		self.tail = i;
	}

	/// Append node to the tail of the small queue (caller maintains `s_len`).
	fn push_back_small(&mut self, idx: usize) {
		let i = idx as u32;
		self.set_next(idx, NIL);
		if self.s_tail == NIL {
			self.s_head = i;
		} else {
			self.set_next(self.s_tail as usize, i);
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
		assert_eq!(c.links.len(), c.nodes.len());
		assert_eq!(c.visited.len(), (c.capacity + 63) / 64);
		// No visited bit may be set beyond the allocated arena.
		for i in c.nodes.len()..c.visited.len() * 64 {
			assert!(!c.is_visited(i), "visited bit set for unallocated slot {i}");
		}
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
			assert!(!c.is_small(i), "main node {i} flagged small");
			assert_eq!(
				c.map.get(&c.nodes[i].key).map(|&v| v as usize),
				Some(i),
				"map disagrees for slot {i}"
			);
			if cur == c.hand {
				hand_linked = true;
				assert_eq!(c.hand_prev, prev, "hand_prev is not the hand's predecessor");
			}
			prev = cur;
			cur = c.next(i);
			main_len += 1;
		}
		assert_eq!(c.tail, prev, "main tail mismatch");
		assert!(hand_linked, "hand points at an unlinked slot");
		if c.hand != NIL {
			if c.hand_prev == NIL {
				assert_eq!(c.hand, c.head, "hand_prev NIL but hand is not head");
			} else {
				assert!(seen[c.hand_prev as usize], "hand_prev not linked");
				assert_eq!(c.next(c.hand_prev as usize), c.hand);
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
			assert!(c.is_small(i), "small node {i} not flagged small");
			assert_eq!(
				c.map.get(&c.nodes[i].key).map(|&v| v as usize),
				Some(i),
				"map disagrees for slot {i}"
			);
			prev = cur;
			cur = c.next(i);
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
			cur = c.next(cur as usize);
		}
		out
	}

	fn small_keys(c: &LruCache<i64, i64>) -> Vec<i64> {
		let mut out = Vec::new();
		let mut cur = c.s_head;
		while cur != NIL {
			out.push(c.nodes[cur as usize].key);
			cur = c.next(cur as usize);
		}
		out
	}

	fn in_ghost(c: &LruCache<i64, i64>, key: i64) -> bool {
		let h = ghost_hash(c.map.hasher().hash_one(&key));
		c.ghost[(h as usize) & c.gmask] == fingerprint(h)
	}

	#[test]
	fn node_is_16_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 16);
	}

	#[test]
	fn ghost_is_half_size() {
		let c: LruCache<i64, i64> = LruCache::new(2048);
		assert_eq!(c.ghost.len(), 1024);
		assert_eq!(c.gmask, 1023);
		let c1: LruCache<i64, i64> = LruCache::new(1);
		assert_eq!(c1.ghost.len(), 1);
		assert_eq!(c1.gmask, 0);
	}

	#[test]
	fn visited_bitset_sizing_and_ops() {
		let mut c: LruCache<i64, i64> = LruCache::new(2048);
		assert_eq!(c.visited.len(), 32); // 256 bytes
		for i in [0usize, 1, 63, 64, 65, 2047] {
			assert!(!c.vbit_test(i));
			c.vbit_set(i);
			assert!(c.vbit_test(i));
		}
		c.vbit_clear(64);
		assert!(!c.vbit_test(64) && c.vbit_test(63) && c.vbit_test(65));
		let c1: LruCache<i64, i64> = LruCache::new(1);
		assert_eq!(c1.visited.len(), 1);
		let c65: LruCache<i64, i64> = LruCache::new(65);
		assert_eq!(c65.visited.len(), 2);
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
	fn ghost_hash_recovers_unrotated_fx_product() {
		let c: LruCache<i64, i64> = LruCache::new(2048);
		for k in [0i64, 1, 2, 42, -1, 4096, 1 << 40, i64::MIN, i64::MAX] {
			// Old finish(): the raw product of a single Fx round from a zero state.
			let old = (0u64.rotate_left(5) ^ (k as u64)).wrapping_mul(FX_SEED);
			let new = c.map.hasher().hash_one(&k);
			assert_eq!(new, old.rotate_left(FX_ROTATE), "finish must apply the rotation");
			assert_eq!(ghost_hash(new), old, "ghost_hash must undo the rotation for {k}");
		}
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
		let i = cache.map[&1] as usize;
		assert!(!cache.is_small(i) && !cache.is_visited(i));
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
		assert!(in_ghost(&cache, 1), "main evictions are ghosted");
		assert_eq!(main_keys(&cache), vec![2, 3]);
		assert_eq!(small_keys(&cache), vec![4]);
		assert_eq!(cache.hand, cache.map[&2]);
		assert_eq!(cache.hand_prev, NIL);
		check_structure(&cache);
	}

	#[test]
	fn main_evicted_key_returns_straight_to_main() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // all promoted; SIEVE evicts 1 from main and ghosts it
		assert!(!cache.map.contains_key(&1));
		assert!(in_ghost(&cache, 1));
		cache.put(1, 11); // ghost hit -> admitted to main (small head 4 evicted)
		let i = cache.map[&1] as usize;
		assert!(!cache.is_small(i), "main-evicted key must return to main");
		assert!(!cache.is_visited(i));
		assert!(!in_ghost(&cache, 1), "ghost entry must be cleared on hit");
		assert_eq!(*main_keys(&cache).last().unwrap(), 1);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		check_structure(&cache);
	}

	#[test]
	fn reused_slot_starts_unvisited() {
		let mut cache: LruCache<i64, i64> = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts 1 (unvisited small head), reuses its slot
		let i = cache.map[&3] as usize;
		assert!(!cache.is_visited(i), "reused slot must start unvisited");
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
		let i = cache.map[&1] as usize;
		assert!(!cache.is_small(i), "returning key must land in main");
		assert!(!cache.is_visited(i));
		assert_eq!(main_keys(&cache), vec![1]);
		assert!(!in_ghost(&cache, 1), "ghost entry must be cleared on hit");
		assert_eq!(*cache.get(&1).unwrap(), 11);
		check_structure(&cache);
	}

	#[test]
	fn capacity_one() {
		let mut cache: LruCache<i64, i64> = LruCache::new(1);
		assert_eq!(cache.ghost.len(), 1);
		assert_eq!(cache.gmask, 0);
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
		for &cap in &[1usize, 2, 3, 7, 16, 64, 65, 130] {
			let mut cache: LruCache<i64, i64> = LruCache::new(cap);
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ cap as u64;
			let universe = (cap as u64 * 3).max(48);
			for _ in 0..20_000 {
				state ^= state << 13;
				state ^= state >> 7;
				state ^= state << 17;
				let key = (state % universe) as i64;
				match cache.get(&key) {
					Some(&v) => assert_eq!(v, key * 3),
					None => cache.put(key, key * 3),
				}
				check_structure(&cache);
			}
			for phantom in [100_000i64, -1] {
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
