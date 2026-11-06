use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
/// Sample period multiplier: the sketch ages (halves) after
/// `capacity * SAMPLE_FACTOR` effective additions.
const SAMPLE_FACTOR: usize = 10;
/// 4-bit counters per 64-byte block: 8 words x 16 nibbles.
const COUNTERS_PER_BLOCK: usize = 8 * 16;
/// Clears the top bit of every nibble after a one-bit right shift.
const RESET_MASK: u64 = 0x7777_7777_7777_7777;

/// FxHash-style multiply-rotate hasher with a splitmix64 avalanche finish.
/// Cheap compared to SipHash; the finish step gives good bit dispersion for
/// both the hashbrown control bytes and the sketch row-index slices.
#[derive(Default, Clone, Copy)]
pub struct FastHasher {
	state: u64,
}

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

impl FastHasher {
	#[inline(always)]
	fn add(&mut self, word: u64) {
		self.state = (self.state.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

#[inline(always)]
fn splitmix64(mut z: u64) -> u64 {
	z = z.wrapping_add(0x9e37_79b9_7f4a_7c15);
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

impl Hasher for FastHasher {
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
			self.add(u64::from_le_bytes(buf) ^ ((rem.len() as u64) << 56));
		}
	}

	#[inline(always)]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u16(&mut self, i: u16) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}

	#[inline(always)]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn write_i64(&mut self, i: i64) {
		self.add(i as u64);
	}

	#[inline(always)]
	fn finish(&self) -> u64 {
		splitmix64(self.state)
	}
}

/// Hash a key once with the fast hasher; the result feeds all sketch rows
/// and (via `HashedKey`) the map probe.
#[inline(always)]
fn key_hash<K: Hash>(key: &K) -> u64 {
	let mut h = FastHasher::default();
	key.hash(&mut h);
	h.finish()
}

/// Pass-through hasher: the map key already carries its precomputed
/// `key_hash`, so the map reuses it instead of hashing the key again.
#[derive(Default, Clone, Copy)]
pub struct PrehashedHasher {
	state: u64,
}

impl Hasher for PrehashedHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		// Fallback only; `HashedKey` always uses `write_u64`.
		for &b in bytes {
			self.state = (self.state.rotate_left(5) ^ b as u64).wrapping_mul(FX_SEED);
		}
	}

	#[inline(always)]
	fn write_u64(&mut self, i: u64) {
		self.state = i;
	}

	#[inline(always)]
	fn finish(&self) -> u64 {
		self.state
	}
}

type PrehashedBuildHasher = BuildHasherDefault<PrehashedHasher>;

/// Map key carrying its precomputed fast hash. Equality compares the key.
#[derive(Debug, Clone)]
struct HashedKey<K> {
	h: u64,
	key: K,
}

impl<K> Hash for HashedKey<K> {
	#[inline(always)]
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_u64(self.h);
	}
}

impl<K: Eq> PartialEq for HashedKey<K> {
	#[inline(always)]
	fn eq(&self, other: &Self) -> bool {
		self.h == other.h && self.key == other.key
	}
}

impl<K: Eq> Eq for HashedKey<K> {}

// ---------------------------------------------------------------------------
// Region tagging: the 2-bit region lives in the top two bits of `Node::prev`.
// ---------------------------------------------------------------------------

const REGION_SHIFT: u32 = usize::BITS - 2;
const REGION_MASK: usize = 0b11 << REGION_SHIFT;
/// "No link" sentinel for raw (untagged) indices. Capacities are far below
/// 2^62, so this never collides with a real arena index.
const NIL: usize = !REGION_MASK;

const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Previous index in the node's region list (`NIL` if head), with the
	/// region tag packed into the top two bits. Always read via `& !REGION_MASK`.
	prev: usize,
	next: Option<usize>,
}

// Packing the region tag into `prev` means the region costs no extra bytes:
// Node<u64,u64> must not grow past the 48 bytes of the baseline/v003 layout.
const _: () = assert!(std::mem::size_of::<Node<u64, u64>>() <= 48);

impl<K, V> Node<K, V> {
	#[inline(always)]
	fn prev_idx(&self) -> usize {
		self.prev & !REGION_MASK
	}

	#[inline(always)]
	fn set_prev_idx(&mut self, p: usize) {
		self.prev = (self.prev & REGION_MASK) | (p & !REGION_MASK);
	}

	#[inline(always)]
	fn region(&self) -> usize {
		(self.prev & REGION_MASK) >> REGION_SHIFT
	}

	#[inline(always)]
	fn set_region(&mut self, r: usize) {
		self.prev = (self.prev & !REGION_MASK) | (r << REGION_SHIFT);
	}

	#[inline(always)]
	fn next_idx(&self) -> usize {
		match self.next {
			Some(n) => n,
			None => NIL,
		}
	}
}

#[derive(Debug, Clone, Copy)]
struct RegionList {
	head: usize,
	tail: usize,
	len: usize,
}

