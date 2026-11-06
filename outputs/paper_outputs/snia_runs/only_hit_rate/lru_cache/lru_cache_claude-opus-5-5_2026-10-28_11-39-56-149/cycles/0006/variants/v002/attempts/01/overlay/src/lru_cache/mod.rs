// O(1) W-TinyLFU cache built on v001's arena-based Segmented LRU, with a hill-climbed
// window/main split.
// Three lists share one arena:
//   - an admission-free LRU window that receives new entries; its target size `win_cap`
//     starts at ~1% of capacity and is adapted by a hill climber between 1% and 40%,
//   - a probationary main segment (~20% of the main area),
//   - a protected main segment (~80% of the main area) reached only by re-referenced entries.
// When the window overflows, its LRU (the candidate) competes against the main-area victim
// (probation LRU, else protected LRU) using a 4-bit count-min frequency sketch fronted by a
// doorkeeper bitset; ties admit. Each access is counted once: a put that directly follows a
// get-miss of the same key does not increment the sketch again.
// Hill climber: every `sample_period` gets (clamp(10 * capacity, 2_000, 50_000)) the sampled
// hit rate is compared with the previous sample; the step direction flips only after two
// consecutive drops (a single drop, e.g. from a half-applied lazy resize, keeps the direction).
// The step magnitude starts at min(max(1, capacity / 64), 256), decays by 2% per round and
// resets to the initial magnitude once it reaches 1. Resizes are lazy: the window shrinks/grows
// only as puts arrive on a full arena, and the TinyLFU duel stays in force while the window grows.
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
/// Multiplier for doorkeeper bit positions (distinct from `ROW_MUL`).
const DOOR_MUL: u64 = 0x2545_f491_4f6c_dd1d;

/// 4-row count-min sketch of saturating 4-bit counters packed 16 per `u64`, fronted by a
/// `width`-bit doorkeeper that absorbs each key's first occurrence per reset period.
#[derive(Debug)]
struct FreqSketch {
	table: Vec<u64>,
	door: Vec<u64>,  // doorkeeper bitset, `width` bits
	width: usize,    // counters per row (power of two)
	row_shift: u32,  // 64 - log2(width)
	count: usize,    // increments since the last halving
	reset_at: usize, // halve when `count` reaches this
}

