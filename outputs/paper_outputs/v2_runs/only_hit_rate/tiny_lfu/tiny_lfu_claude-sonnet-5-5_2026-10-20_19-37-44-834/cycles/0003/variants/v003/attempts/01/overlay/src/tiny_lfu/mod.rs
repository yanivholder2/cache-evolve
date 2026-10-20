use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const COUNTERS_PER_WORD: usize = 16;
const MAX_COUNT: u64 = 15;

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
		let width = capacity.max(MIN_SKETCH_WIDTH).next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the packed 4-bit frequency sketch.
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

/// Count-min sketch with 4-bit saturating counters packed 16 per `u64`.
#[derive(Debug)]
struct CountMinSketch {
	/// Counters per row (power of two, >= 16).
	width: usize,
	/// Words per row.
	row_words: usize,
	words: Vec<u64>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let row_words = width / COUNTERS_PER_WORD;
		Self {
			width,
			row_words,
			words: vec![0u64; row_words * SKETCH_DEPTH],
		}
	}

	fn heap_bytes(&self) -> usize {
		self.words.len() * std::mem::size_of::<u64>()
	}

	fn hash<K: Hash>(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Returns (word index, bit shift) for each row, derived from one 64-bit hash.
	#[inline]
	fn slots(&self, h: u64) -> [(usize, u32); SKETCH_DEPTH] {
		const MULTS: [u64; SKETCH_DEPTH] = [
			0x9E37_79B9_7F4A_7C15,
			0xC2B2_AE3D_27D4_EB4F,
			0x1656_67B1_9E37_79F9,
			0xD6E8_FEB8_6659_FD93,
		];
		let mask = (self.width - 1) as u64;
		let mut out = [(0usize, 0u32); SKETCH_DEPTH];
		for row in 0..SKETCH_DEPTH {
			// Different 16-bit-rotated slice of the hash, re-mixed with an odd multiplier.
			let slice = h.rotate_right((row as u32) * 16);
			let mixed = (slice ^ (h >> 32)).wrapping_mul(MULTS[row]);
			let pos = ((mixed >> 32) & mask) as usize;
			let word = row * self.row_words + pos / COUNTERS_PER_WORD;
			let shift = ((pos % COUNTERS_PER_WORD) * 4) as u32;
			out[row] = (word, shift);
		}
		out
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		let slots = self.slots(Self::hash(key));
		let mut vals = [0u64; SKETCH_DEPTH];
		let mut min = u64::MAX;
		for (i, &(w, s)) in slots.iter().enumerate() {
			vals[i] = (self.words[w] >> s) & 0xF;
			min = min.min(vals[i]);
		}
		if min >= MAX_COUNT {
			return;
		}
		// Conservative update: only counters equal to the current minimum.
		for (i, &(w, s)) in slots.iter().enumerate() {
			if vals[i] == min {
				self.words[w] += 1u64 << s;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		let slots = self.slots(Self::hash(key));
		slots
			.iter()
			.map(|&(w, s)| ((self.words[w] >> s) & 0xF) as u32)
			.min()
			.unwrap_or(0)
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
}
