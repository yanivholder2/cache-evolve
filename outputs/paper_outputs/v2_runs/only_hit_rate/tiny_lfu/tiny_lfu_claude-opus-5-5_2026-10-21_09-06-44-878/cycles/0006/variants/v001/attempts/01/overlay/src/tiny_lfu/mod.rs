use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Number of oldest probation entries sampled when choosing an eviction victim.
const VICTIM_SAMPLE: usize = 8;

/// Segment tags for W-TinyLFU nodes.
const SEG_PROBATION: u8 = 0;
const SEG_PROTECTED: u8 = 1;
const SEG_WINDOW: u8 = 2;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

/// Cheap, non-cryptographic FxHash-style hasher with a splitmix64 avalanche in
/// `finish`. Used both for the count-min sketch and for the key->slot map.
#[derive(Debug, Default, Clone, Copy)]
pub struct LocalFastHasher {
	h: u64,
}

impl LocalFastHasher {
	#[inline(always)]
	fn add(&mut self, x: u64) {
		self.h = (self.h.rotate_left(5) ^ x).wrapping_mul(FX_SEED);
	}
}

#[inline(always)]
fn splitmix64(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

impl Hasher for LocalFastHasher {
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
		splitmix64(self.h)
	}
}

type FastBuildHasher = BuildHasherDefault<LocalFastHasher>;

