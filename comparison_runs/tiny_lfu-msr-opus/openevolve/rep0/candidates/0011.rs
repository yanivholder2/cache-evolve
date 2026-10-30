use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters (as in Caffeine's FrequencySketch).
const MAX_COUNT: u8 = 15;
/// Counters are halved after `SAMPLE_FACTOR * capacity` increments (aging), and the
/// hill climber samples the hit rate over the same number of lookups.
const SAMPLE_FACTOR: usize = 10;
/// A warm candidate that loses the contest is still admitted with probability 1/128.
const ADMIT_RANDOM_THRESHOLD: u8 = 6;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

const PROTECTED_FRACTION: f64 = 0.8;
const CLIMB_STEP_PERCENT: f64 = 0.0625;
const CLIMB_RESTART_THRESHOLD: f64 = 0.05;
const CLIMB_STEP_DECAY: f64 = 0.98;
const SEEDS: [u64; SKETCH_DEPTH] = [
	0xc3a5_c85c_97cb_3127,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
	0xcbf2_9ce4_8422_2325,
];

fn hash_of<Q: Hash + ?Sized>(key: &Q) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	segment: u8,
}

/// One intrusive LRU list (head = LRU, tail = MRU) threaded through the arena.
#[derive(Debug, Clone, Copy)]
struct Segment {
	head: usize,
	tail: usize,
	len: usize,
}

impl Segment {
	const EMPTY: Segment = Segment {
		head: NIL,
		tail: NIL,
		len: 0,
	};
}

