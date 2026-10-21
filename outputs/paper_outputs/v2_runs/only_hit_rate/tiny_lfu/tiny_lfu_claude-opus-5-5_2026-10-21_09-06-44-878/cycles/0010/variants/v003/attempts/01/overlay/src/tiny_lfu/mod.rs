use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Sketch weight of a `get` hit. Misses and puts count with weight 1.
const HIT_WEIGHT: u32 = 2;

/// Number of oldest probation entries sampled when choosing an eviction victim.
const VICTIM_SAMPLE: usize = 8;

/// Number of oldest protected entries sampled when choosing which entry to
/// demote to probation on protected overflow.
const DEMOTE_SAMPLE: usize = 4;

/// Minimum newcomer estimate for a ghost hit to win a tie against the victim.
const GHOST_TIE_MIN_FREQ: u32 = 2;

/// Segment tags for SLRU nodes.
const SEG_PROBATION: u8 = 0;
const SEG_PROTECTED: u8 = 1;

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

/// Ghost-set representation of a key hash: 0 is reserved as the empty-slot
/// sentinel of the ghost ring, so a hash of 0 is mapped to 1.
#[inline(always)]
fn ghost_hash(h: u64) -> u64 {
	if h == 0 { 1 } else { h }
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	/// 0 = probation, 1 = protected.
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
	capacity: usize,
	sketch: CountMinSketch,
	/// Hash of the key whose most recent `get` missed. A following `put` of the
	/// same key has already been counted by that `get`, so it skips the sketch
	/// increment (each miss is counted once, not twice).
	pending_miss_hash: Option<u64>,
	/// FIFO ring of hashes of recently evicted keys (length `capacity`; 0 = empty).
	ghost_ring: Vec<u64>,
	/// Next ring slot to overwrite.
	ghost_pos: usize,
	/// Membership index over the live ghost hashes.
	ghost_set: HashSet<u64, FastBuildHasher>,
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
		let protected_cap = (capacity * 8 / 10).max(1);
		let ghost_cap = capacity;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} protected_cap={protected_cap} ghost_cap={ghost_cap} hit_weight={HIT_WEIGHT}"
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
			capacity,
			sketch: CountMinSketch::new(width),
			pending_miss_hash: None,
			ghost_ring: vec![0; ghost_cap],
			ghost_pos: 0,
			ghost_set: HashSet::with_capacity_and_hasher(ghost_cap, FastBuildHasher::default()),
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

	/// Lookup first, then count: a hit adds `HIT_WEIGHT` to the sketch, a miss
	/// adds 1 (and arms the put-dedup).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = hash_key(key);
		let Some(&idx) = self.map.get(key) else {
			self.sketch.increment_hashed_by(h, 1);
			self.pending_miss_hash = Some(h);
			return None;
		};
		self.sketch.increment_hashed_by(h, HIT_WEIGHT);
		self.pending_miss_hash = None;
		// `on_hit` reads the estimate after the weighted increment.
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
		let gh = ghost_hash(key_hash);
		let is_ghost = self.ghost_set.contains(&gh);
		let admit = incoming_freq > victim_freq
			|| (is_ghost && incoming_freq == victim_freq && incoming_freq >= GHOST_TIE_MIN_FREQ);
		if admit {
			if is_ghost {
				// Stale ring slot is left in place; its later removal is a no-op.
				self.ghost_set.remove(&gh);
			}
			let victim_key = self.nodes[victim_idx].key.clone();
			self.record_ghost(hash_key(&victim_key));
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			if self.nodes[victim_idx].segment == SEG_PROTECTED {
				self.prot_len -= 1;
			}
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].segment = SEG_PROBATION;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Append an evicted key's hash to the ghost ring, displacing the oldest.
	#[inline]
	fn record_ghost(&mut self, h: u64) {
		let h = ghost_hash(h);
		let old = self.ghost_ring[self.ghost_pos];
		if old != 0 {
			self.ghost_set.remove(&old);
		}
		self.ghost_ring[self.ghost_pos] = h;
		self.ghost_set.insert(h);
		self.ghost_pos += 1;
		if self.ghost_pos == self.ghost_ring.len() {
			self.ghost_pos = 0;
		}
	}

	/// SLRU hit handling with a frequency-gated promotion.
	///
	/// Protected hits move to protected MRU. Probation hits are promoted while
	/// protected has room. Once protected is full, the would-be demotion target
	/// (lowest estimate among the `DEMOTE_SAMPLE` oldest protected entries) is
	/// found first; the hit is promoted (and the target demoted to probation MRU)
	/// only if the hit's estimate is at least the target's. Otherwise the hit
	/// stays in probation and moves to probation MRU.
	fn on_hit(&mut self, idx: usize) {
		if self.nodes[idx].segment == SEG_PROTECTED {
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.prot_len < self.protected_cap {
			self.unlink(idx);
			self.nodes[idx].segment = SEG_PROTECTED;
			self.push_back(idx);
			self.prot_len += 1;
			return;
		}

		// Protected is full: select the demotion target before `idx` joins
		// protected, so the sample covers only existing protected entries.
		let Some((demote, demote_freq)) = self.select_demotion() else {
			// Unreachable while protected_cap >= 1, but keep the plain promotion.
			self.unlink(idx);
			self.nodes[idx].segment = SEG_PROTECTED;
			self.push_back(idx);
			self.prot_len += 1;
			return;
		};
		let hit_freq = self.sketch.estimate(&self.nodes[idx].key);

		self.unlink(idx);
		if hit_freq >= demote_freq {
			// Same swap as the ungated SLRU: promote, then demote.
			self.nodes[idx].segment = SEG_PROTECTED;
			self.push_back(idx);
			self.unlink(demote);
			self.nodes[demote].segment = SEG_PROBATION;
			self.push_back(demote);
		} else {
			// Second chance within probation; protected untouched.
			self.push_back(idx);
		}
	}

	/// Walk the `DEMOTE_SAMPLE` oldest protected entries and return the one
	/// with the smallest frequency estimate together with that estimate. Ties
	/// keep the entry closest to the head (strict `<`), so equal estimates fall
	/// back to plain LRU demotion.
	fn select_demotion(&self) -> Option<(usize, u32)> {
		let head = self.prot_head?;
		let mut best_idx = head;
		let mut best_freq = self.sketch.estimate(&self.nodes[head].key);
		let mut cursor = self.nodes[head].next;
		let mut seen = 1;
		while seen < DEMOTE_SAMPLE {
			let Some(idx) = cursor else { break };
			let freq = self.sketch.estimate(&self.nodes[idx].key);
			if freq < best_freq {
				best_freq = freq;
				best_idx = idx;
			}
			cursor = self.nodes[idx].next;
			seen += 1;
		}
		Some((best_idx, best_freq))
	}

	/// Walk the `VICTIM_SAMPLE` oldest probation entries and return the one with
	/// the smallest frequency estimate. Ties keep the entry closest to the head
	/// (strict `<`). Falls back to the protected list if probation is empty.
	fn select_victim(&self) -> (usize, u32) {
		let head = self
			.prob_head
			.or(self.prot_head)
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

	/// Unlink `idx` from the list selected by its `segment`.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].segment == SEG_PROTECTED;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if protected {
					self.prot_head = next;
				} else {
					self.prob_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if protected {
					self.prot_tail = prev;
				} else {
					self.prob_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Push `idx` to the MRU end of the list selected by its `segment`.
	fn push_back(&mut self, idx: usize) {
		let protected = self.nodes[idx].segment == SEG_PROTECTED;
		let tail = if protected { self.prot_tail } else { self.prob_tail };
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => {
				if protected {
					self.prot_head = Some(idx);
				} else {
					self.prob_head = Some(idx);
				}
			}
		}
		if protected {
			self.prot_tail = Some(idx);
		} else {
			self.prob_tail = Some(idx);
		}
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

	/// Add `add` to each of the key's `SKETCH_DEPTH` counters, saturating at
	/// `MAX_COUNT`.
	#[inline]
	fn increment_hashed_by(&mut self, h: u64, add: u32) {
		for idx in self.positions(h) {
			let counter = &mut self.counts[idx];
			*counter = counter.saturating_add(add).min(MAX_COUNT);
		}
	}

	#[inline]
	fn increment_hashed(&mut self, h: u64) {
		self.increment_hashed_by(h, 1);
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
		assert_eq!(prot, cache.prot_len);
		assert!(cache.prot_len <= cache.protected_cap);
		assert_eq!(prob + prot, cache.nodes.len());
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.nodes.len() <= cache.capacity);
		assert!(cache.ghost_set.len() <= cache.ghost_ring.len());
		assert_eq!(cache.ghost_ring.len(), cache.capacity);
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
	fn get_hit_adds_hit_weight_and_miss_adds_one() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&7).is_none());
		assert_eq!(cache.sketch.estimate(&7i64), 1); // miss: +1
		cache.put(7, 70); // deduped: no increment
		assert_eq!(cache.sketch.estimate(&7i64), 1);
		let before = cache.sketch.estimate(&7i64);
		assert_eq!(*cache.get(&7).unwrap(), 70);
		assert_eq!(cache.sketch.estimate(&7i64), before + 2); // hit: +HIT_WEIGHT
		assert_eq!(HIT_WEIGHT, 2);
		check_structure(&cache);
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
	fn low_frequency_new_item_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_none());
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
	fn victim_is_min_frequency_among_oldest_not_lru_head() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		for _ in 0..5 {
			cache.get(&1); // key 1 is hot (1 + 5 * 2 = 11) and promoted to protected
		}
		assert_eq!(cache.sketch.estimate(&1i64), 11);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		// probation = [2, 3, 4], protected = [1]: key 1 is not a candidate.
		assert_eq!(cache.prot_head, cache.map.get(&1).copied());
		assert_eq!(cache.prob_head, cache.map.get(&2).copied());

		cache.get(&5);
		cache.get(&5); // warm newcomer: freq 2 (misses count 1; the put is deduped)
		cache.put(5, 50);

		// Victim is the first min-frequency probation entry: key 2.
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.nodes.len() <= 4);
		check_structure(&cache);
	}

	#[test]
	fn protected_overflow_demotes_to_probation() {
		let mut cache = TinyLfuCache::new(5); // protected_cap = 4
		for k in 0..5 {
			cache.put(k, k);
		}
		for k in 0..5 {
			cache.get(&k);
			check_structure(&cache);
		}
		assert_eq!(cache.prot_len, 4);
		// All keys have equal estimates (1 + 2 = 3), so the `>=` gate promotes
		// key 4 and the tie-break demotes the entry closest to the protected
		// head: key 0.
		assert_eq!(cache.prob_head, cache.map.get(&0).copied());
	}

	#[test]
	fn protected_overflow_spares_hot_head_and_demotes_colder_entry() {
		let mut cache = TinyLfuCache::new(10); // protected_cap = 8
		for k in 0..10 {
			cache.put(k, k);
		}
		for _ in 0..4 {
			cache.get(&0); // key 0: promoted to protected head, estimate 1 + 4 * 2 = 9
		}
		assert_eq!(cache.sketch.estimate(&0i64), 9);
		for k in 1..9 {
			cache.get(&k); // keys 1..8 promoted (estimate 3); key 8 overflows protected
			check_structure(&cache);
		}
		assert_eq!(cache.prot_len, 8);
		let idx0 = *cache.map.get(&0).unwrap();
		let idx1 = *cache.map.get(&1).unwrap();
		// Hot key 0 (protected LRU head) survives; colder key 1 is demoted.
		assert_eq!(cache.nodes[idx0].segment, SEG_PROTECTED);
		assert_eq!(cache.prot_head, Some(idx0));
		assert_eq!(cache.nodes[idx1].segment, SEG_PROBATION);
		assert_eq!(cache.prob_tail, Some(idx1));
	}

	#[test]
	fn cold_probation_hit_does_not_displace_hotter_protected() {
		let mut cache = TinyLfuCache::<i64, i64>::new(10); // protected_cap = 8
		for k in 0..8 {
			cache.put(k, k);
			cache.get(&k); // promoted while protected has room
		}
		assert_eq!(cache.prot_len, 8);
		for k in 0..8 {
			bump_to(&mut cache, k, 5);
		}
		cache.put(8, 80);
		cache.put(9, 90);
		check_structure(&cache);
		let idx8 = *cache.map.get(&8).unwrap();
		assert_eq!(cache.prob_head, Some(idx8));
		let prot_head_before = cache.prot_head;

		// Hit on key 8 raises it to estimate 1 + 2 = 3 < 5: stays in probation at MRU.
		assert_eq!(*cache.get(&8).unwrap(), 80);
		assert_eq!(cache.sketch.estimate(&8i64), 3);
		assert_eq!(cache.nodes[idx8].segment, SEG_PROBATION);
		assert_eq!(cache.prob_tail, Some(idx8));
		assert_eq!(cache.prot_len, 8);
		assert_eq!(cache.prot_head, prot_head_before);
		for k in 0..8 {
			let i = *cache.map.get(&k).unwrap();
			assert_eq!(cache.nodes[i].segment, SEG_PROTECTED);
		}
		check_structure(&cache);
	}

	#[test]
	fn structure_holds_under_mixed_workload() {
		let mut cache = TinyLfuCache::new(8);
		let mut x: u64 = 12345;
		for _ in 0..5000 {
			x = splitmix64(x);
			let key = (x % 40) as i64;
			if cache.get(&key).is_none() {
				cache.put(key, key * 10);
			}
			check_structure(&cache);
		}
		for k in 0..40i64 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
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

	#[test]
	fn weighted_increment_saturates() {
		let mut sketch = CountMinSketch::new(16);
		let h = hash_key(&3i64);
		for idx in sketch.positions(h) {
			sketch.counts[idx] = MAX_COUNT - 1;
		}
		sketch.increment_hashed_by(h, HIT_WEIGHT);
		assert_eq!(sketch.estimate_hashed(h), MAX_COUNT);
	}

	/// Raise `key`'s sketch estimate to at least `target` without touching the cache.
	fn bump_to(cache: &mut TinyLfuCache<i64, i64>, key: i64, target: u32) {
		while cache.sketch.estimate(&key) < target {
			cache.sketch.increment(&key);
		}
	}

	#[test]
	fn ghost_tie_admits_recently_evicted_but_not_never_seen() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		for k in 1..=4 {
			cache.put(k, k * 10);
		}
		// Warm newcomer 5 strictly beats the probation head (key 1) and evicts it.
		bump_to(&mut cache, 5, 2);
		cache.put(5, 50);
		assert!(cache.map.get(&1).is_none());
		assert!(cache.ghost_set.contains(&ghost_hash(hash_key(&1i64))));
		check_structure(&cache);

		// Every resident probation entry now has estimate 3.
		for k in 2..=5 {
			bump_to(&mut cache, k, 3);
		}
		let (_, victim_freq) = cache.select_victim();
		assert_eq!(victim_freq, 3);

		// Ghost key 1 returns and ties the victim at 3: admitted.
		bump_to(&mut cache, 1, 2);
		assert!(cache.get(&1).is_none()); // miss counts -> 3; put is deduped
		assert_eq!(cache.sketch.estimate(&1i64), 3);
		cache.put(1, 11);
		assert!(cache.map.contains_key(&1));
		assert!(!cache.ghost_set.contains(&ghost_hash(hash_key(&1i64))));
		check_structure(&cache);

		// Never-seen key 6 with the same tied estimate is rejected.
		let (_, victim_freq) = cache.select_victim();
		assert_eq!(victim_freq, 3);
		bump_to(&mut cache, 6, 2);
		assert!(cache.get(&6).is_none());
		assert_eq!(cache.sketch.estimate(&6i64), 3);
		cache.put(6, 60);
		assert!(!cache.map.contains_key(&6));
		check_structure(&cache);
	}
}