#[inline(always)]
fn hash_key<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut hasher = LocalFastHasher::default();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// 0 = probation, 1 = protected, 2 = window.
	segment: u8,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize, FastBuildHasher>,
	nodes: Vec<Node<K, V>>,
	prob_head: Option<usize>,
	prob_tail: Option<usize>,
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	prot_len: usize,
	protected_cap: usize,
	/// Admission window (LRU) in front of the SLRU main region.
	win_head: Option<usize>,
	win_tail: Option<usize>,
	win_len: usize,
	/// 0 when `capacity < 2` (no window; plain SLRU + TinyLFU admission).
	window_cap: usize,
	/// `capacity - window_cap`: probation + protected budget.
	main_cap: usize,
	capacity: usize,
	sketch: CountMinSketch,
	/// Hash of the key whose most recent `get` missed. A following `put` of the
	/// same key has already been counted by that `get`, so it skips the sketch
	/// increment (each miss is counted once, not twice).
	pending_miss_hash: Option<u64>,
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
		let window_cap = if capacity < 2 { 0 } else { (capacity / 100).max(1) };
		let main_cap = capacity - window_cap;
		let protected_cap = (main_cap * 8 / 10).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window_cap={window_cap} main_cap={main_cap} protected_cap={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			protected_cap,
			win_head: None,
			win_tail: None,
			win_len: 0,
			window_cap,
			main_cap,
			capacity,
			sketch: CountMinSketch::new(width),
			pending_miss_hash: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = hash_key(key);
		self.sketch.increment_hashed(h);
		let Some(&idx) = self.map.get(key) else {
			self.pending_miss_hash = Some(h);
			return None;
		};
		self.pending_miss_hash = None;
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let key_hash = hash_key(&key);
		let deduped = self.pending_miss_hash.take() == Some(key_hash);
		if !deduped {
			self.sketch.increment_hashed(key_hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		if self.window_cap == 0 {
			self.put_main_only(key, key_hash, value);
			return;
		}

		// Newcomers are admitted to the window unconditionally. If that would
		// overflow the window, the window LRU (the candidate) leaves first: it
		// moves into probation if main has room, otherwise it contests the
		// probation victim. The freed slot (if any) is reused for the newcomer.
		let mut slot: Option<usize> = None;
		if self.win_len >= self.window_cap {
			if let Some(cand) = self.win_head {
				let main_len = self.nodes.len() - self.win_len;
				if main_len < self.main_cap {
					self.unlink(cand);
					self.win_len -= 1;
					self.nodes[cand].segment = SEG_PROBATION;
					self.push_back(cand);
				} else {
					let (victim_idx, victim_freq) = self.select_victim();
					let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
					if cand_freq > victim_freq {
						self.evict(victim_idx);
						self.unlink(cand);
						self.win_len -= 1;
						self.nodes[cand].segment = SEG_PROBATION;
						self.push_back(cand);
						slot = Some(victim_idx);
					} else {
						self.evict(cand);
						slot = Some(cand);
					}
				}
			}
		}

		if slot.is_none() && self.nodes.len() >= self.capacity {
			// Defensive: should be unreachable (full cache always has a full
			// window), but never exceed capacity.
			let (victim_idx, _) = self.select_victim();
			self.evict(victim_idx);
			slot = Some(victim_idx);
		}

		let idx = match slot {
			Some(i) => {
				self.nodes[i].key = key.clone();
				self.nodes[i].value = value;
				self.nodes[i].segment = SEG_WINDOW;
				i
			}
			None => {
				let i = self.nodes.len();
				self.nodes.push(Node {
					key: key.clone(),
					value,
					prev: None,
					next: None,
					segment: SEG_WINDOW,
				});
				i
			}
		};
		self.push_back(idx);
		self.win_len += 1;
		self.map.insert(key, idx);
	}

	/// Remove a resident node from the map and its list, fixing segment counts.
	/// The slot stays allocated in `nodes` for reuse by the caller.
	fn evict(&mut self, idx: usize) {
		let k = self.nodes[idx].key.clone();
		self.map.remove(&k);
		self.unlink(idx);
		match self.nodes[idx].segment {
			SEG_PROTECTED => self.prot_len -= 1,
			SEG_WINDOW => self.win_len -= 1,
			_ => {}
		}
	}

	/// Pre-window behaviour (used when `capacity < 2`): newcomers contest the
	/// probation victim directly.
	fn put_main_only(&mut self, key: K, key_hash: u64, value: V) {
		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				segment: SEG_PROBATION,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		let (victim_idx, victim_freq) = self.select_victim();
		let incoming_freq = self.sketch.estimate_hashed(key_hash);
		if incoming_freq > victim_freq {
			self.evict(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].segment = SEG_PROBATION;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Hit handling: window hits move to window MRU (no promotion); probation
	/// hits are promoted to protected MRU (demoting the protected LRU back to
	/// probation MRU on overflow); protected hits move to protected MRU.
	fn on_hit(&mut self, idx: usize) {
		self.unlink(idx);
		match self.nodes[idx].segment {
			SEG_WINDOW | SEG_PROTECTED => {
				self.push_back(idx);
			}
			_ => {
				self.nodes[idx].segment = SEG_PROTECTED;
				self.push_back(idx);
				self.prot_len += 1;
				if self.prot_len > self.protected_cap {
					if let Some(demote) = self.prot_head {
						self.unlink(demote);
						self.nodes[demote].segment = SEG_PROBATION;
						self.push_back(demote);
						self.prot_len -= 1;
					}
				}
			}
		}
	}

	/// Walk the `VICTIM_SAMPLE` oldest probation entries and return the one with
	/// the smallest frequency estimate. Ties keep the entry closest to the head
	/// (strict `<`). Falls back to the protected list (then the window) if
	/// probation is empty.
	fn select_victim(&self) -> (usize, u32) {
		let head = self
			.prob_head
			.or(self.prot_head)
			.or(self.win_head)
			.expect("TinyLfuCache: full cache has no victim");
		let mut best_idx = head;
		let mut best_freq = self.sketch.estimate(&self.nodes[head].key);
		let mut cursor = self.nodes[head].next;
		let mut seen = 1;
		while seen < VICTIM_SAMPLE {
			let Some(idx) = cursor else { break };
			let freq = self.sketch.estimate(&self.nodes[idx].key);
			if freq < best_freq {
				best_freq = freq;
				best_idx = idx;
			}
			cursor = self.nodes[idx].next;
			seen += 1;
		}
		(best_idx, best_freq)
	}

	#[inline(always)]
	fn set_head(&mut self, seg: u8, v: Option<usize>) {
		match seg {
			SEG_PROTECTED => self.prot_head = v,
			SEG_WINDOW => self.win_head = v,
			_ => self.prob_head = v,
		}
	}

	#[inline(always)]
	fn set_tail(&mut self, seg: u8, v: Option<usize>) {
		match seg {
			SEG_PROTECTED => self.prot_tail = v,
			SEG_WINDOW => self.win_tail = v,
			_ => self.prob_tail = v,
		}
	}

	#[inline(always)]
	fn tail_of(&self, seg: u8) -> Option<usize> {
		match seg {
			SEG_PROTECTED => self.prot_tail,
			SEG_WINDOW => self.win_tail,
			_ => self.prob_tail,
		}
	}

	/// Unlink `idx` from the list selected by its `segment`.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let seg = self.nodes[idx].segment;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.set_head(seg, next),
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.set_tail(seg, prev),
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Push `idx` to the MRU end of the list selected by its `segment`.
	fn push_back(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment;
		let tail = self.tail_of(seg);
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.set_head(seg, Some(idx)),
		}
		self.set_tail(seg, Some(idx));
	}
}

/// Count-min sketch over a flat `depth * width` counter array; row `i` lives at
/// offset `i * width`. Row positions come from one 64-bit hash via
/// Kirsch–Mitzenmacher double hashing.
#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<u32>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![0; depth * width];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	/// Flat indices (row offset already applied) of the key's counter in each row.
	#[inline(always)]
	fn positions(&self, h: u64) -> [usize; SKETCH_DEPTH] {
		let h1 = h as u32;
		let h2 = ((h >> 32) as u32) | 1;
		let mask = (self.width - 1) as u32;
		let mut out = [0usize; SKETCH_DEPTH];
		for (i, slot) in out.iter_mut().enumerate() {
			let pos = (h1.wrapping_add((i as u32).wrapping_mul(h2)) & mask) as usize;
			*slot = i * self.width + pos;
		}
		out
	}

	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		for idx in self.positions(h) {
			let counter = &mut self.counts[idx];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	#[inline]
	fn estimate_hashed(&self, h: u64) -> u32 {
		let mut min = MAX_COUNT;
		for idx in self.positions(h) {
			let c = self.counts[idx];
			if c < min {
				min = c;
			}
		}
		min
	}

	#[allow(dead_code)]
	#[inline]
	fn increment<K: Hash>(&mut self, key: &K) {
		self.increment_hashed(hash_key(key));
	}

	#[inline]
	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self.estimate_hashed(hash_key(key))
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

	fn list_len<K, V>(cache: &TinyLfuCache<K, V>, mut cur: Option<usize>, seg: u8) -> usize {
		let mut n = 0;
		while let Some(i) = cur {
			assert_eq!(cache.nodes[i].segment, seg);
			n += 1;
			cur = cache.nodes[i].next;
		}
		n
	}

	fn check_structure<K, V>(cache: &TinyLfuCache<K, V>) {
		let prob = list_len(cache, cache.prob_head, SEG_PROBATION);
		let prot = list_len(cache, cache.prot_head, SEG_PROTECTED);
		let win = list_len(cache, cache.win_head, SEG_WINDOW);
		assert_eq!(prot, cache.prot_len);
		assert_eq!(win, cache.win_len);
		assert!(cache.prot_len <= cache.protected_cap);
		assert!(cache.win_len <= cache.window_cap);
		assert!(prob + prot <= cache.main_cap);
		assert_eq!(prob + prot + win, cache.nodes.len());
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.nodes.len() <= cache.capacity);
	}

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
	fn window_sizing() {
		let c = TinyLfuCache::<i64, i64>::new(2048);
		assert_eq!(c.window_cap, 20);
		assert_eq!(c.main_cap, 2028);
		assert_eq!(c.protected_cap, 2028 * 8 / 10);
		let c = TinyLfuCache::<i64, i64>::new(1);
		assert_eq!(c.window_cap, 0);
		assert_eq!(c.main_cap, 1);
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
	fn newcomer_admitted_to_window_and_cold_window_victim_rejected() {
		let mut cache = TinyLfuCache::new(2); // window 1, main 1
		cache.put(1, 10);
		cache.put(2, 20); // 1 spills to probation, window = [2]
		cache.get(&1); // 1 promoted, freq 2
		cache.put(3, 30); // 3 enters window; candidate 2 (freq 1) loses to 1

		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		check_structure(&cache);
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
	fn window_candidate_beats_min_frequency_probation_victim() {
		let mut cache = TinyLfuCache::new(4); // window 1, main 3, protected 2
		cache.put(1, 10);
		for _ in 0..5 {
			cache.get(&1); // key 1 is hot (freq 6); window hits do not promote
		}
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		// probation = [1, 2, 3], window = [4].
		assert_eq!(cache.prob_head, cache.map.get(&1).copied());
		assert_eq!(cache.win_head, cache.map.get(&4).copied());
		check_structure(&cache);

		cache.get(&4);
		cache.get(&4); // window candidate 4 warms to freq 3
		cache.get(&5);
		cache.put(5, 50);

		// Victim is the first min-frequency probation entry (key 2), not hot key 1.
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		check_structure(&cache);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		let mut cache = TinyLfuCache::new(5); // window 1, main 4, protected_cap = 3
		for k in 0..5 {
			cache.put(k, k);
		}
		for k in 0..5 {
			cache.get(&k);
			check_structure(&cache);
		}
		assert_eq!(cache.prot_len, 3);
		// Key 0 was the protected LRU when key 3 was promoted, so it is demoted.
		assert_eq!(cache.prob_head, cache.map.get(&0).copied());
		// Key 4 is in the window and was not promoted.
		assert_eq!(cache.win_head, cache.map.get(&4).copied());
	}

	#[test]
	fn structure_holds_under_mixed_workload() {
		for &cap in &[2usize, 3, 8, 150, 300] {
			let mut cache = TinyLfuCache::new(cap);
			let mut x: u64 = 12345 ^ cap as u64;
			let universe = (cap * 5) as u64;
			for i in 0..20000u64 {
				x = splitmix64(x);
				let key = if i % 3 == 0 { (x % (universe / 5 + 1)) as i64 } else { (x % universe) as i64 };
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
				check_structure(&cache);
			}
			for k in 0..universe as i64 {
				if let Some(&v) = cache.get(&k) {
					assert_eq!(v, k * 10);
				}
			}
			assert_eq!(cache.nodes.len(), cap);
			assert_eq!(cache.win_len, cache.window_cap);
		}
	}

	#[test]
	fn capacity_one_works() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10); // promoted; probation empty
		cache.get(&2);
		cache.get(&2);
		cache.get(&2);
		cache.put(2, 20);
		check_structure(&cache);
		assert!(cache.nodes.len() <= 1);
	}

	#[test]
	fn sketch_width_is_sixteen_times_capacity() {
		let cache = TinyLfuCache::<i64, i64>::new(2048);
		// 2048 * 16 = 32768 counters per row, 4 rows of u32.
		assert_eq!(cache.sketch_heap_bytes(), 4 * 32768 * 4);
	}
}
