// W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// A small LRU admission *window* sits in front of a segmented-LRU *main* region
// (probation + protected). A Count-Min sketch with 4-bit counters and periodic halving
// estimates key popularity; when the window overflows, its LRU entry (candidate) duels
// the probation LRU entry (victim) and only the more frequent one stays resident.
// A hill climber resizes the window/main split from sampled hit rates, so recency-biased
// workloads get a larger window and frequency-biased ones a smaller one.
// All queues live in one arena (Vec<Node>) linked by indices; freed slots are recycled.
// No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	queue: usize, // WINDOW, PROBATION or PROTECTED
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

fn mix(mut x: u64) -> u64 {
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	x ^ (x >> 33)
}

const SEEDS: [u64; 4] = [
	0x9e37_79b9_7f4a_7c15,
	0xbf58_476d_1ce4_e5b9,
	0x94d0_49bb_1331_11eb,
	0x2545_f491_4f6c_dd1d,
];

/// Count-Min sketch: 4 rows of packed 4-bit counters, conservative update, halved
/// every `sample_size` additions so popularity estimates age out.
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u8>, // two 4-bit counters per byte
	width: usize,   // counters per row (power of two)
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.max(64).next_power_of_two() * 4;
		Self {
			table: vec![0; width * 2], // 4 rows * width counters / 2 per byte
			width,
			additions: 0,
			sample_size: 10 * capacity.max(64),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			*slot = row * self.width + (mix(hash ^ SEEDS[row]) as usize & (self.width - 1));
		}
		out
	}

	fn counter(&self, s: usize) -> u8 {
		(self.table[s >> 1] >> ((s & 1) << 2)) & 0x0f
	}

	fn frequency(&self, hash: u64) -> u8 {
		let slots = self.slots(hash);
		slots.iter().map(|&s| self.counter(s)).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let slots = self.slots(hash);
		let min = slots.iter().map(|&s| self.counter(s)).min().unwrap_or(0);
		if min >= 15 {
			return;
		}
		for &s in &slots {
			if self.counter(s) == min {
				self.table[s >> 1] += 1u8 << ((s & 1) << 2);
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			for b in self.table.iter_mut() {
				*b = (*b >> 1) & 0x77;
			}
			self.additions /= 2;
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // recycled arena slots
	heads: [usize; 3],      // LRU end of each queue
	tails: [usize; 3],      // MRU end of each queue
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: FrequencySketch,
	last_miss: Option<u64>, // hash of the key whose get just missed (already counted)
	// hill-climber state
	sample_size: usize,
	hits_in_sample: usize,
	misses_in_sample: usize,
	prev_hit_rate: f64,
	step_size: f64,
	window_target: f64,
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
		log::info!("LruCache.try_new: capacity={capacity}");
		let window_max = (capacity / 100).max(1);
		Some(Self {
			map: HashMap::with_capacity(capacity.saturating_add(1)),
			nodes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max: (capacity - window_max) * 4 / 5,
			sketch: FrequencySketch::new(capacity),
			last_miss: None,
			sample_size: capacity.saturating_mul(10),
			hits_in_sample: 0,
			misses_in_sample: 0,
			prev_hit_rate: 0.0,
			step_size: -0.0625 * capacity as f64,
			window_target: window_max as f64,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present; records the access in the frequency sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.climb();
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits_in_sample += 1;
				self.last_miss = None;
				self.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.misses_in_sample += 1;
				self.last_miss = Some(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the admission window; when the cache is
	/// over capacity the TinyLFU duel decides which entry is evicted.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_key(&key);
		if self.last_miss != Some(hash) {
			self.sketch.increment(hash);
		}
		self.last_miss = None;
		let idx = match self.free.pop() {
			Some(i) => {
				self.nodes[i].key = key.clone();
				self.nodes[i].value = value;
				i
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					prev: NIL,
					next: NIL,
					queue: WINDOW,
				});
				self.nodes.len() - 1
			}
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		self.evict();
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			WINDOW => {
				self.unlink(idx);
				self.push_back(WINDOW, idx);
			}
			PROBATION => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
				self.demote_protected();
			}
			_ => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
			}
		}
	}

	fn demote_protected(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			let idx = self.heads[PROTECTED];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn victim(&self) -> usize {
		for q in [PROBATION, PROTECTED, WINDOW] {
			if self.heads[q] != NIL {
				return self.heads[q];
			}
		}
		NIL
	}

	fn admit(&self, candidate: usize, victim: usize) -> bool {
		let c = self.sketch.frequency(hash_key(&self.nodes[candidate].key));
		let v = self.sketch.frequency(hash_key(&self.nodes[victim].key));
		c > v
	}

	fn evict(&mut self) {
		// Window overflow flows into probation; the first moved entry is the candidate.
		let mut candidate = NIL;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
			if candidate == NIL {
				candidate = idx;
			}
		}
		while self.map.len() > self.capacity {
			let victim = self.victim();
			if candidate == NIL || candidate == victim {
				if candidate == victim {
					candidate = self.nodes[victim].next;
				}
				self.remove(victim);
			} else if self.admit(candidate, victim) {
				self.remove(victim);
			} else {
				let next = self.nodes[candidate].next;
				self.remove(candidate);
				candidate = next;
			}
		}
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Hill climber: adapt the window size toward a higher sampled hit rate.
	fn climb(&mut self) {
		let requests = self.hits_in_sample + self.misses_in_sample;
		if requests < self.sample_size {
			return;
		}
		let hit_rate = self.hits_in_sample as f64 / requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step_size } else { -self.step_size };
		self.step_size = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * if amount >= 0.0 { 1.0 } else { -1.0 }
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits_in_sample = 0;
		self.misses_in_sample = 0;
		if self.capacity < 2 {
			return;
		}
		let max_window = (self.capacity - 1) as f64;
		self.window_target = (self.window_target + amount).clamp(1.0, max_window);
		let w = self.window_target as usize;
		self.resize(w);
	}

	fn resize(&mut self, window_max: usize) {
		self.window_max = window_max;
		self.protected_max = (self.capacity - window_max) * 4 / 5;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
		self.demote_protected();
	}

	/// Remove node from its current queue.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.heads[q] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tails[q] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
		self.lens[q] -= 1;
	}

	/// Append node to the MRU end of queue `q`.
	fn push_back(&mut self, q: usize, idx: usize) {
		let tail = self.tails[q];
		self.nodes[idx].queue = q;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail == NIL {
			self.heads[q] = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.tails[q] = idx;
		self.lens[q] += 1;
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

	/// Simulator-style access: get, and on miss insert.
	fn access(cache: &mut LruCache<i64, i64>, key: i64) -> bool {
		if cache.get(&key).is_some() {
			true
		} else {
			cache.put(key, key * 10);
			false
		}
	}

	/// Verifies queue links, lengths, map consistency and size bounds.
	fn check_structure(cache: &LruCache<i64, i64>) {
		let mut total = 0;
		for q in 0..3 {
			let mut prev = NIL;
			let mut idx = cache.heads[q];
			let mut count = 0;
			while idx != NIL {
				let node = &cache.nodes[idx];
				assert_eq!(node.queue, q);
				assert_eq!(node.prev, prev);
				assert_eq!(cache.map[&node.key], idx);
				prev = idx;
				idx = node.next;
				count += 1;
			}
			assert_eq!(cache.tails[q], prev);
			assert_eq!(cache.lens[q], count);
			total += count;
		}
		assert_eq!(total, cache.map.len());
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.lens[WINDOW] <= cache.window_max);
		assert!(cache.lens[PROTECTED] <= cache.protected_max);
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::<i64, i64>::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::<i64, i64>::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow() {
		let mut cache = LruCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
		check_structure(&cache);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::<i64, i64>::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn newest_entry_is_always_resident() {
		let mut cache = LruCache::<i64, i64>::new(3);
		for k in 0..50 {
			cache.put(k, k);
			assert_eq!(cache.get(&k).copied(), Some(k));
			check_structure(&cache);
		}
	}

	#[test]
	fn frequent_items_survive_scan() {
		let mut cache = LruCache::<i64, i64>::new(10);
		for _ in 0..20 {
			for k in 0..6 {
				access(&mut cache, k);
			}
		}
		for k in 1000..1100 {
			access(&mut cache, k);
		}
		for k in 0..6 {
			assert!(cache.get(&k).is_some(), "hot key {k} was evicted by a scan");
		}
		check_structure(&cache);
	}

	#[test]
	fn random_workload_keeps_invariants() {
		let mut cache = LruCache::<i64, i64>::new(16);
		let mut x: u64 = 0x1234_5678;
		for step in 0..20_000i64 {
			x = x
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let r = (x >> 33) as i64;
			let key = if r % 4 == 0 { r % 500 } else { r % 24 };
			if step % 7 == 0 {
				cache.put(key, step);
				assert_eq!(*cache.get(&key).unwrap(), step);
			} else {
				access(&mut cache, key);
			}
			check_structure(&cache);
		}
	}

	#[test]
	fn try_new_zero_is_none() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
