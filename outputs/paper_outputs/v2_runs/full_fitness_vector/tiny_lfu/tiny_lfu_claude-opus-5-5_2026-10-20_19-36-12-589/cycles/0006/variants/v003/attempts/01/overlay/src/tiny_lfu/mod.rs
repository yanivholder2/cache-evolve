use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
/// Saturation point of the `u16` counters, expressed in the `u32` comparison domain.
const MAX_COUNT: u32 = u16::MAX as u32;

/// Top bit of a node's `next` link records its region: set = main, clear = window.
const REGION_BIT: u32 = 1 << 31;
/// Mask extracting the link value from `next`.
const LINK_MASK: u32 = !REGION_BIT;
/// Sentinel for "no link" in the 31-bit-indexed intrusive lists.
const NIL: u32 = LINK_MASK;

/// Fraction of capacity given to the LRU admission window (`capacity / WINDOW_DIVISOR`, min 1).
const WINDOW_DIVISOR: usize = 10;

const FX_SEED: u64 = 0x517c_c1b7_2722_0a95;

/// Minimal FxHash-style hasher (local; no external crates).
#[derive(Default, Clone, Copy)]
pub struct FxLikeHasher {
	state: u64,
}

impl FxLikeHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxLikeHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut buf = [0u8; 8];
			buf.copy_from_slice(c);
			self.add(u64::from_le_bytes(buf));
		}
		let rem = chunks.remainder();
		if !rem.is_empty() {
			let mut buf = [0u8; 8];
			buf[..rem.len()].copy_from_slice(rem);
			self.add(u64::from_le_bytes(buf));
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u16(&mut self, i: u16) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}

	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline]
	fn finish(&self) -> u64 {
		self.state
	}
}

type FxBuild = BuildHasherDefault<FxLikeHasher>;

/// murmur3 fmix64 finaliser for avalanche.
#[inline(always)]
fn fmix64(mut h: u64) -> u64 {
	h ^= h >> 33;
	h = h.wrapping_mul(0xff51_afd7_ed55_8ccd);
	h ^= h >> 33;
	h = h.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	h ^= h >> 33;
	h
}

/// Hash a key once with the fast hasher plus a finaliser mix.
#[inline]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut hasher = FxLikeHasher::default();
	key.hash(&mut hasher);
	fmix64(hasher.finish())
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	/// Low 31 bits: next link (`NIL` = none). Top bit: region (`REGION_BIT` = main).
	next: u32,
}

