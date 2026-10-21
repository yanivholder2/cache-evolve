// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queue).
// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.
// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is
// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.
//
// H03 (cycle 9): the ghost is a fixed-length ring of u64 fingerprints (0 = empty)
// plus a fingerprint -> live-occurrence count map. No key clones are stored for
// the ghost; policy is otherwise identical to cycle 8 v003.

use std::collections::hash_map::RandomState;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasher, Hash};

pub mod lru_cache_jni;

const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	freq: u8,
	#[allow(dead_code)] // read by tests / Debug; queue membership is authoritative
	in_main: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> arena index
	nodes: Vec<Node<K, V>>, // arena
	small: VecDeque<u32>,   // probation FIFO (front = oldest)
	main: VecDeque<u32>,    // main FIFO (front = oldest)
	ghost_ring: Vec<u64>,   // fingerprints of recent small evictions; 0 = empty slot
	ghost_head: usize,      // next ring slot to overwrite (oldest when ring is full)
	ghost_count: HashMap<u64, u32>, // fingerprint -> live occurrences in ring
	ghost_hasher: RandomState,
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
		// H03 (cycle 8): capacity / 80 probation queue (was capacity / 40 in cycle 7, / 20 in cycle 6, / 10 before).
		let small_target = std::cmp::max(1, capacity / 80);
		// H03 (cycle 8): 13 * capacity / 80 ghost window (was 3 * capacity / 20), keeping small + ghost ~constant.
		let ghost_cap = std::cmp::max(1, capacity.saturating_mul(13) / 80);
		log::info!("LruCache.try_new: capacity={capacity} small_target={small_target} (capacity / 80) ghost_cap={ghost_cap} (13 * capacity / 80, u64 fingerprint ring)");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			small: VecDeque::with_capacity(capacity),
			main: VecDeque::with_capacity(capacity),
			ghost_ring: vec![0u64; ghost_cap],
			ghost_head: 0,
			ghost_count: HashMap::with_capacity(ghost_cap),
			ghost_hasher: RandomState::new(),
			small_target,
			ghost_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Non-zero 64-bit fingerprint of `key` (0 is reserved for empty ring slots).
	#[inline]
	fn fingerprint(&self, key: &K) -> u64 {
		let fp = self.ghost_hasher.hash_one(key);
		if fp == 0 {
			1
		} else {
			fp
		}
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

		let fp = self.fingerprint(&key);
		let ghost_hit = Self::ghost_dec(&mut self.ghost_count, fp);

		let idx = if self.map.len() >= self.capacity {
			let slot = self.evict_one();
			let node = &mut self.nodes[slot as usize];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			node.in_main = ghost_hit;
			slot
		} else {
			let slot = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
				in_main: ghost_hit,
			});
			slot
		};

		if ghost_hit {
			self.main.push_back(idx);
		} else {
			self.small.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Saturating decrement of `fp`'s live count; removes the entry at zero.
	/// Returns true iff `fp` had a positive count (i.e. was a ghost).
	#[inline]
	fn ghost_dec(counts: &mut HashMap<u64, u32>, fp: u64) -> bool {
		match counts.get_mut(&fp) {
			Some(c) if *c > 0 => {
				*c -= 1;
				if *c == 0 {
					counts.remove(&fp);
				}
				true
			}
			Some(_) => {
				counts.remove(&fp);
				false
			}
			None => false,
		}
	}

	/// Evicts exactly one resident entry and returns its (now free) arena slot.
	/// Terminates: every iteration either lowers some node's freq or frees a slot.
	fn evict_one(&mut self) -> u32 {
		loop {
			if self.small.len() >= self.small_target || self.main.is_empty() {
				let idx = self.small.pop_front().expect("small must be non-empty when main is empty and cache is full");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq = 0;
					node.in_main = true;
					self.main.push_back(idx);
					continue;
				}
				let fp = self.fingerprint(&self.nodes[idx as usize].key);
				self.map.remove(&self.nodes[idx as usize].key);
				self.push_ghost(fp);
				return idx;
			} else {
				let idx = self.main.pop_front().expect("main checked non-empty");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq -= 1;
					self.main.push_back(idx);
					continue;
				}
				self.map.remove(&self.nodes[idx as usize].key);
				return idx;
			}
		}
	}

	/// Records a small-queue eviction in the ghost ring, overwriting the oldest slot.
	fn push_ghost(&mut self, fp: u64) {
		let old = self.ghost_ring[self.ghost_head];
		if old != 0 {
			// Saturating: a ghost hit may already have consumed this occurrence.
			Self::ghost_dec(&mut self.ghost_count, old);
		}
		self.ghost_ring[self.ghost_head] = fp;
		*self.ghost_count.entry(fp).or_insert(0) += 1;
		self.ghost_head += 1;
		if self.ghost_head == self.ghost_cap {
			self.ghost_head = 0;
		}
	}

	#[cfg(test)]
	fn in_ghost(&self, k: &K) -> bool {
		let fp = self.fingerprint(k);
		self.ghost_count.get(&fp).is_some_and(|&c| c > 0)
	}

	#[cfg(test)]
	fn ghost_occupied(&self) -> usize {
		self.ghost_ring.iter().filter(|&&fp| fp != 0).count()
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

	fn ghost_bounded(cache: &LruCache<i64, i64>) {
		assert_eq!(cache.ghost_ring.len(), cache.ghost_cap);
		assert!(cache.ghost_occupied() <= cache.ghost_cap);
		let live: u32 = cache.ghost_count.values().sum();
		assert!(live as usize <= cache.ghost_occupied());
		assert!(cache.ghost_count.len() <= cache.ghost_cap);
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
	}

	#[test]
	fn ghost_hit_admitted_into_main() {
		let mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 208 / 80 = 2
		assert_eq!(cache.ghost_cap, 2);
		assert_eq!(cache.small_target, 1);
		for k in 1..=16 {
			cache.put(k, k);
		}
		cache.put(17, 17); // evicts 1 into ghost
		assert!(cache.get(&1).is_none());
		assert!(cache.in_ghost(&1));
		cache.put(1, 100); // ghost hit -> straight into main
		assert!(in_main(&cache, 1));
		assert!(!cache.in_ghost(&1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 16);
		ghost_bounded(&cache);
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
			ghost_bounded(&cache);
		}
		assert!(cache.get(&1).is_none());
		assert!(!cache.in_ghost(&1));
		assert!(cache.in_ghost(&2));
		assert!(cache.in_ghost(&3));
		cache.put(1, 100); // no longer a ghost -> lands in small
		assert!(!in_main(&cache, 1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 16);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
	}

	#[test]
	fn consumed_ghost_slot_overwrite_saturates() {
		let mut cache = LruCache::new(16); // small_target = 1, ghost_cap = 2
		for k in 1..=16 {
			cache.put(k, k);
		}
		cache.put(17, 17); // 1 -> ghost
		cache.put(1, 1); // ghost hit consumes 1's count; its ring slot stays
		assert!(!cache.in_ghost(&1));
		// Churn enough small evictions to overwrite the stale slot; must not underflow.
		for k in 100..120 {
			cache.put(k, k);
			ghost_bounded(&cache);
		}
		assert!(!cache.in_ghost(&1));
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
		assert!(!cache.in_ghost(&1));
		assert_eq!(*cache.get(&3).unwrap(), 3);
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
			assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
			ghost_bounded(&cache);
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
		assert_eq!(cache.small_target, 25);
		assert_eq!(cache.ghost_cap, 332);
		// Combined admission window ~unchanged vs cycle 7 (51 + 307 = 358).
		assert_eq!(cache.small_target + cache.ghost_cap, 357);
		assert_eq!(cache.ghost_ring.len(), 332);
	}

	fn skewed_churn(capacity: usize, iters: usize) {
		let mut cache = LruCache::new(capacity);
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		let hot = (capacity as i64) * 3 / 4;
		let cold = (capacity as i64) * 5;
		for _ in 0..iters {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let r = (state >> 33) as i64;
			// Skewed mix: hot set plus a larger cold tail.
			let k = if r % 3 == 0 { (r >> 2) % cold + 1_000_000 } else { (r >> 2) % hot };
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.map.len() <= capacity);
			assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
			ghost_bounded(&cache);
		}
		for (k, &idx) in cache.map.iter() {
			assert_eq!(cache.nodes[idx as usize].value, *k * 10);
		}
	}

	#[test]
	fn capacity_bound_under_churn_400() {
		let mut cache = LruCache::new(400); // small_target = 5, ghost_cap = 65
		assert_eq!(cache.small_target, 5);
		assert_eq!(cache.ghost_cap, 65);
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
			assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
			ghost_bounded(&cache);
		}
		for k in 0..3000i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	fn capacity_bound_under_churn_1600() {
		let cache = LruCache::<i64, i64>::new(1600);
		assert_eq!(cache.small_target, 20);
		assert_eq!(cache.ghost_cap, 260);
		skewed_churn(1600, 100_000);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
