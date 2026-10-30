// Adaptive W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// * A small LRU *window* (starts at ~1% of capacity) admits every new key, so a
//   `put` is always immediately visible to the next `get`.
// * The *main* space is a segmented LRU: probation + protected (~80% of main).
// * When the window overflows, its LRU entry becomes a candidate for the main
//   space and must beat the probation LRU victim on estimated frequency
//   (TinyLFU admission: 4-bit count-min sketch with periodic halving). This
//   keeps one-hit wonders and scans from flushing the frequently used set.
// * A hill climber resizes the window/main split from the sampled hit rate, so
//   recency-biased workloads get a large window, frequency-biased a small one.
// * The main-space victim is the least-frequent of the few least-recent probation
//   entries (sampled LRU/LFU hybrid), and the sketch is 4x wider than Caffeine's
//   default to reduce collision noise in admission decisions.
// Arena + index-linked lists, O(1) per operation, no unsafe, std only.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

const ADMIT_HASHDOS_THRESHOLD: u64 = 6;
const CLIMBER_STEP_PERCENT: f64 = 0.0625;
const CLIMBER_RESTART_THRESHOLD: f64 = 0.05;
const CLIMBER_STEP_DECAY: f64 = 0.98;
/// Sketch words per unit of capacity (16 counters per word).
const SKETCH_WIDTH: usize = 4;
/// Number of least-recent probation entries examined when picking a victim.
const VICTIM_SAMPLE: usize = 4;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

/// 4-bit count-min sketch (4 hashed rows folded into one table) with aging.
#[derive(Debug)]
struct Sketch {
	table: Vec<u64>,
	mask: usize,
	additions: usize,
	sample: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		// Wider than Caffeine's default: fewer collisions -> sharper admission.
		let words = capacity
			.max(16)
			.saturating_mul(SKETCH_WIDTH)
			.next_power_of_two();
		Self {
			table: vec![0; words],
			mask: words * 16 - 1,
			additions: 0,
			sample: capacity.max(16).saturating_mul(10),
		}
	}

	fn slot(&self, hash: u64, row: u64) -> (usize, u32) {
		let mut x = hash.wrapping_add(row.wrapping_mul(0x9E37_79B9_7F4A_7C15));
		x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		x ^= x >> 31;
		let c = (x as usize) & self.mask;
		(c >> 4, ((c & 15) as u32) << 2)
	}

	fn frequency(&self, hash: u64) -> u64 {
		let mut freq = 15;
		for row in 0..4 {
			let (word, shift) = self.slot(hash, row);
			freq = freq.min((self.table[word] >> shift) & 15);
		}
		freq
	}

	/// Conservative update: only the minimal counters are incremented.
	fn increment(&mut self, hash: u64) {
		let freq = self.frequency(hash);
		if freq >= 15 {
			return;
		}
		for row in 0..4 {
			let (word, shift) = self.slot(hash, row);
			if (self.table[word] >> shift) & 15 == freq {
				self.table[word] += 1u64 << shift;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for word in self.table.iter_mut() {
				*word = (*word >> 1) & 0x7777_7777_7777_7777;
			}
			self.additions /= 2;
		}
	}
}