/// TinyLFU with an LRU admission window: new keys always enter the window; the window's
/// LRU overflow duels the main list's LRU head via the sketch.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, u32, FxBuild>,
	nodes: Vec<Node<K, V>>,
	w_head: u32,
	w_tail: u32,
	w_len: usize,
	m_head: u32,
	m_tail: u32,
	m_len: usize,
	window_cap: usize,
	main_cap: usize,
	capacity: usize,
	sketch: CountMinSketch,
	/// Hash of the key from the most recent `get` miss; when the very next `put` is for
	/// the same key, that access was already counted and must not be counted twice.
	last_miss_hash: u64,
	last_miss_valid: bool,
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
		if capacity >= REGION_BIT as usize {
			log::error!(
				"TinyLfuCache.try_new: capacity={capacity} must be < 2^31 for 31-bit node indices"
			);
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		let window_cap = (capacity / WINDOW_DIVISOR).max(1);
		let main_cap = capacity - window_cap;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FxBuild::default()),
			nodes: Vec::with_capacity(capacity),
			w_head: NIL,
			w_tail: NIL,
			w_len: 0,
			m_head: NIL,
			m_tail: NIL,
			m_len: 0,
			window_cap,
			main_cap,
			capacity,
			sketch: CountMinSketch::new(width),
			last_miss_hash: 0,
			last_miss_valid: false,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u16>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = key_hash(key);
		self.sketch.increment_hash(h);
		match self.map.get(key) {
			Some(&idx) => {
				self.last_miss_valid = false;
				self.relink(idx);
				Some(&self.nodes[idx as usize].value)
			}
			None => {
				self.last_miss_hash = h;
				self.last_miss_valid = true;
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let h = key_hash(&key);
		// If the preceding `get` missed on this same key, its access is already in the
		// sketch; read the estimate instead of counting it a second time.
		let _incoming_freq = if self.last_miss_valid && h == self.last_miss_hash {
			self.sketch.estimate_hash(h)
		} else {
			self.sketch.increment_hash(h)
		};
		self.last_miss_valid = false;
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.relink(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			// Still filling: insert at window MRU; overflow into main without a duel.
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx, false);
			self.map.insert(key, idx);
			if self.w_len > self.window_cap {
				debug_assert!(self.m_len < self.main_cap);
				let c = self.w_head;
				self.unlink(c);
				self.push_back(c, true);
			}
			return;
		}

		// Full: the window LRU `c` duels the main LRU `v`; the loser's slot hosts the arrival.
		let c = self.w_head;
		debug_assert!(c != NIL, "TinyLfuCache: full cache has empty window");
		let v = self.m_head;
		let admit_c = v != NIL && {
			let fc = self
				.sketch
				.estimate_hash(key_hash(&self.nodes[c as usize].key));
			let fv = self
				.sketch
				.estimate_hash(key_hash(&self.nodes[v as usize].key));
			fc > fv
		};
		let free = if admit_c {
			self.map.remove(&self.nodes[v as usize].key);
			self.unlink(v);
			self.unlink(c);
			self.push_back(c, true);
			v
		} else {
			self.map.remove(&self.nodes[c as usize].key);
			self.unlink(c);
			c
		};
		let n = &mut self.nodes[free as usize];
		n.key = key.clone();
		n.value = value;
		self.push_back(free, false);
		self.map.insert(key, free);
	}

	#[inline(always)]
	fn is_main(&self, idx: u32) -> bool {
		self.nodes[idx as usize].next & REGION_BIT != 0
	}

	/// Move `idx` to the MRU end of its own list (no region change).
	#[inline]
	fn relink(&mut self, idx: u32) {
		let main = self.is_main(idx);
		let tail = if main { self.m_tail } else { self.w_tail };
		if tail == idx {
			return;
		}
		self.unlink(idx);
		self.push_back(idx, main);
	}

	#[inline(always)]
	fn set_next_link(&mut self, idx: u32, link: u32) {
		let n = &mut self.nodes[idx as usize].next;
		*n = (*n & REGION_BIT) | link;
	}

	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let raw_next = self.nodes[i].next;
		let main = raw_next & REGION_BIT != 0;
		let prev = self.nodes[i].prev;
		let next = raw_next & LINK_MASK;
		if prev != NIL {
			self.set_next_link(prev, next);
		} else if main {
			self.m_head = next;
		} else {
			self.w_head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else if main {
			self.m_tail = prev;
		} else {
			self.w_tail = prev;
		}
		if main {
			self.m_len -= 1;
		} else {
			self.w_len -= 1;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = (raw_next & REGION_BIT) | NIL;
	}

	fn push_back(&mut self, idx: u32, main: bool) {
		let i = idx as usize;
		let tail = if main { self.m_tail } else { self.w_tail };
		self.nodes[i].prev = tail;
		self.nodes[i].next = if main { REGION_BIT | NIL } else { NIL };
		if tail != NIL {
			self.set_next_link(tail, idx);
		} else if main {
			self.m_head = idx;
		} else {
			self.w_head = idx;
		}
		if main {
			self.m_tail = idx;
			self.m_len += 1;
		} else {
			self.w_tail = idx;
			self.w_len += 1;
		}
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	/// Flattened `depth * width` saturating `u16` counters; row `r` occupies
	/// `[r * width, (r + 1) * width)`.
	counts: Vec<u16>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		debug_assert!(width.is_power_of_two());
		let depth = SKETCH_DEPTH;
		let counts = vec![0u16; depth * width];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u16>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u16>()
	}

	/// Flat index for `row`, derived by double hashing from one 64-bit hash.
	#[inline(always)]
	fn index(&self, h1: u32, h2: u32, row: usize) -> usize {
		let col = (h1.wrapping_add((row as u32).wrapping_mul(h2)) as usize) & (self.width - 1);
		row * self.width + col
	}

	/// The four flat indices for a key hash.
	#[inline(always)]
	fn indices(&self, h: u64) -> [usize; SKETCH_DEPTH] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		[
			self.index(h1, h2, 0),
			self.index(h1, h2, 1),
			self.index(h1, h2, 2),
			self.index(h1, h2, 3),
		]
	}

	/// Conservative update: read the key's four counters, find their minimum, and bump
	/// only the counters equal to that minimum (branch-free). Saturates at `MAX_COUNT`.
	/// Returns the key's new estimate (`min + 1`, saturating).
	#[inline]
	fn increment_hash(&mut self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		debug_assert!(i0.max(i1).max(i2).max(i3) < self.counts.len());
		// SAFETY: each index is `row * width + (x & (width - 1))` with `row < SKETCH_DEPTH`,
		// so it is `< SKETCH_DEPTH * width == counts.len()`.
		unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			let min = c0.min(c1).min(c2.min(c3));
			let inc = (min < MAX_COUNT) as u32;
			*self.counts.get_unchecked_mut(i0) = (c0 + (((c0 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i1) = (c1 + (((c1 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i2) = (c2 + (((c2 == min) as u32) & inc)) as u16;
			*self.counts.get_unchecked_mut(i3) = (c3 + (((c3 == min) as u32) & inc)) as u16;
			min + inc
		}
	}

	#[inline]
	fn estimate_hash(&self, h: u64) -> u32 {
		let [i0, i1, i2, i3] = self.indices(h);
		debug_assert!(i0.max(i1).max(i2).max(i3) < self.counts.len());
		// SAFETY: see `increment_hash`.
		unsafe {
			let c0 = *self.counts.get_unchecked(i0) as u32;
			let c1 = *self.counts.get_unchecked(i1) as u32;
			let c2 = *self.counts.get_unchecked(i2) as u32;
			let c3 = *self.counts.get_unchecked(i3) as u32;
			c0.min(c1).min(c2.min(c3))
		}
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
	fn node_is_compact() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}

	#[test]
	fn rejects_capacity_beyond_31_bits() {
		assert!(TinyLfuCache::<u64, u64>::try_new(1usize << 31).is_none());
	}

	#[test]
	fn miss_then_put_counts_once() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		let k = 99i64;
		assert!(cache.get(&k).is_none());
		cache.put(k, 1);
		assert_eq!(cache.sketch.estimate_hash(key_hash(&k)), 1);
	}

	#[test]
	fn conservative_update_bumps_only_minimum_counters() {
		let mut s = CountMinSketch::new(16);
		let h = key_hash(&7u64);
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		// Inflate one row's counter so it sits above the others.
		let i0 = s.index(h1, h2, 0);
		s.counts[i0] = 5;
		assert_eq!(s.increment_hash(h), 1);
		assert_eq!(s.counts[i0], 5);
		assert_eq!(s.estimate_hash(h), 1);
		assert_eq!(s.increment_hash(h), 2);
		assert_eq!(s.counts[i0], 5);
	}

	#[test]
	fn conservative_update_saturates() {
		let mut s = CountMinSketch::new(16);
		for c in s.counts.iter_mut() {
			*c = u16::MAX;
		}
		let h = key_hash(&3u64);
		assert_eq!(s.increment_hash(h), u16::MAX as u32);
		assert!(s.counts.iter().all(|&c| c == u16::MAX));
	}

	#[test]
	fn capacity_one_keeps_newest() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
	fn low_frequency_window_overflow_is_rejected() {
		// capacity 2 → window_cap 1, main_cap 1. Key 1 overflows into main; key 2 sits in
		// the window. Key 3 always enters the window (it holds the newest key); the
		// displaced window candidate 2 loses its duel against the hotter main key 1.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&1).is_some());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_some());
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
	fn window_burst_key_survives_one_hit_scan() {
		// capacity 20 → window_cap 2, main_cap 18.
		let mut cache = TinyLfuCache::<i64, i64>::new(20);
		let access = |cache: &mut TinyLfuCache<i64, i64>, k: i64| {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		};
		for k in 100..120 {
			access(&mut cache, k);
		}
		// Burst: key 7 arrives and is re-referenced while it sits in the window.
		access(&mut cache, 7);
		access(&mut cache, 7);
		access(&mut cache, 7);
		// Scan of one-hit keys pushes 7 out of the window; it should win its duel.
		for k in 200..212 {
			access(&mut cache, k);
		}
		assert_eq!(cache.get(&7), Some(&7));
		assert!(cache.nodes.len() <= 20);
		assert_eq!(cache.w_len + cache.m_len, cache.map.len());
	}
}
