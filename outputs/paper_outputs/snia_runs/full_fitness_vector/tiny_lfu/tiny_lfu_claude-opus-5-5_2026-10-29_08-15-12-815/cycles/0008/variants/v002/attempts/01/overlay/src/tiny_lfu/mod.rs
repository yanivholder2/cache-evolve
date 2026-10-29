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
/// Absolute cap on the 5% window floor (entries).
const WINDOW_FLOOR_MAX: usize = 820;
/// Minimum estimate at which a window candidate wins an admission tie.
const TIE_WIN_MIN_FREQ: u32 = 2;

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
// All link arithmetic stays in u32; indices widen to `usize` only at the
// point of indexing `nodes`.
// ---------------------------------------------------------------------------

const REGION_SHIFT: u32 = 30;
const REGION_MASK: u32 = 0b11 << REGION_SHIFT;
/// "No link" sentinel for raw (untagged) 30-bit indices (= 0x3FFF_FFFF).
/// `try_new` rejects capacities that could collide with it.
const NIL: u32 = !REGION_MASK;
/// `NIL` widened, for the capacity guard in `try_new`.
const NIL_USIZE: usize = NIL as usize;

const WINDOW: u32 = 0;
const PROBATION: u32 = 1;
const PROTECTED: u32 = 2;

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
	fn prev_idx(&self) -> u32 {
		self.prev & !REGION_MASK
	}

	#[inline(always)]
	fn region(&self) -> u32 {
		self.prev >> REGION_SHIFT
	}

	#[cfg_attr(not(test), allow(dead_code))]
	#[inline(always)]
	fn set_prev_idx(&mut self, p: u32) {
		self.prev = (self.prev & REGION_MASK) | (p & !REGION_MASK);
	}

	#[cfg_attr(not(test), allow(dead_code))]
	#[inline(always)]
	fn set_region(&mut self, r: u32) {
		self.prev = (self.prev & !REGION_MASK) | (r << REGION_SHIFT);
	}

	#[cfg_attr(not(test), allow(dead_code))]
	#[inline(always)]
	fn next_idx(&self) -> u32 {
		self.next
	}

	#[cfg_attr(not(test), allow(dead_code))]
	#[inline(always)]
	fn set_next_idx(&mut self, n: u32) {
		self.next = n;
	}
}

#[derive(Debug, Clone, Copy)]
struct RegionList {
	head: u32,
	tail: u32,
	len: u32,
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
	map: HashMap<HashedKey<K>, u32, PrehashedBuildHasher>,
	nodes: Vec<Node<K, V>>,
	/// Indexed by the 2-bit region tag; length 4 so `tag` indexing needs no
	/// bounds check (slot 3 is never used).
	lists: [RegionList; 4],
	capacity: usize,
	window_cap: u32,
	protected_cap: u32,
	sketch: CountMinSketch,
	/// Hash of the key whose most recent `get` missed (already counted in
	/// the sketch). Meaningful only while `last_miss_valid` is set.
	last_miss_h: u64,
	/// Set by a `get` miss; cleared by a `get` hit and by every `put`. A
	/// `put` of the same hash while set skips its sketch increment, so a
	/// miss-then-insert counts once rather than twice.
	last_miss_valid: bool,
}

/// Window size: 1% of capacity, floored at min(5% of capacity, 820), and
/// kept strictly below capacity whenever capacity > 1.
fn window_size(capacity: usize) -> usize {
	let pct1 = (capacity / 100).max(1);
	let pct5 = capacity * 5 / 100;
	let mut window_cap = pct1.max(pct5.min(WINDOW_FLOOR_MAX));
	if capacity > 1 && window_cap >= capacity {
		window_cap = capacity.saturating_sub(1).max(1);
	}
	window_cap
}

