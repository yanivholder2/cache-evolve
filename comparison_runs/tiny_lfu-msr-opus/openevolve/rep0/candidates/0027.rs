use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
/// Counters are bytes (they saturate at 15), so the sketch can be 4x wider than
/// a u32 table of the same footprint: far fewer collisions, sharper estimates.
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters: together with periodic halving this keeps the
/// sketch biased towards recent popularity instead of all-time popularity.
const MAX_COUNT: u8 = 15;
/// Counters are halved after `SAMPLE_FACTOR * capacity` increments.
const SAMPLE_FACTOR: usize = 10;
const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const WINDOW_PERCENT: f64 = 0.01;
const PROTECTED_PERCENT: f64 = 0.8;
/// Static window sizes (fractions of capacity) raced by the mini-simulator.
const MINI_WINDOWS: [f64; 9] = [0.01, 0.03, 0.06, 0.1, 0.16, 0.25, 0.4, 0.6, 0.8];
/// Target capacity of each sampled replica; keys are sampled at a rate of
/// roughly `MINI_TARGET / capacity` so every replica stays this small.
const MINI_TARGET: usize = 1024;
/// Sampled accesses per adaptation period, as a multiple of replica capacity.
const MINI_PERIOD_FACTOR: usize = 5;
/// Weight kept by earlier periods' hit counts at each decision.
const MINI_DECAY: f64 = 0.5;
const MINI_SAMPLE_SALT: u64 = 0x5851_f42d_4c95_7f2d;
/// The admission contest weighs the candidate against the least-frequent of
/// this many entries at the LRU end of probation (ties keep the oldest).
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

/// Intrusive LRU list (head = LRU, tail = MRU) over the arena.
#[derive(Debug, Clone, Copy)]
struct List {
	head: usize,
	tail: usize,
	len: usize,
}

const EMPTY_LIST: List = List { head: NIL, tail: NIL, len: 0 };

/// W-TinyLFU: an admission window (LRU) in front of a segmented-LRU main
/// region (probation + protected). Keys leaving the window must beat the
/// probation LRU victim's sketch estimate to enter the main region. The window
/// size is tuned online by a sampled mini-simulator.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: CountMinSketch,
	last_miss: Option<u64>,
	/// Window-size tuner; `None` for the replicas living inside the tuner.
	sim: Option<Box<MiniSim>>,
}

/// Mini-simulator (Einziger et al., "Adaptive Software Cache Management"):
/// scaled-down replicas of this cache, each pinned to a different static
/// window size, all fed the same spatially sampled key stream (a key is either
/// always or never sampled, so its frequency profile is preserved). Every
/// period the main cache adopts the window size of the replica with the best
/// recent hit count. Unlike a hill climber it jumps straight to the best size
/// and stays there, instead of oscillating around it on noisy period deltas.
#[derive(Debug)]
struct MiniSim {
	caches: Vec<TinyLfuCache<u64, ()>>,
	hits: Vec<usize>,
	scores: Vec<f64>,
	rate: u64,
	accesses: usize,
	period: usize,
}

impl MiniSim {
	fn new(capacity: usize) -> Self {
		let rate = (capacity / MINI_TARGET).max(1);
		let mini_capacity = (capacity / rate).max(1);
		let caches: Vec<TinyLfuCache<u64, ()>> = MINI_WINDOWS
			.iter()
			.map(|&percent| TinyLfuCache::<u64, ()>::build(mini_capacity, percent, None))
			.collect();
		Self {
			caches,
			hits: vec![0; MINI_WINDOWS.len()],
			scores: vec![0.0; MINI_WINDOWS.len()],
			rate: rate as u64,
			accesses: 0,
			period: MINI_PERIOD_FACTOR * mini_capacity,
		}
	}

	fn sketch_heap_bytes(&self) -> usize {
		self.caches.iter().map(|c| c.sketch.heap_bytes()).sum()
	}

