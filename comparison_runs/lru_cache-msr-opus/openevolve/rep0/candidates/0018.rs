// W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// A small LRU admission *window* sits in front of a segmented-LRU *main* region
// (probation + protected). A Count-Min sketch with 4-bit counters and periodic halving
// estimates key popularity; when the window overflows, its LRU entry (candidate) duels
// the probation LRU entry (victim) and only the more frequent one stays resident.
// The configuration is chosen by racing key-only shadow replicas of this policy, each with a
// fixed (sketch aging period, window fraction) pair, on a hash-sampled slice of the stream
// (all of it for caches up to 4096 entries). Every sketch is fed every access, so the real
// cache can adopt the best pair's sketch and window split instantly: recency-biased phases
// get a large window, frequency-biased ones a small window, shifting popularity a short
// sketch memory and stable popularity a long one.
// All queues live in one arena (Vec<Node>) linked by indices; freed slots are recycled.
// No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	queue: usize, // WINDOW, PROBATION or PROTECTED
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

fn mix(mut x: u64) -> u64 {
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	x ^ (x >> 33)
}

const SEEDS: [u64; 4] = [
	0x9e37_79b9_7f4a_7c15,
	0xbf58_476d_1ce4_e5b9,
	0x94d0_49bb_1331_11eb,
	0x2545_f491_4f6c_dd1d,
];

/// Count-Min sketch: 4 rows of packed 4-bit counters, conservative update, halved
/// every `sample_size` additions so popularity estimates age out.
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u8>, // two 4-bit counters per byte
	width: usize,   // counters per row (power of two)
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	/// `period` is the aging period in multiples of the capacity (Caffeine uses 10).
	fn new(capacity: usize, period: usize) -> Self {
		let width = capacity.max(64).next_power_of_two() * 4;
		Self {
			table: vec![0; width * 2], // 4 rows * width counters / 2 per byte
			width,
			additions: 0,
			sample_size: period.saturating_mul(capacity.max(64)),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			*slot = row * self.width + (mix(hash ^ SEEDS[row]) as usize & (self.width - 1));
		}
		out
	}

	fn counter(&self, s: usize) -> u8 {
		(self.table[s >> 1] >> ((s & 1) << 2)) & 0x0f
	}

	fn frequency(&self, hash: u64) -> u8 {
		let slots = self.slots(hash);
		slots.iter().map(|&s| self.counter(s)).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let slots = self.slots(hash);
		let min = slots.iter().map(|&s| self.counter(s)).min().unwrap_or(0);
		if min >= 15 {
			return;
		}
		for &s in &slots {
			if self.counter(s) == min {
				self.table[s >> 1] += 1u8 << ((s & 1) << 2);
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			for b in self.table.iter_mut() {
				*b = (*b >> 1) & 0x77;
			}
			self.additions /= 2;
		}
	}
}

/// Window fractions raced against each other by the shadow replicas.
const WINDOW_FRACTIONS: [f64; 6] = [0.01, 0.04, 0.12, 0.30, 0.55, 0.85];
/// Sketch aging periods (multiples of capacity between counter halvings) raced by the
/// shadows: short memory tracks shifting popularity, long memory exploits stable skew.
const SKETCH_PERIODS: [usize; 4] = [3, 10, 30, 100];
/// Index into SKETCH_PERIODS used before the shadows have any evidence (Caffeine's 10x).
const DEFAULT_SKETCH: usize = 1;

/// Pass-through hasher for maps keyed by already-mixed 64-bit key hashes (shadows).
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = self.0.rotate_left(8) ^ u64::from(b);
		}
	}

	fn write_u64(&mut self, n: u64) {
		self.0 = n;
	}
}

type IdMap = HashMap<u64, usize, BuildHasherDefault<IdHasher>>;
/// Shadows track at most this many entries; larger caches are hash-sampled down to it.
const SHADOW_TARGET: usize = 4096;
const SAMPLE_SEED: u64 = 0x5851_f42d_4c95_7f2d;
/// Per-epoch decay of the shadow hit scores (~6-7 epochs of memory).
const SCORE_DECAY: f64 = 0.85;
/// A rival split must beat the current one by this factor before the cache switches.
const SWITCH_MARGIN: f64 = 1.01;

fn window_for(fraction: f64, capacity: usize) -> usize {
	if capacity < 2 {
		1
	} else {
		((fraction * capacity as f64).round() as usize).clamp(1, capacity - 1)
	}
}

