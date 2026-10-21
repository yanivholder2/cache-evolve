// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queue),
// with a TinyLFU count-min sketch gating every admission into `main` (H03, cycle 5).
// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.
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
const SKETCH_MUL: u64 = 0x9E37_79B9_7F4A_7C15;
const SKETCH_SEEDS: [u64; SKETCH_ROWS] = [
	0xC3A5_C85C_97CB_3127,
	0xB492_B66F_BE98_F273,
	0x9AE1_6A3B_2F90_404F,
	0xCBF2_9CE4_8422_2325,
];

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut s = DefaultHasher::new();
	key.hash(&mut s);
	s.finish()
}

/// 4-row count-min sketch of 4-bit-saturating (u8, max 15) counters with periodic halving.
#[derive(Debug)]
struct FreqSketch {
	table: Vec<u8>, // SKETCH_ROWS * width, row-major
	width: usize,
	mask: usize,
	shift: u32,
	additions: usize,
	sample_size: usize,
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = std::cmp::max(16, capacity.saturating_mul(2)).next_power_of_two();
		let shift = 64 - width.trailing_zeros();
		Self {
			table: vec![0u8; SKETCH_ROWS * width],
			width,
			mask: width - 1,
			shift,
			additions: 0,
			sample_size: std::cmp::max(1, capacity.saturating_mul(10)),
		}
	}

	#[inline]
	fn slot(&self, row: usize, h: u64) -> usize {
		let i = ((h ^ SKETCH_SEEDS[row]).wrapping_mul(SKETCH_MUL) >> self.shift) as usize & self.mask;
		row * self.width + i
	}

	fn increment(&mut self, h: u64) {
		for row in 0..SKETCH_ROWS {
			let s = self.slot(row, h);
			if self.table[s] < SKETCH_MAX {
				self.table[s] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}

	fn estimate(&self, h: u64) -> u8 {
		let mut m = u8::MAX;
		for row in 0..SKETCH_ROWS {
			m = std::cmp::min(m, self.table[self.slot(row, h)]);
		}
		m
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
		// H01 (cycle 4): capacity / 4 ghost window (was capacity / 2).
		let ghost_cap = std::cmp::max(1, capacity / 4);
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
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(hash_key(key));
		let idx = *self.map.get(key)? as usize;
		let node = &mut self.nodes[idx];
		if node.freq < MAX_FREQ {
			node.freq += 1;
		}
		Some(&node.value)
	}

	/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit that
	/// passes the TinyLFU admission gate.
	pub fn put(&mut self, key: K, value: V) {
		let h = hash_key(&key);
		self.sketch.increment(h);

		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			if node.freq < MAX_FREQ {
				node.freq += 1;
			}
			return;
		}

		let ghost_hit = self.ghost_set.remove(&key).is_some();

		let reuse = if self.map.len() >= self.capacity { Some(self.evict_one()) } else { None };
		// The freed slot (if any) is in no queue, so the gate's sweep cannot touch it.
		let to_main = ghost_hit && self.admit_to_main(h);

		let idx = match reuse {
			Some(slot) => {
				let node = &mut self.nodes[slot as usize];
				node.key = key.clone();
				node.value = value;
				node.freq = 0;
				node.in_main = to_main;
				slot
			}
			None => {
				let slot = self.nodes.len() as u32;
				self.nodes.push(Node {
					key: key.clone(),
					value,
					freq: 0,
					in_main: to_main,
				});
				slot
			}
		};

		if to_main {
			self.main.push_back(idx);
		} else {
			self.small.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.small.len() + self.main.len(), self.map.len());
	}

	/// TinyLFU gate for an entry about to join `main`. Active only when admitting would
	/// push `main` past its share (`capacity - small_target`), i.e. displace a resident.
	/// Then the candidate must be strictly more frequent than the real CLOCK victim.
	fn admit_to_main(&mut self, cand_hash: u64) -> bool {
		let main_limit = self.capacity - self.small_target;
		if self.main.len() + 1 <= main_limit {
			return true;
		}
		match self.main_victim() {
			None => true,
			Some(v) => {
				let vh = hash_key(&self.nodes[v as usize].key);
				self.sketch.estimate(cand_hash) > self.sketch.estimate(vh)
			}
		}
	}

	/// Runs main's CLOCK sweep until the front node has freq == 0, leaving it at the
	/// front, and returns its index (the node `evict_one` would evict from main next).
	/// Terminates within MAX_FREQ * main.len() steps; the decrements are the same ones
	/// `evict_one` would perform later.
	fn main_victim(&mut self) -> Option<u32> {
		loop {
			let idx = *self.main.front()?;
			let node = &mut self.nodes[idx as usize];
			if node.freq == 0 {
				return Some(idx);
			}
			node.freq -= 1;
			self.main.pop_front();
			self.main.push_back(idx);
		}
	}

	/// Evicts exactly one resident entry and returns its (now free) arena slot.
	/// Terminates: every iteration either lowers some node's freq or frees a slot.
	fn evict_one(&mut self) -> u32 {
		loop {
			if self.small.len() >= self.small_target || self.main.is_empty() {
				let idx = self.small.pop_front().expect("small must be non-empty when main is empty and cache is full");
				if self.nodes[idx as usize].freq > 0 {
					let h = hash_key(&self.nodes[idx as usize].key);
					if self.admit_to_main(h) {
						let node = &mut self.nodes[idx as usize];
						node.freq = 0;
						node.in_main = true;
						self.main.push_back(idx);
						continue;
					}
					// Gate rejected: treat as cold (fall through to the eviction path).
				}
				let evicted = self.nodes[idx as usize].key.clone();
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
		cache.put(11, 110); // 1 promoted to main (main far from full: gate inactive), 2 evicted
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
		let mut cache = LruCache::new(10);
		assert_eq!(cache.ghost_cap, 2);
		for k in 1..=10 {
			cache.put(k, k);
		}
		cache.put(11, 11); // evicts 1 into ghost
		assert!(cache.get(&1).is_none());
		assert!(cache.ghost_set.contains_key(&1));
		cache.put(1, 100); // ghost hit, main empty (gate inactive) -> straight into main
		assert!(in_main(&cache, 1));
		assert!(!cache.ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 10);
	}

	#[test]
	fn ghost_hit_rejected_by_sketch_lands_in_small() {
		let mut cache = LruCache::new(10); // small_target = 1, main share = 9
		for k in 1..=10 {
			cache.put(k, k);
		}
		// Make 1..=9 hot (resident freq and sketch counts); 10 stays a one-hit wonder.
		for _ in 0..5 {
			for k in 1..=9 {
				cache.get(&k);
			}
		}
		cache.put(11, 11); // promotes 1..=9 into main (filling it), evicts 10 -> ghost
		assert_eq!(cache.main.len(), 9);
		assert!(cache.ghost_set.contains_key(&10));
		let victim_key = 1i64;
		assert!(
			cache.sketch.estimate(hash_key(&10i64)) + 1 < cache.sketch.estimate(hash_key(&victim_key)),
			"test precondition: ghost key must be colder than the main victim"
		);
		cache.put(10, 100); // ghost hit, main full, colder than victim -> small
		assert!(!in_main(&cache, 10));
		assert_eq!(*cache.get(&10).unwrap(), 100);
		assert_eq!(cache.main.len(), 9);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
		assert!(cache.map.len() <= 10);
	}

	#[test]
	fn small_promotion_rejected_by_sketch_is_ghosted() {
		let mut cache = LruCache::new(10);
		for k in 1..=10 {
			cache.put(k, k);
		}
		for _ in 0..5 {
			for k in 1..=9 {
				cache.get(&k);
			}
		}
		cache.put(11, 11); // main now full with 1..=9; small = [11]
		assert_eq!(cache.main.len(), 9);
		cache.get(&11); // freq > 0 but sketch count low
		cache.put(12, 12); // 11 would be promoted, but gate rejects -> evicted + ghosted
		assert!(!cache.map.contains_key(&11));
		assert!(cache.ghost_set.contains_key(&11));
		assert_eq!(cache.main.len(), 9);
		assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
	}

	#[test]
	fn ghost_window_expires_after_ghost_cap_plus_one_evictions() {
		let mut cache = LruCache::new(10); // small_target = 1, ghost_cap = 2
		let ghost_cap = cache.ghost_cap as i64;
		assert_eq!(ghost_cap, 2);
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
		cache.put(2, 2); // 1 -> ghost
		cache.put(1, 1); // ghost hit -> main (main empty: no victim, admit); 2 -> ghost
		assert!(in_main(&cache, 1));
		cache.put(3, 3); // small empty, main non-empty: evict 1 from main
		assert!(cache.get(&1).is_none());
		assert!(!cache.ghost_set.contains_key(&1));
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
			assert!(cache.ghost_q.len() <= cache.ghost_cap);
		}
		for k in 0..37 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	fn sketch_saturates_and_ages() {
		let mut s = FreqSketch::new(4); // width 16, sample_size 40
		let h = hash_key(&7i64);
		for _ in 0..20 {
			s.increment(h);
		}
		assert_eq!(s.estimate(h), SKETCH_MAX);
		for i in 0..20i64 {
			s.increment(hash_key(&(1000 + i))); // reaches 40 additions -> halve
		}
		assert!(s.estimate(h) <= SKETCH_MAX / 2 + 1);
		assert_eq!(s.additions, 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