impl FreqSketch {
	fn new(capacity: usize) -> Self {
		let width = std::cmp::max(capacity.saturating_mul(4), 64).next_power_of_two();
		let words = (4 * width) / 16;
		Self {
			table: vec![0u64; words],
			door: vec![0u64; width / 64],
			width,
			row_shift: 64 - width.trailing_zeros(),
			count: 0,
			reset_at: std::cmp::max(1, capacity.saturating_mul(16)),
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

	/// Doorkeeper bit position (word index, bit) for probe `i`.
	#[inline]
	fn door_bit(&self, hash: u64, i: usize) -> (usize, u32) {
		let h = (hash ^ ROW_SEEDS[i]).wrapping_mul(DOOR_MUL);
		let pos = (h >> self.row_shift) as usize;
		(pos >> 6, (pos & 63) as u32)
	}

	#[inline]
	fn door_contains(&self, hash: u64) -> bool {
		(0..4).all(|i| {
			let (w, b) = self.door_bit(hash, i);
			(self.door[w] >> b) & 1 == 1
		})
	}

	fn increment(&mut self, hash: u64) {
		if self.door_contains(hash) {
			for row in 0..4 {
				let (w, shift) = self.slot(hash, row);
				if (self.table[w] >> shift) & 0xF < 15 {
					self.table[w] += 1u64 << shift;
				}
			}
		} else {
			for i in 0..4 {
				let (w, b) = self.door_bit(hash, i);
				self.door[w] |= 1u64 << b;
			}
		}
		self.count += 1;
		if self.count >= self.reset_at {
			for w in self.table.iter_mut() {
				*w = (*w >> 1) & 0x7777_7777_7777_7777;
			}
			self.door.fill(0);
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
		min + self.door_contains(hash) as u64
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
	win_cap: usize, // 0 => window disabled (pure v001 SLRU); otherwise the climbed target
	capacity: usize,
	sketch: FreqSketch,
	last_miss_hash: u64,   // hash of the most recent get-miss (already counted in the sketch)
	last_miss_valid: bool, // whether `last_miss_hash` is still pending a put
	// Hill climber (active only when win_cap > 0)
	sample_hits: u32,
	sample_accesses: u32,
	prev_hit_rate: f64,
	step: isize,           // signed window-size step
	step_init: usize,      // initial step magnitude (reset target)
	consecutive_drops: u8, // hit-rate drops in a row since the last flip / non-drop
	sample_period: usize,
	min_win: usize,
	max_win: usize,
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
		let sample_period = capacity.saturating_mul(10).clamp(2_000, 50_000);
		let step_init = std::cmp::min(std::cmp::max(1, capacity / 64), 256);
		let min_win = std::cmp::max(1, capacity / 100);
		let max_win = std::cmp::max(min_win, capacity * 40 / 100);
		log::info!(
			"LruCache.try_new: capacity={capacity} win_cap={win_cap} prot_cap={prot_cap} \
			 sample_period={sample_period} step={step_init} win_range=[{min_win}, {max_win}]"
		);
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
			last_miss_hash: 0,
			last_miss_valid: false,
			sample_hits: 0,
			sample_accesses: 0,
			prev_hit_rate: 0.0,
			step: step_init as isize,
			step_init,
			consecutive_drops: 0,
			sample_period,
			min_win,
			max_win,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it per W-TinyLFU / SLRU rules.
	/// Every access (hit or miss) is recorded in the frequency sketch exactly once.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = if self.win_cap > 0 {
			let h = hash_key(key);
			self.sketch.increment(h);
			Some(h)
		} else {
			None
		};
		let found = self.map.get(key).copied();
		if self.win_cap > 0 {
			self.sample_accesses += 1;
			if found.is_some() {
				self.sample_hits += 1;
			}
			if self.sample_accesses as usize >= self.sample_period {
				self.climb();
			}
		}
		match found {
			None => {
				if let Some(h) = h {
					self.last_miss_hash = h;
					self.last_miss_valid = true;
				}
				None
			}
			Some(idx) => {
				self.last_miss_valid = false;
				self.touch(idx);
				Some(&self.nodes[idx as usize].value)
			}
		}
	}

	/// One hill-climbing round: flip direction after two consecutive hit-rate drops, decay the
	/// step, move the window target, and rebalance the protected cap. Nodes are not evicted here.
	fn climb(&mut self) {
		let hr = self.sample_hits as f64 / self.sample_accesses as f64;
		if hr < self.prev_hit_rate {
			self.consecutive_drops = self.consecutive_drops.saturating_add(1);
			if self.consecutive_drops >= 2 {
				self.step = -self.step;
				self.consecutive_drops = 0;
			}
		} else {
			self.consecutive_drops = 0;
		}
		let sign: isize = if self.step < 0 { -1 } else { 1 };
		let mut mag = std::cmp::max(1, self.step.unsigned_abs() * 98 / 100);
		if mag <= 1 {
			mag = self.step_init;
		}
		self.step = sign * mag as isize;

		let target = (self.win_cap as isize + self.step)
			.clamp(self.min_win as isize, self.max_win as isize) as usize;
		self.win_cap = target;
		self.prev_hit_rate = hr;
		self.sample_hits = 0;
		self.sample_accesses = 0;

		let main = self.capacity - self.win_cap;
		self.prot_cap = std::cmp::max(1, main * 8 / 10);
		while self.prot_len > self.prot_cap {
			let demote = self.head[PROT as usize];
			debug_assert!(demote != NIL);
			self.move_to(demote, PROB);
		}
	}

	/// Inserts or updates `key`.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as a re-reference
		if let Some(&idx) = self.map.get(&key) {
			self.last_miss_valid = false;
			self.nodes[idx as usize].value = value;
			self.touch(idx);
			return;
		}
		if self.win_cap == 0 {
			self.last_miss_valid = false;
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

	/// Main-area victim: probation LRU, else protected LRU (NIL if the main area is empty).
	#[inline]
	fn main_victim(&self) -> u32 {
		if self.head[PROB as usize] != NIL {
			self.head[PROB as usize]
		} else {
			self.head[PROT as usize]
		}
	}

	/// W-TinyLFU insert: new key enters the window; window overflow sends the window LRU
	/// (candidate) to probation, subject to frequency admission when the arena is full.
	fn put_tinylfu(&mut self, key: K, value: V) {
		let h = hash_key(&key);
		// The access was already counted by the preceding get-miss of this key.
		if !(self.last_miss_valid && self.last_miss_hash == h) {
			self.sketch.increment(h);
		}
		self.last_miss_valid = false;

		if self.nodes.len() < self.capacity {
			// Free room: new key into window; overflow candidates go straight to probation.
			let idx = self.nodes.len() as u32;
			self.place(idx, key, value, WIN);
			while self.win_len > self.win_cap {
				let cand = self.head[WIN as usize];
				debug_assert!(cand != NIL);
				self.move_to(cand, PROB);
			}
			return;
		}

		// Arena full.
		let cand = self.head[WIN as usize];
		let victim = self.main_victim();

		if self.win_len < self.win_cap {
			// Window growing: the duel still gates which entry gives up its slot.
			let slot = if cand == NIL {
				debug_assert!(victim != NIL);
				self.evict(victim);
				victim
			} else if victim == NIL {
				self.evict(cand);
				cand
			} else {
				let cand_est = self.sketch.estimate(hash_key(&self.nodes[cand as usize].key));
				let victim_est = self.sketch.estimate(hash_key(&self.nodes[victim as usize].key));
				if cand_est >= victim_est || victim_est <= 1 {
					// Candidate stays in the window; the window grows by one.
					self.evict(victim);
					victim
				} else {
					self.evict(cand);
					cand
				}
			};
			self.place(slot, key, value, WIN);
			return;
		}

		// Window at (or above) target: the incoming key pushes out the window LRU.
		debug_assert!(cand != NIL, "window must be non-empty when arena is full");
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
		// Window shrinking: surplus window-LRU nodes move to probation MRU (no eviction; the
		// total count is unchanged).
		while self.win_len >= self.win_cap {
			let extra = self.head[WIN as usize];
			if extra == NIL {
				break;
			}
			self.move_to(extra, PROB);
		}
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
			while self.prot_len > self.prot_cap {
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
		// capacity 3 -> window 1, probation 1, protected 1 (min_win == max_win == 1).
		// After 1,2,3: window [3], probation [1, 2]. Each later put pushes the window LRU
		// into main; equal one-shot frequencies (doorkeeper only, estimate 1) tie, so the
		// candidate is admitted and the probation LRU is evicted.
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
		assert_eq!(cache.win_len, cache.win_cap);
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
		cache.put(4, 40); // candidate 3 ties victim 2 (both estimate 1) -> admit, evict 2
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
		cache.get(&1); // protected [1], 2 demoted: probation [2] (estimate 2)
		cache.put(4, 40); // candidate 3 (estimate 1) < victim 2 (estimate 2) -> evict 3
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
		// Promote 0..=6 to protected and make them clearly hotter than one-shot scan keys
		// (probation is empty afterwards, so the protected LRU is the admission victim).
		for _ in 0..5 {
			for k in 0..7 {
				cache.get(&k);
			}
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
	fn miss_then_put_counts_once() {
		let mut cache = LruCache::<i64, i64>::new(10);
		let k = 42i64;
		assert!(cache.get(&k).is_none());
		cache.put(k, 1);
		assert_eq!(cache.sketch.estimate(hash_key(&k)), 1);
		// A put not preceded by a get-miss of the same key still counts once.
		assert!(cache.get(&7).is_none());
		cache.put(8, 8);
		assert_eq!(cache.sketch.estimate(hash_key(&8)), 1);
	}

	#[test]
	fn sketch_saturates_and_halves() {
		// max(4 * 4, 64) -> width 64; reset_at = 4 * 16 = 64.
		let mut s = FreqSketch::new(4);
		assert_eq!(s.width, 64);
		assert_eq!(s.reset_at, 64);
		let h = hash_key(&7i64);
		s.increment(h); // first occurrence: doorkeeper only
		assert_eq!(s.estimate(h), 1);
		for _ in 0..19 {
			s.increment(h);
		}
		// counters saturate at 15, plus the doorkeeper bit
		assert_eq!(s.estimate(h), 16);
		assert!(s.estimate(h) <= 16);
		for _ in 0..44 {
			s.increment(h);
		}
		// 64th increment halved the counters (15 -> 7) and cleared the doorkeeper
		assert_eq!(s.count, 32);
		assert!(s.estimate(h) <= 7);
		s.increment(h); // re-enters the doorkeeper
		assert!(s.estimate(h) <= 8);
	}

	#[test]
	fn climber_flips_only_after_two_consecutive_drops() {
		let mut cache = LruCache::<i64, i64>::new(1000);
		assert!(cache.step > 0);
		assert_eq!(cache.consecutive_drops, 0);

		// Baseline sample: 50% hit rate (a rise from 0.0) keeps growing.
		cache.sample_hits = 500;
		cache.sample_accesses = 1000;
		cache.climb();
		assert!(cache.step > 0);
		assert_eq!(cache.consecutive_drops, 0);

		// First drop (50% -> 40%): direction is kept.
		cache.sample_hits = 400;
		cache.sample_accesses = 1000;
		cache.climb();
		assert!(cache.step > 0, "a single drop must not flip the step");
		assert_eq!(cache.consecutive_drops, 1);

		// Second drop in a row (40% -> 30%): direction flips and the counter resets.
		cache.sample_hits = 300;
		cache.sample_accesses = 1000;
		cache.climb();
		assert!(cache.step < 0, "two consecutive drops must flip the step");
		assert_eq!(cache.consecutive_drops, 0);

		// A drop followed by a non-drop resets the counter without flipping.
		cache.sample_hits = 200;
		cache.sample_accesses = 1000;
		cache.climb();
		assert!(cache.step < 0);
		assert_eq!(cache.consecutive_drops, 1);
		cache.sample_hits = 200;
		cache.sample_accesses = 1000;
		cache.climb();
		assert!(cache.step < 0);
		assert_eq!(cache.consecutive_drops, 0);
		assert!(cache.min_win <= cache.win_cap && cache.win_cap <= cache.max_win);
	}

	#[test]
	fn climber_keeps_window_in_bounds() {
		let capacity = 1000usize;
		let mut cache = LruCache::<i64, i64>::new(capacity);
		assert_eq!(cache.sample_period, 10_000);
		assert_eq!(cache.step_init, 15);
		let mut x: u64 = 0x1234_5678;
		for i in 0..(5 * cache.sample_period) as u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			// Mix of a hot set and a wide scan so the hit rate moves between samples.
			let k = if (i / 3000) % 2 == 0 { (x % 600) as i64 } else { (x % 20_000) as i64 };
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert!(cache.map.len() <= capacity);
			assert!(cache.prot_len <= cache.prot_cap);
		}
		assert!(cache.min_win <= cache.win_cap && cache.win_cap <= cache.max_win);
		assert_eq!(cache.map.len(), capacity);
		for (k, &idx) in cache.map.iter() {
			assert_eq!(*k, cache.nodes[idx as usize].key);
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		let _ = LruCache::<i64, i64>::new(0);
	}
}
