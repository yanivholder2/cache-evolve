// O(1) S3-FIFO cache (Yang et al., SOSP 2023) with a SIEVE main queue (Zhang et al., NSDI 2024),
// using HashMap + arena-based singly-linked lists, plus a TinyLFU-style frequency duel at
// small-queue exit.
//
// Two queues share the arena's forward links:
// - *small* (probationary) queue: a pure FIFO sized to ~10% of capacity. New keys enter here.
//   When popped from its head, a visited key is promoted to the main tail (no eviction yet). An
//   unvisited key *duels* the SIEVE hand's main victim: if its count-min frequency estimate is
//   strictly greater, the main victim is evicted (and ghosted) and the small key moves to the
//   main tail; otherwise the small key is evicted and its fingerprint recorded in the ghost.
// - *main* queue: SIEVE. Hits set `visited` and never relink; on eviction a hand walks
//   from the oldest end towards the newest, clearing visited bits, and evicts the first unvisited
//   node. Main evictions are also recorded in the ghost, so a key SIEVE evicted from main that
//   returns while its fingerprint survives is re-admitted straight to main.
// - *ghost*: a direct-mapped `Vec<u32>` of fingerprints (0 = empty), fed by both small-head and
//   main evictions. A missed key whose fingerprint is in the ghost skips probation and is
//   admitted straight to the main tail.
// - *sketch*: a 4-row count-min sketch of 4-bit counters (16 per u64 word), incremented on each
//   miss (put of a new key) and on each 0->1 VISITED transition. After `10 * capacity`
//   increments every counter is halved (aging).
//
// Representation: `Node<K, V>` holds only `{ key, value }` (16 bytes for u64/u64). Per-node
// policy state lives outside the node:
//   - a parallel packed `links: Vec<u32>`: bit 30 = SMALL, bits 0..30 = forward link
//     (NIL = LINK_MASK);
//   - a dense `visited: Vec<u64>` bitset (1 bit per slot; 256 bytes at capacity 2048, so it
//     stays L1-resident). A `get` hit touches only the map, this bitset and `nodes[idx]`
//     (plus one hash + sketch update on the key's first hit since its bit was last cleared).
// Evicted slots are reused in place, and the victim's key is swapped out (then removed from the
// map) rather than removed-then-cloned. No unsafe, no external crates.
//
// The index map uses an in-file multiplicative (Fx-style) hasher instead of SipHash-1-3,
// and stores `u32` slot indices.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const SMALL: u32 = 1 << 30;
const LINK_MASK: u32 = (1 << 30) - 1;
const NIL: u32 = LINK_MASK;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Number of count-min sketch rows.
const SKETCH_ROWS: usize = 4;
/// Mask that halves every 4-bit counter in a word after a right shift by one.
const AGE_MASK: u64 = 0x7777_7777_7777_7777;

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

