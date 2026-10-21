use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
const DOOR_BITS_PER_ENTRY: usize = 8;
const SAMPLE_FACTOR: usize = 10;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>,
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width, capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch (counters plus doorkeeper bits).
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = None;
		match self.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
	/// Doorkeeper bit array (power-of-two bit count).
	door: Vec<u64>,
	door_mask: usize,
	additions: usize,
	sample_size: usize,
}

impl CountMinSketch {
	fn new(width: usize, capacity: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		let bits = (capacity * DOOR_BITS_PER_ENTRY).max(64).next_power_of_two();
		Self {
			width,
			depth,
			counts,
			door: vec![0u64; bits / 64],
			door_mask: bits - 1,
			additions: 0,
			sample_size: capacity.saturating_mul(SAMPLE_FACTOR).max(1),
		}
	}

	/// Heap bytes backing the counter table and the doorkeeper.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>() + self.door.len() * std::mem::size_of::<u64>()
	}

	fn door_probes<K: Hash>(&self, key: &K) -> (usize, usize) {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(0xD00D_5EED);
		key.hash(&mut hasher);
		let h = hasher.finish();
		((h as usize) & self.door_mask, ((h >> 32) as usize) & self.door_mask)
	}

	fn door_contains(&self, a: usize, b: usize) -> bool {
		(self.door[a >> 6] >> (a & 63)) & 1 == 1 && (self.door[b >> 6] >> (b & 63)) & 1 == 1
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		let (a, b) = self.door_probes(key);
		if !self.door_contains(a, b) {
			self.door[a >> 6] |= 1u64 << (a & 63);
			self.door[b >> 6] |= 1u64 << (b & 63);
			return;
		}
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	fn reset(&mut self) {
		for row in self.counts.iter_mut() {
			for c in row.iter_mut() {
				*c >>= 1;
			}
		}
		for w in self.door.iter_mut() {
			*w = 0;
		}
		self.additions = 0;
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		let base = self
			.counts
			.iter()
			.enumerate()
			.map(|(row, _)| self.counts[row][self.position(key, row as u64)])
			.min()
			.unwrap_or(0);
		let (a, b) = self.door_probes(key);
		base + if self.door_contains(a, b) { 1 } else { 0 }
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