fn hash_of<T: Hash + ?Sized>(key: &T) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena (capacity + 1 slots max)
	free: Vec<usize>,       // evicted slots awaiting reuse
	heads: [usize; 3],      // LRU end of window / probation / protected
	tails: [usize; 3],      // MRU end of window / probation / protected
	sizes: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	last_miss: Option<u64>, // hash of last get-miss (avoid double counting on put)
	hits: usize,
	misses: usize,
	prev_hit_rate: f64,
	step: f64,
	rng: u64,
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
		// Caffeine defaults: window = 1% of capacity, protected = 80% of main.
		let window_max = capacity / 100 + usize::from(capacity % 100 != 0);
		let main = capacity - window_max;
		let protected_max = main / 5 * 4 + main % 5 * 4 / 5;
		Some(Self {
			map: HashMap::with_capacity(capacity.saturating_add(1)),
			nodes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			sizes: [0; 3],
			capacity,
			window_max,
			protected_max,
			sketch: Sketch::new(capacity),
			last_miss: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -CLIMBER_STEP_PERCENT * capacity as f64,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits += 1;
				self.on_hit(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				self.misses += 1;
				self.last_miss = Some(hash);
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; overflow is resolved
	/// by TinyLFU admission between the window victim and the main victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_of(&key);
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		let idx = self.alloc(Node {
			key: key.clone(),
			value,
			hash,
			prev: NIL,
			next: NIL,
			queue: WINDOW as u8,
		});
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		let mut candidate = NIL;
		while self.sizes[WINDOW] > self.window_max {
			let c = self.heads[WINDOW];
			self.unlink(c);
			self.push_back(PROBATION, c);
			if candidate == NIL {
				candidate = c;
			}
		}
		while self.map.len() > self.capacity {
			self.evict_one(candidate);
			candidate = NIL;
		}
	}

	fn alloc(&mut self, node: Node<K, V>) -> usize {
		match self.free.pop() {
			Some(i) => {
				self.nodes[i] = node;
				i
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		}
	}

	fn on_hit(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue as usize;
		self.unlink(idx);
		if queue == WINDOW {
			self.push_back(WINDOW, idx);
		} else {
			self.push_back(PROTECTED, idx);
			self.demote_protected();
		}
	}

	fn demote_protected(&mut self) {
		while self.sizes[PROTECTED] > self.protected_max {
			let c = self.heads[PROTECTED];
			self.unlink(c);
			self.push_back(PROBATION, c);
		}
	}

	/// Pick the main-space victim: the least-frequent among the `VICTIM_SAMPLE`
	/// least-recent probation entries (excluding `candidate`; ties keep the
	/// oldest), falling back to the protected LRU when probation has none.
	fn select_victim(&self, candidate: usize) -> usize {
		let mut node = self.heads[PROBATION];
		let mut best = NIL;
		let mut best_freq = u64::MAX;
		let mut seen = 0;
		while node != NIL && seen < VICTIM_SAMPLE {
			if node != candidate {
				let freq = self.sketch.frequency(self.nodes[node].hash);
				if freq < best_freq {
					best = node;
					best_freq = freq;
				}
				seen += 1;
			}
			node = self.nodes[node].next;
		}
		if best == NIL {
			best = self.heads[PROTECTED];
		}
		best
	}

	fn evict_one(&mut self, candidate: usize) {
		let victim = self.select_victim(candidate);
		let evict = if candidate == NIL {
			if victim == NIL {
				self.heads[WINDOW]
			} else {
				victim
			}
		} else if victim == NIL || !self.admit(candidate, victim) {
			candidate
		} else {
			victim
		};
		self.unlink(evict);
		self.map.remove(&self.nodes[evict].key);
		self.free.push(evict);
	}

	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.frequency(self.nodes[candidate].hash);
		let victim_freq = self.sketch.frequency(self.nodes[victim].hash);
		if candidate_freq > victim_freq {
			return true;
		}
		if candidate_freq < ADMIT_HASHDOS_THRESHOLD {
			return false;
		}
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		self.rng & 127 == 0
	}

	/// Hill climber: adapt the window size towards a higher sampled hit rate.
	fn climb(&mut self) {
		let total = self.hits + self.misses;
		if total < self.sketch.sample {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= CLIMBER_RESTART_THRESHOLD {
			let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
			CLIMBER_STEP_PERCENT * self.capacity as f64 * sign
		} else {
			CLIMBER_STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		if amount >= 1.0 {
			self.increase_window(amount as usize);
		} else if amount <= -1.0 {
			self.decrease_window((-amount) as usize);
		}
	}

	fn increase_window(&mut self, amount: usize) {
		let quota = amount.min(self.protected_max);
		if quota == 0 {
			return;
		}
		self.protected_max -= quota;
		self.window_max += quota;
		self.demote_protected();
		let mut moved = 0;
		while moved < quota {
			let mut c = self.heads[PROBATION];
			if c == NIL {
				c = self.heads[PROTECTED];
			}
			if c == NIL {
				break;
			}
			self.unlink(c);
			self.push_back(WINDOW, c);
			moved += 1;
		}
		let unused = quota - moved;
		self.protected_max += unused;
		self.window_max -= unused;
	}

	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_max - 1);
		if quota == 0 {
			return;
		}
		self.window_max -= quota;
		self.protected_max += quota;
		while self.sizes[WINDOW] > self.window_max {
			let c = self.heads[WINDOW];
			self.unlink(c);
			self.push_back(PROBATION, c);
		}
	}

	/// Remove node from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.heads[queue] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tails[queue] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.sizes[queue] -= 1;
	}

	/// Append node to the MRU end of `queue`.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.tails[queue];
		self.nodes[idx].queue = queue as u8;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail == NIL {
			self.heads[queue] = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.tails[queue] = idx;
		self.sizes[queue] += 1;
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

	fn check_invariants<K, V>(cache: &LruCache<K, V>) {
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.nodes.len() <= cache.capacity + 1);
		assert_eq!(cache.sizes.iter().sum::<usize>(), cache.map.len());
		assert!(cache.window_max >= 1);
		assert!(cache.sizes[WINDOW] <= cache.window_max);
		assert!(cache.sizes[PROTECTED] <= cache.protected_max);
		assert!(cache.window_max + cache.protected_max <= cache.capacity);
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
	fn overwrite_does_not_grow() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
		check_invariants(&cache);
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
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_invariants(&cache);
	}

	#[test]
	fn eviction_keeps_size_bounded_and_newest_key() {
		let mut cache = LruCache::new(3);
		for k in 1..=10 {
			cache.put(k, k * 10);
			check_invariants(&cache);
		}
		assert_eq!(cache.map.len(), 3);
		assert_eq!(*cache.get(&10).unwrap(), 100);
	}

	#[test]
	fn frequent_key_survives_scan() {
		let mut cache = LruCache::new(10);
		cache.put(0, 0);
		for _ in 0..20 {
			cache.get(&0);
		}
		for k in 1..=100 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			check_invariants(&cache);
		}
		assert!(cache.get(&0).is_some());
	}

	#[test]
	fn randomized_workload_preserves_invariants() {
		for &cap in &[1usize, 2, 3, 7, 50, 200] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 12345;
			for _ in 0..20_000 {
				x = x
					.wrapping_mul(6_364_136_223_846_793_005)
					.wrapping_add(1_442_695_040_888_963_407);
				let r = x >> 33;
				let k = if r & 1 == 0 {
					(r >> 1) % (cap as u64 + 1)
				} else {
					(r >> 1) % (cap as u64 * 8)
				};
				match cache.get(&k).copied() {
					Some(v) => assert_eq!(v, k + 1),
					None => {
						cache.put(k, k + 1);
						assert_eq!(cache.get(&k).copied(), Some(k + 1));
					}
				}
				check_invariants(&cache);
			}
		}
	}

	#[test]
	fn try_new_zero_returns_none() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
