// S3-FIFO-style cache: small probation FIFO + ghost queue + `main` scanned by a SIEVE hand.
// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.
// `main` is an intrusive doubly linked list over the arena (head = newest, tail = oldest).
// H02 (cycle 8): survivors in `main` keep their position (no reinsertion); the hand moves
// from the tail toward the head, so new arrivals are examined before a survivor is revisited.
// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is
// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const MAX_FREQ: u8 = 3;
const NIL: u32 = u32::MAX;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	freq: u8,
	#[allow(dead_code)] // read by tests / Debug; queue membership is authoritative
	in_main: bool,
	prev: u32, // toward main_head (newer); only meaningful while in `main`
	next: u32, // toward main_tail (older); only meaningful while in `main`
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> arena index
	nodes: Vec<Node<K, V>>, // arena
	small: VecDeque<u32>,   // probation FIFO (front = oldest)
	main_head: u32,         // newest node in `main`
	main_tail: u32,         // oldest node in `main`
	main_len: usize,
	hand: u32,              // SIEVE hand; NIL = start at main_tail
	ghost_q: VecDeque<K>,   // recently evicted-from-small keys, FIFO
	ghost_set: HashMap<K, ()>,
	small_target: usize,
	ghost_cap: usize,
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
			log::error!("LruCache.try_new: capacity {capacity} exceeds u32 arena index range");
			return None;
		}
		// H02 (cycle 7): capacity / 40 probation queue (was capacity / 20 in cycle 6, capacity / 10 before).
		let small_target = std::cmp::max(1, capacity / 40);
		// H02 (cycle 7): 3 * capacity / 20 ghost window (was capacity / 8), keeping small + ghost ~constant.
		let ghost_cap = std::cmp::max(1, capacity.saturating_mul(3) / 20);
		log::info!("LruCache.try_new: capacity={capacity} small_target={small_target} (capacity / 40) ghost_cap={ghost_cap} (3 * capacity / 20) main=SIEVE");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			small: VecDeque::with_capacity(capacity),
			main_head: NIL,
			main_tail: NIL,
			main_len: 0,
			hand: NIL,
			ghost_q: VecDeque::with_capacity(ghost_cap + 1),
			ghost_set: HashMap::with_capacity(ghost_cap + 1),
			small_target,
			ghost_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, bumping its (saturating) access frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)? as usize;
		let node = &mut self.nodes[idx];
		if node.freq < MAX_FREQ {
			node.freq += 1;
		}
		Some(&node.value)
	}

	/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			if node.freq < MAX_FREQ {
				node.freq += 1;
			}
			return;
		}

		let ghost_hit = self.ghost_set.remove(&key).is_some();

		let idx = if self.map.len() >= self.capacity {
			let slot = self.evict_one();
			let node = &mut self.nodes[slot as usize];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			node.in_main = ghost_hit;
			node.prev = NIL;
			node.next = NIL;
			slot
		} else {
			let slot = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
				in_main: ghost_hit,
				prev: NIL,
				next: NIL,
			});
			slot
		};

		if ghost_hit {
			self.main_push_head(idx);
		} else {
			self.small.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.small.len() + self.main_len, self.map.len());
	}

	/// Links `idx` at the head (newest end) of `main`.
	fn main_push_head(&mut self, idx: u32) {
		let old_head = self.main_head;
		{
			let node = &mut self.nodes[idx as usize];
			node.prev = NIL;
			node.next = old_head;
		}
		if old_head != NIL {
			self.nodes[old_head as usize].prev = idx;
		} else {
			self.main_tail = idx;
		}
		self.main_head = idx;
		self.main_len += 1;
	}

	/// Unlinks `idx` from `main`. If the hand sits on `idx`, it first moves to `idx.prev`.
	fn main_unlink(&mut self, idx: u32) {
		let (prev, next) = {
			let node = &self.nodes[idx as usize];
			(node.prev, node.next)
		};
		if self.hand == idx {
			self.hand = prev;
		}
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.main_head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.main_tail = prev;
		}
		let node = &mut self.nodes[idx as usize];
		node.prev = NIL;
		node.next = NIL;
		self.main_len -= 1;
	}

	/// Evicts exactly one resident entry and returns its (now free) arena slot.
	/// Terminates: every iteration either lowers some node's freq or frees a slot.
	fn evict_one(&mut self) -> u32 {
		loop {
			if self.small.len() >= self.small_target || self.main_len == 0 {
				let idx = self.small.pop_front().expect("small must be non-empty when main is empty and cache is full");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq = 0;
					node.in_main = true;
					self.main_push_head(idx);
					continue;
				}
				let evicted = node.key.clone();
				self.map.remove(&evicted);
				self.push_ghost(evicted);
				return idx;
			} else {
				return self.sieve_evict_main();
			}
		}
	}

	/// SIEVE sweep over `main`: survivors stay in place with freq decremented; the hand
	/// walks tail -> head (wrapping), and the first freq == 0 node is evicted.
	/// Bounded by main_len * MAX_FREQ + 1 steps.
	fn sieve_evict_main(&mut self) -> u32 {
		debug_assert!(self.main_len > 0);
		let mut cur = if self.hand == NIL { self.main_tail } else { self.hand };
		loop {
			let node = &mut self.nodes[cur as usize];
			if node.freq > 0 {
				node.freq -= 1;
				cur = node.prev;
				if cur == NIL {
					cur = self.main_tail;
				}
				continue;
			}
			// Place hand on the victim; main_unlink then steps it to victim.prev.
			self.hand = cur;
			self.main_unlink(cur);
			let evicted = self.nodes[cur as usize].key.clone();
			self.map.remove(&evicted);
			return cur;
		}
	}

	fn push_ghost(&mut self, key: K) {
		self.ghost_set.insert(key.clone(), ());
		self.ghost_q.push_back(key);
		while self.ghost_q.len() > self.ghost_cap {
			if let Some(old) = self.ghost_q.pop_front() {
				self.ghost_set.remove(&old);
			}
		}
	}

	/// Debug check: walks `main` from head and returns the node count, verifying links.
	#[cfg(test)]
	fn main_walk_len(&self) -> usize {
		let mut count = 0usize;
		let mut prev = NIL;
		let mut cur = self.main_head;
		while cur != NIL {
			let node = &self.nodes[cur as usize];
			assert_eq!(node.prev, prev, "main list back-link broken");
			assert!(node.in_main);
			prev = cur;
			cur = node.next;
			count += 1;
			assert!(count <= self.nodes.len(), "main list cycle");
		}
		assert_eq!(prev, self.main_tail, "main_tail mismatch");
		count
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

	fn in_main(cache: &LruCache<i64, i64>, key: i64) -> bool {
		let idx = cache.map[&key] as usize;
		cache.nodes[idx].in_main
	}

	fn freq_of(cache: &LruCache<i64, i64>, key: i64) -> u8 {
		cache.nodes[cache.map[&key] as usize].freq
	}

	fn check_structure(cache: &LruCache<i64, i64>) {
		assert_eq!(cache.small.len() + cache.main_len, cache.map.len());
		assert_eq!(cache.main_walk_len(), cache.main_len);
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
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1 (one-hit wonder from small)
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(cache.map.len(), 1);
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
	fn one_hit_wonders_evicted_from_small_first() {
		let mut cache = LruCache::new(10); // small_target = 1
		for k in 1..=10 {
			cache.put(k, k * 10);
		}
		cache.get(&1); // key 1 is reused; keys 2..=10 are one-hit wonders
		cache.put(11, 110); // 1 promoted to main, 2 evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn accessed_small_entry_promoted_to_main() {
		let mut cache = LruCache::new(10);
		for k in 1..=10 {
			cache.put(k, k);
		}
		cache.get(&2);
		assert!(!in_main(&cache, 2));
		cache.put(11, 11); // evicts 1 (freq 0)
		assert!(!in_main(&cache, 2));
		cache.put(12, 12); // 2 has freq > 0 -> promoted to main; 3 evicted
		assert!(in_main(&cache, 2));
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 2);
		check_structure(&cache);
	}

	#[test]
	fn ghost_hit_admitted_into_main() {
		let mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 48 / 20 = 2
		assert_eq!(cache.ghost_cap, 2);
		assert_eq!(cache.small_target, 1);
		for k in 1..=16 {
			cache.put(k, k);
		}
		cache.put(17, 17); // evicts 1 into ghost
		assert!(cache.get(&1).is_none());
		assert!(cache.ghost_set.contains_key(&1));
		cache.put(1, 100); // ghost hit -> straight into main
		assert!(in_main(&cache, 1));
		assert_eq!(cache.main_head, cache.map[&1]); // new arrival linked at head
		assert!(!cache.ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 16);
		check_structure(&cache);
	}

	#[test]
	fn ghost_window_expires_after_ghost_cap_plus_one_evictions() {
		let mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 2
		let ghost_cap = cache.ghost_cap as i64;
		assert_eq!(ghost_cap, 2);
		assert_eq!(cache.small_target, 1);
		for k in 1..=16 {
			cache.put(k, k);
		}
		// ghost_cap + 1 small-queue evictions: keys 1..=ghost_cap+1.
		for k in 17..=(17 + ghost_cap) {
			cache.put(k, k);
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		assert!(cache.get(&1).is_none());
		assert!(!cache.ghost_set.contains_key(&1));
		assert!(cache.ghost_set.contains_key(&2));
		cache.put(1, 100); // no longer a ghost -> lands in small
		assert!(!in_main(&cache, 1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 16);
		check_structure(&cache);
	}

	#[test]
	fn main_evictions_not_ghosted() {
		let mut cache = LruCache::new(1);
		cache.put(1, 1);
		cache.put(2, 2); // 1 -> ghost
		cache.put(1, 1); // ghost hit -> main; 2 evicted from small -> ghost
		assert!(in_main(&cache, 1));
		cache.put(3, 3); // small empty, main non-empty: evict 1 from main
		assert!(cache.get(&1).is_none());
		assert!(!cache.ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&3).unwrap(), 3);
		check_structure(&cache);
	}

	#[test]
	fn sieve_survivor_stays_in_place_and_new_arrival_evicted_first() {
		// capacity 4: small_target = 1, ghost_cap = 1.
		let mut cache = LruCache::new(4);
		for k in 1..=4 {
			cache.put(k, k);
		}
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(5, 5); // 1,2,3 promoted (main head->tail: 3,2,1); 4 evicted from small
		check_structure(&cache);
		assert!(cache.get(&4).is_none());
		cache.get(&1); // 1: freq 1
		cache.put(6, 6); // 5 evicted from small
		assert!(cache.get(&5).is_none());
		cache.get(&3); // 3: freq 1
		cache.get(&6); // 6: freq 1
		// 6 promoted to head (main: 6,3,2,1). Sweep from tail: 1 freq 1 -> 0 (stays put),
		// 2 freq 0 -> evicted, hand = 3.
		cache.put(7, 7);
		check_structure(&cache);
		assert!(!cache.map.contains_key(&2));
		assert_eq!(cache.main_tail, cache.map[&1]); // survivor not reinserted
		assert_eq!(cache.hand, cache.map[&3]);
		cache.get(&1); // old survivor at the tail: freq 1
		cache.get(&7);
		// 7 promoted to head (main: 7,6,3,1). Sweep from hand=3: freq 1 -> 0, then
		// 6 (newly promoted, freq 0, between hand and head) evicted before 1 is revisited.
		cache.put(8, 8);
		check_structure(&cache);
		assert!(!cache.map.contains_key(&6));
		assert_eq!(freq_of(&cache, 1), 1); // survivor retained and untouched
		assert_eq!(cache.main_tail, cache.map[&1]);
		assert_eq!(cache.hand, cache.map[&7]);
		assert_eq!(*cache.get(&1).unwrap(), 1);
		assert_eq!(*cache.get(&3).unwrap(), 3);
		assert_eq!(*cache.get(&7).unwrap(), 7);
		assert_eq!(*cache.get(&8).unwrap(), 8);
	}

	#[test]
	fn capacity_bound_under_churn() {
		let mut cache = LruCache::new(8);
		for i in 0..10_000i64 {
			let k = (i * 7919) % 37;
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.map.len() <= 8);
			check_structure(&cache);
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		for k in 0..37 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	fn split_ratios_at_2048() {
		let cache = LruCache::<i64, i64>::new(2048);
		assert_eq!(cache.small_target, 51);
		assert_eq!(cache.ghost_cap, 307);
		// Combined admission window unchanged vs cycle 6 (102 + 256).
		assert_eq!(cache.small_target + cache.ghost_cap, 358);
	}

	#[test]
	fn capacity_bound_under_churn_400() {
		let mut cache = LruCache::new(400); // small_target = 10, ghost_cap = 60
		assert_eq!(cache.small_target, 10);
		assert_eq!(cache.ghost_cap, 60);
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..50_000 {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let r = (state >> 33) as i64;
			// Skewed mix: hot set of 300 keys plus a 2000-key cold tail.
			let k = if r % 3 == 0 { (r >> 2) % 2000 + 1000 } else { (r >> 2) % 300 };
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.map.len() <= 400);
			check_structure(&cache);
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		for k in 0..3000i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
