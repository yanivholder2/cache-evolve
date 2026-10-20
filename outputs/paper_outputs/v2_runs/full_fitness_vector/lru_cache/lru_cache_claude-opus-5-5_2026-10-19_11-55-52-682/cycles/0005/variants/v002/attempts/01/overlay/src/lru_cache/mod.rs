// O(1) SIEVE cache (Zhang et al., NSDI 2024) using an in-file open-addressing index +
// arena-based singly-linked list.
//
// Single FIFO queue: new keys are appended at the tail (newest end). Each node carries a
// `visited` bit set on hit; hits never relink. On eviction a "hand" walks from the oldest
// end towards the newest, clearing visited bits, and evicts the first unvisited node.
// Surviving (visited) nodes stay in place (lazy promotion); new keys are behind the hand's
// direction of travel, so one-touch keys get demoted quickly (quick demotion).
//
// Nodes are only ever removed at the hand and appended at the tail, so no back-links are
// needed: the hand carries its own predecessor (`hand_prev`) as it walks forward. Links are
// `u32` with `NIL = u32::MAX`, so `Node<u64, u64>` fits in 24 bytes. Evicted slots are reused
// in place, so no free-list management is needed. No unsafe, no external crates.
//
// The key -> slot index is a linear-probing table of `u32` slot ids (`EMPTY = u32::MAX`),
// sized to a power of two >= 2 * capacity (load factor <= 0.5). Keys live only in `nodes`.
// Deletion uses backward-shift (no tombstones). A miss-path `put` hashes the new key once
// and reuses that hash for both the lookup and the insert.

