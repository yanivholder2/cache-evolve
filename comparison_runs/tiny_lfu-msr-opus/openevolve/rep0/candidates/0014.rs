use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
/// Counters per sketch row, per unit of capacity. Admission contests are decided at the
/// cache's frequency boundary where true counts are tiny (~1-3 per sample period), so a
/// single collision flips the decision. Up to `RESET_MULTIPLIER * capacity` distinct keys
/// land in each row per period; a wide row keeps the all-rows-collide (count-min
/// overestimate) event rare even before conservative update.
const SKETCH_WIDTH_FACTOR: usize = 32;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters (as in Caffeine): bounded counts let periodic halving age history out.
const MAX_COUNT: u8 = 15;
/// The sketch halves every counter after `RESET_MULTIPLIER * capacity` recorded increments.
const RESET_MULTIPLIER: usize = 10;
const ROW_SEEDS: [u64; SKETCH_DEPTH] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0x85EB_CA77_C2B2_AE63,
];

/// Resident-set segments (W-TinyLFU): an admission-free LRU window in front of a
/// segmented-LRU main space (probation + protected) guarded by the TinyLFU filter.
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const WINDOW_PERCENT: f64 = 0.01;
const PROTECTED_PERCENT: f64 = 0.80;

/// Hill climber that adapts the window / main split to the observed hit rate.
const CLIMBER_STEP_PERCENT: f64 = 0.0625;
const CLIMBER_RESTART_THRESHOLD: f64 = 0.05;
const CLIMBER_STEP_DECAY: f64 = 0.98;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: Option<usize>,
	next: Option<usize>,
	queue: u8,
}