/// Counter index of hash `h` in sketch row `r` (before masking).
#[inline(always)]
fn sketch_index(h: u64, r: usize) -> usize {
	(h.rotate_left(16 * r as u32) as usize) ^ ((h >> 32) as usize).wrapping_mul(r)
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
	// Ghost fingerprints of keys evicted from small or main; direct-mapped, 0 = empty.
	ghost: Vec<u32>,
	gmask: usize,
	// Count-min frequency sketch: SKETCH_ROWS rows of (sketch_mask + 1) 4-bit counters,
	// 16 counters per word, rows stored contiguously.
	sketch: Vec<u64>,
	sketch_mask: usize,
	sample: u32,
	sample_limit: u32,
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
		let gsize = capacity.next_power_of_two();
		let width = capacity.next_power_of_two().max(16);
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
			sketch: vec![0u64; SKETCH_ROWS * (width / 16)],
			sketch_mask: width - 1,
			sample: 0,
			sample_limit: (capacity as u64 * 10).min(u32::MAX as u64) as u32,
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

	/// Set VISITED only if clear, so repeated hits on hot keys do not dirty the word. The 0->1
	/// transition is also counted in the frequency sketch; already-visited hits do no sketch work.
	#[inline(always)]
	fn set_visited(&mut self, i: usize) {
		if !self.vbit_test(i) {
			self.vbit_set(i);
			let h = self.map.hasher().hash_one(&self.nodes[i].key);
			self.freq_incr(h);
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

	/// Words per sketch row.
	#[inline(always)]
	fn row_words(&self) -> usize {
		(self.sketch_mask + 1) >> 4
	}

	/// Increment the 4 count-min counters of `h` (saturating at 15); age the sketch by halving
	/// every counter once `sample_limit` increments have accumulated.
	fn freq_incr(&mut self, h: u64) {
		let rw = self.row_words();
		for r in 0..SKETCH_ROWS {
			let c = sketch_index(h, r) & self.sketch_mask;
			let w = &mut self.sketch[r * rw + (c >> 4)];
			let shift = (c & 15) << 2;
			if (*w >> shift) & 0xF < 15 {
				*w += 1u64 << shift;
			}
		}
		self.sample += 1;
		if self.sample >= self.sample_limit {
			for w in self.sketch.iter_mut() {
				*w = (*w >> 1) & AGE_MASK;
			}
			self.sample /= 2;
		}
	}

	/// Count-min frequency estimate of `h`: the minimum of its 4 counters.
	fn freq_est(&self, h: u64) -> u8 {
		let rw = self.row_words();
		let mut m = 15u8;
		for r in 0..SKETCH_ROWS {
			let c = sketch_index(h, r) & self.sketch_mask;
			let v = ((self.sketch[r * rw + (c >> 4)] >> ((c & 15) << 2)) & 0xF) as u8;
			m = m.min(v);
		}
		m
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
		let h = self.map.hasher().hash_one(&key);
		self.freq_incr(h);
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
	/// promoted to main without evicting; unvisited ones duel the SIEVE main victim on frequency.
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
				let hx = self.map.hasher().hash_one(&self.nodes[x].key);
				if self.head != NIL {
					// Frequency duel against the SIEVE hand's main victim.
					let (victim, victim_prev) = self.find_victim();
					let hv = self.map.hasher().hash_one(&self.nodes[victim].key);
					if self.freq_est(hx) > self.freq_est(hv) {
						// x wins: evict the main victim, admit x to the main tail.
						self.unlink_main(victim, victim_prev);
						self.ghost[(hv as usize) & self.gmask] = fingerprint(hv);
						self.set_small(x, false);
						self.push_back_main(x);
						return victim;
					}
					// x loses: the victim stays linked; park the hand on it so the next main
					// eviction resumes there (the visited bits it passed are already cleared).
					self.hand = victim as u32;
					self.hand_prev = victim_prev;
				}
				self.ghost[(hx as usize) & self.gmask] = fingerprint(hx);
				self.set_next(x, NIL);
				return x;
			}
			// Main eviction via the SIEVE hand; the victim's fingerprint is recorded in the
			// same ghost as small-head evictions.
			let (victim, victim_prev) = self.find_victim();
			let h = self.map.hasher().hash_one(&self.nodes[victim].key);
			self.ghost[(h as usize) & self.gmask] = fingerprint(h);
			self.unlink_main(victim, victim_prev);
			return victim;
		}
	}

	/// Unlink `victim` from main using the predecessor the hand carried, advance the hand past
	/// it, and clear its forward link.
	fn unlink_main(&mut self, victim: usize, victim_prev: u32) {
		let v = victim as u32;
		let nxt = self.next(victim);
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
	}

	/// Walk the hand from its position (or head) towards the newest end of main, clearing
	/// visited bits and wrapping to head, until an unvisited node is found. Returns the victim
	/// and its predecessor (NIL if the victim is head). Terminates within `capacity + 1` steps
	/// since one full pass clears every bit. Does not move `self.hand`; callers do.
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
		let h = c.map.hasher().hash_one(&key);
		c.ghost[(h as usize) & c.gmask] == fingerprint(h)
	}

	fn est(c: &LruCache<i64, i64>, key: i64) -> u8 {
		c.freq_est(c.map.hasher().hash_one(&key))
	}

	#[test]
	fn node_is_16_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 16);
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
	fn sketch_sizing() {
		let c: LruCache<i64, i64> = LruCache::new(2048);
		assert_eq!(c.sketch.len(), 512); // 4 KiB
		assert_eq!(c.sketch_mask, 2047);
		assert_eq!(c.sample_limit, 20_480);
		let c1: LruCache<i64, i64> = LruCache::new(1);
		assert_eq!(c1.sketch.len(), 4);
		assert_eq!(c1.sketch_mask, 15);
	}

	#[test]
	fn sketch_counts_saturate_and_age() {
		let mut c: LruCache<i64, i64> = LruCache::new(2048);
		let h = c.map.hasher().hash_one(&7i64);
		assert_eq!(c.freq_est(h), 0);
		for _ in 0..6 {
			c.freq_incr(h);
		}
		assert_eq!(c.freq_est(h), 6);
		// Force aging on the next increment: 7 is halved to 3.
		c.sample = c.sample_limit - 1;
		c.freq_incr(h);
		assert_eq!(c.freq_est(h), 3, "aging must halve counters");
		assert_eq!(c.sample, c.sample_limit / 2);
		for w in &c.sketch {
			for s in 0..16 {
				assert!((w >> (s * 4)) & 0xF <= 3);
			}
		}
		for _ in 0..40 {
			c.freq_incr(h);
		}
		assert_eq!(c.freq_est(h), 15, "counters saturate at 15");
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
		// 1 promoted (bit cleared); then 2 (freq 1) loses the duel to main victim 1 (freq 2)
		// and is evicted.
		cache.put(4, 40);
		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&2));
		assert_eq!(main_keys(&cache), vec![1]);
		assert_eq!(small_keys(&cache), vec![3, 4]);
		let i = cache.map[&1] as usize;
		assert!(!cache.is_small(i) && !cache.is_visited(i));
		// The losing duel parks the hand on the surviving victim.
		assert_eq!(cache.hand, cache.map[&1]);
		assert_eq!(cache.hand_prev, NIL);
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
		// Ghost hit -> admitted to main. Small head 4 (freq 1) loses the duel to main victim
		// 2 (freq 2) and is evicted.
		cache.put(1, 11);
		let i = cache.map[&1] as usize;
		assert!(!cache.map.contains_key(&4));
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
	fn frequent_small_key_wins_duel_against_fresh_main_entry() {
		// Capacity 4 -> small_target 1. Keys here are < 16, so sketch row 0 (low bits of
		// k * FX_SEED) is collision-free and estimates are exact.
		let mut cache: LruCache<i64, i64> = LruCache::new(4);
		for k in [1, 2, 3, 4] {
			cache.put(k, k);
		}
		cache.put(5, 5); // main empty: small head 1 evicted (freq 1)
		assert!(!cache.map.contains_key(&1));
		// Emulate the ghost entry being overwritten so 1 re-enters probation.
		cache.ghost.fill(0);
		cache.put(1, 1); // freq 2; small head 2 evicted
		cache.ghost.fill(0);
		for k in [6, 7, 8, 9] {
			cache.put(k, k); // evicts 3, 4, 5, then 1 (main still empty)
		}
		assert!(!cache.map.contains_key(&1));
		assert_eq!(small_keys(&cache), vec![6, 7, 8, 9]);
		cache.ghost.fill(0);
		cache.put(1, 1); // freq 3; small head 6 evicted
		assert_eq!(small_keys(&cache), vec![7, 8, 9, 1]);
		assert_eq!(est(&cache, 1), 3);
		// Give main a fresh entry: 7 is hit once (freq 2) and promoted.
		cache.get(&7);
		assert_eq!(est(&cache, 7), 2);
		cache.put(10, 10); // 7 promoted; 8 (freq 1) loses to 7 (freq 2)
		assert_eq!(main_keys(&cache), vec![7]);
		assert!(!cache.map.contains_key(&8));
		check_structure(&cache);
		cache.put(11, 11); // 9 (freq 1) loses to 7
		assert!(!cache.map.contains_key(&9));
		assert_eq!(small_keys(&cache), vec![1, 10, 11]);
		check_structure(&cache);
		cache.ghost.fill(0);
		cache.put(12, 12); // 1 (freq 3) beats 7 (freq 2): 7 evicted and ghosted, 1 -> main
		assert!(!cache.map.contains_key(&7));
		assert!(in_ghost(&cache, 7));
		assert_eq!(main_keys(&cache), vec![1]);
		assert_eq!(small_keys(&cache), vec![10, 11, 12]);
		let i = cache.map[&1] as usize;
		assert!(!cache.is_small(i));
		assert_eq!(*cache.get(&1).unwrap(), 1);
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