impl RegionList {
	const fn empty() -> Self {
		Self {
			head: NIL,
			tail: NIL,
			len: 0,
		}
	}
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<HashedKey<K>, usize, PrehashedBuildHasher>,
	nodes: Vec<Node<K, V>>,
	lists: [RegionList; 3],
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
		let window_cap = (capacity / 100).max(1);
		let main_cap = capacity - window_cap;
		let protected_cap = (main_cap as f64 * 0.8) as usize;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={window_cap} protected={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, PrehashedBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			lists: [RegionList::empty(); 3],
			capacity,
			window_cap,
			protected_cap,
			sketch: CountMinSketch::new(width, capacity),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `blocks * 64`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = key_hash(key);
		self.sketch.increment_hashed(h);
		let probe = HashedKey { h, key: key.clone() };
		let &idx = self.map.get(&probe)?;
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let key_h = key_hash(&key);
		self.sketch.increment_hashed(key_h);
		let hk = HashedKey { h: key_h, key };
		if let Some(&idx) = self.map.get(&hk) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: hk.key.clone(),
				value,
				prev: NIL,
				next: None,
			});
			self.push_back(idx, WINDOW);
			self.map.insert(hk, idx);
			// Spare room in main: window overflow just migrates to probation.
			if self.lists[WINDOW].len > self.window_cap {
				let cand = self.lists[WINDOW].head;
				self.unlink(cand);
				self.push_back(cand, PROBATION);
			}
			return;
		}

		// Full: pick a slot to free via the window-candidate vs main-victim contest.
		let (slot, old_h) = self.evict_one();
		let old_key = std::mem::replace(&mut self.nodes[slot].key, hk.key.clone());
		self.map.remove(&HashedKey { h: old_h, key: old_key });
		self.nodes[slot].value = value;
		self.push_back(slot, WINDOW);
		self.map.insert(hk, slot);
	}

	/// Runs the admission contest and unlinks the loser; returns its slot and
	/// its hash (so the map removal does not rehash it).
	fn evict_one(&mut self) -> (usize, u64) {
		let cand = self.lists[WINDOW].head;
		let victim = if self.lists[PROBATION].head != NIL {
			self.lists[PROBATION].head
		} else {
			self.lists[PROTECTED].head
		};

		if cand == NIL {
			// Window empty (not reachable in steady state); evict main victim.
			let victim_h = key_hash(&self.nodes[victim].key);
			self.unlink(victim);
			return (victim, victim_h);
		}
		let cand_h = key_hash(&self.nodes[cand].key);
		if victim == NIL {
			// Main region empty (e.g. capacity 1): the candidate leaves.
			self.unlink(cand);
			return (cand, cand_h);
		}
		let victim_h = key_hash(&self.nodes[victim].key);
		let cand_freq = self.sketch.estimate_hashed(cand_h);
		let victim_freq = self.sketch.estimate_hashed(victim_h);
		if cand_freq > victim_freq {
			self.unlink(victim);
			self.unlink(cand);
			self.push_back(cand, PROBATION);
			(victim, victim_h)
		} else {
			self.unlink(cand);
			(cand, cand_h)
		}
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region() {
			PROBATION => {
				self.unlink(idx);
				self.push_back(idx, PROTECTED);
				if self.lists[PROTECTED].len > self.protected_cap {
					let d = self.lists[PROTECTED].head;
					self.unlink(d);
					self.push_back(d, PROBATION);
				}
			}
			r => {
				if self.lists[r].tail != idx {
					self.unlink(idx);
					self.push_back(idx, r);
				}
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let r = self.nodes[idx].region();
		let prev = self.nodes[idx].prev_idx();
		let next = self.nodes[idx].next_idx();
		if prev == NIL {
			self.lists[r].head = next;
		} else {
			self.nodes[prev].next = if next == NIL { None } else { Some(next) };
		}
		if next == NIL {
			self.lists[r].tail = prev;
		} else {
			self.nodes[next].set_prev_idx(prev);
		}
		self.lists[r].len -= 1;
		self.nodes[idx].set_prev_idx(NIL);
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize, r: usize) {
		let tail = self.lists[r].tail;
		{
			let n = &mut self.nodes[idx];
			n.set_region(r);
			n.set_prev_idx(tail);
			n.next = None;
		}
		if tail == NIL {
			self.lists[r].head = idx;
		} else {
			self.nodes[tail].next = Some(idx);
		}
		self.lists[r].tail = idx;
		self.lists[r].len += 1;
	}
}

/// One cache line of 4-bit counters: 8 words x 16 nibbles = 128 counters.
#[repr(align(64))]
#[derive(Debug, Clone, Copy, Default)]
struct Block([u64; 8]);

const _: () = assert!(std::mem::size_of::<Block>() == 64);

/// Cache-line-blocked count-min sketch with 4-bit saturating counters and
/// periodic aging (halve everything every `sample_size` additions). All
/// `SKETCH_DEPTH` counters for a key live in one 64-byte block.
#[derive(Debug)]
struct CountMinSketch {
	table: Vec<Block>,
	block_mask: usize,
	additions: u32,
	sample_size: u32,
}

impl CountMinSketch {
	fn new(width: usize, capacity: usize) -> Self {
		let counters = SKETCH_DEPTH * width.max(1);
		let blocks = counters.div_ceil(COUNTERS_PER_BLOCK).max(1).next_power_of_two();
		let sample_size = capacity
			.saturating_mul(SAMPLE_FACTOR)
			.clamp(1, u32::MAX as usize) as u32;
		Self {
			table: vec![Block::default(); blocks],
			block_mask: blocks - 1,
			additions: 0,
			sample_size,
		}
	}

	/// Heap bytes backing the counter table: one 64-byte line per block.
	fn heap_bytes(&self) -> usize {
		self.table.len() * 64
	}

	/// Block selection from the high 32 hash bits (disjoint from `position`).
	#[inline(always)]
	fn block_index(&self, h: u64) -> usize {
		((h >> 32) as usize) & self.block_mask
	}

	/// In-block (word, nibble shift) for row `i`, from the low 32 hash bits.
	/// Row `i` owns words `2i` and `2i+1`; one byte of the hash picks the word
	/// (bit 4) and nibble (bits 0..4) within that pair.
	#[inline(always)]
	fn position(h: u64, i: usize) -> (usize, u32) {
		let lo = h as u32;
		let b = (lo >> (8 * i as u32)) & 0xFF;
		let word = (i << 1) | ((b >> 4) & 1) as usize;
		let shift = (b & 0xF) << 2;
		(word, shift)
	}

	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		let bi = self.block_index(h);
		let block = &mut self.table[bi].0;
		let mut moved = false;
		for i in 0..SKETCH_DEPTH {
			let (w, s) = Self::position(h, i);
			let c = (block[w] >> s) & 0xF;
			if c < 15 {
				block[w] += 1u64 << s;
				moved = true;
			}
		}
		if moved {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.reset();
			}
		}
	}

	#[cold]
	fn reset(&mut self) {
		for block in self.table.iter_mut() {
			for w in block.0.iter_mut() {
				*w = (*w >> 1) & RESET_MASK;
			}
		}
		self.additions >>= 1;
	}

	#[inline]
	fn estimate_hashed(&self, h: u64) -> u32 {
		let block = &self.table[self.block_index(h)].0;
		let mut min = 15u32;
		for i in 0..SKETCH_DEPTH {
			let (w, s) = Self::position(h, i);
			let c = ((block[w] >> s) & 0xF) as u32;
			if c < min {
				min = c;
			}
		}
		min
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
	fn sketch_counts_increment() {
		let mut s = CountMinSketch::new(16, 16);
		let h = key_hash(&7i64);
		s.increment_hashed(h);
		s.increment_hashed(h);
		assert!(s.estimate_hashed(h) >= 2);
	}

	#[test]
	fn sketch_saturates_at_15() {
		// Large sample so aging never triggers during the test.
		let mut s = CountMinSketch::new(1024, 1 << 20);
		let h = key_hash(&99i64);
		for _ in 0..100 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 15);
	}

	#[test]
	fn sketch_ages_by_halving() {
		let mut s = CountMinSketch::new(1024, 1 << 20);
		let h = key_hash(&5i64);
		for _ in 0..8 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 8);
		let before = s.additions;
		s.reset();
		assert_eq!(s.estimate_hashed(h), 4);
		assert_eq!(s.additions, before >> 1);
	}

	#[test]
	fn sketch_heap_within_budget() {
		let cache = TinyLfuCache::<i64, i64>::new(1024);
		assert!(cache.sketch_heap_bytes() <= 64 * 1024);
		assert_eq!(cache.sketch_heap_bytes(), 8 * 1024);
	}

	#[test]
	fn region_tag_roundtrips_without_corrupting_links() {
		let mut n = Node { key: 1u64, value: 2u64, prev: NIL, next: None };
		n.set_prev_idx(5);
		n.set_region(PROTECTED);
		assert_eq!(n.prev_idx(), 5);
		assert_eq!(n.region(), PROTECTED);
		n.set_prev_idx(NIL);
		assert_eq!(n.prev_idx(), NIL);
		assert_eq!(n.region(), PROTECTED);
	}

	#[test]
	fn eviction_prefers_hot_items() {
		// cap 2: window 1, probation/protected share 1.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // 1 -> probation, 2 in window
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30); // candidate 2 loses to victim 1

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn cold_window_candidate_is_evicted() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30); // newcomer always enters window; candidate 2 evicted
		cache.put(4, 40); // candidate 3 (freq 1) loses to victim 1

		assert!(cache.get(&1).is_some());
		assert!(cache.get(&4).is_some());
		assert!(cache.get(&3).is_none());
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
	fn capacity_one_works() {
		let mut cache = TinyLfuCache::new(1);
		for k in 0..10i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..10i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 1);
	}
}