/// Key-only W-TinyLFU replica with a fixed window size. It shares the real cache's
/// frequency sketch (which depends only on the access stream), so with no sampling it
/// behaves exactly like the real cache would with that window.
#[derive(Debug)]
struct Shadow {
	map: IdMap,
	keys: Vec<u64>,
	prev: Vec<usize>,
	next: Vec<usize>,
	queue: Vec<usize>,
	free: Vec<usize>,
	heads: [usize; 3],
	tails: [usize; 3],
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	hits: usize, // hits in the current epoch
	score: f64,  // exponentially decayed hits
	fraction: usize, // index into WINDOW_FRACTIONS
	sketch: usize,   // index into SKETCH_PERIODS
}

impl Shadow {
	fn new(capacity: usize, fraction: usize, sketch: usize) -> Self {
		let window_max = window_for(WINDOW_FRACTIONS[fraction], capacity);
		Self {
			map: IdMap::with_capacity_and_hasher(capacity + 1, Default::default()),
			keys: Vec::with_capacity(capacity + 1),
			prev: Vec::with_capacity(capacity + 1),
			next: Vec::with_capacity(capacity + 1),
			queue: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max: (capacity - window_max) * 4 / 5,
			hits: 0,
			score: 0.0,
			fraction,
			sketch,
		}
	}

	fn unlink(&mut self, i: usize) {
		let q = self.queue[i];
		let (p, n) = (self.prev[i], self.next[i]);
		if p == NIL {
			self.heads[q] = n;
		} else {
			self.next[p] = n;
		}
		if n == NIL {
			self.tails[q] = p;
		} else {
			self.prev[n] = p;
		}
		self.lens[q] -= 1;
	}

	fn push_back(&mut self, q: usize, i: usize) {
		let t = self.tails[q];
		self.queue[i] = q;
		self.prev[i] = t;
		self.next[i] = NIL;
		if t == NIL {
			self.heads[q] = i;
		} else {
			self.next[t] = i;
		}
		self.tails[q] = i;
		self.lens[q] += 1;
	}

	fn move_to(&mut self, q: usize, i: usize) {
		self.unlink(i);
		self.push_back(q, i);
	}

	fn remove(&mut self, i: usize) {
		self.unlink(i);
		self.map.remove(&self.keys[i]);
		self.free.push(i);
	}