	/// Replays one access on every replica (when its key is sampled); at the
	/// end of a period returns the window fraction that performed best.
	fn record(&mut self, hash: u64) -> Option<f64> {
		if mix(hash ^ MINI_SAMPLE_SALT) % self.rate != 0 {
			return None;
		}
		for (cache, hits) in self.caches.iter_mut().zip(self.hits.iter_mut()) {
			if cache.get(&hash).is_some() {
				*hits += 1;
			} else {
				cache.put(hash, ());
			}
		}
		self.accesses += 1;
		if self.accesses < self.period {
			return None;
		}
		self.accesses = 0;
		let mut best = 0;
		for i in 0..self.scores.len() {
			self.scores[i] = self.scores[i] * MINI_DECAY + self.hits[i] as f64;
			self.hits[i] = 0;
			if self.scores[i] > self.scores[best] {
				best = i;
			}
		}
		Some(MINI_WINDOWS[best])
	}
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		let sim = Box::new(MiniSim::new(capacity));
		let cache = Self::build(capacity, WINDOW_PERCENT, Some(sim));
		log::info!(
			"TinyLfuCache.try_new: capacity={} sketch_width={} window={}",
			capacity,
			cache.sketch.width,
			cache.window_max
		);
		Some(cache)
	}

	/// Builds a cache with a fixed initial window fraction; `sim` is the
	/// window tuner (absent for the tuner's own replicas, which stay static).
	fn build(capacity: usize, window_percent: f64, sim: Option<Box<MiniSim>>) -> Self {
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let window_max = ((capacity as f64 * window_percent) as usize).min(capacity);
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max,
			sketch: CountMinSketch::new(width, SAMPLE_FACTOR * capacity),
			last_miss: None,
			sim,
		}
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		// The mini-simulator's replica sketches are frequency metadata too.
		let replicas: usize = match self.sim.as_ref() {
			Some(sim) => sim.sketch_heap_bytes(),
			None => 0,
		};
		self.sketch.heap_bytes() + replicas
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.last_miss = None;
				self.on_access(idx);
			}
			None => {
				self.last_miss = Some(hash);
			}
		}
		self.adapt(hash);
		match found {
			Some(idx) => Some(&self.nodes[idx].value),
			None => None,
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		// A put that directly follows a get-miss of the same key is the same
		// logical access: count it once so misses are not double-weighted.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}

		let idx = match self.free.pop() {
			Some(i) => {
				let node = &mut self.nodes[i];
				node.key = key.clone();
				node.value = value;
				node.hash = hash;
				i
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					hash,
					prev: NIL,
					next: NIL,
					queue: WINDOW,
				});
				self.nodes.len() - 1
			}
		};
		self.map.insert(key, idx);
		self.push_back(idx, WINDOW);

		// Window overflow flows into probation as the admission candidate.
		let mut candidate = NIL;
		while self.lists[WINDOW as usize].len > self.window_max {
			candidate = self.lists[WINDOW as usize].head;
			self.relink(candidate, PROBATION);
		}
		if self.map.len() > self.capacity {
			self.evict(candidate);
		}
	}

	fn on_access(&mut self, idx: usize) {
		if self.nodes[idx].queue == WINDOW {
			self.relink(idx, WINDOW);
		} else {
			self.relink(idx, PROTECTED);
			self.demote_protected_overflow();
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.lists[PROTECTED as usize].len > self.protected_max {
			let lru = self.lists[PROTECTED as usize].head;
			self.relink(lru, PROBATION);
		}
	}

	/// Admission contest: the candidate (fresh from the window) against the
	/// least-frequent of the few oldest probation entries; the strictly more
	/// frequent one stays. Sampling a handful of LRU-end entries keeps the
	/// recency order (ties pick the oldest) while sparing popular keys that just
	/// slid out of protected and evicting cold one-hit wonders sitting behind them.
	fn evict(&mut self, candidate: usize) {
		let mut victim = NIL;
		let mut victim_freq = 0u8;
		let mut cur = self.lists[PROBATION as usize].head;
		let mut sampled = 0;
		while cur != NIL && sampled < VICTIM_SAMPLE {
			if cur != candidate {
				let f = self.sketch.estimate(self.nodes[cur].hash);
				if victim == NIL || f < victim_freq {
					victim = cur;
					victim_freq = f;
				}
				sampled += 1;
			}
			cur = self.nodes[cur].next;
		}
		if victim == NIL {
			victim = self.lists[PROTECTED as usize].head;
			if victim == NIL {
				victim = self.lists[WINDOW as usize].head;
			}
			if victim != NIL {
				victim_freq = self.sketch.estimate(self.nodes[victim].hash);
			}
		}
		let loser = if victim == NIL {
			candidate
		} else if candidate == NIL {
			victim
		} else if self.sketch.estimate(self.nodes[candidate].hash) > victim_freq {
			victim
		} else {
			candidate
		};
		self.unlink(loser);
		self.map.remove(&self.nodes[loser].key);
		self.free.push(loser);
	}

	/// Feeds the mini-simulator; when it reaches a verdict, re-partitions the
	/// cache to the best-performing static window size.
	fn adapt(&mut self, hash: u64) {
		let verdict = match self.sim.as_mut() {
			Some(sim) => sim.record(hash),
			None => None,
		};
		if let Some(percent) = verdict {
			self.resize_window((self.capacity as f64 * percent) as usize);
		}
	}

	/// Sets the window to `window` entries and the protected segment to its
	/// share of the remaining main region. Entries move between regions
	/// (oldest first, Caffeine-style) but none are evicted.
	fn resize_window(&mut self, window: usize) {
		self.window_max = window.min(self.capacity);
		self.protected_max =
			((self.capacity - self.window_max) as f64 * PROTECTED_PERCENT) as usize;
		self.demote_protected_overflow();
		while self.lists[WINDOW as usize].len < self.window_max {
			let mut c = self.lists[PROBATION as usize].head;
			if c == NIL {
				c = self.lists[PROTECTED as usize].head;
			}
			if c == NIL {
				break;
			}
			self.relink(c, WINDOW);
		}
		while self.lists[WINDOW as usize].len > self.window_max {
			let lru = self.lists[WINDOW as usize].head;
			self.relink(lru, PROBATION);
		}
	}

	fn relink(&mut self, idx: usize, queue: u8) {
		self.unlink(idx);
		self.push_back(idx, queue);
	}

	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue as usize)
		};
		if prev == NIL {
			self.lists[q].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.lists[q].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lists[q].len -= 1;
	}

	fn push_back(&mut self, idx: usize, queue: u8) {
		let tail = self.lists[queue as usize].tail;
		if tail == NIL {
			self.lists[queue as usize].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.lists[queue as usize].tail = idx;
		self.lists[queue as usize].len += 1;
		let node = &mut self.nodes[idx];
		node.prev = tail;
		node.next = NIL;
		node.queue = queue;
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<u8>,
	additions: usize,
	sample_size: usize,
}

fn mix(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

impl CountMinSketch {
	fn new(width: usize, sample_size: usize) -> Self {
		let depth = SKETCH_DEPTH;
		Self {
			width,
			depth,
			counts: vec![0; depth * width],
			additions: 0,
			sample_size: sample_size.max(1),
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u8>()
	}

	fn slot(&self, hash: u64, row: usize) -> usize {
		let h = mix(hash.wrapping_add((row as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15)));
		row * self.width + (h as usize & (self.width - 1))
	}

	/// Conservative update: only the counters equal to the key's current
	/// minimum are raised. The estimate stays an upper bound on the true count
	/// but suffers far less collision inflation, which sharpens the admission
	/// contest for the low-frequency keys where it matters most.
	fn increment(&mut self, hash: u64) {
		let mut slots = [0usize; SKETCH_DEPTH];
		let mut min = u8::MAX;
		for (row, slot) in slots.iter_mut().enumerate() {
			let s = self.slot(hash, row);
			*slot = s;
			min = min.min(self.counts[s]);
		}
		if min >= MAX_COUNT {
			return;
		}
		for &s in slots.iter() {
			if self.counts[s] == min {
				self.counts[s] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	/// Aging: halve every counter so stale popularity decays. Truncation of odd
	/// counters loses mass; account for it (Caffeine-style) so the next aging
	/// period has the intended length.
	fn reset(&mut self) {
		let mut odd = 0usize;
		for c in self.counts.iter_mut() {
			odd += (*c & 1) as usize;
			*c >>= 1;
		}
		self.additions = self.additions.saturating_sub(odd / self.depth) / 2;
	}

	fn estimate(&self, hash: u64) -> u8 {
		(0..self.depth)
			.map(|row| self.counts[self.slot(hash, row)])
			.min()
			.unwrap_or(0)
	}
}

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		TinyLfuCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		TinyLfuCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn eviction_prefers_hot_items() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn low_frequency_new_item_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn update_promotes_existing_item_and_admits_hot_arrivals() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}
}
