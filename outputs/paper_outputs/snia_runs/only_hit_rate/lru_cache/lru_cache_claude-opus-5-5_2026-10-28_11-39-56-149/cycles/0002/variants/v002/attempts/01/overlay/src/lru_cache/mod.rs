// O(1) W-TinyLFU cache built on v001's arena-based Segmented LRU.
// Three lists share one arena:
//   - a small admission-free LRU window (~1% of capacity) that receives new entries,
//   - a probationary main segment (~20% of the main area),
//   - a protected main segment (~80% of the main area) reached only by re-referenced entries.
// When the window overflows, its LRU (the candidate) competes against the main-area victim
// (probation LRU, else protected LRU) using a 4-bit count-min frequency sketch; ties admit.
// For capacities below 3 the window is disabled and the cache behaves exactly like v001's SLRU.
// Nodes are stored in a Vec (arena) indexed by u32. On eviction the slot is reused in-place,
// so no free-list management is needed. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

/// Sentinel for "no node" in the u32-indexed linked lists.
const NIL: u32 = u32::MAX;

/// Segment identifiers (index into the `head`/`tail` arrays).
const PROB: u8 = 0;
const PROT: u8 = 1;
const WIN: u8 = 2;

// ---------------------------------------------------------------------------------------------
// Frequency sketch
// ---------------------------------------------------------------------------------------------

/// Minimal FxHash-style hasher (no crates).
struct FxHasher(u64);

const FX_K: u64 = 0x517c_c1b7_2722_0a95;

impl FxHasher {
	#[inline]
	fn add(&mut self, x: u64) {
		self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(FX_K);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.add(b as u64);
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
}

/// Hash a key once; the result is well mixed (splitmix64 finalizer) so that row indices can be
/// derived from it with cheap per-row seeds.
#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = FxHasher(0);
	key.hash(&mut h);
	let mut x = h.finish();
	x ^= x >> 30;
	x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
	x ^= x >> 27;
	x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
	x ^= x >> 31;
	x
}

const ROW_SEEDS: [u64; 4] = [
	0x9e37_79b9_7f4a_7c15,
	0xc2b2_ae3d_27d4_eb4f,
	0x1656_67b1_9e37_79f9,
	0x27d4_eb2f_1656_67c5,
];
const ROW_MUL: u64 = 0xff51_afd7_ed55_8ccd;

/// 4-row count-min sketch of saturating 4-bit counters packed 16 per `u64`.
#[derive(Debug)]
struct FreqSketch {
	table: Vec<u64>,
	width: usize,    // counters per row (power of two)
	row_shift: u32,  // 64 - log2(width)
	count: usize,    // increments since the last halving
	reset_at: usize, // halve when `count` reaches this
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = std::cmp::max(capacity, 64).next_power_of_two();
		let words = (4 * width) / 16;
		Self {
			table: vec![0u64; words],
			width,
			row_shift: 64 - width.trailing_zeros(),
			count: 0,
			reset_at: std::cmp::max(1, capacity.saturating_mul(10)),
		}
	}

	/// Counter position (word index, bit shift) for `row`.
	#[inline]
	fn slot(&self, hash: u64, row: usize) -> (usize, u32) {
		let h = (hash ^ ROW_SEEDS[row]).wrapping_mul(ROW_MUL);
		let col = (h >> self.row_shift) as usize;
		let counter = row * self.width + col;
		(counter >> 4, ((counter & 15) as u32) * 4)
	}

	fn increment(&mut self, hash: u64) {
		for row in 0..4 {
			let (w, shift) = self.slot(hash, row);
			if (self.table[w] >> shift) & 0xF < 15 {
				self.table[w] += 1u64 << shift;
			}
		}
		self.count += 1;
		if self.count >= self.reset_at {
			for w in self.table.iter_mut() {
				*w = (*w >> 1) & 0x7777_7777_7777_7777;
			}
			self.count /= 2;
		}
	}

	fn estimate(&self, hash: u64) -> u64 {
		let mut min = 15u64;
		for row in 0..4 {
			let (w, shift) = self.slot(hash, row);
			let c = (self.table[w] >> shift) & 0xF;
			if c < min {
				min = c;
			}
		}
		min
	}
}

// ---------------------------------------------------------------------------------------------
// Cache
// ---------------------------------------------------------------------------------------------

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
	segment: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: [u32; 3],         // LRU (front) per segment: [probation, protected, window]
	tail: [u32; 3],         // MRU (back) per segment
	prot_len: usize,
	prot_cap: usize,
	win_len: usize,
	win_cap: usize, // 0 => window disabled (pure v001 SLRU)
	capacity: usize,
	sketch: FreqSketch,
}