/// W-TinyLFU: an admission window LRU in front of a segmented-LRU main space
/// (probation + protected); window overflow contests the main victim via the sketch.
/// The window/main split is tuned online by a hill climber.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	segments: [Segment; 3],
	capacity: usize,
	window_cap: usize,
	protected_cap: usize,
	min_window: usize,
	sketch: CountMinSketch,
	pending_miss: Option<u64>,
	rng: u64,
	sample_hits: usize,
	sample_misses: usize,
	climb_sample: usize,
	prev_hit_rate: f64,
	step: f64,
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
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		let window_cap = capacity / 100;
		let protected_cap = ((capacity - window_cap) as f64 * PROTECTED_FRACTION) as usize;
		let sample = SAMPLE_FACTOR.saturating_mul(capacity);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			segments: [Segment::EMPTY; 3],
			capacity,
			window_cap,
			protected_cap,
			min_window: window_cap.min(1),
			sketch: CountMinSketch::new(width, sample),
			pending_miss: None,
			rng: 0x9E37_79B9_7F4A_7C15,
			sample_hits: 0,
			sample_misses: 0,
			climb_sample: sample.max(1),
			prev_hit_rate: 0.0,
			step: -CLIMB_STEP_PERCENT * capacity as f64,
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
		let hash = hash_of(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.pending_miss = None;
				self.sample_hits += 1;
				self.on_access(idx);
			}
			None => {
				self.pending_miss = Some(hash);
				self.sample_misses += 1;
			}
		}
		self.maybe_climb();
		match found {
			Some(idx) => Some(&self.nodes[idx].value),
			None => None,
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_of(&key);
		// A miss is reported as `get` followed by `put`: count it as one access.
		if self.pending_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}

		let idx = self.alloc(key.clone(), value, hash);
		self.map.insert(key, idx);
		self.push_back(idx, WINDOW);

		// Window overflow flows into probation; the last one moved is the admission candidate.
		let mut candidate = None;
		while self.segments[WINDOW as usize].len > self.window_cap {
			let c = self.segments[WINDOW as usize].head;
			self.unlink(c);
			self.push_back(c, PROBATION);
			candidate = Some(c);
		}
		self.evict_excess(candidate);
	}

	fn alloc(&mut self, key: K, value: V, hash: u64) -> usize {
		let node = Node {
			key,
			value,
			hash,
			prev: NIL,
			next: NIL,
			segment: WINDOW,
		};
		match self.free.pop() {
			Some(idx) => {
				self.nodes[idx] = node;
				idx
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		}
	}

	fn evict_excess(&mut self, mut candidate: Option<usize>) {
		while self.map.len() > self.capacity {
			let victim = self.select_victim(candidate);
			match (candidate.take(), victim) {
				(Some(c), Some(v)) => {
					if self.admit(c, v) {
						self.evict(v);
					} else {
						self.evict(c);
					}
				}
				(Some(c), None) => self.evict(c),
				(None, Some(v)) => self.evict(v),
				(None, None) => break,
			}
		}
	}

	fn select_victim(&self, candidate: Option<usize>) -> Option<usize> {
		[PROBATION, PROTECTED, WINDOW]
			.iter()
			.map(|&seg| self.segments[seg as usize].head)
			.find(|&head| head != NIL && Some(head) != candidate)
	}

	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.estimate(self.nodes[candidate].hash);
		let victim_freq = self.sketch.estimate(self.nodes[victim].hash);
		if candidate_freq > victim_freq {
			true
		} else if candidate_freq >= ADMIT_RANDOM_THRESHOLD {
			(self.next_random() & 127) == 0
		} else {
			false
		}
	}

	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	fn on_access(&mut self, idx: usize) {
		match self.nodes[idx].segment {
			PROBATION => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
				self.demote_protected_overflow();
			}
			seg => {
				self.unlink(idx);
				self.push_back(idx, seg);
			}
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.segments[PROTECTED as usize].len > self.protected_cap {
			let head = self.segments[PROTECTED as usize].head;
			self.unlink(head);
			self.push_back(head, PROBATION);
		}
	}

	/// Hill climber: every sample period, move the window/main boundary in the
	/// direction that last improved the hit rate, with a decaying step.
	fn maybe_climb(&mut self) {
		if self.map.len() < self.capacity {
			// Warm-up: the hit rate rises regardless of the window size, so it carries
			// no signal for the climber (Caffeine likewise waits for a warm cache).
			self.sample_hits = 0;
			self.sample_misses = 0;
			return;
		}
		let total = self.sample_hits + self.sample_misses;
		if total < self.climb_sample {
			return;
		}
		let hit_rate = self.sample_hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= CLIMB_RESTART_THRESHOLD {
			let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
			CLIMB_STEP_PERCENT * self.capacity as f64 * sign
		} else {
			CLIMB_STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.sample_hits = 0;
		self.sample_misses = 0;
		let adjustment = amount as i64;
		if adjustment > 0 {
			self.grow_window(adjustment as usize);
		} else if adjustment < 0 {
			self.shrink_window(adjustment.unsigned_abs() as usize);
		}
	}

	fn grow_window(&mut self, amount: usize) {
		let quota = amount.min(self.protected_cap);
		if quota == 0 {
			return;
		}
		self.protected_cap -= quota;
		self.window_cap += quota;
		self.demote_protected_overflow();
		for _ in 0..quota {
			if self.segments[WINDOW as usize].len >= self.window_cap {
				break;
			}
			let probation_head = self.segments[PROBATION as usize].head;
			let idx = if probation_head != NIL {
				probation_head
			} else {
				self.segments[PROTECTED as usize].head
			};
			if idx == NIL {
				break;
			}
			self.unlink(idx);
			self.push_back(idx, WINDOW);
		}
	}

	fn shrink_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_cap.saturating_sub(self.min_window));
		if quota == 0 {
			return;
		}
		self.window_cap -= quota;
		self.protected_cap += quota;
		while self.segments[WINDOW as usize].len > self.window_cap {
			let head = self.segments[WINDOW as usize].head;
			self.unlink(head);
			self.push_back(head, PROBATION);
		}
	}

	fn next_random(&mut self) -> u64 {
		let mut x = self.rng;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.rng = x;
		x
	}

	fn unlink(&mut self, idx: usize) {
		let (prev, next, seg) = {
			let node = &self.nodes[idx];
			(node.prev, node.next, node.segment as usize)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.segments[seg].head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.segments[seg].tail = prev;
		}
		self.segments[seg].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	fn push_back(&mut self, idx: usize, seg: u8) {
		let tail = self.segments[seg as usize].tail;
		{
			let node = &mut self.nodes[idx];
			node.prev = tail;
			node.next = NIL;
			node.segment = seg;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.segments[seg as usize].head = idx;
		}
		let segment = &mut self.segments[seg as usize];
		segment.tail = idx;
		segment.len += 1;
	}
}

/// Count-min sketch of 4-bit counters with periodic halving (TinyLFU "reset").
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
			counts: vec![0; width * depth],
			additions: 0,
			sample_size: sample_size.max(1),
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.counts.len() * std::mem::size_of::<u8>()
	}

	fn index(&self, hash: u64, row: usize) -> usize {
		let mut x = hash ^ SEEDS[row];
		x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
		x ^= x >> 31;
		row * self.width + ((x as usize) & (self.width - 1))
	}

	/// Conservative update: raise only the counters currently holding the key's minimum,
	/// so colliding keys inflate each other's estimates far less.
	fn increment(&mut self, hash: u64) {
		let mut slots = [0usize; SKETCH_DEPTH];
		let mut min = MAX_COUNT;
		for (row, slot) in slots.iter_mut().enumerate() {
			*slot = self.index(hash, row);
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

	fn estimate(&self, hash: u64) -> u8 {
		(0..self.depth)
			.map(|row| self.counts[self.index(hash, row)])
			.min()
			.unwrap_or(0)
	}

	/// Age the history: halve every counter so stale popularity decays.
	fn reset(&mut self) {
		for c in self.counts.iter_mut() {
			*c >>= 1;
		}
		self.additions /= 2;
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
