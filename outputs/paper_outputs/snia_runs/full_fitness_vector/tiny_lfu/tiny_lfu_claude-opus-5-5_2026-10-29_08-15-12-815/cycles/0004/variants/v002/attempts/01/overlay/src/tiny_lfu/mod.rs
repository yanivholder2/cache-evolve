use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;

/// Aging period: the sketch halves after `SAMPLE_FACTOR * capacity` additions.
const SAMPLE_FACTOR: usize = 10;
/// One 64-byte block holds 8 words x 16 four-bit counters.
const COUNTERS_PER_BLOCK: usize = 128;
/// After `>> 1`, clears the bit shifted in from the neighbouring nibble.
const RESET_MASK: u64 = 0x7777_7777_7777_7777;

/// FxHash-style multiply-rotate hasher with a splitmix64 avalanche finish.
/// Cheap compared to SipHash; the finish step gives good bit dispersion for
/// both the hashbrown control bytes and the sketch index bits.
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

/// Hash a key once with the fast hasher; the result feeds the sketch
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
// 32-bit links. `next` is a raw index (`NIL` = none). `prev` carries the
// 2-bit region tag in bits 30–31 and the index in bits 0–29.
// ---------------------------------------------------------------------------

const REGION_SHIFT: u32 = 30;
const REGION_MASK: u32 = 0b11 << REGION_SHIFT;
/// "No link" sentinel for raw (untagged) 30-bit indices (= 0x3FFF_FFFF).
/// `try_new` rejects capacities that could collide with it.
const NIL: u32 = !REGION_MASK;
/// `NIL` widened for `usize`-typed call sites and `RegionList`.
const NIL_USIZE: usize = NIL as usize;

const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Previous index in the node's region list (`NIL` if head), with the
	/// region tag packed into the top two bits. Always read via `& !REGION_MASK`.
	prev: u32,
	/// Next index in the node's region list (`NIL` if tail).
	next: u32,
}

// u64 key + u64 value + two u32 links = 24 bytes, no padding.
const _: () = assert!(std::mem::size_of::<Node<u64, u64>>() <= 24);

impl<K, V> Node<K, V> {
	#[inline(always)]
	fn prev_idx(&self) -> usize {
		(self.prev & !REGION_MASK) as usize
	}

	#[inline(always)]
	fn set_prev_idx(&mut self, p: usize) {
		self.prev = (self.prev & REGION_MASK) | ((p as u32) & !REGION_MASK);
	}

	#[inline(always)]
	fn region(&self) -> usize {
		((self.prev & REGION_MASK) >> REGION_SHIFT) as usize
	}

	#[inline(always)]
	fn set_region(&mut self, r: usize) {
		self.prev = (self.prev & !REGION_MASK) | ((r as u32) << REGION_SHIFT);
	}

	#[inline(always)]
	fn next_idx(&self) -> usize {
		self.next as usize
	}

