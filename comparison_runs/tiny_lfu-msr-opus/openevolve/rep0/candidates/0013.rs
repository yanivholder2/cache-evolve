use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters (Caffeine-style): bounds the weight of stale popularity.
const MAX_COUNT: u8 = 15;
/// Sketch counters are halved after `SAMPLE_FACTOR * capacity` increments (aging).
const SAMPLE_FACTOR: usize = 10;
const NIL: usize = usize::MAX;

/// Queue tags / indices into `TinyLfuCache::lists`.
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

/// Candidates at least this warm that lose the admission contest are still admitted with
/// probability 1/ADMIT_JITTER_ODDS (Caffeine-style), so warm keys are not starved forever by a
/// stale-but-equal probation victim.
const ADMIT_JITTER_MIN_FREQ: u8 = 6;
const ADMIT_JITTER_MASK: u64 = 127;

const PERCENT_MAIN_PROTECTED: f64 = 0.80;

/// Window fractions replayed by the sampled mini caches; the main window follows the winner.
const MINI_WINDOW_FRACTIONS: [f64; 12] = [
	0.0, 0.01, 0.02, 0.04, 0.07, 0.11, 0.16, 0.23, 0.32, 0.45, 0.60, 0.78,
];
/// Index of the 1% configuration, the main cache's initial window.
const MINI_INITIAL_CHOICE: usize = 1;
/// Mini caches see a hash-sampled 1/rate slice of the keys (SHARDS-style) and hold about
/// `capacity / rate` keys, with the rate chosen so that this stays near the target.
const MINI_TARGET_CAPACITY: usize = 2048;
/// The window is re-chosen every `ADAPT_PERIOD_FACTOR * capacity` requests from decayed scores.
const ADAPT_PERIOD_FACTOR: usize = 2;
const MINI_SCORE_DECAY: f64 = 0.9;
/// Smaller caches keep their fixed initial window.
const MIN_ADAPT_CAPACITY: usize = 32;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	queue: usize,
	prev: usize,
	next: usize,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize,
	tail: usize,
	len: usize,
}

impl List {
	const EMPTY: List = List {
		head: NIL,
		tail: NIL,
		len: 0,
	};
}

