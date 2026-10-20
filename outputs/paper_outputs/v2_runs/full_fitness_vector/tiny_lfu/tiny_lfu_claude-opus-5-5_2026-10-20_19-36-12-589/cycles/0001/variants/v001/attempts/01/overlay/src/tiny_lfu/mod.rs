use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Region tags: which intrusive list a node currently lives on.
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const REGIONS: usize = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: u8,
}

/// W-TinyLFU: a small LRU admission window in front of a segmented-LRU main
/// region (probation + protected). All three lists are intrusive doubly-linked
/// lists over the shared `nodes` arena; `heads/tails/lens` are indexed by region.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	heads: [Option<usize>; REGIONS],
	tails: [Option<usize>; REGIONS],
	lens: [usize; REGIONS],
	capacity: usize,
	window_cap: usize,
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
		// Window gets ~1% (at least 1) unless that would leave no main region.
		let window_cap = if capacity >= 2 { (capacity / 100).max(1) } else { 0 };
		let main_cap = capacity - window_cap;
		let protected_cap = if main_cap >= 2 {
			(main_cap * 80 / 100).max(1)
		} else {
			0
		};
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window={window_cap} protected={protected_cap} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			heads: [None; REGIONS],
			tails: [None; REGIONS],
			lens: [0; REGIONS],
			capacity,
			window_cap,
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
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			// Not yet full: insert at window MRU, spill window LRU into probation.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				region: WINDOW,
			});
			self.push_back(idx, WINDOW);
			self.map.insert(key, idx);
			if self.lens[WINDOW as usize] > self.window_cap {
				let cand = self.heads[WINDOW as usize].expect("window non-empty");
				self.unlink(cand);
				self.push_back(cand, PROBATION);
			}
			return;
		}

		// Full: main-region victim is probation LRU, else protected LRU.
		let victim_idx = self.heads[PROBATION as usize]
			.or(self.heads[PROTECTED as usize])
			.expect("TinyLfuCache: full cache has no main-region victim");
		let victim_freq = self.sketch.estimate(&self.nodes[victim_idx].key);

		if self.window_cap == 0 {
			// Degenerate (capacity 1): the newcomer itself is the candidate.
			let incoming_freq = self.sketch.estimate(&key);
			if incoming_freq > victim_freq {
				self.evict(victim_idx);
				self.fill_slot(victim_idx, key, value, PROBATION);
			}
			return;
		}

		// Window is at window_cap when full; its LRU is the admission candidate.
		let cand_idx = self.heads[WINDOW as usize].expect("TinyLfuCache: full cache has empty window");
		let cand_freq = self.sketch.estimate(&self.nodes[cand_idx].key);
		let freed = if cand_freq > victim_freq {
			self.evict(victim_idx);
			self.unlink(cand_idx);
			self.push_back(cand_idx, PROBATION);
			victim_idx
		} else {
			self.evict(cand_idx);
			cand_idx
		};
		self.fill_slot(freed, key, value, WINDOW);
	}

	/// Movement on a hit (get hit or put on an existing key).
	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region {
			WINDOW => {
				self.unlink(idx);
				self.push_back(idx, WINDOW);
			}
			PROBATION => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
				if self.lens[PROTECTED as usize] > self.protected_cap {
					let demote = self.heads[PROTECTED as usize].expect("protected non-empty");
					self.unlink(demote);
					self.push_back(demote, PROBATION);
				}
			}
			_ => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
			}
		}
	}

	/// Remove a resident from the map and its list; its arena slot is then reused.
	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		let k = self.nodes[idx].key.clone();
		self.map.remove(&k);
	}

	fn fill_slot(&mut self, idx: usize, key: K, value: V, region: u8) {
		self.nodes[idx].key = key.clone();
		self.nodes[idx].value = value;
		self.push_back(idx, region);
		self.map.insert(key, idx);
	}

	fn unlink(&mut self, idx: usize) {
		let r = self.nodes[idx].region as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.heads[r] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tails[r] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.lens[r] -= 1;
	}

	fn push_back(&mut self, idx: usize, region: u8) {
		let r = region as usize;
		self.nodes[idx].region = region;
		self.nodes[idx].prev = self.tails[r];
		self.nodes[idx].next = None;
		match self.tails[r] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.heads[r] = Some(idx),
		}
		self.tails[r] = Some(idx);
		self.lens[r] += 1;
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

	fn check_structure(cache: &TinyLfuCache<i64, i64>) {
		assert!(cache.map.len() <= cache.capacity);
		assert_eq!(cache.lens.iter().sum::<usize>(), cache.map.len());
		for r in 0..REGIONS {
			let mut count = 0;
			let mut cur = cache.heads[r];
			while let Some(i) = cur {
				assert_eq!(cache.nodes[i].region as usize, r);
				assert_eq!(cache.map.get(&cache.nodes[i].key), Some(&i));
				count += 1;
				cur = cache.nodes[i].next;
			}
			assert_eq!(count, cache.lens[r]);
		}
		assert!(cache.lens[WINDOW as usize] <= cache.window_cap);
		assert!(cache.lens[PROTECTED as usize] <= cache.protected_cap);
	}

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
	fn cold_window_candidate_is_rejected_newcomer_lands_in_window() {
		// W-TinyLFU: the newcomer always enters the window; the admission
		// contest is between the window LRU (key 2) and the probation LRU (key 1).
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&1).is_some());
		assert!(cache.get(&3).is_some());
		assert!(cache.get(&2).is_none());
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
	fn structure_holds_across_capacities() {
		for cap in [1usize, 2, 3, 8, 50, 300] {
			let mut cache = TinyLfuCache::<i64, i64>::new(cap);
			let mut x: u64 = 12345;
			for _ in 0..5000 {
				x = x.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let k = ((x >> 33) % (cap as u64 * 4 + 3)) as i64;
				if cache.get(&k).is_none() {
					cache.put(k, k * 10);
				}
				check_structure(&cache);
			}
			for k in 0..(cap as i64 * 4 + 3) {
				if let Some(&v) = cache.get(&k) {
					assert_eq!(v, k * 10);
				}
			}
		}
	}
}