#[derive(Debug, Default, Clone, Copy)]
struct Queue {
	head: Option<usize>,
	tail: Option<usize>,
	len: usize,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	queues: [Queue; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: CountMinSketch,
	/// Sketch hash of the key the immediately preceding `get` looked up: a follow-up
	/// `put` of that key belongs to the same access and must not be counted twice.
	last_get: Option<u64>,
	sample_hits: usize,
	sample_count: usize,
	sample_size: usize,
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
		let window_max = (capacity as f64 * WINDOW_PERCENT) as usize;
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		let sample_size = capacity.saturating_mul(RESET_MULTIPLIER).max(1);
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			queues: [Queue::default(); 3],
			capacity,
			window_max,
			protected_max,
			sketch: CountMinSketch::new(width, sample_size),
			last_get: None,
			sample_hits: 0,
			sample_count: 0,
			sample_size,
			prev_hit_rate: 0.0,
			step: -CLIMBER_STEP_PERCENT * capacity as f64,
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
		let hash = CountMinSketch::hash(key);
		self.sketch.increment(hash);
		self.last_get = Some(hash);
		match self.map.get(key).copied() {
			Some(idx) => {
				self.on_hit(idx);
				self.record_sample(true);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.record_sample(false);
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = CountMinSketch::hash(&key);
		// A put right after a get (hit or miss) of the same key is one access: count it once.
		if self.last_get.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		let node = Node {
			key: key.clone(),
			value,
			hash,
			prev: None,
			next: None,
			queue: WINDOW,
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
		self.attach(idx, WINDOW);
		self.evict();
	}

	/// Shrink the window to its quota (overflow drops into probation as admission
	/// candidates), then run the TinyLFU contest until the cache fits its capacity.
	fn evict(&mut self) {
		let mut candidate = None;
		while self.queues[WINDOW as usize].len > self.window_max {
			let idx = self.queues[WINDOW as usize]
				.head
				.expect("TinyLfuCache: non-empty window has no head");
			self.detach(idx);
			self.attach(idx, PROBATION);
			candidate = Some(idx);
		}
		while self.map.len() > self.capacity {
			let victim = self.queues[PROBATION as usize]
				.head
				.or(self.queues[PROTECTED as usize].head)
				.or(self.queues[WINDOW as usize].head)
				.expect("TinyLfuCache: over-capacity cache has no victim");
			let loser = match candidate.filter(|&c| c != victim) {
				Some(c) => {
					let incoming = self.sketch.estimate(self.nodes[c].hash);
					let resident = self.sketch.estimate(self.nodes[victim].hash);
					if incoming > resident {
						victim
					} else {
						candidate = None;
						c
					}
				}
				None => {
					if candidate == Some(victim) {
						candidate = None;
					}
					victim
				}
			};
			self.remove(loser);
		}
	}

	fn remove(&mut self, idx: usize) {
		self.detach(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Promote on access: probation hits graduate to protected (demoting protected's
	/// LRU back to probation on overflow); window / protected hits move to their MRU.
	fn on_hit(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		self.detach(idx);
		if queue == PROBATION {
			self.attach(idx, PROTECTED);
			while self.queues[PROTECTED as usize].len > self.protected_max {
				self.demote_protected();
			}
		} else {
			self.attach(idx, queue);
		}
	}

	fn demote_protected(&mut self) {
		if let Some(idx) = self.queues[PROTECTED as usize].head {
			self.detach(idx);
			self.attach(idx, PROBATION);
		}
	}

	/// Hill climbing over the window size: keep moving in the same direction while
	/// the sampled hit rate improves, reverse when it drops.
	fn record_sample(&mut self, hit: bool) {
		if self.map.len() < self.capacity {
			return;
		}
		self.sample_count += 1;
		if hit {
			self.sample_hits += 1;
		}
		if self.sample_count < self.sample_size {
			return;
		}
		let hit_rate = self.sample_hits as f64 / self.sample_count as f64;
		let delta = hit_rate - self.prev_hit_rate;
		let amount = if delta >= 0.0 { self.step } else { -self.step };
		let restart = CLIMBER_STEP_PERCENT * self.capacity as f64;
		self.step = if delta.abs() >= CLIMBER_RESTART_THRESHOLD {
			if amount >= 0.0 {
				restart
			} else {
				-restart
			}
		} else {
			amount * CLIMBER_STEP_DECAY
		};
		self.prev_hit_rate = hit_rate;
		self.sample_hits = 0;
		self.sample_count = 0;
		self.adjust_window(amount);
	}

	fn adjust_window(&mut self, amount: f64) {
		let quota = amount.abs() as usize;
		if quota == 0 {
			return;
		}
		if amount > 0.0 {
			let quota = quota.min(self.protected_max);
			self.protected_max -= quota;
			self.window_max += quota;
			while self.queues[PROTECTED as usize].len > self.protected_max {
				self.demote_protected();
			}
		} else {
			let quota = quota.min(self.window_max);
			self.window_max -= quota;
			self.protected_max += quota;
		}
	}

	fn detach(&mut self, idx: usize) {
		let q = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.queues[q].head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.queues[q].tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.queues[q].len -= 1;
	}

	fn attach(&mut self, idx: usize, queue: u8) {
		let q = queue as usize;
		self.nodes[idx].queue = queue;
		self.nodes[idx].prev = self.queues[q].tail;
		self.nodes[idx].next = None;
		match self.queues[q].tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.queues[q].head = Some(idx),
		}
		self.queues[q].tail = Some(idx);
		self.queues[q].len += 1;
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
		self.counts.len() * std::mem::size_of::<u8>()
	}

	fn hash<K: Hash>(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Per-row index derived from one key hash via a seeded splitmix64 finaliser.
	fn slot(&self, hash: u64, row: usize) -> usize {
		let mut z = hash.wrapping_add(ROW_SEEDS[row]);
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^= z >> 31;
		row * self.width + ((z as usize) & (self.width - 1))
	}

	/// Conservative update: raise only the row counters sitting at the current minimum.
	fn increment(&mut self, hash: u64) {
		let mut slots = [0usize; SKETCH_DEPTH];
		for (row, slot) in slots.iter_mut().enumerate() {
			*slot = self.slot(hash, row);
		}
		let min = slots.iter().map(|&s| self.counts[s]).min().unwrap_or(0);
		if min >= MAX_COUNT {
			return;
		}
		for &s in &slots {
			if self.counts[s] == min {
				self.counts[s] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.age();
		}
	}

	fn estimate(&self, hash: u64) -> u8 {
		(0..self.depth)
			.map(|row| self.counts[self.slot(hash, row)])
			.min()
			.unwrap_or(0)
	}

	/// Halve every counter so stale popularity decays and new hot keys can compete.
	fn age(&mut self) {
		for count in self.counts.iter_mut() {
			*count >>= 1;
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

	#[test]
	fn segments_stay_consistent_and_bounded() {
		let mut cache = TinyLfuCache::new(200);
		let mut state: u64 = 0x2545_F491_4F6C_DD1D;
		for i in 0..50_000u64 {
			state ^= state << 13;
			state ^= state >> 7;
			state ^= state << 17;
			let key = if state % 4 == 0 { state % 5_000 } else { state % 150 };
			if cache.get(&key).is_none() {
				cache.put(key, i);
			}
			assert!(cache.map.len() <= 200);
			let queued: usize = cache.queues.iter().map(|q| q.len).sum();
			assert_eq!(queued, cache.map.len());
			assert!(cache.window_max + cache.protected_max <= 200);
		}
	}
}
