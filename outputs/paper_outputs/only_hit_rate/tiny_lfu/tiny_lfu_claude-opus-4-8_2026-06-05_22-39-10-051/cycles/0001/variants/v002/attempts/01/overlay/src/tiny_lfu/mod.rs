use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Which LRU segment a resident node currently lives in.
///
/// W-TinyLFU fronts the main (admission-gated) region with a small LRU
/// *window* so that bursty newcomers are not immediately rejected by the
/// frequency sketch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Segment {
	Window,
	Main,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: Segment,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	free: Vec<usize>,
	// Window segment LRU (head = LRU victim, tail = MRU).
	head_window: Option<usize>,
	tail_window: Option<usize>,
	// Main segment LRU (head = LRU victim, tail = MRU).
	head_main: Option<usize>,
	tail_main: Option<usize>,
	window_len: usize,
	main_len: usize,
	window_cap: usize,
	main_cap: usize,
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
		// A small admission window in front of the main region. For tiny
		// capacities this collapses to window_cap == capacity (main_cap == 0),
		// i.e. a pure LRU window.
		let window_cap = (capacity / 100).max(1).min(capacity);
		let main_cap = capacity - window_cap;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window_cap={window_cap} main_cap={main_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			head_window: None,
			tail_window: None,
			head_main: None,
			tail_main: None,
			window_len: 0,
			main_len: 0,
			window_cap,
			main_cap,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		// LRU-promote within whichever segment the key currently occupies.
		let seg = self.nodes[idx].segment;
		self.unlink(idx);
		self.push_back(seg, idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let seg = self.nodes[idx].segment;
			self.unlink(idx);
			self.push_back(seg, idx);
			return;
		}

		// Never-seen key: always enters the window's MRU position first, with no
		// sketch gate. The admission contest happens only when the window
		// overflows.
		let idx = self.alloc_slot(key.clone(), value);
		self.push_back(Segment::Window, idx);
		self.map.insert(key, idx);
		self.window_len += 1;

		if self.window_len <= self.window_cap {
			return;
		}

		// Window over capacity: the window's LRU node is the admission candidate.
		let candidate = self
			.head_window
			.expect("TinyLfuCache: overflowing window has no candidate");

		if self.main_len < self.main_cap {
			// Main region still has room: the window node simply migrates into
			// main with no eviction.
			self.unlink(candidate);
			self.push_back(Segment::Main, candidate);
			self.window_len -= 1;
			self.main_len += 1;
		} else if self.main_cap == 0 {
			// Degenerate pure-LRU window (capacity == 1): evict the window LRU.
			self.evict(candidate);
			self.window_len -= 1;
		} else {
			// Main is full: contest the candidate against the main LRU victim.
			let victim = self
				.head_main
				.expect("TinyLfuCache: full main region has no victim");
			let candidate_key = self.nodes[candidate].key.clone();
			let victim_key = self.nodes[victim].key.clone();
			if self.sketch.estimate(&candidate_key) > self.sketch.estimate(&victim_key) {
				// Candidate wins: evict the victim and promote candidate into main.
				self.evict(victim);
				self.main_len -= 1;
				self.unlink(candidate);
				self.push_back(Segment::Main, candidate);
				self.window_len -= 1;
				self.main_len += 1;
			} else {
				// Candidate loses: drop it.
				self.evict(candidate);
				self.window_len -= 1;
			}
		}
	}

	/// Acquire a free arena slot (reusing an evicted one when available) and
	/// populate it with the newcomer's key/value.
	fn alloc_slot(&mut self, key: K, value: V) -> usize {
		if let Some(idx) = self.free.pop() {
			self.nodes[idx].key = key;
			self.nodes[idx].value = value;
			self.nodes[idx].prev = None;
			self.nodes[idx].next = None;
			self.nodes[idx].segment = Segment::Window;
			idx
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				prev: None,
				next: None,
				segment: Segment::Window,
			});
			idx
		}
	}

	/// Remove a resident node from its segment list and the map, returning its
	/// slot to the free list. Counters are adjusted by the caller.
	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		let key = self.nodes[idx].key.clone();
		self.map.remove(&key);
		self.free.push(idx);
	}

	fn segment_head(&self, seg: Segment) -> Option<usize> {
		match seg {
			Segment::Window => self.head_window,
			Segment::Main => self.head_main,
		}
	}

	fn segment_tail(&self, seg: Segment) -> Option<usize> {
		match seg {
			Segment::Window => self.tail_window,
			Segment::Main => self.tail_main,
		}
	}

	fn set_segment_head(&mut self, seg: Segment, value: Option<usize>) {
		match seg {
			Segment::Window => self.head_window = value,
			Segment::Main => self.head_main = value,
		}
	}

	fn set_segment_tail(&mut self, seg: Segment, value: Option<usize>) {
		match seg {
			Segment::Window => self.tail_window = value,
			Segment::Main => self.tail_main = value,
		}
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.set_segment_head(seg, next),
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.set_segment_tail(seg, prev),
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, seg: Segment, idx: usize) {
		self.nodes[idx].segment = seg;
		let tail = self.segment_tail(seg);
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.set_segment_head(seg, Some(idx)),
		}
		self.set_segment_tail(seg, Some(idx));
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
	fn capacity_is_never_exceeded() {
		let mut cache: TinyLfuCache<i64, i64> = TinyLfuCache::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8);
	}
}
