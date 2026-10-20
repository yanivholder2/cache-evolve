use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u8 = u8::MAX;
const NIL: u32 = u32::MAX;
const MAIN: usize = 0;
const WIN: usize = 1;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// Segment membership per slot (true = window), kept outside Node to preserve its layout.
	in_window: Vec<bool>,
	free: Vec<u32>,
	heads: [u32; 2],
	tails: [u32; 2],
	lens: [usize; 2],
	capacity: usize,
	sketch: CountMinSketch,
	// Adaptive window state.
	window_cap: usize,
	step: usize,
	max_window: usize,
	direction: i8,
	period_hits: u64,
	period_ops: u64,
	period_len: u64,
	prev_hits: u64,
	prev_ops: u64,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 || capacity >= u32::MAX as usize {
			log::error!("TinyLfuCache.try_new: capacity must be in 1..u32::MAX");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		let max_window = (capacity / 2).max(1);
		let window_cap = (capacity / 100).max(1).min(max_window);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			in_window: Vec::with_capacity(capacity),
			free: Vec::new(),
			heads: [NIL; 2],
			tails: [NIL; 2],
			lens: [0; 2],
			capacity,
			sketch: CountMinSketch::new(width),
			window_cap,
			step: (capacity / 100).max(1),
			max_window,
			direction: 1,
			period_hits: 0,
			period_ops: 0,
			period_len: (capacity as u64).saturating_mul(10),
			prev_hits: 0,
			prev_ops: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width` bytes (u8 counters).
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = CountMinSketch::mix(key);
		self.sketch.increment(h);
		self.period_ops += 1;
		let hit = self.map.get(key).copied();
		if hit.is_some() {
			self.period_hits += 1;
		}
		if self.period_ops >= self.period_len {
			self.adapt();
		}
		let idx = hit?;
		let list = if self.in_window[idx] { WIN } else { MAIN };
		self.unlink(idx);
		self.push_back(list, idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = CountMinSketch::mix(&key);
		self.sketch.increment(h);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			let list = if self.in_window[idx] { WIN } else { MAIN };
			self.unlink(idx);
			self.push_back(list, idx);
			return;
		}

		let slot = if let Some(s) = self.free.pop() {
			let s = s as usize;
			self.nodes[s].key = key.clone();
			self.nodes[s].value = value;
			s
		} else if self.nodes.len() < self.capacity {
			let s = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.in_window.push(false);
			s
		} else {
			// Full: free one slot, then reuse it for the incoming key.
			let s = if self.lens[WIN] >= self.window_cap && self.lens[WIN] > 0 {
				self.compete_window_head()
			} else if self.heads[MAIN] != NIL {
				let v = self.heads[MAIN] as usize;
				self.evict(v);
				v
			} else {
				let v = self.heads[WIN] as usize;
				self.evict(v);
				v
			};
			// `evict` leaves key/value stale in the slot; overwrite.
			self.free.retain(|&f| f as usize != s);
			self.nodes[s].key = key.clone();
			self.nodes[s].value = value;
			s
		};
		self.map.insert(key, slot);
		self.push_back(WIN, slot);
		// Window overflow while filling: promote head into main (room exists, no eviction).
		if self.lens[WIN] > self.window_cap && self.heads[WIN] != NIL {
			let c = self.heads[WIN] as usize;
			if self.nodes.len() < self.capacity || !self.free.is_empty() {
				self.unlink(c);
				self.push_back(MAIN, c);
			}
		}
	}

	/// Remove node `idx` from cache entirely; slot becomes free (caller may reclaim it).
	fn evict(&mut self, idx: usize) {
		let k = self.nodes[idx].key.clone();
		self.map.remove(&k);
		self.unlink(idx);
		self.free.push(idx as u32);
	}

	/// Window head competes with main head by the strict TinyLFU test.
	/// Exactly one slot ends up free; its index is returned (and is in `free`).
	fn compete_window_head(&mut self) -> usize {
		let cand = self.heads[WIN] as usize;
		let victim = self.heads[MAIN];
		if victim == NIL {
			self.evict(cand);
			return cand;
		}
		let victim = victim as usize;
		let cand_f = self.sketch.estimate(CountMinSketch::mix(&self.nodes[cand].key));
		let vict_f = self.sketch.estimate(CountMinSketch::mix(&self.nodes[victim].key));
		if cand_f > vict_f {
			self.evict(victim);
			self.unlink(cand);
			self.push_back(MAIN, cand);
			victim
		} else {
			self.evict(cand);
			cand
		}
	}

	fn adapt(&mut self) {
		let (hits, ops) = (self.period_hits, self.period_ops);
		self.period_hits = 0;
		self.period_ops = 0;
		if self.prev_ops > 0 {
			// worsened iff hits/ops < prev_hits/prev_ops
			if (hits as u128) * (self.prev_ops as u128) < (self.prev_hits as u128) * (ops as u128) {
				self.direction = -self.direction;
			}
		}
		self.prev_hits = hits;
		self.prev_ops = ops;
		let new_cap = if self.direction > 0 {
			(self.window_cap + self.step).min(self.max_window)
		} else {
			self.window_cap.saturating_sub(self.step).max(1)
		};
		self.window_cap = new_cap;
		// Shrink: demote window LRU entries through the strict admission test.
		while self.lens[WIN] > self.window_cap {
			let cand = self.heads[WIN] as usize;
			if self.heads[MAIN] == NIL {
				self.unlink(cand);
				self.push_back(MAIN, cand);
			} else {
				self.compete_window_head();
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let list = if self.in_window[idx] { WIN } else { MAIN };
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.heads[list] = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tails[list] = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
		self.lens[list] -= 1;
	}

	fn push_back(&mut self, list: usize, idx: usize) {
		self.in_window[idx] = list == WIN;
		let tail = self.tails[list];
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx as u32;
		} else {
			self.heads[list] = idx as u32;
		}
		self.tails[list] = idx as u32;
		self.lens[list] += 1;
	}
}

/// Cheap multiplicative-fold hasher (replaces SipHash); output is then finalized by splitmix64.
struct FoldHasher(u64);

impl Hasher for FoldHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517C_C1B7_2722_0A95);
		}
	}

	#[inline]
	fn write_u64(&mut self, v: u64) {
		self.0 = (self.0.rotate_left(5) ^ v).wrapping_mul(0x517C_C1B7_2722_0A95);
	}

	#[inline]
	fn write_i64(&mut self, v: i64) {
		self.write_u64(v as u64);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	bits: u32,
	counts: Vec<u8>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		Self {
			width,
			depth,
			bits: width.trailing_zeros(),
			counts: vec![0; depth * width],
		}
	}

	/// Heap bytes backing the counter table: `depth * width` (u8 counters).
	fn heap_bytes(&self) -> usize {
		self.depth * self.width
	}

	/// One hash + one splitmix64 finalizer per operation.
	#[inline]
	fn mix<K: Hash>(key: &K) -> u64 {
		let mut hasher = FoldHasher(0x9E37_79B9_7F4A_7C15);
		key.hash(&mut hasher);
		let mut z = hasher.finish();
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}

	/// Row index from a disjoint bit-slice of the mixed hash (power-of-two mask).
	#[inline]
	fn index(&self, h: u64, row: usize) -> usize {
		let slice = h.rotate_right((row as u32 * self.bits) & 63) as usize;
		row * self.width + (slice & (self.width - 1))
	}

	fn increment(&mut self, h: u64) {
		let mut idx = [0usize; SKETCH_DEPTH];
		let mut min = MAX_COUNT;
		for row in 0..SKETCH_DEPTH {
			idx[row] = self.index(h, row);
			min = min.min(self.counts[idx[row]]);
		}
		if min == MAX_COUNT {
			return;
		}
		for &i in idx.iter() {
			if self.counts[i] == min {
				self.counts[i] += 1;
			}
		}
	}

	fn estimate(&self, h: u64) -> u32 {
		(0..SKETCH_DEPTH)
			.map(|row| self.counts[self.index(h, row)])
			.min()
			.unwrap_or(0) as u32
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
