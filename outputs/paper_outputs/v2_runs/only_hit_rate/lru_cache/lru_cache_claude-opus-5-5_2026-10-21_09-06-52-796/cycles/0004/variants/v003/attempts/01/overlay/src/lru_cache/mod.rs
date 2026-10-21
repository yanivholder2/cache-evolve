// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queue).
// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.
// H03: ghost-hit re-admission into `main` is gated by a TinyLFU-style count-min
// frequency sketch when the cache is full.
// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is
// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const MAX_FREQ: u8 = 3;

const SKETCH_ROWS: usize = 4;
const SKETCH_MAX: u8 = 15;
const ROW_MULTIPLIERS: [u64; SKETCH_ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0xFF51_AFD7_ED55_8CCD,
];

/// Count-min sketch: 4 rows of saturating 4-bit-range (`u8`, capped at 15) counters,
/// with periodic halving (aging) once `additions` reaches `sample`.
#[derive(Debug)]
struct FreqSketch {
	table: Vec<u8>, // SKETCH_ROWS * width, row-major
	width: usize,   // power of two
	mask: usize,
	additions: usize,
	sample: usize,
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = std::cmp::max(16, capacity).next_power_of_two();
		Self {
			table: vec![0u8; SKETCH_ROWS * width],
			width,
			mask: width - 1,
			additions: 0,
			sample: std::cmp::max(1, 10 * capacity),
		}
	}

	fn hash_of<K: Hash>(key: &K) -> u64 {
		// DefaultHasher::new() uses fixed zero keys -> deterministic across runs.
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		h.finish()
	}

	#[inline]
	fn slot(&self, h: u64, row: usize) -> usize {
		let x = h.wrapping_mul(ROW_MULTIPLIERS[row]);
		let x = x ^ (x >> 29);
		row * self.width + ((x >> 32) as usize & self.mask)
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		let h = Self::hash_of(key);
		for row in 0..SKETCH_ROWS {
			let i = self.slot(h, row);
			if self.table[i] < SKETCH_MAX {
				self.table[i] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u8 {
		let h = Self::hash_of(key);
		let mut min = SKETCH_MAX;
		for row in 0..SKETCH_ROWS {
			let c = self.table[self.slot(h, row)];
			if c < min {
				min = c;
			}
		}
		min
	}
}

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
	ghost_q: VecDeque<K>,   // recently evicted-from-small keys, FIFO
	ghost_set: HashMap<K, ()>,
	sketch: FreqSketch,
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
		let small_target = std::cmp::max(1, capacity / 10);
		// H01: tighter ghost window (capacity / 2) instead of capacity - small_target.
		let ghost_cap = std::cmp::max(1, capacity / 2);
		log::info!("LruCache.try_new: capacity={capacity} small_target={small_target} ghost_cap={ghost_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			small: VecDeque::with_capacity(capacity),
			main: VecDeque::with_capacity(capacity),
			ghost_q: VecDeque::with_capacity(ghost_cap + 1),
			ghost_set: HashMap::with_capacity(ghost_cap + 1),
			sketch: FreqSketch::new(capacity),
			small_target,
			ghost_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, bumping its (saturating) access frequency.
	/// Every lookup (hit or miss) is recorded in the frequency sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let idx = *self.map.get(key)? as usize;
		let node = &mut self.nodes[idx];
		if node.freq < MAX_FREQ {
			node.freq += 1;
		}
		Some(&node.value)
	}

	/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit that
	/// either finds free space or out-scores the `main` front in the frequency sketch.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			if node.freq < MAX_FREQ {
				node.freq += 1;
			}
			return;
		}

		self.sketch.increment(&key);

		let ghost_hit = self.ghost_set.remove(&key).is_some();
		let full = self.map.len() >= self.capacity;

		// Decide before evict_one(), which may rotate main.front().
		let to_main = ghost_hit
			&& (!full
				|| match self.main.front() {
					Some(&front) => {
						self.sketch.estimate(&key) > self.sketch.estimate(&self.nodes[front as usize].key)
					}
					None => false,
				});

		let idx = if full {
			let slot = self.evict_one();
			let node = &mut self.nodes[slot as usize];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			node.in_main = to_main;
			slot
		} else {
			let slot = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
				in_main: to_main,
			});
			slot
		};

		if to_main {
			self.main.push_back(idx);
		} else {
			self.small.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
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
				let evicted = node.key.clone();
				self.map.remove(&evicted);
				self.push_ghost(evicted);
				return idx;
			} else {
				let idx = self.main.pop_front().expect("main checked non-empty");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq -= 1;
					self.main.push_back(idx);
					continue;
				}
				let evicted = node.key.clone();
				self.map.remove(&evicted);
				return idx;
			}
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

	/// Builds: key 2 promoted into `main` (front), key 1 evicted from `small` into the ghost.
	fn setup_ghost_vs_main_front(cache: &mut LruCache<i64, i64>, main_front_gets: usize) {
		cache.put(2, 2);
		cache.put(1, 1);
		for k in 3..=10 {
			cache.put(k, k);
		}
		for _ in 0..main_front_gets {
			cache.get(&2);
		}
		cache.put(11, 11); // 2 (freq > 0) -> main; 1 (freq 0) -> ghost
		assert!(in_main(cache, 2));
		assert_eq!(cache.main.front().copied(), Some(cache.map[&2]));
		assert!(!cache.map.contains_key(&1));
		assert!(cache.ghost_set.contains_key(&1));
	}

	#[test]
	fn ghost_hit_admitted_into_main() {
		let mut cache = LruCache::new(10);
		assert_eq!(cache.ghost_cap, 5);
		// Key 1 is looked up often (misses still count in the sketch).
		for _ in 0..10 {
			assert!(cache.get(&1).is_none());
		}
		setup_ghost_vs_main_front(&mut cache, 1);
		assert!(cache.sketch.estimate(&1) > cache.sketch.estimate(&2));
		cache.put(1, 100); // frequent ghost hit -> into main
		assert!(in_main(&cache, 1));
		assert!(!cache.ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 10);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
	}

	#[test]
	fn ghost_hit_admitted_into_main_when_not_full() {
		let mut cache = LruCache::new(10);
		cache.ghost_set.insert(1, ());
		cache.ghost_q.push_back(1);
		cache.put(1, 1); // not full -> ghost hit goes to main without comparison
		assert!(in_main(&cache, 1));
	}

	#[test]
	fn infrequent_ghost_hit_loses_to_frequent_main_front() {
		let mut cache = LruCache::new(10);
		setup_ghost_vs_main_front(&mut cache, 6);
		assert!(cache.sketch.estimate(&1) < cache.sketch.estimate(&2));
		cache.put(1, 100); // ghost hit, but rarer than main front -> small
		assert!(!in_main(&cache, 1));
		assert!(cache.small.contains(&cache.map[&1]));
		assert!(!cache.ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 10);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
	}

	#[test]
	fn sketch_never_underestimates_and_ages() {
		let mut s = FreqSketch::new(4); // width 16, sample 40
		assert_eq!(s.width, 16);
		assert_eq!(s.sample, 40);
		let counts: [(i64, usize); 4] = [(7, 10), (8, 3), (9, 1), (10, 5)];
		let mut total = 0;
		for &(k, n) in &counts {
			for _ in 0..n {
				s.increment(&k);
				total += 1;
			}
		}
		assert!(total < s.sample);
		for &(k, n) in &counts {
			assert!(s.estimate(&k) as usize >= n.min(SKETCH_MAX as usize));
		}
		assert_eq!(s.additions, total);
		// Fill up to exactly `sample` additions -> one halving.
		let mut k = 1000i64;
		while s.additions + 1 < s.sample {
			s.increment(&k);
			k += 1;
		}
		assert!(s.table.iter().any(|&c| c > 7));
		s.increment(&k);
		assert_eq!(s.additions, s.sample / 2);
		assert!(s.table.iter().all(|&c| c <= SKETCH_MAX / 2));
		assert!(s.estimate(&7) >= 5); // 10 halved
	}

	#[test]
	fn ghost_window_expires_after_ghost_cap_plus_one_evictions() {
		let mut cache = LruCache::new(10); // small_target = 1, ghost_cap = 5
		let ghost_cap = cache.ghost_cap as i64;
		assert_eq!(ghost_cap, 5);
		for k in 1..=10 {
			cache.put(k, k);
		}
		// ghost_cap + 1 small-queue evictions: keys 1..=ghost_cap+1.
		for k in 11..=(11 + ghost_cap) {
			cache.put(k, k);
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		assert!(cache.get(&1).is_none());
		assert!(!cache.ghost_set.contains_key(&1));
		assert!(cache.ghost_set.contains_key(&2));
		cache.put(1, 100); // no longer a ghost -> lands in small
		assert!(!in_main(&cache, 1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 10);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
	}

	#[test]
	fn main_evictions_not_ghosted() {
		let mut cache = LruCache::new(1);
		cache.put(1, 1);
		cache.get(&1); // freq > 0
		cache.put(2, 2); // 1 promoted to main, then evicted from main (not ghosted)
		assert!(!cache.map.contains_key(&1));
		assert!(!cache.ghost_set.contains_key(&1));
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 2);
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
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		for k in 0..37 {
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