use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;
const EMPTY: u32 = u32::MAX;

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

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	next: u32,
	visited: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	table: Vec<u32>,        // open-addressing index: bucket -> node slot (EMPTY = free)
	mask: usize,            // table.len() - 1 (table.len() is a power of two)
	nodes: Vec<Node<K, V>>, // arena
	head: u32,              // oldest (front)
	tail: u32,              // newest (back)
	hand: u32,              // SIEVE hand: NIL means "start at head"
	hand_prev: u32,         // predecessor of `hand`; NIL when hand is at head or NIL
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
		let buckets = match capacity
			.checked_mul(2)
			.and_then(|c| c.checked_next_power_of_two())
		{
			Some(b) => b,
			None => {
				log::error!("LruCache.try_new: capacity={capacity} overflows index table size");
				return None;
			}
		};
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			table: vec![EMPTY; buckets],
			mask: buckets - 1,
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			hand: NIL,
			hand_prev: NIL,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Fx hash of the key plus a final fold so the high bits reach the bucket index.
	#[inline]
	fn hash(key: &K) -> u64 {
		let mut s = FxHasher::default();
		key.hash(&mut s);
		let h = s.finish();
		h ^ (h >> 32)
	}

	/// Linear probe from `h & mask`. `Ok(bucket)` if the key is resident, else
	/// `Err(first empty bucket)`. Terminates because load factor <= 0.5.
	#[inline]
	fn find(&self, key: &K, h: u64) -> Result<usize, usize> {
		let mask = self.mask;
		let mut b = h as usize & mask;
		loop {
			let s = self.table[b];
			if s == EMPTY {
				return Err(b);
			}
			if self.nodes[s as usize].key == *key {
				return Ok(b);
			}
			b = (b + 1) & mask;
		}
	}

	/// First empty bucket on the probe path of `h`, for a key known to be absent.
	#[inline]
	fn find_empty(&self, h: u64) -> usize {
		let mask = self.mask;
		let mut b = h as usize & mask;
		while self.table[b] != EMPTY {
			b = (b + 1) & mask;
		}
		b
	}

	/// Bucket holding slot `slot`, probing from the slot's key hash `h`.
	#[inline]
	fn bucket_of_slot(&self, slot: u32, h: u64) -> usize {
		let mask = self.mask;
		let mut b = h as usize & mask;
		while self.table[b] != slot {
			debug_assert!(self.table[b] != EMPTY, "slot not indexed");
			b = (b + 1) & mask;
		}
		b
	}

	/// Backward-shift deletion of the entry in bucket `hole` (no tombstones).
	fn remove_bucket(&mut self, mut hole: usize) {
		let mask = self.mask;
		let mut j = hole;
		loop {
			j = (j + 1) & mask;
			let s = self.table[j];
			if s == EMPTY {
				break;
			}
			let home = Self::hash(&self.nodes[s as usize].key) as usize & mask;
			// Entry at j may move into the hole iff its home is cyclically at or before the hole.
			if (j.wrapping_sub(home) & mask) >= (j.wrapping_sub(hole) & mask) {
				self.table[hole] = s;
				hole = j;
			}
		}
		self.table[hole] = EMPTY;
	}

	/// Returns the value for `key` if present, marking it visited (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = Self::hash(key);
		match self.find(key, h) {
			Ok(b) => {
				let node = &mut self.nodes[self.table[b] as usize];
				node.visited = true;
				Some(&node.value)
			}
			Err(_) => None,
		}
	}

	/// Inserts or updates `key`. If at capacity, evicts via the SIEVE hand.
	pub fn put(&mut self, key: K, value: V) {
		let h = Self::hash(&key);
		let empty_bucket = match self.find(&key, h) {
			Ok(b) => {
				// Existing key: update in place and mark visited; no relinking.
				let node = &mut self.nodes[self.table[b] as usize];
				node.value = value;
				node.visited = true;
				return;
			}
			Err(b) => b,
		};
		let (idx, bucket) = if self.nodes.len() >= self.capacity {
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
			// Drop the victim's old key from the index (backward-shift), then reuse the slot.
			let old_h = Self::hash(&self.nodes[victim].key);
			let old_b = self.bucket_of_slot(v, old_h);
			self.remove_bucket(old_b);
			let node = &mut self.nodes[victim];
			node.key = key;
			node.value = value;
			node.next = NIL;
			node.visited = false;
			// The shift may have moved the first empty bucket on our probe path: re-probe.
			(victim, self.find_empty(h))
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				next: NIL,
				visited: false,
			});
			(idx, empty_bucket)
		};
		self.table[bucket] = idx as u32;
		self.push_back(idx);
		debug_assert!(self.nodes.len() <= self.capacity);
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
	use std::collections::HashSet;

	type C = LruCache<i64, i64>;

	/// Slot holding `key`, if resident, via the index.
	fn slot(c: &C, key: i64) -> Option<u32> {
		c.find(&key, C::hash(&key)).ok().map(|b| c.table[b])
	}

	fn contains(c: &C, key: i64) -> bool {
		slot(c, key).is_some()
	}

	/// Walk from head via `next`, verifying links, index agreement, tail, and that the hand
	/// and its predecessor are consistent.
	fn check_structure(c: &C) {
		assert!(c.nodes.len() <= c.capacity);
		assert!(c.table.len().is_power_of_two());
		assert!(c.table.len() >= 2 * c.capacity);
		let occupied = c.table.iter().filter(|&&s| s != EMPTY).count();
		assert_eq!(occupied, c.nodes.len(), "occupied buckets != nodes");
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
			assert_eq!(slot(c, n.key), Some(cur), "index disagrees for slot {i}");
			if cur == c.hand {
				hand_linked = true;
				assert_eq!(c.hand_prev, prev, "hand_prev is not the hand's predecessor");
			}
			prev = cur;
			cur = n.next;
			count += 1;
		}
		assert_eq!(c.tail, prev, "tail mismatch");
		assert_eq!(count, c.nodes.len(), "list length != node count");
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

	fn keys_in_order(c: &C) -> Vec<i64> {
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
	fn get_miss_returns_none() {
		let mut cache = C::new(4);
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
		let mut cache = C::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // nothing visited: evicts oldest (1)
		assert!(!contains(&cache, 1));
		assert_eq!(keys_in_order(&cache), vec![2, 3, 4]);
		check_structure(&cache);
	}

	#[test]
	fn visited_key_survives_eviction_without_moving() {
		let mut cache = C::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1); // mark visited, no relink
		assert_eq!(keys_in_order(&cache), vec![1, 2, 3]);
		cache.put(4, 40); // hand clears 1, evicts 2
		assert!(contains(&cache, 1));
		assert!(!contains(&cache, 2));
		// 1 stayed at the oldest position; new key appended at newest end.
		assert_eq!(keys_in_order(&cache), vec![1, 3, 4]);
		assert!(
			!cache.nodes[slot(&cache, 1).unwrap() as usize].visited,
			"hand must clear visited bit"
		);
		check_structure(&cache);
	}

	#[test]
	fn hand_resumes_after_victim() {
		let mut cache = C::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.put(4, 40); // evicts 2; hand now at 3
		assert_eq!(Some(cache.hand), slot(&cache, 3));
		assert_eq!(Some(cache.hand_prev), slot(&cache, 1));
		cache.put(5, 50); // hand starts at 3 (unvisited): evicts 3, not 1
		assert!(contains(&cache, 1));
		assert!(!contains(&cache, 3));
		assert_eq!(keys_in_order(&cache), vec![1, 4, 5]);
		assert_eq!(Some(cache.hand), slot(&cache, 4));
		check_structure(&cache);
	}

	#[test]
	fn hand_wraps_to_head() {
		let mut cache = C::new(3);
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
		assert!(!contains(&cache, 2));
		assert_eq!(keys_in_order(&cache), vec![3, 4, 5]);
		check_structure(&cache);
	}

	#[test]
	fn scan_of_unique_keys_does_not_evict_visited_key() {
		let mut cache = C::new(8);
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
		let mut cache = C::new(1);
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
	fn randomized_workload_preserves_structure() {
		for &cap in &[1usize, 2, 3, 7, 16] {
			let mut cache = C::new(cap);
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

	/// Hammers insert/evict so backward-shift deletion runs on long clusters; every resident
	/// key must stay findable and every evicted key must not be.
	#[test]
	fn randomized_index_insert_evict() {
		for &cap in &[1usize, 2, 3, 7, 16, 1000] {
			let mut cache = C::new(cap);
			let key_space = (cap * 3 + 5) as u64;
			let ops = if cap >= 1000 { 6_000 } else { 20_000 };
			let mut ever_put: HashSet<i64> = HashSet::new();
			let mut state: u64 = 0xD1B5_4A32_D192_ED03 ^ (cap as u64).wrapping_mul(0x9E37);
			for op in 0..ops {
				state ^= state << 13;
				state ^= state >> 7;
				state ^= state << 17;
				// Mix strided and small keys so clusters form under the Fx hash.
				let raw = state % key_space;
				let key = if state & 1 == 0 { raw as i64 } else { (raw as i64) << 20 };
				match cache.get(&key) {
					Some(&v) => assert_eq!(v, key ^ 0x55),
					None => {
						cache.put(key, key ^ 0x55);
						ever_put.insert(key);
					}
				}
				let full = cap < 1000 || op % 50 == 0 || op + 1 == ops;
				if full {
					check_structure(&cache);
					let resident: HashSet<i64> = cache.nodes.iter().map(|n| n.key).collect();
					assert_eq!(resident.len(), cache.nodes.len(), "duplicate resident key");
					for &k in &ever_put {
						assert_eq!(
							contains(&cache, k),
							resident.contains(&k),
							"index/residency mismatch for key {k} (cap {cap})"
						);
					}
				}
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		C::new(0);
	}

	#[test]
	fn capacity_too_large_rejected() {
		assert!(C::try_new(NIL as usize).is_none());
	}
}
