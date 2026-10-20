use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

const WIN: usize = 0;
const MAIN: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: u8,
}

/// Window + main TinyLFU. New keys enter an LRU window; a window victim must beat the main
/// LRU victim (by sketch frequency) to enter main. The window size is adapted by a
/// hill-climber that compares per-period hit ratios.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	heads: [Option<usize>; 2],
	tails: [Option<usize>; 2],
	lens: [usize; 2],
	capacity: usize,
	sketch: CountMinSketch,
	// hill-climber state
	target: f32,
	step: f32,
	dir: f32,
	prev_ratio: f32,
	hits: u32,
	accesses: u32,
	period: u32,
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
		let step = (capacity as f32 * 0.01).max(0.25);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			heads: [None; 2],
			tails: [None; 2],
			lens: [0; 2],
			capacity,
			sketch: CountMinSketch::new(width),
			target: (capacity as f32 * 0.01).max(1.0),
			step,
			dir: 1.0,
			prev_ratio: 0.0,
			hits: 0,
			accesses: 0,
			period: (capacity.saturating_mul(10)).min(u32::MAX as usize) as u32,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	fn target_int(&self) -> usize {
		let max = ((self.capacity as f32) * 0.8) as usize;
		(self.target.round() as usize).clamp(1, max.max(1))
	}

	fn adapt(&mut self) {
		let ratio = self.hits as f32 / self.accesses as f32;
		if ratio < self.prev_ratio {
			self.dir = -self.dir;
			self.step *= 0.98;
		}
		self.prev_ratio = ratio;
		let max = ((self.capacity as f32) * 0.8).max(1.0);
		self.target = (self.target + self.dir * self.step).clamp(1.0, max);
		self.hits = 0;
		self.accesses = 0;
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		self.accesses += 1;
		let found = self.map.get(key).copied();
		if found.is_some() {
			self.hits += 1;
		}
		if self.accesses >= self.period {
			self.adapt();
		}
		let idx = found?;
		let seg = self.nodes[idx].seg as usize;
		self.unlink(idx);
		self.push_back(idx, seg);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let seg = self.nodes[idx].seg as usize;
			self.unlink(idx);
			self.push_back(idx, seg);
			return;
		}

		let t = self.target_int();

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				seg: WIN as u8,
			});
			self.push_back(idx, WIN);
			self.map.insert(key, idx);
			if self.lens[WIN] > t {
				let c = self.heads[WIN].expect("window nonempty");
				self.unlink(c);
				self.push_back(c, MAIN);
			}
			return;
		}

		let w = self.lens[WIN];
		// Window under target: grow it by taking the main LRU slot (no eviction contest).
		if w < t && self.lens[MAIN] > 0 {
			let v = self.heads[MAIN].expect("main nonempty");
			self.replace(v, key, value, WIN);
			return;
		}
		// No main segment: window LRU is simply replaced.
		if self.lens[MAIN] == 0 {
			let c = self.heads[WIN].expect("full cache has a victim");
			self.replace(c, key, value, WIN);
			return;
		}

		let c = self.heads[WIN].expect("window nonempty");
		let v = self.heads[MAIN].expect("main nonempty");
		let cf = self.sketch.estimate(&self.nodes[c].key);
		let vf = self.sketch.estimate(&self.nodes[v].key);
		let dest = if w > t { MAIN } else { WIN };
		if cf > vf {
			// Candidate beats main victim: candidate moves to main, victim's slot is reused.
			self.unlink(c);
			self.push_back(c, MAIN);
			self.replace(v, key, value, dest);
		} else {
			self.replace(c, key, value, dest);
		}
	}

	/// Evict the entry in slot `idx` and reuse the slot for `key` in segment `seg`.
	fn replace(&mut self, idx: usize, key: K, value: V, seg: usize) {
		let old = self.nodes[idx].key.clone();
		self.map.remove(&old);
		self.unlink(idx);
		self.nodes[idx].key = key.clone();
		self.nodes[idx].value = value;
		self.push_back(idx, seg);
		self.map.insert(key, idx);
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].seg as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.heads[seg] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tails[seg] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.lens[seg] -= 1;
	}

	fn push_back(&mut self, idx: usize, seg: usize) {
		self.nodes[idx].seg = seg as u8;
		self.nodes[idx].prev = self.tails[seg];
		self.nodes[idx].next = None;
		match self.tails[seg] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.heads[seg] = Some(idx),
		}
		self.tails[seg] = Some(idx);
		self.lens[seg] += 1;
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self
			.counts
			.iter()
			.enumerate()
			.map(|(row, _)| self.counts[row][self.position(key, row as u64)])
			.min()
			.unwrap_or(0)
	}

	fn position<K: Hash>(&self, key: &K, seed: u64) -> usize {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(seed);
		key.hash(&mut hasher);
		(hasher.finish() as usize) & (self.width - 1)
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