/// Admission test: the candidate wins outright on a higher estimate, and
/// also wins a tie once it has been seen at least `TIE_WIN_MIN_FREQ` times.
/// Non-short-circuit `|` / `&` keep this to compares plus one branch.
#[inline(always)]
fn admit(cand_freq: u32, victim_freq: u32) -> bool {
	(cand_freq > victim_freq) | ((cand_freq == victim_freq) & (cand_freq >= TIE_WIN_MIN_FREQ))
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
		let window_cap = window_size(capacity);
		let main_cap = capacity - window_cap;
		let protected_cap = (main_cap as f64 * 0.8) as usize;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window={window_cap} protected={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, PrehashedBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			lists: [RegionList::empty(); 4],
			capacity,
			window_cap: window_cap as u32,
			protected_cap: protected_cap as u32,
			sketch: CountMinSketch::new(width, capacity),
			last_miss_h: 0,
			last_miss_valid: false,
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

	#[inline(always)]
	fn node(&self, i: u32) -> &Node<K, V> {
		debug_assert!((i as usize) < self.nodes.len());
		// SAFETY: indices come only from the map or from list links, which
		// always point at live slots (`nodes` never shrinks; a slot is pushed
		// before it is linked or mapped).
		unsafe { self.nodes.get_unchecked(i as usize) }
	}

	#[inline(always)]
	fn node_mut(&mut self, i: u32) -> &mut Node<K, V> {
		debug_assert!((i as usize) < self.nodes.len());
		// SAFETY: indices come only from the map or from list links, which
		// always point at live slots (`nodes` never shrinks; a slot is pushed
		// before it is linked or mapped).
		unsafe { self.nodes.get_unchecked_mut(i as usize) }
	}

	#[inline(always)]
	fn list(&mut self, r: u32) -> &mut RegionList {
		&mut self.lists[(r & 3) as usize]
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = key_hash(key);
		self.sketch.increment_hashed(h);
		let probe = HashedKey { h, key: key.clone() };
		let found = self.map.get(&probe).copied();
		match found {
			Some(idx) => {
				self.last_miss_valid = false;
				self.on_hit(idx);
				Some(&self.node(idx).value)
			}
			None => {
				self.last_miss_h = h;
				self.last_miss_valid = true;
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let key_h = key_hash(&key);
		// A put right after a missed get of the same key: that get already
		// counted this access, so do not count it a second time.
		let already_counted = self.last_miss_valid && key_h == self.last_miss_h;
		self.last_miss_valid = false;
		if !already_counted {
			self.sketch.increment_hashed(key_h);
		}
		let hk = HashedKey { h: key_h, key };
		if let Some(&idx) = self.map.get(&hk) {
			self.node_mut(idx).value = value;
			self.on_hit(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: hk.key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx, WINDOW);
			self.map.insert(hk, idx);
			// Spare room in main: window overflow just migrates to probation.
			if self.lists[WINDOW as usize].len > self.window_cap {
				let cand = self.lists[WINDOW as usize].head;
				self.unlink(cand);
				self.push_back(cand, PROBATION);
			}
			return;
		}

		// Full: pick a slot to free via the window-candidate vs main-victim contest.
		let (slot, old_h) = self.evict_one();
		let new_key = hk.key.clone();
		let old_key = std::mem::replace(&mut self.node_mut(slot).key, new_key);
		self.map.remove(&HashedKey { h: old_h, key: old_key });
		self.node_mut(slot).value = value;
		self.push_back(slot, WINDOW);
		self.map.insert(hk, slot);
	}

	/// Runs the admission contest and unlinks the loser; returns its slot and
	/// its hash (so the map removal does not rehash it).
	fn evict_one(&mut self) -> (u32, u64) {
		let cand = self.lists[WINDOW as usize].head;
		let victim = if self.lists[PROBATION as usize].head != NIL {
			self.lists[PROBATION as usize].head
		} else {
			self.lists[PROTECTED as usize].head
		};

		if cand == NIL {
			// Window empty (not reachable in steady state); evict main victim.
			let victim_h = key_hash(&self.node(victim).key);
			self.unlink(victim);
			return (victim, victim_h);
		}
		let cand_h = key_hash(&self.node(cand).key);
		if victim == NIL {
			// Main region empty (e.g. capacity 1): the candidate leaves.
			self.unlink(cand);
			return (cand, cand_h);
		}
		let victim_h = key_hash(&self.node(victim).key);
		let cand_freq = self.sketch.estimate_hashed(cand_h);
		let victim_freq = self.sketch.estimate_hashed(victim_h);
		if admit(cand_freq, victim_freq) {
			self.unlink(victim);
			self.unlink(cand);
			self.push_back(cand, PROBATION);
			(victim, victim_h)
		} else {
			self.unlink(cand);
			(cand, cand_h)
		}
	}

	#[inline]
	fn on_hit(&mut self, idx: u32) {
		let r = self.node(idx).region();
		if r == PROBATION {
			self.unlink(idx);
			self.push_back(idx, PROTECTED);
			if self.lists[PROTECTED as usize].len > self.protected_cap {
				let d = self.lists[PROTECTED as usize].head;
				self.unlink(d);
				self.push_back(d, PROBATION);
			}
		} else if self.list(r).tail != idx {
			self.unlink(idx);
			self.push_back(idx, r);
		}
	}

	/// Detaches `idx` from its region list. The node's own links are left
	/// stale: every caller immediately `push_back`s it, which overwrites both.
	#[inline(always)]
	fn unlink(&mut self, idx: u32) {
		let tagged = self.node(idx).prev;
		let next = self.node(idx).next;
		let r = tagged >> REGION_SHIFT;
		let prev = tagged & !REGION_MASK;
		if prev == NIL {
			self.list(r).head = next;
		} else {
			self.node_mut(prev).next = next;
		}
		if next == NIL {
			self.list(r).tail = prev;
		} else {
			let nn = self.node_mut(next);
			nn.prev = (nn.prev & REGION_MASK) | prev;
		}
		self.list(r).len -= 1;
	}

	#[inline(always)]
	fn push_back(&mut self, idx: u32, r: u32) {
		let tail = self.list(r).tail;
		{
			let n = self.node_mut(idx);
			n.prev = (r << REGION_SHIFT) | tail;
			n.next = NIL;
		}
		if tail == NIL {
			self.list(r).head = idx;
		} else {
			self.node_mut(tail).next = idx;
		}
		let l = self.list(r);
		l.tail = idx;
		l.len += 1;
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

	/// Pairwise conservative update: rows (0,1) and (2,3) each form a pair;
	/// within a pair only the counter(s) equal to the pair minimum are bumped
	/// (if that minimum is below saturation). Branchless.
	#[inline(always)]
	fn increment_hashed(&mut self, h: u64) {
		let bi = self.block_index(h);
		let block = &mut self.table[bi].0;
		let (w0, s0) = Self::position(h, 0);
		let (w1, s1) = Self::position(h, 1);
		let (w2, s2) = Self::position(h, 2);
		let (w3, s3) = Self::position(h, 3);
		// Rows own disjoint word pairs, so these reads stay valid across the writes.
		let c0 = (block[w0] >> s0) & 0xF;
		let c1 = (block[w1] >> s1) & 0xF;
		let c2 = (block[w2] >> s2) & 0xF;
		let c3 = (block[w3] >> s3) & 0xF;
		let m01 = c0.min(c1);
		let m23 = c2.min(c3);
		let live01 = (m01 < 15) as u64;
		let live23 = (m23 < 15) as u64;
		block[w0] += (((c0 == m01) as u64) & live01) << s0;
		block[w1] += (((c1 == m01) as u64) & live01) << s1;
		block[w2] += (((c2 == m23) as u64) & live23) << s2;
		block[w3] += (((c3 == m23) as u64) & live23) << s3;
		self.additions += (live01 | live23) as u32;
		if self.additions >= self.sample_size {
			self.reset();
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

	#[inline(always)]
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

	fn freq(cache: &TinyLfuCache<i64, i64>, k: i64) -> u32 {
		cache.sketch.estimate_hashed(key_hash(&k))
	}

	#[test]
	fn standalone_put_still_increments() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		cache.put(5, 50);
		assert_eq!(freq(&cache, 5), 1);
	}

	#[test]
	fn miss_then_put_counts_once() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&5).is_none());
		cache.put(5, 50);
		assert_eq!(freq(&cache, 5), 1);
		assert!(!cache.last_miss_valid);
	}

	#[test]
	fn miss_then_put_of_other_key_increments_it() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		let before = freq(&cache, 2);
		assert!(cache.get(&1).is_none());
		cache.put(2, 20);
		assert!(freq(&cache, 2) > before);
		assert!(!cache.last_miss_valid);
	}

	#[test]
	fn hit_clears_pending_miss() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		cache.put(1, 10);
		assert!(cache.get(&2).is_none());
		assert!(cache.last_miss_valid);
		assert!(cache.get(&1).is_some());
		assert!(!cache.last_miss_valid);
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
		// sample_size = 10: the 10th addition triggers a reset. With a single
		// key all four counters stay equal, so conservative update matches
		// plain increment here.
		let mut s = CountMinSketch::new(16, 1);
		let h = key_hash(&11i64);
		for _ in 0..9 {
			s.increment_hashed(h);
		}
		assert_eq!(s.estimate_hashed(h), 9);
		s.increment_hashed(h);
		assert_eq!(s.estimate_hashed(h), 5);
	}

	fn counter(s: &CountMinSketch, h: u64, i: usize) -> u64 {
		let (w, shift) = CountMinSketch::position(h, i);
		(s.table[s.block_index(h)].0[w] >> shift) & 0xF
	}

	#[test]
	fn conservative_update_skips_non_minimum_counter() {
		// Both hashes map to block 0. They share row 0's counter (byte0 = 0)
		// but use distinct counters in rows 1..3.
		let mut s = CountMinSketch::new(16, 1_000);
		let ha: u64 = 0x0000_0000_0000_0100;
		let hb: u64 = 0x0000_0000_0505_0200;
		for _ in 0..3 {
			s.increment_hashed(hb);
		}
		assert_eq!(counter(&s, ha, 0), 3);
		assert_eq!(counter(&s, ha, 1), 0);
		s.increment_hashed(ha);
		// Row 0 (shared, 3) is not the pair minimum: not bumped.
		assert_eq!(counter(&s, ha, 0), 3);
		assert_eq!(counter(&s, ha, 1), 1);
		assert_eq!(counter(&s, ha, 2), 1);
		assert_eq!(counter(&s, ha, 3), 1);
		assert_eq!(s.estimate_hashed(ha), 1);
		assert_eq!(s.estimate_hashed(hb), 3);
	}

	#[test]
	fn window_floor_sizes() {
		assert_eq!(window_size(1), 1);
		assert_eq!(window_size(2), 1);
		assert_eq!(window_size(64), 3);
		assert_eq!(window_size(2048), 102);
		assert_eq!(window_size(16384), 819);
		assert_eq!(window_size(131072), 1310);
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
		n.set_prev_idx(NIL);
		assert_eq!(n.prev_idx(), NIL);
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
	fn admit_tie_rule() {
		assert!(admit(3, 2));
		assert!(!admit(2, 3));
		assert!(admit(2, 2));
		assert!(admit(15, 15));
		assert!(!admit(1, 1));
		assert!(!admit(0, 0));
		assert!(admit(1, 0));
	}

	#[test]
	fn tie_at_two_admits_candidate() {
		// cap 2: window 1, main 1.
		let mut cache = TinyLfuCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // 1 -> probation, 2 in window
		assert!(cache.get(&1).is_some()); // victim 1 -> estimate 2
		assert!(cache.get(&2).is_some()); // candidate 2 -> estimate 2
		assert_eq!(freq(&cache, 1), 2);
		assert_eq!(freq(&cache, 2), 2);
		assert!(cache.get(&3).is_none());
		cache.put(3, 30); // tie at 2: candidate 2 admitted, victim 1 evicted

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn tie_at_one_keeps_victim() {
		let mut cache = TinyLfuCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // 1 -> probation, 2 in window
		assert_eq!(freq(&cache, 1), 1);
		assert_eq!(freq(&cache, 2), 1);
		cache.put(3, 30); // tie at 1: candidate 2 rejected, victim 1 stays

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
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
		cache.put(3, 30); // follows a miss of 3: not counted again; candidate 2 loses to victim 1

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

	/// Walks all three region lists and checks lengths, region tags and
	/// back-links against the node table.
	fn check_lists(cache: &TinyLfuCache<i64, i64>) {
		let mut total = 0usize;
		for r in [WINDOW, PROBATION, PROTECTED] {
			let list = cache.lists[r as usize];
			let mut cur = list.head;
			let mut prev = NIL;
			let mut count = 0u32;
			while cur != NIL {
				let n = &cache.nodes[cur as usize];
				assert_eq!(n.region(), r, "node {cur} tagged {} found in list {r}", n.region());
				assert_eq!(n.prev_idx(), prev, "node {cur} has a broken back-link");
				prev = cur;
				cur = n.next;
				count += 1;
				assert!(count as usize <= cache.nodes.len(), "cycle in list {r}");
			}
			assert_eq!(list.tail, prev, "list {r} tail mismatch");
			assert_eq!(list.len, count, "list {r} len mismatch");
			total += count as usize;
		}
		assert_eq!(total, cache.nodes.len());
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert_eq!(cache.lists[3].len, 0);
	}

	#[test]
	fn mixed_ops_keep_lists_consistent() {
		let mut cache = TinyLfuCache::<i64, i64>::new(64);
		let mut s: u64 = 0x2545_f491_4f6c_dd1d;
		for _ in 0..10_000 {
			s = s.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let r = s >> 33;
			// Skewed key space: half the ops hit a hot set of 32 keys.
			let key = if r & 1 == 0 { ((r >> 1) % 32) as i64 } else { ((r >> 1) % 512) as i64 };
			if cache.get(&key).is_none() {
				cache.put(key, key * 3);
			} else {
				assert_eq!(*cache.get(&key).unwrap(), key * 3);
			}
			check_lists(&cache);
		}
	}
}