/// W-TinyLFU: LRU admission window -> TinyLFU contest -> SLRU main (probation/protected).
/// The window size tracks whichever sampled mini cache (fixed window fraction) scores best.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	window_min: usize,
	protected_max: usize,
	sketch: CountMinSketch,
	pending_miss: Option<u64>,
	minis: Vec<MiniSim>,
	mini_rate: u64,
	mini_choice: usize,
	requests: usize,
	adapt_period: usize,
	rng: u64,
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
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let (window_max, protected_max) = initial_split(capacity);
		let window_min = window_max.min(1);
		let sample_size = capacity.saturating_mul(SAMPLE_FACTOR).max(1);
		let mini_rate = (capacity / MINI_TARGET_CAPACITY).max(1);
		let minis: Vec<MiniSim> = if capacity >= MIN_ADAPT_CAPACITY {
			let mini_capacity = capacity / mini_rate;
			MINI_WINDOW_FRACTIONS
				.iter()
				.map(|&fraction| MiniSim::new(mini_capacity, fraction))
				.collect()
		} else {
			Vec::new()
		};
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={window_max} protected={protected_max}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			lists: [List::EMPTY; 3],
			capacity,
			window_max,
			window_min,
			protected_max,
			sketch: CountMinSketch::new(width, sample_size),
			pending_miss: None,
			minis,
			mini_rate: mini_rate as u64,
			mini_choice: MINI_INITIAL_CHOICE,
			requests: 0,
			adapt_period: capacity.saturating_mul(ADAPT_PERIOD_FACTOR).max(1),
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u8>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.record_access(hash);
		let found = self.map.get(key).copied();
		let idx = match found {
			Some(idx) => idx,
			None => {
				// The caller usually follows a miss with a put: count that as ONE access.
				self.pending_miss = Some(hash);
				self.tick();
				return None;
			}
		};
		self.pending_miss = None;
		self.on_hit(idx);
		self.tick();
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		if self.pending_miss.take() != Some(hash) {
			self.record_access(hash);
		}
		if let Some(idx) = self.map.get(&key).copied() {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		let node = Node {
			key: key.clone(),
			value,
			hash,
			queue: WINDOW,
			prev: NIL,
			next: NIL,
		};
		let idx = match self.free.pop() {
			Some(slot) => {
				self.nodes[slot] = node;
				slot
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		};
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		self.evict();
	}

	/// Promote on access: window/protected move to their MRU end; probation hits graduate
	/// to protected (overflow demotes the protected LRU back to probation).
	fn on_hit(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		self.unlink(idx);
		if queue == PROBATION {
			self.push_back(PROTECTED, idx);
			self.demote_protected();
		} else {
			self.push_back(queue, idx);
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let idx = self.lists[PROTECTED].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	/// Window overflow flows into probation as the admission candidate; while over capacity,
	/// the candidate contests the probation LRU victim via the sketch and the loser is evicted.
	fn evict(&mut self) {
		let mut candidate = NIL;
		while self.lists[WINDOW].len > self.window_max {
			let idx = self.lists[WINDOW].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
			candidate = idx;
		}
		while self.map.len() > self.capacity {
			if candidate == NIL {
				candidate = self.lists[WINDOW].head;
			}
			let victim = self.select_victim(candidate);
			let loser = if victim == NIL {
				candidate
			} else if candidate == NIL {
				victim
			} else if self.admit(candidate, victim) {
				victim
			} else {
				candidate
			};
			if loser == NIL {
				break;
			}
			self.remove(loser);
			candidate = NIL;
		}
	}

	fn select_victim(&self, candidate: usize) -> usize {
		let probation = self.lists[PROBATION].head;
		if probation != NIL && probation != candidate {
			return probation;
		}
		let protected = self.lists[PROTECTED].head;
		if protected != NIL {
			return protected;
		}
		let window = self.lists[WINDOW].head;
		if window != candidate {
			window
		} else if window != NIL {
			self.nodes[window].next
		} else {
			NIL
		}
	}

	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.estimate(self.nodes[candidate].hash);
		let victim_freq = self.sketch.estimate(self.nodes[victim].hash);
		if candidate_freq > victim_freq {
			return true;
		}
		if candidate_freq < ADMIT_JITTER_MIN_FREQ {
			return false;
		}
		// Warm candidate lost or tied: admit with a small probability (xorshift64).
		let mut x = self.rng;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.rng = x;
		(x & ADMIT_JITTER_MASK) == 0
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Count one access in the sketch and replay it in the hash-sampled mini caches.
	fn record_access(&mut self, hash: u64) {
		self.sketch.increment(hash);
		if self.minis.is_empty() || (hash >> 24) % self.mini_rate != 0 {
			return;
		}
		let sketch = &self.sketch;
		for mini in self.minis.iter_mut() {
			mini.access(hash, sketch);
		}
	}

	/// Every adaptation period, move the main window to the best-scoring mini configuration.
	/// All minis see the identical sampled stream, so their score gaps are low-noise.
	fn tick(&mut self) {
		if self.minis.is_empty() {
			return;
		}
		self.requests += 1;
		if self.requests < self.adapt_period {
			return;
		}
		self.requests = 0;
		for mini in self.minis.iter_mut() {
			mini.score = mini.score * MINI_SCORE_DECAY + mini.hits as f64;
			mini.hits = 0;
		}
		let mut best = self.mini_choice;
		for (i, mini) in self.minis.iter().enumerate() {
			if mini.score > self.minis[best].score {
				best = i;
			}
		}
		self.mini_choice = best;
		let budget = self.window_max + self.protected_max;
		let target = ((self.capacity as f64 * MINI_WINDOW_FRACTIONS[best]).round() as usize)
			.max(self.window_min)
			.min(budget);
		let current = self.window_max;
		if target > current {
			self.increase_window(target - current);
		} else if target < current {
			self.decrease_window(current - target);
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
			let mut idx = self.lists[PROBATION].head;
			if idx == NIL {
				idx = self.lists[PROTECTED].head;
			}
			if idx == NIL {
				break;
			}
			self.unlink(idx);
			self.push_back(WINDOW, idx);
			moved += 1;
		}
		let unused = quota - moved;
		self.protected_max += unused;
		self.window_max -= unused;
	}

	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_max.saturating_sub(self.window_min));
		if quota == 0 {
			return;
		}
		self.window_max -= quota;
		self.protected_max += quota;
		while self.lists[WINDOW].len > self.window_max {
			let idx = self.lists[WINDOW].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let (prev, next, queue) = {
			let node = &self.nodes[idx];
			(node.prev, node.next, node.queue)
		};
		if prev == NIL {
			self.lists[queue].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.lists[queue].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lists[queue].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.lists[queue].tail;
		{
			let node = &mut self.nodes[idx];
			node.queue = queue;
			node.prev = tail;
			node.next = NIL;
		}
		if tail == NIL {
			self.lists[queue].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.lists[queue].tail = idx;
		self.lists[queue].len += 1;
	}
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

/// Window/protected split shared by the main cache and the mini caches: a 1% window and an
/// 80%-protected main region; later window growth is taken from the protected quota, so
/// `window + protected` stays equal to the returned sum.
fn initial_split(capacity: usize) -> (usize, usize) {
	let window = capacity / 100;
	let protected = ((capacity - window) as f64 * PERCENT_MAIN_PROTECTED) as usize;
	(window, protected)
}

/// Mini-cache keys are already SipHash outputs: use them as their own hash.
#[derive(Debug, Default)]
struct PassThroughHasher(u64);

impl Hasher for PassThroughHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = self.0.rotate_left(8) ^ u64::from(b);
		}
	}

	fn write_u64(&mut self, n: u64) {
		self.0 = n;
	}
}

type HashIndex = HashMap<u64, usize, BuildHasherDefault<PassThroughHasher>>;

#[derive(Debug, Clone, Copy)]
struct MiniNode {
	hash: u64,
	queue: usize,
	prev: usize,
	next: usize,
}

/// A key-only W-TinyLFU with a FIXED window fraction, fed a hash-sampled slice of the trace
/// and sharing the main frequency sketch for its admission contests.
#[derive(Debug)]
struct MiniSim {
	map: HashIndex,
	nodes: Vec<MiniNode>,
	free: Vec<usize>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	hits: u64,
	score: f64,
}

impl MiniSim {
	fn new(capacity: usize, window_fraction: f64) -> Self {
		let (window0, protected0) = initial_split(capacity);
		let budget = window0 + protected0;
		let window_max = ((capacity as f64 * window_fraction).round() as usize).min(budget);
		Self {
			map: HashIndex::with_capacity_and_hasher(capacity + 1, Default::default()),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [List::EMPTY; 3],
			capacity,
			window_max,
			protected_max: budget - window_max,
			hits: 0,
			score: 0.0,
		}
	}

	/// One request for `hash`; a miss inserts it (as the caller's get-miss + put would).
	fn access(&mut self, hash: u64, sketch: &CountMinSketch) {
		if let Some(idx) = self.map.get(&hash).copied() {
			self.hits += 1;
			let queue = self.nodes[idx].queue;
			self.unlink(idx);
			if queue == PROBATION {
				self.push_back(PROTECTED, idx);
				while self.lists[PROTECTED].len > self.protected_max {
					let head = self.lists[PROTECTED].head;
					self.unlink(head);
					self.push_back(PROBATION, head);
				}
			} else {
				self.push_back(queue, idx);
			}
			return;
		}
		let node = MiniNode {
			hash,
			queue: WINDOW,
			prev: NIL,
			next: NIL,
		};
		let idx = match self.free.pop() {
			Some(slot) => {
				self.nodes[slot] = node;
				slot
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		};
		self.map.insert(hash, idx);
		self.push_back(WINDOW, idx);
		let mut candidate = NIL;
		while self.lists[WINDOW].len > self.window_max {
			let head = self.lists[WINDOW].head;
			self.unlink(head);
			self.push_back(PROBATION, head);
			candidate = head;
		}
		if self.map.len() <= self.capacity {
			return;
		}
		let mut victim = self.lists[PROBATION].head;
		if victim == candidate || victim == NIL {
			victim = self.lists[PROTECTED].head;
		}
		let loser = if candidate == NIL {
			if victim == NIL {
				self.lists[WINDOW].head
			} else {
				victim
			}
		} else if victim == NIL
			|| sketch.estimate(self.nodes[candidate].hash) <= sketch.estimate(self.nodes[victim].hash)
		{
			candidate
		} else {
			victim
		};
		if loser == NIL {
			return;
		}
		let key = self.nodes[loser].hash;
		self.unlink(loser);
		self.map.remove(&key);
		self.free.push(loser);
	}

	fn unlink(&mut self, idx: usize) {
		let MiniNode {
			prev, next, queue, ..
		} = self.nodes[idx];
		if prev == NIL {
			self.lists[queue].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.lists[queue].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lists[queue].len -= 1;
	}

	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.lists[queue].tail;
		{
			let node = &mut self.nodes[idx];
			node.queue = queue;
			node.prev = tail;
			node.next = NIL;
		}
		if tail == NIL {
			self.lists[queue].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.lists[queue].tail = idx;
		self.lists[queue].len += 1;
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

impl CountMinSketch {
	fn new(width: usize, sample_size: usize) -> Self {
		let depth = SKETCH_DEPTH;
		Self {
			width,
			depth,
			counts: vec![0; depth * width],
			additions: 0,
			sample_size,
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u8>()
	}

	/// Minimal-increment (conservative) update: only the counters holding the key's current
	/// minimum are bumped. The key's estimate still rises by one, but colliding cells inflate far
	/// less, so cold newcomers and probation victims are estimated more accurately.
	fn increment(&mut self, hash: u64) {
		let mut slots = [0usize; SKETCH_DEPTH];
		let mut min = MAX_COUNT;
		for (row, slot) in slots.iter_mut().enumerate() {
			*slot = row * self.width + self.position(hash, row);
			min = min.min(self.counts[*slot]);
		}
		if min >= MAX_COUNT {
			return;
		}
		for &slot in slots.iter() {
			if self.counts[slot] == min {
				self.counts[slot] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	/// Aging: halve every counter so the sketch tracks recent popularity.
	fn reset(&mut self) {
		let mut odd = 0usize;
		for counter in self.counts.iter_mut() {
			odd += (*counter & 1) as usize;
			*counter >>= 1;
		}
		self.additions = self.additions.saturating_sub(odd >> 2) >> 1;
	}

	fn estimate(&self, hash: u64) -> u8 {
		(0..self.depth)
			.map(|row| self.counts[row * self.width + self.position(hash, row)])
			.min()
			.unwrap_or(0)
	}

	fn position(&self, hash: u64, row: usize) -> usize {
		let mut x = hash ^ (row as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		x ^= x >> 31;
		(x as usize) & (self.width - 1)
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

	#[test]
	fn residency_stays_within_capacity() {
		let capacity = 200;
		let mut cache = TinyLfuCache::new(capacity);
		let mut x: u64 = 12345;
		for i in 0..50_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = if i % 3 == 0 { x % 50 } else { x % 5_000 };
			if cache.get(&key).is_none() {
				cache.put(key, i);
			}
			assert!(cache.map.len() <= capacity);
			let listed: usize = cache.lists.iter().map(|l| l.len).sum();
			assert_eq!(listed, cache.map.len());
			assert!(cache.lists[WINDOW].len <= cache.window_max);
			assert!(cache.lists[PROTECTED].len <= cache.protected_max);
		}
	}
}
