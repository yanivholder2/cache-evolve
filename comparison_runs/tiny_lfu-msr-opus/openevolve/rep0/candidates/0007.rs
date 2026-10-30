use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters: together with periodic halving this keeps the
/// sketch biased towards recent popularity instead of all-time popularity.
const MAX_COUNT: u32 = 15;
/// Counters are halved after `SAMPLE_FACTOR * capacity` increments; the hill
/// climber uses the same sample period.
const SAMPLE_FACTOR: usize = 10;
const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const WINDOW_PERCENT: f64 = 0.01;
const PROTECTED_PERCENT: f64 = 0.8;
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;

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
/// size is tuned online by a hill climber.
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
	hits: usize,
	misses: usize,
	prev_hit_rate: f64,
	step: f64,
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
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let window_max = (capacity as f64 * WINDOW_PERCENT) as usize;
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={window_max}");
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max,
			sketch: CountMinSketch::new(width, SAMPLE_FACTOR * capacity),
			last_miss: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -STEP_PERCENT * capacity as f64,
		})
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
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits += 1;
				self.last_miss = None;
				self.on_access(idx);
			}
			None => {
				self.misses += 1;
				self.last_miss = Some(hash);
			}
		}
		self.climb();
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
	/// probation LRU victim; the strictly more frequent one stays.
	fn evict(&mut self, candidate: usize) {
		let mut victim = self.lists[PROBATION as usize].head;
		if victim == candidate {
			victim = NIL;
		}
		if victim == NIL {
			victim = self.lists[PROTECTED as usize].head;
		}
		if victim == NIL {
			victim = self.lists[WINDOW as usize].head;
		}
		let loser = if victim == NIL {
			candidate
		} else if candidate == NIL {
			victim
		} else if self.sketch.estimate(self.nodes[candidate].hash)
			> self.sketch.estimate(self.nodes[victim].hash)
		{
			victim
		} else {
			candidate
		};
		self.unlink(loser);
		self.map.remove(&self.nodes[loser].key);
		self.free.push(loser);
	}

	/// Hill climbing on the window size (Caffeine's adaptive W-TinyLFU).
	fn climb(&mut self) {
		if self.map.len() < self.capacity / 2 {
			self.hits = 0;
			self.misses = 0;
			self.prev_hit_rate = 0.0;
			return;
		}
		let total = self.hits + self.misses;
		if total < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= RESTART_THRESHOLD {
			let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
			STEP_PERCENT * self.capacity as f64 * sign
		} else {
			STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;

		let adjust = amount as i64;
		if adjust > 0 {
			let quota = (adjust as usize).min(self.protected_max);
			self.protected_max -= quota;
			self.window_max += quota;
			self.demote_protected_overflow();
			for _ in 0..quota {
				let mut c = self.lists[PROBATION as usize].head;
				if c == NIL {
					c = self.lists[PROTECTED as usize].head;
				}
				if c == NIL {
					break;
				}
				self.relink(c, WINDOW);
			}
		} else if adjust < 0 {
			let quota = ((-adjust) as usize).min(self.window_max.saturating_sub(1));
			self.protected_max += quota;
			self.window_max -= quota;
			while self.lists[WINDOW as usize].len > self.window_max {
				let lru = self.lists[WINDOW as usize].head;
				self.relink(lru, PROBATION);
			}
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
	counts: Vec<u32>,
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

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
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
		let mut min = u32::MAX;
		for row in 0..self.depth {
			let s = self.slot(hash, row);
			slots[row] = s;
			min = min.min(self.counts[s]);
		}
		if min >= MAX_COUNT {
			return;
		}
		for &s in slots.iter().take(self.depth) {
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

	fn estimate(&self, hash: u64) -> u32 {
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
