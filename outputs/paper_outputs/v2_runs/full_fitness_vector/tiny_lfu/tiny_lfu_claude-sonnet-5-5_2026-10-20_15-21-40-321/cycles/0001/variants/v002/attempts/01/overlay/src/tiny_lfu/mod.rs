use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

const SEG_WINDOW: u8 = 0;
const SEG_MAIN: u8 = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: u8,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	// main segment list
	head: Option<usize>,
	tail: Option<usize>,
	// window segment list
	win_head: Option<usize>,
	win_tail: Option<usize>,
	win_len: usize,
	win_size: usize,
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
		let win_size = (capacity / 100).max(1);
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={win_size}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			win_head: None,
			win_tail: None,
			win_len: 0,
			win_size,
			capacity,
			sketch: CountMinSketch::new(width),
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
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				seg: SEG_WINDOW,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			if self.win_len > self.win_size {
				let cand = self.win_head.expect("window non-empty");
				self.unlink(cand);
				self.push_back_main(cand);
			}
			return;
		}

		// Full: choose the slot to reuse.
		let cand = match self.win_head {
			Some(c) => c,
			None => {
				// Window empty (should not happen): replace main head directly.
				let v = self.head.expect("TinyLfuCache: full cache has no victim");
				self.replace_slot(v, key, value);
				return;
			}
		};
		let slot = match self.head {
			Some(victim) => {
				let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
				let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
				if cand_freq > victim_freq {
					// Evict the main victim; candidate moves into main.
					let vkey = self.nodes[victim].key.clone();
					self.map.remove(&vkey);
					self.unlink(victim);
					self.unlink(cand);
					self.push_back_main(cand);
					victim
				} else {
					cand
				}
			}
			None => cand,
		};
		self.replace_slot(slot, key, value);
	}

	/// Reuse `slot` (which must be detached or linked anywhere) for a new key in the window tail.
	fn replace_slot(&mut self, slot: usize, key: K, value: V) {
		if self.nodes[slot].prev.is_some()
			|| self.nodes[slot].next.is_some()
			|| self.win_head == Some(slot)
			|| self.head == Some(slot)
		{
			let old = self.nodes[slot].key.clone();
			self.map.remove(&old);
			self.unlink(slot);
		}
		self.nodes[slot].key = key.clone();
		self.nodes[slot].value = value;
		self.push_back(slot);
		self.map.insert(key, slot);
	}

	fn touch(&mut self, idx: usize) {
		let seg = self.nodes[idx].seg;
		self.unlink(idx);
		if seg == SEG_WINDOW {
			self.push_back(idx);
		} else {
			self.push_back_main(idx);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let window = self.nodes[idx].seg == SEG_WINDOW;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if window {
					self.win_head = next
				} else {
					self.head = next
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if window {
					self.win_tail = prev
				} else {
					self.tail = prev
				}
			}
		}
		if window {
			self.win_len -= 1;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append to the window tail.
	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].seg = SEG_WINDOW;
		self.nodes[idx].prev = self.win_tail;
		self.nodes[idx].next = None;
		match self.win_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.win_head = Some(idx),
		}
		self.win_tail = Some(idx);
		self.win_len += 1;
	}

	/// Append to the main tail.
	fn push_back_main(&mut self, idx: usize) {
		self.nodes[idx].seg = SEG_MAIN;
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
