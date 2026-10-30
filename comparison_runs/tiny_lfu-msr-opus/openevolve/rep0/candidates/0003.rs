use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
/// 4-bit saturating counters (Caffeine-style): bounds the weight of stale popularity.
const MAX_COUNT: u8 = 15;
/// Sketch counters are halved after `SAMPLE_FACTOR * capacity` increments (aging);
/// the hill climber also samples its hit rate over this many requests.
const SAMPLE_FACTOR: usize = 10;
const NIL: usize = usize::MAX;

/// Queue tags / indices into `TinyLfuCache::lists`.
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

const PERCENT_MAIN_PROTECTED: f64 = 0.80;
const CLIMBER_STEP_PERCENT: f64 = 0.0625;
const CLIMBER_STEP_DECAY: f64 = 0.98;
const CLIMBER_RESTART_THRESHOLD: f64 = 0.05;

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

/// W-TinyLFU: LRU admission window -> TinyLFU contest -> SLRU main (probation/protected),
/// with a hill climber that adapts the window size to the workload.
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
	hits_in_sample: usize,
	misses_in_sample: usize,
	sample_size: usize,
	prev_hit_rate: f64,
	step_size: f64,
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
		let window_max = capacity / 100;
		let window_min = window_max.min(1);
		let protected_max = ((capacity - window_max) as f64 * PERCENT_MAIN_PROTECTED) as usize;
		let sample_size = capacity.saturating_mul(SAMPLE_FACTOR).max(1);
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
			hits_in_sample: 0,
			misses_in_sample: 0,
			sample_size,
			prev_hit_rate: 0.0,
			step_size: -CLIMBER_STEP_PERCENT * capacity as f64,
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
		let found = self.map.get(key).copied();
		let idx = match found {
			Some(idx) => idx,
			None => {
				// The caller usually follows a miss with a put: count that as ONE access.
				self.pending_miss = Some(hash);
				self.record_sample(false);
				return None;
			}
		};
		self.pending_miss = None;
		self.on_hit(idx);
		self.record_sample(true);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		if self.pending_miss.take() != Some(hash) {
			self.sketch.increment(hash);
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

	fn admit(&self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.estimate(self.nodes[candidate].hash);
		let victim_freq = self.sketch.estimate(self.nodes[victim].hash);
		candidate_freq > victim_freq
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Hill climber (Caffeine-style): every sample period, keep moving the window size in the
	/// direction that improved the hit rate, decaying the step; restart on large shifts.
	fn record_sample(&mut self, hit: bool) {
		if hit {
			self.hits_in_sample += 1;
		} else {
			self.misses_in_sample += 1;
		}
		let requests = self.hits_in_sample + self.misses_in_sample;
		if requests < self.sample_size {
			return;
		}
		let hit_rate = self.hits_in_sample as f64 / requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step_size } else { -self.step_size };
		self.step_size = if change.abs() >= CLIMBER_RESTART_THRESHOLD {
			let restart = CLIMBER_STEP_PERCENT * self.capacity as f64;
			if amount >= 0.0 {
				restart
			} else {
				-restart
			}
		} else {
			CLIMBER_STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits_in_sample = 0;
		self.misses_in_sample = 0;
		let adjustment = amount as i64;
		if adjustment > 0 {
			self.increase_window(adjustment as usize);
		} else if adjustment < 0 {
			self.decrease_window(adjustment.unsigned_abs() as usize);
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

	fn increment(&mut self, hash: u64) {
		let mut added = false;
		for row in 0..self.depth {
			let slot = row * self.width + self.position(hash, row);
			let counter = &mut self.counts[slot];
			if *counter < MAX_COUNT {
				*counter += 1;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.reset();
			}
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