	#[inline(always)]
	fn set_next_idx(&mut self, n: usize) {
		self.next = n as u32;
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
			head: NIL_USIZE,
			tail: NIL_USIZE,
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
		if capacity >= NIL_USIZE {
			log::error!(
				"TinyLfuCache.try_new: capacity={capacity} must be < {NIL_USIZE} (32-bit node links)"
			);
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

	/// Heap bytes held by the frequency sketch: `blocks * 64` (4-bit counters,
	/// `depth * width` of them packed into 64-byte cache-line blocks).
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
				next: NIL,
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
		let victim = if self.lists[PROBATION].head != NIL_USIZE {
			self.lists[PROBATION].head
		} else {
			self.lists[PROTECTED].head
		};

		if cand == NIL_USIZE {
			// Window empty (not reachable in steady state); evict main victim.
			let victim_h = key_hash(&self.nodes[victim].key);
			self.unlink(victim);
			return (victim, victim_h);
		}
		let cand_h = key_hash(&self.nodes[cand].key);
		if victim == NIL_USIZE {
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
		if prev == NIL_USIZE {
			self.lists[r].head = next;
		} else {
			self.nodes[prev].set_next_idx(next);
		}
		if next == NIL_USIZE {
			self.lists[r].tail = prev;
		} else {
			self.nodes[next].set_prev_idx(prev);
		}
		self.lists[r].len -= 1;
		self.nodes[idx].set_prev_idx(NIL_USIZE);
		self.nodes[idx].next = NIL;
	}

	fn push_back(&mut self, idx: usize, r: usize) {
		let tail = self.lists[r].tail;
		{
			let n = &mut self.nodes[idx];
			n.set_region(r);
			n.set_prev_idx(tail);
			n.next = NIL;
		}
		if tail == NIL_USIZE {
			self.lists[r].head = idx;
		} else {
			self.nodes[tail].set_next_idx(idx);
		}
		self.lists[r].tail = idx;
		self.lists[r].len += 1;
	}
}

/// One cache line of counters: 8 words x 16 four-bit counters.
#[repr(align(64))]
#[derive(Debug, Clone, Copy, Default)]
struct Block([u64; 8]);

/// Cache-line-blocked count-min sketch with 4-bit saturating counters and
/// periodic halving (TinyLFU aging). All `SKETCH_DEPTH` counters of a key
/// live in one 64-byte block; row `i` uses word pair `2i, 2i+1`.
#[derive(Debug)]
struct CountMinSketch {
	table: Vec<Block>,
	block_mask: usize,
	/// Increments since the last reset.
	additions: u32,
	/// Reset threshold (`SAMPLE_FACTOR * capacity`).
	sample_size: u32,
}

impl CountMinSketch {
	/// `width` is counters per row (power of two); total counters
	/// `SKETCH_DEPTH * width` are packed into 128-counter blocks.
	fn new(width: usize, capacity: usize) -> Self {
		let blocks = (width * SKETCH_DEPTH / COUNTERS_PER_BLOCK)
			.max(1)
			.next_power_of_two();
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

	/// Block selected by the high 32 bits of the hash.
	#[inline(always)]
	fn block_index(&self, h: u64) -> usize {
		((h >> 32) as usize) & self.block_mask
	}

	/// For row `i`, byte `i` of the low hash half picks the word (bit 4,
	/// within the row's pair) and the nibble (bits 0..4).
	#[inline(always)]
	fn position(h: u64, i: usize) -> (usize, u32) {
		let b = (h >> (i * 8)) as u32 & 0xFF;
		let word = 2 * i + ((b >> 4) & 1) as usize;
		let shift = (b & 0xF) << 2;
		(word, shift)
	}

	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		let bi = self.block_index(h);
		let block = &mut self.table[bi].0;
		let mut added = false;
		for i in 0..SKETCH_DEPTH {
			let (w, shift) = Self::position(h, i);
			if (block[w] >> shift) & 0xF < 15 {
				block[w] += 1u64 << shift;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.reset();
			}
		}
	}

	/// Halves every counter and the addition count.
	#[cold]
	fn reset(&mut self) {
		let mut odd: u32 = 0;
		for block in self.table.iter_mut() {
			for w in block.0.iter_mut() {
				odd += (*w & 0x1111_1111_1111_1111).count_ones();
				*w = (*w >> 1) & RESET_MASK;
			}
		}
		self.additions = (self.additions - (odd >> 2).min(self.additions)) >> 1;
	}

	#[inline]
	fn estimate_hashed(&self, h: u64) -> u32 {
		let block = &self.table[self.block_index(h)].0;
		let mut min = 15u32;
		for i in 0..SKETCH_DEPTH {
			let (w, shift) = Self::position(h, i);
			let c = ((block[w] >> shift) & 0xF) as u32;
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
		let mut s = CountMinSketch::new(16, 4);
		let h = key_hash(&7i64);
		s.increment_hashed(h);
		s.increment_hashed(h);
		assert!(s.estimate_hashed(h) >= 2);
	}

	#[test]
	fn sketch_saturates_at_15() {
		let mut s = CountMinSketch::new(64, 1_000);
		let h = key_hash(&3i64);
		for _ in 0..100 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 15);
	}

	#[test]
	fn sketch_ages_by_halving() {
		// sample_size = 10: the 10th addition triggers a reset.
		let mut s = CountMinSketch::new(16, 1);
		let h = key_hash(&11i64);
		for _ in 0..9 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 9);
		s.increment_hashed(h);
		assert_eq!(s.estimate_hashed(h), 5);
	}

	#[test]
	fn sketch_heap_within_budget() {
		let cache = TinyLfuCache::<i64, i64>::new(1024);
		assert_eq!(cache.sketch_heap_bytes(), 8 * 1024);
	}

	#[test]
	fn region_tag_roundtrips_without_corrupting_links() {
		let mut n = Node { key: 1u64, value: 2u64, prev: NIL, next: NIL };
		n.set_prev_idx(5);
		n.set_region(PROTECTED);
		assert_eq!(n.prev_idx(), 5);
		assert_eq!(n.region(), PROTECTED);
		n.set_prev_idx(NIL_USIZE);
		assert_eq!(n.prev_idx(), NIL_USIZE);
		assert_eq!(n.region(), PROTECTED);
		n.set_next_idx(7);
		assert_eq!(n.next_idx(), 7);
		assert_eq!(n.region(), PROTECTED);
	}

	#[test]
	fn node_is_24_bytes() {
		assert_eq!(std::mem::size_of::<Node<u64, u64>>(), 24);
	}

	#[test]
	fn capacity_colliding_with_sentinel_is_rejected() {
		assert!(TinyLfuCache::<i64, i64>::try_new(NIL_USIZE).is_none());
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