	/// Simulates one access (miss => insert); returns whether it hit.
	fn access(&mut self, h: u64, sketch: &FrequencySketch) -> bool {
		if let Some(&i) = self.map.get(&h) {
			if self.queue[i] == WINDOW {
				self.move_to(WINDOW, i);
			} else {
				self.move_to(PROTECTED, i);
				while self.lens[PROTECTED] > self.protected_max {
					let j = self.heads[PROTECTED];
					self.move_to(PROBATION, j);
				}
			}
			return true;
		}
		let i = match self.free.pop() {
			Some(i) => {
				self.keys[i] = h;
				i
			}
			None => {
				self.keys.push(h);
				self.prev.push(NIL);
				self.next.push(NIL);
				self.queue.push(WINDOW);
				self.keys.len() - 1
			}
		};
		self.push_back(WINDOW, i);
		self.map.insert(h, i);
		let mut cand = NIL;
		while self.lens[WINDOW] > self.window_max {
			let j = self.heads[WINDOW];
			self.move_to(PROBATION, j);
			if cand == NIL {
				cand = j;
			}
		}
		while self.map.len() > self.capacity {
			let victim = [PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.heads[q])
				.find(|&v| v != NIL)
				.unwrap_or(NIL);
			if cand == NIL || cand == victim {
				if cand == victim {
					cand = self.next[victim];
				}
				self.remove(victim);
			} else if sketch.frequency(self.keys[cand]) > sketch.frequency(self.keys[victim]) {
				self.remove(victim);
			} else {
				let n = self.next[cand];
				self.remove(cand);
				cand = n;
			}
		}
		false
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // recycled arena slots
	heads: [usize; 3],      // LRU end of each queue
	tails: [usize; 3],      // MRU end of each queue
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketches: Vec<FrequencySketch>, // one per SKETCH_PERIODS entry, all fed every access
	sketch_choice: usize,           // sketch consulted by the real cache's admission filter
	last_miss: Option<u64>, // hash of the key whose get just missed (already counted)
	// configuration selection by shadow replicas
	shadows: Vec<Shadow>, // one per (sketch period, window fraction) pair
	choice: usize,        // index of the shadow whose configuration the real cache uses
	sample_mask: u64,     // keys with mix(hash ^ SAMPLE_SEED) & mask == 0 feed the shadows
	epoch_len: usize,
	epoch_count: usize,
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
		log::info!("LruCache.try_new: capacity={capacity}");
		let mut rate = 1usize;
		while capacity / rate > SHADOW_TARGET {
			rate *= 2;
		}
		let shadow_cap = (capacity / rate).max(1);
		let window_max = window_for(WINDOW_FRACTIONS[0], capacity);
		Some(Self {
			map: HashMap::with_capacity(capacity.saturating_add(1)),
			nodes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max: (capacity - window_max) * 4 / 5,
			sketches: SKETCH_PERIODS
				.iter()
				.map(|&p| FrequencySketch::new(capacity, p))
				.collect(),
			sketch_choice: DEFAULT_SKETCH,
			last_miss: None,
			shadows: (0..SKETCH_PERIODS.len())
				.flat_map(|sk| (0..WINDOW_FRACTIONS.len()).map(move |f| (sk, f)))
				.map(|(sk, f)| Shadow::new(shadow_cap, f, sk))
				.collect(),
			choice: DEFAULT_SKETCH * WINDOW_FRACTIONS.len(),
			sample_mask: rate as u64 - 1,
			epoch_len: shadow_cap.max(64),
			epoch_count: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present; records the access in the frequency sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.record(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.last_miss = None;
				self.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the admission window; when the cache is
	/// over capacity the TinyLFU duel decides which entry is evicted.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_key(&key);
		if self.last_miss != Some(hash) {
			// insert without a preceding get: count it as an access everywhere
			self.record(hash);
		}
		self.last_miss = None;
		let idx = match self.free.pop() {
			Some(i) => {
				self.nodes[i].key = key.clone();
				self.nodes[i].value = value;
				i
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					prev: NIL,
					next: NIL,
					queue: WINDOW,
				});
				self.nodes.len() - 1
			}
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		self.evict();
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			WINDOW => {
				self.unlink(idx);
				self.push_back(WINDOW, idx);
			}
			PROBATION => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
				self.demote_protected();
			}
			_ => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
			}
		}
	}

	fn demote_protected(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			let idx = self.heads[PROTECTED];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn victim(&self) -> usize {
		for q in [PROBATION, PROTECTED, WINDOW] {
			if self.heads[q] != NIL {
				return self.heads[q];
			}
		}
		NIL
	}

	fn admit(&self, candidate: usize, victim: usize) -> bool {
		let sketch = &self.sketches[self.sketch_choice];
		let c = sketch.frequency(hash_key(&self.nodes[candidate].key));
		let v = sketch.frequency(hash_key(&self.nodes[victim].key));
		c > v
	}

	fn evict(&mut self) {
		// Window overflow flows into probation; the first moved entry is the candidate.
		let mut candidate = NIL;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
			if candidate == NIL {
				candidate = idx;
			}
		}
		while self.map.len() > self.capacity {
			let victim = self.victim();
			if candidate == NIL || candidate == victim {
				if candidate == victim {
					candidate = self.nodes[victim].next;
				}
				self.remove(victim);
			} else if self.admit(candidate, victim) {
				self.remove(victim);
			} else {
				let next = self.nodes[candidate].next;
				self.remove(candidate);
				candidate = next;
			}
		}
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Counts one access in every sketch, then lets the shadows see it.
	fn record(&mut self, hash: u64) {
		for sketch in self.sketches.iter_mut() {
			sketch.increment(hash);
		}
		self.observe(hash);
	}

	/// Feeds a (sampled) access to every shadow replica and, once per epoch, re-selects
	/// the sketch period and window fraction for the real cache.
	fn observe(&mut self, hash: u64) {
		if mix(hash ^ SAMPLE_SEED) & self.sample_mask != 0 {
			return;
		}
		let sketches = &self.sketches;
		for s in self.shadows.iter_mut() {
			if s.access(hash, &sketches[s.sketch]) {
				s.hits += 1;
			}
		}
		self.epoch_count += 1;
		if self.epoch_count >= self.epoch_len {
			self.select();
		}
	}

	/// Decays shadow scores and switches the real cache to the best split (with hysteresis).
	fn select(&mut self) {
		self.epoch_count = 0;
		for s in self.shadows.iter_mut() {
			s.score = s.score * SCORE_DECAY + s.hits as f64;
			s.hits = 0;
		}
		let mut best = self.choice;
		for (i, s) in self.shadows.iter().enumerate() {
			if s.score > self.shadows[best].score {
				best = i;
			}
		}
		if best != self.choice
			&& self.shadows[best].score > self.shadows[self.choice].score * SWITCH_MARGIN
		{
			self.choice = best;
			self.sketch_choice = self.shadows[best].sketch;
			let w = window_for(WINDOW_FRACTIONS[self.shadows[best].fraction], self.capacity);
			self.resize(w);
		}
	}

	fn resize(&mut self, window_max: usize) {
		self.window_max = window_max;
		self.protected_max = (self.capacity - window_max) * 4 / 5;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
		self.demote_protected();
	}

	/// Remove node from its current queue.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.heads[q] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tails[q] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
		self.lens[q] -= 1;
	}

	/// Append node to the MRU end of queue `q`.
	fn push_back(&mut self, q: usize, idx: usize) {
		let tail = self.tails[q];
		self.nodes[idx].queue = q;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail == NIL {
			self.heads[q] = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.tails[q] = idx;
		self.lens[q] += 1;
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

	/// Simulator-style access: get, and on miss insert.
	fn access(cache: &mut LruCache<i64, i64>, key: i64) -> bool {
		if cache.get(&key).is_some() {
			true
		} else {
			cache.put(key, key * 10);
			false
		}
	}

	/// Verifies queue links, lengths, map consistency and size bounds.
	fn check_structure(cache: &LruCache<i64, i64>) {
		let mut total = 0;
		for q in 0..3 {
			let mut prev = NIL;
			let mut idx = cache.heads[q];
			let mut count = 0;
			while idx != NIL {
				let node = &cache.nodes[idx];
				assert_eq!(node.queue, q);
				assert_eq!(node.prev, prev);
				assert_eq!(cache.map[&node.key], idx);
				prev = idx;
				idx = node.next;
				count += 1;
			}
			assert_eq!(cache.tails[q], prev);
			assert_eq!(cache.lens[q], count);
			total += count;
		}
		assert_eq!(total, cache.map.len());
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.lens[WINDOW] <= cache.window_max);
		assert!(cache.lens[PROTECTED] <= cache.protected_max);
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::<i64, i64>::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::<i64, i64>::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow() {
		let mut cache = LruCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
		check_structure(&cache);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::<i64, i64>::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::<i64, i64>::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_structure(&cache);
	}

	#[test]
	fn newest_entry_is_always_resident() {
		let mut cache = LruCache::<i64, i64>::new(3);
		for k in 0..50 {
			cache.put(k, k);
			assert_eq!(cache.get(&k).copied(), Some(k));
			check_structure(&cache);
		}
	}

	#[test]
	fn frequent_items_survive_scan() {
		let mut cache = LruCache::<i64, i64>::new(10);
		for _ in 0..20 {
			for k in 0..6 {
				access(&mut cache, k);
			}
		}
		for k in 1000..1100 {
			access(&mut cache, k);
		}
		for k in 0..6 {
			assert!(cache.get(&k).is_some(), "hot key {k} was evicted by a scan");
		}
		check_structure(&cache);
	}

	#[test]
	fn random_workload_keeps_invariants() {
		let mut cache = LruCache::<i64, i64>::new(16);
		let mut x: u64 = 0x1234_5678;
		for step in 0..20_000i64 {
			x = x
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let r = (x >> 33) as i64;
			let key = if r % 4 == 0 { r % 500 } else { r % 24 };
			if step % 7 == 0 {
				cache.put(key, step);
				assert_eq!(*cache.get(&key).unwrap(), step);
			} else {
				access(&mut cache, key);
			}
			check_structure(&cache);
		}
	}

	#[test]
	fn shadows_stay_consistent() {
		let mut cache = LruCache::<i64, i64>::new(32);
		let mut x: u64 = 0x9abc_def0;
		for step in 0..30_000i64 {
			x = x
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let r = (x >> 33) as i64;
			// alternate recency-friendly and frequency-skewed phases
			let key = if (step / 5_000) % 2 == 0 { step / 3 - r % 20 } else { r % 64 };
			access(&mut cache, key);
			if step % 1_000 == 0 {
				check_structure(&cache);
			}
		}
		check_structure(&cache);
		assert!(cache.choice < cache.shadows.len());
		assert_eq!(cache.shadows.len(), SKETCH_PERIODS.len() * WINDOW_FRACTIONS.len());
		assert_eq!(cache.sketch_choice, cache.shadows[cache.choice].sketch);
		assert!(cache.sketch_choice < cache.sketches.len());
		for s in &cache.shadows {
			assert!(s.map.len() <= s.capacity);
			assert_eq!(s.lens.iter().sum::<usize>(), s.map.len());
			assert!(s.lens[WINDOW] <= s.window_max);
			assert!(s.lens[PROTECTED] <= s.protected_max);
		}
	}

	#[test]
	fn try_new_zero_is_none() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
