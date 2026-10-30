use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 64;
/// 4-bit style saturation (as in Caffeine): bounds how much stale history a key can bank.
const MAX_COUNT: u8 = 15;
/// The sketch ages (halves every counter) after `SAMPLE_FACTOR * capacity` increments, and the
/// window hill climber re-evaluates over a request sample of the same length.
const SAMPLE_FACTOR: usize = 10;
const WINDOW_PERCENT: f64 = 0.01;
const PROTECTED_PERCENT: f64 = 0.8;
const CLIMB_STEP_PERCENT: f64 = 0.0625;
const CLIMB_STEP_DECAY: f64 = 0.98;
const CLIMB_RESTART_THRESHOLD: f64 = 0.05;

/// Number of the oldest probation entries examined when picking the admission victim; the
/// least frequent of them (oldest on ties) is the one weighed against the candidate.
const VICTIM_SAMPLE: usize = 5;
/// Number of the oldest protected entries also examined. A protected entry is chosen only when
/// it is strictly colder than every sampled probation entry, so long-stale protected residents
/// (whose sketch counts have aged to ~0) can be reclaimed instead of warmer probation entries.
const PROTECTED_VICTIM_SAMPLE: usize = 2;

const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	queue: usize,
}

/// W-TinyLFU: an admission window LRU in front of a segmented-LRU main region
/// (probation + protected). The window's LRU entry contests the probation LRU entry
/// through the frequency sketch; the window size is tuned online by a hill climber.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	heads: [Option<usize>; 3],
	tails: [Option<usize>; 3],
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: CountMinSketch,
	last_miss: Option<u64>,
	sample_hits: usize,
	sample_misses: usize,
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
		let width = capacity
			.saturating_mul(SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let window_max = (capacity as f64 * WINDOW_PERCENT) as usize;
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		let sample_size = capacity.saturating_mul(SAMPLE_FACTOR).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={window_max}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			heads: [None; 3],
			tails: [None; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max,
			sketch: CountMinSketch::new(width, sample_size),
			last_miss: None,
			sample_hits: 0,
			sample_misses: 0,
			sample_size,
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
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let full = self.map.len() >= self.capacity;
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.last_miss = None;
				self.on_hit(idx);
				if full {
					self.record_sample(true);
				}
				Some(&self.nodes[idx].value)
			}
			None => {
				// The simulator follows a miss with a put of the same key; remember it so
				// that one logical access is counted once in the sketch.
				self.last_miss = Some(hash);
				if full {
					self.record_sample(false);
				}
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let idx = self.alloc(key.clone(), value);
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		self.maintain();
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		let node = Node {
			key,
			value,
			prev: None,
			next: None,
			queue: WINDOW,
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

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			PROBATION => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
				self.demote_protected_overflow();
			}
			q => {
				self.unlink(idx);
				self.push_back(q, idx);
			}
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			match self.heads[PROTECTED] {
				Some(h) => {
					self.unlink(h);
					self.push_back(PROBATION, h);
				}
				None => break,
			}
		}
	}

	/// Move window overflow into probation; when over capacity, the moved entry
	/// (candidate) contests the main region's victim through the sketch.
	fn maintain(&mut self) {
		while self.lens[WINDOW] > self.window_max {
			let candidate = match self.heads[WINDOW] {
				Some(c) => c,
				None => break,
			};
			self.unlink(candidate);
			self.push_back(PROBATION, candidate);
			if self.map.len() > self.capacity {
				self.admit_or_reject(candidate);
			}
		}
		while self.map.len() > self.capacity {
			match self.main_victim(None) {
				Some(v) => self.evict(v),
				None => match self.heads[WINDOW] {
					Some(w) => self.evict(w),
					None => break,
				},
			}
		}
	}

	fn main_victim(&self, exclude: Option<usize>) -> Option<usize> {
		self.heads[PROBATION]
			.filter(|&v| Some(v) != exclude)
			.or_else(|| self.heads[PROTECTED])
	}

	/// Pick the admission victim: the least frequent of the `VICTIM_SAMPLE` oldest probation
	/// entries (the oldest wins ties, so equal frequencies keep plain LRU order). A stale hot
	/// entry at the probation head therefore no longer blocks every admission while colder,
	/// never-reused entries sit right behind it. Falls back to the protected LRU entry when
	/// probation holds nothing but the candidate.
	fn select_victim(&self, candidate: usize) -> Option<(usize, u8)> {
		let mut best: Option<(usize, u8)> = None;
		// Probation first (it wins ties), then the protected LRU tail: an aged-out protected
		// entry is displaced only if strictly colder than every sampled probation entry.
		for &(queue, limit) in [(PROBATION, VICTIM_SAMPLE), (PROTECTED, PROTECTED_VICTIM_SAMPLE)].iter() {
			let mut cursor = self.heads[queue];
			let mut examined = 0;
			while let Some(idx) = cursor {
				if examined >= limit || best.map_or(false, |(_, f)| f == 0) {
					break;
				}
				cursor = self.nodes[idx].next;
				if idx == candidate {
					continue;
				}
				examined += 1;
				let freq = self.sketch.estimate(hash_key(&self.nodes[idx].key));
				if best.map_or(true, |(_, f)| freq < f) {
					best = Some((idx, freq));
				}
			}
		}
		best
	}

	fn admit_or_reject(&mut self, candidate: usize) {
		let (victim, victim_freq) = match self.select_victim(candidate) {
			Some(v) => v,
			None => {
				self.evict(candidate);
				return;
			}
		};
		let candidate_freq = self.sketch.estimate(hash_key(&self.nodes[candidate].key));
		if candidate_freq > victim_freq {
			self.evict(victim);
		} else {
			self.evict(candidate);
		}
	}

	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Hill climber (Caffeine-style): after each sample, keep moving the window size in the
	/// direction that improved the hit rate, otherwise reverse; step decays, restarts on shifts.
	fn record_sample(&mut self, hit: bool) {
		if hit {
			self.sample_hits += 1;
		} else {
			self.sample_misses += 1;
		}
		let total = self.sample_hits + self.sample_misses;
		if total < self.sample_size {
			return;
		}
		let hit_rate = self.sample_hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		let limit = CLIMB_STEP_PERCENT * self.capacity as f64;
		self.step = if change.abs() >= CLIMB_RESTART_THRESHOLD {
			if amount >= 0.0 {
				limit
			} else {
				-limit
			}
		} else {
			CLIMB_STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.sample_hits = 0;
		self.sample_misses = 0;
		self.adjust_window(amount as i64);
	}

	fn adjust_window(&mut self, amount: i64) {
		if amount > 0 {
			let quota = (amount as usize).min(self.protected_max);
			if quota == 0 {
				return;
			}
			self.protected_max -= quota;
			self.window_max += quota;
			self.demote_protected_overflow();
			for _ in 0..quota {
				match self.heads[PROBATION].or(self.heads[PROTECTED]) {
					Some(idx) => {
						self.unlink(idx);
						self.push_back(WINDOW, idx);
					}
					None => break,
				}
			}
		} else if amount < 0 {
			if self.window_max <= 1 {
				return;
			}
			let quota = ((-amount) as usize).min(self.window_max - 1);
			self.window_max -= quota;
			self.protected_max += quota;
			while self.lens[WINDOW] > self.window_max {
				match self.heads[WINDOW] {
					Some(idx) => {
						self.unlink(idx);
						self.push_back(PROBATION, idx);
					}
					None => break,
				}
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.heads[q] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tails[q] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.lens[q] -= 1;
	}

	fn push_back(&mut self, q: usize, idx: usize) {
		self.nodes[idx].queue = q;
		self.nodes[idx].prev = self.tails[q];
		self.nodes[idx].next = None;
		match self.tails[q] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.heads[q] = Some(idx),
		}
		self.tails[q] = Some(idx);
		self.lens[q] += 1;
	}
}

fn hash_key<T: Hash + ?Sized>(key: &T) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
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
			sample_size: sample_size.max(1),
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u8>()
	}

	/// Conservative (minimal) increment with periodic halving (TinyLFU reset/aging).
	fn increment(&mut self, hash: u64) {
		let mut positions = [0usize; SKETCH_DEPTH];
		let mut min = u8::MAX;
		for (row, slot) in positions.iter_mut().enumerate() {
			let pos = self.position(hash, row);
			*slot = pos;
			min = min.min(self.counts[pos]);
		}
		if min >= MAX_COUNT {
			return;
		}
		for &pos in positions.iter() {
			if self.counts[pos] == min {
				self.counts[pos] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	fn estimate(&self, hash: u64) -> u8 {
		(0..SKETCH_DEPTH)
			.map(|row| self.counts[self.position(hash, row)])
			.min()
			.unwrap_or(0)
	}

	fn reset(&mut self) {
		for c in self.counts.iter_mut() {
			*c >>= 1;
		}
		self.additions /= 2;
	}

	fn position(&self, hash: u64, row: usize) -> usize {
		let mut z = hash.wrapping_add((row as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^= z >> 31;
		row * self.width + ((z as usize) & (self.width - 1))
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
	fn residency_is_bounded_and_consistent() {
		let mut cache = TinyLfuCache::new(200);
		for i in 0..50_000u64 {
			let key = if i % 3 == 0 { i % 40 } else { i.wrapping_mul(7919) % 3_000 };
			if cache.get(&key).is_none() {
				cache.put(key, key * 2);
			}
			assert!(cache.map.len() <= 200);
			assert_eq!(cache.lens.iter().sum::<usize>(), cache.map.len());
		}
		for (k, &idx) in cache.map.iter() {
			assert_eq!(cache.nodes[idx].key, *k);
			assert_eq!(cache.nodes[idx].value, *k * 2);
		}
	}
}
