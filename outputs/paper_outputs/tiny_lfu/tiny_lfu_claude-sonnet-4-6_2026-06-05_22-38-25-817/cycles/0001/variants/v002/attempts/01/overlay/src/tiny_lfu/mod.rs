use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// True when this node is in the admission window; false when in the main LRU.
	/// A node is always in exactly one of the two lists, so prev/next serve both.
	in_window: bool,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// Main LRU list head (oldest / LRU end).
	head: Option<usize>,
	/// Main LRU list tail (newest / MRU end).
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
	/// Maximum number of slots reserved for the admission window.
	window_capacity: usize,
	/// Current number of nodes resident in the admission window.
	window_len: usize,
	/// Admission window head (LRU end of window).
	window_head: Option<usize>,
	/// Admission window tail (MRU end of window).
	window_tail: Option<usize>,
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
		let window_capacity = (capacity / 100).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} \
			 window_capacity={window_capacity}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width),
			window_capacity,
			window_len: 0,
			window_head: None,
			window_tail: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		if self.nodes[idx].in_window {
			// Promote from window to main LRU (MRU tail).
			self.window_unlink(idx);
			self.window_len -= 1;
			self.nodes[idx].in_window = false;
			self.push_back(idx);
		} else {
			// Already in main: refresh to MRU position.
			self.unlink(idx);
			self.push_back(idx);
		}
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);

		// Update an existing resident key (harness guarantees this won't happen,
		// but handle it for unit-test robustness).
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			if self.nodes[idx].in_window {
				self.window_unlink(idx);
				self.window_push_back(idx);
			} else {
				self.unlink(idx);
				self.push_back(idx);
			}
			return;
		}

		// ── Fill phase ────────────────────────────────────────────────────────
		// While the arena is not yet full, unconditionally admit every newcomer
		// into the admission window.
		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				in_window: true,
			});
			self.window_push_back(idx);
			self.window_len += 1;
			self.map.insert(key, idx);
			return;
		}

		// ── Arena full ────────────────────────────────────────────────────────

		// Edge case: main list is empty (all capacity slots are in the window,
		// which happens immediately after fill).  Fall back to direct TinyLFU
		// admission between the window LRU head and the incoming item.
		if self.head.is_none() {
			let evictee_idx = self
				.window_head
				.expect("TinyLfuCache: cache full but both lists empty");
			let evictee_key = self.nodes[evictee_idx].key.clone();
			let evictee_freq = self.sketch.estimate(&evictee_key);
			let incoming_freq = self.sketch.estimate(&key);

			if incoming_freq > evictee_freq {
				// Evict the window LRU; new item takes its slot (stays in window).
				self.window_unlink(evictee_idx);
				self.window_len -= 1;
				self.map.remove(&evictee_key);

				self.nodes[evictee_idx].key = key.clone();
				self.nodes[evictee_idx].value = value;
				// in_window was already true; keep it true
				self.window_push_back(evictee_idx);
				self.window_len += 1;
				self.map.insert(key, evictee_idx);
			}
			// else: drop the incoming item, nothing changes.
			return;
		}

		// Window below its target occupancy: unconditionally admit new item into
		// the window, evicting the main LRU head to free a slot.
		if self.window_len < self.window_capacity {
			let main_victim_idx = self.head.unwrap();
			let main_victim_key = self.nodes[main_victim_idx].key.clone();

			self.unlink(main_victim_idx);
			self.map.remove(&main_victim_key);

			self.nodes[main_victim_idx].key = key.clone();
			self.nodes[main_victim_idx].value = value;
			self.nodes[main_victim_idx].in_window = true;

			self.window_push_back(main_victim_idx);
			self.window_len += 1;
			self.map.insert(key, main_victim_idx);
			return;
		}

		// ── Windowed TinyLFU eviction ─────────────────────────────────────────
		// Window is full.  Eject its LRU head ("window evictee"), then compare
		// its sketch frequency against the main LRU head ("main victim").
		let window_evictee_idx = self
			.window_head
			.expect("TinyLfuCache: window_len > 0 but window_head is None");
		let window_evictee_key = self.nodes[window_evictee_idx].key.clone();
		let evictee_freq = self.sketch.estimate(&window_evictee_key);

		let main_victim_idx = self.head.unwrap();
		let main_victim_key = self.nodes[main_victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&main_victim_key);

		// Remove evictee from window list first (clears its prev/next).
		self.window_unlink(window_evictee_idx);
		self.window_len -= 1;
		self.nodes[window_evictee_idx].in_window = false;

		if evictee_freq > victim_freq {
			// Evictee beats the main victim:
			//   • Promote evictee → main MRU tail (reuse its current node index).
			//   • Evict main victim, reuse its slot for the incoming key → window.
			self.push_back(window_evictee_idx);
			// map[window_evictee_key] → window_evictee_idx remains valid ✓

			self.unlink(main_victim_idx);
			self.map.remove(&main_victim_key);

			self.nodes[main_victim_idx].key = key.clone();
			self.nodes[main_victim_idx].value = value;
			self.nodes[main_victim_idx].in_window = true;

			self.window_push_back(main_victim_idx);
			self.window_len += 1;
			self.map.insert(key, main_victim_idx);
		} else {
			// Evictee loses: drop it, put the incoming key into its recycled slot
			// at the window MRU tail.
			self.map.remove(&window_evictee_key);

			self.nodes[window_evictee_idx].key = key.clone();
			self.nodes[window_evictee_idx].value = value;
			self.nodes[window_evictee_idx].in_window = true;

			self.window_push_back(window_evictee_idx);
			self.window_len += 1;
			self.map.insert(key, window_evictee_idx);
		}
	}

	// ── Main LRU list helpers ─────────────────────────────────────────────────

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

	// ── Admission window list helpers ─────────────────────────────────────────
	// These mirror unlink/push_back but operate on window_head/window_tail.
	// A node lives in exactly one list at a time, so prev/next fields are shared.

	fn window_unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.window_head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.window_tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn window_push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.window_tail;
		self.nodes[idx].next = None;
		match self.window_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.window_head = Some(idx),
		}
		self.window_tail = Some(idx);
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
	fn capacity_never_exceeded() {
		let cap = 8usize;
		let mut cache = TinyLfuCache::<i64, i64>::new(cap);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= cap, "resident={resident} > cap={cap}");
	}

	#[test]
	fn value_fidelity() {
		let mut cache = TinyLfuCache::<i64, i64>::new(8);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		for k in 0..50i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10, "value mismatch for key {k}");
			}
		}
	}
}