impl<K, V> LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("LruCache.try_new: capacity must be > 0");
			return None;
		}
		if capacity >= NIL as usize {
			log::error!("LruCache.try_new: capacity {capacity} exceeds u32 index range");
			return None;
		}
		let win_cap = if capacity < 3 { 0 } else { std::cmp::max(1, capacity / 100) };
		let main = capacity - win_cap;
		let prot_cap = if main == 1 { 0 } else { std::cmp::max(1, main * 8 / 10) };
		log::info!("LruCache.try_new: capacity={capacity} win_cap={win_cap} prot_cap={prot_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: [NIL; 3],
			tail: [NIL; 3],
			prot_len: 0,
			prot_cap,
			win_len: 0,
			win_cap,
			capacity,
			sketch: FreqSketch::new(if win_cap == 0 { 1 } else { capacity }),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it per W-TinyLFU / SLRU rules.
	/// Every access (hit or miss) is recorded in the frequency sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		if self.win_cap > 0 {
			self.sketch.increment(hash_key(key));
		}
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx as usize].value)
	}

	/// Inserts or updates `key`.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a re-reference
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx as usize].value = value;
			self.touch(idx);
			return;
		}
		if self.win_cap == 0 {
			self.put_slru(key, value);
		} else {
			self.put_tinylfu(key, value);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// v001 behaviour (capacity < 3): new keys enter probation; evict probation LRU, else
	/// protected LRU.
	fn put_slru(&mut self, key: K, value: V) {
		let idx = if self.nodes.len() >= self.capacity {
			let victim = if self.head[PROB as usize] != NIL {
				self.head[PROB as usize]
			} else {
				self.head[PROT as usize]
			};
			debug_assert!(victim != NIL, "a list must be non-empty when arena is full");
			self.evict(victim);
			victim
		} else {
			self.nodes.len() as u32
		};
		self.place(idx, key, value, PROB);
	}

	/// W-TinyLFU insert: new key enters the window; window overflow sends the window LRU
	/// (candidate) to probation, subject to frequency admission when the arena is full.
	fn put_tinylfu(&mut self, key: K, value: V) {
		self.sketch.increment(hash_key(&key));

		if self.nodes.len() < self.capacity {
			// Free room: new key into window; overflow candidate goes straight to probation.
			let idx = self.nodes.len() as u32;
			self.place(idx, key, value, WIN);
			if self.win_len > self.win_cap {
				let cand = self.head[WIN as usize];
				debug_assert!(cand != NIL);
				self.move_to(cand, PROB);
			}
			return;
		}

		// Arena full: window is at win_cap, the incoming key pushes out the window LRU.
		let cand = self.head[WIN as usize];
		debug_assert!(cand != NIL, "window must be non-empty when arena is full");
		let victim = if self.head[PROB as usize] != NIL {
			self.head[PROB as usize]
		} else {
			self.head[PROT as usize]
		};

		let admit = victim != NIL && {
			let cand_est = self.sketch.estimate(hash_key(&self.nodes[cand as usize].key));
			let victim_est = self.sketch.estimate(hash_key(&self.nodes[victim as usize].key));
			cand_est >= victim_est
		};

		let slot = if admit {
			self.evict(victim);
			self.move_to(cand, PROB);
			victim
		} else {
			self.evict(cand);
			cand
		};
		self.place(slot, key, value, WIN);
	}

	/// Write `key`/`value` into arena slot `idx` (pushing a new node if `idx == nodes.len()`),
	/// link it at the MRU end of `segment`, and register it in the map.
	fn place(&mut self, idx: u32, key: K, value: V, segment: u8) {
		let i = idx as usize;
		if i == self.nodes.len() {
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				segment,
			});
		} else {
			let node = &mut self.nodes[i];
			node.key = key.clone();
			node.value = value;
			node.segment = segment;
		}
		self.push_back(idx);
		match segment {
			PROT => self.prot_len += 1,
			WIN => self.win_len += 1,
			_ => {}
		}
		self.map.insert(key, idx);
	}

	/// Remove a linked node from the map and its list (slot stays in the arena for reuse).
	fn evict(&mut self, idx: u32) {
		let i = idx as usize;
		let evicted_key = self.nodes[i].key.clone();
		self.map.remove(&evicted_key);
		self.unlink(idx);
		match self.nodes[i].segment {
			PROT => self.prot_len -= 1,
			WIN => self.win_len -= 1,
			_ => {}
		}
	}

	/// Move a linked node to the MRU end of `segment`, maintaining segment lengths.
	fn move_to(&mut self, idx: u32, segment: u8) {
		let i = idx as usize;
		self.unlink(idx);
		match self.nodes[i].segment {
			PROT => self.prot_len -= 1,
			WIN => self.win_len -= 1,
			_ => {}
		}
		self.nodes[i].segment = segment;
		self.push_back(idx);
		match segment {
			PROT => self.prot_len += 1,
			WIN => self.win_len += 1,
			_ => {}
		}
	}

	/// Handle a re-reference: window hit -> window MRU; probation -> protected (demoting the
	/// protected LRU on overflow); protected -> protected MRU.
	fn touch(&mut self, idx: u32) {
		let i = idx as usize;
		let seg = self.nodes[i].segment;
		if seg == WIN {
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		self.unlink(idx);
		if seg == PROB {
			if self.prot_cap == 0 {
				// Degenerate (capacity 1): everything stays in probation
				self.push_back(idx);
				return;
			}
			self.nodes[i].segment = PROT;
			self.push_back(idx);
			self.prot_len += 1;
			if self.prot_len > self.prot_cap {
				let demote = self.head[PROT as usize];
				debug_assert!(demote != NIL);
				self.unlink(demote);
				self.nodes[demote as usize].segment = PROB;
				self.push_back(demote);
				self.prot_len -= 1;
			}
		} else {
			self.push_back(idx);
		}
	}

	/// Remove node from its segment's list.
	fn unlink(&mut self, idx: u32) {
		let i = idx as usize;
		let seg = self.nodes[i].segment as usize;
		let prev = self.nodes[i].prev;
		let next = self.nodes[i].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head[seg] = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail[seg] = prev;
		}
		self.nodes[i].prev = NIL;
		self.nodes[i].next = NIL;
	}

	/// Append node to the MRU end of its segment's list.
	fn push_back(&mut self, idx: u32) {
		let i = idx as usize;
		let seg = self.nodes[i].segment as usize;
		let tail = self.tail[seg];
		self.nodes[i].prev = tail;
		self.nodes[i].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx;
		} else {
			self.head[seg] = idx;
		}
		self.tail[seg] = idx;
	}
}

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		LruCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		LruCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn node_fits_baseline_size() {
		assert!(std::mem::size_of::<Node<u64, u64>>() <= 48);
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_probation_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1 (probation LRU)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_promotes_to_protected() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // 1 -> protected
		cache.put(3, 30); // evicts key 2 (probation LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_existing_promotes_to_protected() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // 1 -> protected
		cache.put(3, 30); // evicts key 2

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
		// capacity 3 -> window 1, probation 1, protected 1.
		// After 1,2,3: window [3], probation [1, 2]. Each later put pushes the window LRU
		// into main; equal one-shot frequencies tie, so the candidate is admitted and the
		// probation LRU is evicted.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn new_key_enters_window_then_probation() {
		let mut cache = LruCache::<i64, i64>::new(3);
		cache.put(1, 10);
		assert_eq!(cache.nodes[cache.map[&1] as usize].segment, WIN);
		cache.put(2, 20); // window overflow: 1 -> probation
		assert_eq!(cache.nodes[cache.map[&1] as usize].segment, PROB);
		assert_eq!(cache.nodes[cache.map[&2] as usize].segment, WIN);
		assert_eq!(cache.win_len, 1);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // window [3], probation [1, 2]
		for _ in 0..10 {
			cache.get(&1); // 1 -> protected
		}
		cache.put(4, 40); // candidate 3 ties victim 2 -> admit, evict 2
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // window [3], probation [1, 2]
		cache.get(&2); // 2 -> protected; probation: 1
		cache.put(4, 40); // candidate 3 ties victim 1 -> evict 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn admission_rejects_colder_candidate() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // window [3], probation [1, 2]
		cache.get(&1); // protected [1], probation [2]
		cache.get(&2); // protected [2], 1 demoted: probation [1]
		cache.get(&1); // protected [1], 2 demoted: probation [2] (freq 2)
		cache.put(4, 40); // candidate 3 (freq 1) < victim 2 (freq 2) -> evict 3
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		// capacity 2 -> prot_cap 1 (no window)
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3; probation: 2, 3
		cache.get(&3); // 3 -> protected; probation: 2
		cache.put(4, 40); // evicts 2 -> slot 1 reused for key 4; probation: 4
		cache.put(5, 50); // evicts 4 (probation LRU) -> slot 1 reused again; 3 stays protected
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		// capacity 2 -> prot_cap 1 (no window)
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // protected: 1; probation: 2
		cache.get(&2); // protected: 2; 1 demoted -> probation: 1
		cache.put(3, 30); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn scan_does_not_flush_protected() {
		let mut cache = LruCache::new(10); // window 1, main 9, prot_cap 7
		for k in 0..8 {
			cache.put(k, k); // window [7], probation [0..=6]
		}
		for k in 0..7 {
			cache.get(&k); // promote 0..=6 to protected
		}
		for k in 100..200 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for k in 0..7 {
			assert_eq!(*cache.get(&k).unwrap(), k);
		}
	}

	#[test]
	fn hot_key_survives_long_one_shot_scan() {
		let capacity = 10usize;
		let mut cache = LruCache::new(capacity);
		cache.put(0i64, 0i64); // window [0]
		cache.put(1, 1); // 0 -> probation
		for _ in 0..5 {
			cache.get(&0); // 0 -> protected, frequency grows
		}
		for k in 1000..1000 + (10 * capacity) as i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&0).unwrap(), 0);
		assert!(cache.map.len() <= capacity);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // window [3], probation [1, 2]
		cache.put(3, 31); // window hit -> stays window MRU
		cache.put(4, 40); // candidate 3 ties victim 1 -> evict 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn sketch_saturates_and_halves() {
		let mut s = FreqSketch::new(4); // width 64, reset_at 40
		let h = hash_key(&7i64);
		for _ in 0..20 {
			s.increment(h);
		}
		assert_eq!(s.estimate(h), 15);
		for _ in 0..20 {
			s.increment(h);
		}
		// 40th increment triggered a halving
		assert!(s.estimate(h) <= 7);
		assert_eq!(s.count, 20);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
