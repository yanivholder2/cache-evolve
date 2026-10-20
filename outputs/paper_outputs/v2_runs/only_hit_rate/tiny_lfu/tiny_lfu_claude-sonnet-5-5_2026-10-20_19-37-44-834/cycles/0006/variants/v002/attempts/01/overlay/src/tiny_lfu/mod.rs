use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

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
	// Per-segment lists: head = LRU end, tail = MRU end.
	head: [Option<usize>; 3],
	tail: [Option<usize>; 3],
	len: [usize; 3],
	capacity: usize,
	window_cap: usize,
	main_cap: usize,
	protected_cap: usize,
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
		let window_cap = (capacity / 100).max(1);
		let main_cap = capacity - window_cap;
		// Protected takes 90% of main; probation keeps the remainder (at least 1 slot).
		let protected_cap = (main_cap * 90 / 100).min(main_cap.saturating_sub(1));
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: [None; 3],
			tail: [None; 3],
			len: [0; 3],
			capacity,
			window_cap,
			main_cap,
			protected_cap,
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
				seg: WINDOW,
			});
			self.push_back(idx, WINDOW);
			self.map.insert(key, idx);
			if self.len[WINDOW as usize] > self.window_cap {
				let cand = self.head[WINDOW as usize].expect("window overflow without head");
				self.unlink(cand);
				self.push_back(cand, PROBATION);
			}
			return;
		}

		// Full: the window is full; its LRU becomes the candidate.
		let cand = self.head[WINDOW as usize].expect("TinyLfuCache: full cache has empty window");
		let slot;
		if self.main_cap == 0 {
			// No main region: candidate is simply evicted.
			slot = cand;
			self.unlink(cand);
			let old = self.nodes[cand].key.clone();
			self.map.remove(&old);
		} else {
			// Victim: probation LRU, or protected LRU when probation is empty.
			let victim = match self.head[PROBATION as usize] {
				Some(v) => v,
				None => self.head[PROTECTED as usize].expect("main region empty while full"),
			};
			let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
			let victim_freq = self.sketch.estimate(&self.nodes[victim].key);
			// Tie-admit repeat keys (estimate >= 2); strict > otherwise.
			let admit = cand_freq > victim_freq || (cand_freq >= 2 && cand_freq == victim_freq);
			if admit {
				// Evict victim, candidate moves into main.
				let old = self.nodes[victim].key.clone();
				self.map.remove(&old);
				self.unlink(victim);
				self.unlink(cand);
				self.push_back(cand, PROBATION);
				slot = victim;
			} else {
				slot = cand;
				self.unlink(cand);
				let old = self.nodes[cand].key.clone();
				self.map.remove(&old);
			}
		}
		self.nodes[slot].key = key.clone();
		self.nodes[slot].value = value;
		self.push_back(slot, WINDOW);
		self.map.insert(key, slot);
	}

	fn touch(&mut self, idx: usize) {
		match self.nodes[idx].seg {
			PROBATION => {
				self.unlink(idx);
				if self.protected_cap == 0 {
					self.push_back(idx, PROBATION);
					return;
				}
				if self.len[PROTECTED as usize] >= self.protected_cap {
					if let Some(d1) = self.head[PROTECTED as usize] {
						// Consider the two least-recently-used protected entries;
						// demote the one with the lower estimate (tie -> older).
						let mut d = d1;
						if let Some(d2) = self.nodes[d1].next {
							let f1 = self.sketch.estimate(&self.nodes[d1].key);
							let f2 = self.sketch.estimate(&self.nodes[d2].key);
							if f2 < f1 {
								d = d2;
							}
						}
						self.unlink(d);
						self.push_back(d, PROBATION);
					}
				}
				self.push_back(idx, PROTECTED);
			}
			seg => {
				self.unlink(idx);
				self.push_back(idx, seg);
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let s = self.nodes[idx].seg as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head[s] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail[s] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.len[s] -= 1;
	}

	fn push_back(&mut self, idx: usize, seg: u8) {
		let s = seg as usize;
		self.nodes[idx].seg = seg;
		self.nodes[idx].prev = self.tail[s];
		self.nodes[idx].next = None;
		match self.tail[s] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head[s] = Some(idx),
		}
		self.tail[s] = Some(idx);
		self.len[s] += 1;
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
	fn capacity_bound() {
		let mut cache = TinyLfuCache::<i64, i64>::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let n = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(n <= 8);
	}
}
