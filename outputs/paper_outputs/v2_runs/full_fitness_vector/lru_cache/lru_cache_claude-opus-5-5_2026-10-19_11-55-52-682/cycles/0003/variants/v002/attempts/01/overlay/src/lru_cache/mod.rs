// O(1) SIEVE cache (Zhang et al., NSDI 2024) using HashMap + arena-based doubly-linked list,
// with SIEVE's single visited bit upgraded to a 2-level saturating frequency counter.
//
// Single FIFO queue: new keys are appended at the tail (newest end) with `freq = 0`. Each hit
// bumps `freq` (saturating at `FREQ_MAX`); hits never relink. On eviction a "hand" walks from
// the oldest end towards the newest, decrementing non-zero counters by one, and evicts the
// first node whose counter is zero. Surviving nodes stay in place (lazy promotion); new keys
// are behind the hand's direction of travel, so one-touch keys get demoted quickly (quick
// demotion), while reused keys can earn up to `FREQ_MAX` passes of credit.
//
// Links are `u32` with `NIL = u32::MAX` so `Node<u64, u64>` fits in 32 bytes (the `u8`
// counter sits in the padding slot the old `bool` used). Evicted slots are reused in place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;

/// Saturation cap for the per-node frequency counter. Bounds the hand's sweep to at most
/// `FREQ_MAX` full revolutions before an eviction.
const FREQ_MAX: u8 = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
	freq: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: u32,              // oldest (front)
	tail: u32,              // newest (back)
	hand: u32,              // SIEVE hand: NIL means "start at head"
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			hand: NIL,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, bumping its saturating counter (no relinking).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		let node = &mut self.nodes[idx];
		node.freq += (node.freq < FREQ_MAX) as u8;
		Some(&node.value)
	}

	/// Inserts or updates `key`. If at capacity, evicts via the SIEVE hand.
	pub fn put(&mut self, key: K, value: V) {
		// Existing key: update in place and bump counter; no relinking.
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx];
			node.value = value;
			node.freq += (node.freq < FREQ_MAX) as u8;
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let victim = self.find_victim();
			// The hand resumes after the victim; NIL means restart at head next time.
			self.hand = self.nodes[victim].next;
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			let node = &mut self.nodes[victim];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				freq: 0,
			});
			idx
		};
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Walk the hand from its position (or head) towards the newest end, decrementing each
	/// non-zero counter by one and wrapping to head, until a node with `freq == 0` is found.
	/// Each full pass lowers every counter by one, so the walk terminates within
	/// `FREQ_MAX * capacity + 1` steps.
	fn find_victim(&mut self) -> usize {
		debug_assert!(self.head != NIL, "head must be linked when arena is full");
		let mut h = if self.hand == NIL { self.head } else { self.hand };
		#[cfg(debug_assertions)]
		let mut steps: usize = 0;
		loop {
			#[cfg(debug_assertions)]
			{
				steps += 1;
				debug_assert!(
					steps <= FREQ_MAX as usize * self.capacity + 1,
					"SIEVE hand walk exceeded FREQ_MAX * capacity + 1 steps"
				);
			}
			let node = &mut self.nodes[h as usize];
			if node.freq == 0 {
				return h as usize;
			}
			node.freq -= 1;
			h = node.next;
			if h == NIL {
				h = self.head;
			}
		}
	}

	/// Remove node from its current position in the list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.head = next;
		} else {
			self.nodes[prev as usize].next = next;
		}
		if next == NIL {
			self.tail = prev;
		} else {
			self.nodes[next as usize].prev = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the tail (newest end) of the list.
	fn push_back(&mut self, idx: usize) {
		let i = idx as u32;
		self.nodes[idx].prev = self.tail;
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

	/// Walk from head, verifying links, map agreement, tail, counter range, and that hand is
	/// linked or NIL.
	fn check_structure(c: &LruCache<i64, i64>) {
		assert!(c.map.len() <= c.capacity);
		assert_eq!(c.map.len(), c.nodes.len());
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
			assert_eq!(n.prev, prev, "prev link mismatch at {i}");
			assert_eq!(c.map.get(&n.key), Some(&i), "map disagrees for slot {i}");
			assert!(n.freq <= FREQ_MAX, "freq {} exceeds FREQ_MAX at slot {i}", n.freq);
			if cur == c.hand {
				hand_linked = true;
			}
			prev = cur;
			cur = n.next;
			count += 1;
		}
		assert_eq!(c.tail, prev, "tail mismatch");
		assert_eq!(count, c.map.len(), "list length != map length");
		assert!(hand_linked, "hand points at an unlinked slot");
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

	fn freq_of(c: &LruCache<i64, i64>, key: i64) -> u8 {
		c.nodes[c.map[&key]].freq
	}

	#[test]
	fn node_is_32_bytes() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 32);
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
	fn new_keys_start_at_zero_and_hits_saturate() {
		let mut cache: LruCache<i64, i64> = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(freq_of(&cache, 1), 0);
		cache.get(&1);
		assert_eq!(freq_of(&cache, 1), 1);
		cache.get(&1);
		assert_eq!(freq_of(&cache, 1), 2);
		cache.get(&1);
		cache.put(1, 11);
		assert_eq!(freq_of(&cache, 1), FREQ_MAX);
		check_structure(&cache);
	}

	#[test]
	fn unvisited_oldest_is_evicted_fifo() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // nothing hit: evicts oldest (1)
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
		cache.get(&1); // freq 1, no relink
		assert_eq!(keys_in_order(&cache), vec![1, 2, 3]);
		cache.put(4, 40); // hand decrements 1 to 0, evicts 2
		assert!(cache.map.contains_key(&1));
		assert!(!cache.map.contains_key(&2));
		// 1 stayed at the oldest position; new key appended at newest end.
		assert_eq!(keys_in_order(&cache), vec![1, 3, 4]);
		assert_eq!(freq_of(&cache, 1), 0, "one hit + one pass must leave freq == 0");
		check_structure(&cache);
	}

	#[test]
	fn twice_hit_key_survives_two_passes_once_hit_only_one() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&1); // freq(1) = 2
		cache.get(&2); // freq(2) = 1
		// Pass 1: 1 -> 1, 2 -> 0, 3 (freq 0) evicted; hand resets (3 was tail).
		cache.put(4, 40);
		assert!(!cache.map.contains_key(&3));
		assert_eq!(freq_of(&cache, 1), 1);
		assert_eq!(freq_of(&cache, 2), 0);
		assert_eq!(cache.hand, NIL);
		// Pass 2: 1 -> 0 (survives again), 2 (freq 0) evicted.
		cache.put(5, 50);
		assert!(cache.map.contains_key(&1), "twice-hit key must survive a second pass");
		assert!(!cache.map.contains_key(&2), "once-hit key survives only one pass");
		assert_eq!(freq_of(&cache, 1), 0);
		assert_eq!(keys_in_order(&cache), vec![1, 4, 5]);
		check_structure(&cache);
	}

	#[test]
	fn all_saturated_sweep_terminates_and_evicts_start_slot() {
		let mut cache: LruCache<i64, i64> = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.put(4, 40); // 1 -> 0, evicts 2 (freq 0); hand at 3 (mid-list)
		assert_eq!(cache.hand, cache.map[&3] as u32);
		assert_eq!(keys_in_order(&cache), vec![1, 3, 4]);
		for k in [1i64, 3, 4] {
			cache.get(&k);
			cache.get(&k);
			assert_eq!(freq_of(&cache, k), FREQ_MAX);
		}
		let start = cache.hand;
		// Two full revolutions drain every counter; the hand then evicts its start slot (3).
		cache.put(5, 50);
		assert!(!cache.map.contains_key(&3));
		assert_eq!(cache.map[&5] as u32, start, "start slot is reused in place");
		assert_eq!(keys_in_order(&cache), vec![1, 4, 5]);
		assert_eq!(freq_of(&cache, 1), 0);
		assert_eq!(freq_of(&cache, 4), 0);
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
		cache.put(5, 50); // hand starts at 3 (freq 0): evicts 3, not 1
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
		cache.put(4, 40); // evicts 1 (freq 0 head); hand -> 2
		cache.get(&4);
		// hand at 2 (1 -> 0), 3 (1 -> 0), 4 (1 -> 0), wrap to head 2 (freq 0) -> evict 2.
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
			assert_eq!(cache.get(&1).copied(), Some(10), "hit key evicted by scan at {k}");
		}
		check_structure(&cache);
	}

	#[test]
	fn capacity_one_hand_resets() {
		let mut cache: LruCache<i64, i64> = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10); // freq 1
		cache.put(2, 20); // hand decrements 1, wraps to head (1), evicts it
		assert_eq!(cache.hand, NIL, "victim.next is NIL so hand resets");
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
