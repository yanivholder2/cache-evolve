// Adaptive W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// * A small LRU *window* (starts at ~1% of capacity) admits every new key, so a
//   `put` is always immediately visible to the next `get`.
// * The *main* space is a segmented LRU: probation + protected (~80% of main).
// * When the window overflows, its LRU entry becomes a candidate for the main
//   space and must beat the probation LRU victim on estimated frequency
//   (TinyLFU admission: 4-bit count-min sketch with periodic halving). This
//   keeps one-hit wonders and scans from flushing the frequently used set.
// * Mini-simulation adaptation: a grid of small shadow W-TinyLFU caches, each with
//   a fixed (window fraction, sketch reset period) pair (plus one pure LRU),
//   replays a hash-sampled slice of the key stream. Periodically the configuration
//   of the shadow with the fewest (decayed) misses is applied to the real cache,
//   so it tracks the best static configuration for the current workload phase.
//   Shadows with the same reset period share one sketch (it only depends on the
//   key stream), which keeps the simulation cheap.
// Arena + index-linked lists, O(1) per operation, no unsafe, std only.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

const ADMIT_HASHDOS_THRESHOLD: u64 = 6;
/// Window sizes (fraction of capacity) evaluated by the shadow caches; a 100%
/// window (pure LRU) is evaluated separately since it ignores the sketch.
const WINDOW_FRACTIONS: [f64; 10] = [0.0, 0.01, 0.03, 0.06, 0.1, 0.2, 0.3, 0.45, 0.6, 0.8];
/// Sketch reset periods (multiples of capacity) evaluated by the shadow caches.
const SAMPLE_MULTIPLIERS: [usize; 3] = [4, 10, 25];
/// Starting configuration (Caffeine defaults: 1% window, 10x reset period).
const INITIAL_WINDOW: f64 = 0.01;
const INITIAL_MULTIPLIER: usize = 10;
/// Shadow caches are exact below ~2x this capacity, hash-sampled above.
const MINI_TARGET_SIZE: usize = 1024;
/// Per-period decay of the shadow miss scores.
const SCORE_DECAY: f64 = 0.8;
/// A shadow must beat the current configuration by 1% to trigger a switch.
const SWITCH_MARGIN: f64 = 0.99;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

/// 4-bit count-min sketch (4 hashed rows folded into one table) with aging.
#[derive(Debug, Default)]
struct Sketch {
	table: Vec<u64>,
	mask: usize,
	additions: usize,
	sample: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let words = capacity.max(16).next_power_of_two();
		Self {
			table: vec![0; words],
			mask: words * 16 - 1,
			additions: 0,
			sample: capacity.max(16).saturating_mul(10),
		}
	}

	fn slot(&self, hash: u64, row: u64) -> (usize, u32) {
		let mut x = hash.wrapping_add(row.wrapping_mul(0x9E37_79B9_7F4A_7C15));
		x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		x ^= x >> 31;
		let c = (x as usize) & self.mask;
		(c >> 4, ((c & 15) as u32) << 2)
	}

	fn frequency(&self, hash: u64) -> u64 {
		let mut freq = 15;
		for row in 0..4 {
			let (word, shift) = self.slot(hash, row);
			freq = freq.min((self.table[word] >> shift) & 15);
		}
		freq
	}

	/// Conservative update: only the minimal counters are incremented.
	fn increment(&mut self, hash: u64) {
		let freq = self.frequency(hash);
		if freq >= 15 {
			return;
		}
		for row in 0..4 {
			let (word, shift) = self.slot(hash, row);
			if (self.table[word] >> shift) & 15 == freq {
				self.table[word] += 1u64 << shift;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for word in self.table.iter_mut() {
				*word = (*word >> 1) & 0x7777_7777_7777_7777;
			}
			self.additions /= 2;
		}
	}
}

fn hash_of<T: Hash + ?Sized>(key: &T) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

/// Window size for `fraction` of `capacity` (capacity >= 1), at least one entry.
fn window_for(capacity: usize, fraction: f64) -> usize {
	((capacity as f64 * fraction).round() as usize).clamp(1, capacity)
}

/// Protected segment = 80% of the main space.
fn protected_for(main: usize) -> usize {
	main / 5 * 4 + main % 5 * 4 / 5
}

/// Sketch reset period: `multiplier` x capacity counter additions.
fn sample_for(capacity: usize, multiplier: usize) -> usize {
	capacity.max(16).saturating_mul(multiplier)
}

/// Fixed-configuration shadow cache replaying a hash-sampled slice of the stream.
#[derive(Debug)]
struct Shadow {
	cache: LruCache<u64, ()>,
	sketch: usize, // index into the shared shadow sketches (one per multiplier)
	window: f64,
	multiplier: usize,
	misses: u64, // misses in the current period
	score: f64,  // decayed misses
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena (capacity + 1 slots max)
	free: Vec<usize>,       // evicted slots awaiting reuse
	heads: [usize; 3],      // LRU end of window / probation / protected
	tails: [usize; 3],      // MRU end of window / probation / protected
	sizes: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	last_miss: Option<u64>, // hash of last get-miss (avoid double counting on put)
	shadows: Vec<Shadow>,         // fixed-configuration shadows (empty in shadows)
	shadow_sketches: Vec<Sketch>, // shared shadow sketches, one per reset multiplier
	config: usize,                // index of the shadow whose configuration is applied
	sample_rate: u64,
	period: usize,
	accesses: usize,
	rng: u64,
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
		let mut cache = Self::with_window(capacity, window_for(capacity, INITIAL_WINDOW));
		cache.sketch.sample = sample_for(capacity, INITIAL_MULTIPLIER);
		// Shadows see a hash-sampled 1/rate of the keys at 1/rate of the size.
		let rate = (capacity / MINI_TARGET_SIZE).max(1);
		let mini_cap = ((capacity + rate / 2) / rate).max(1);
		cache.shadow_sketches = SAMPLE_MULTIPLIERS
			.iter()
			.map(|&m| {
				let mut sketch = Sketch::new(mini_cap);
				sketch.sample = sample_for(mini_cap, m);
				sketch
			})
			.collect();
		let make_shadow = |window: f64, sketch: usize| {
			let mut mini = LruCache::<u64, ()>::with_window(mini_cap, window_for(mini_cap, window));
			mini.sketch = Sketch::new(1); // unused: shadows share `shadow_sketches`
			Shadow {
				cache: mini,
				sketch,
				window,
				multiplier: SAMPLE_MULTIPLIERS[sketch],
				misses: 0,
				score: 0.0,
			}
		};
		for s in 0..SAMPLE_MULTIPLIERS.len() {
			for &w in WINDOW_FRACTIONS.iter() {
				cache.shadows.push(make_shadow(w, s));
			}
		}
		// Pure LRU (100% window): the sketch plays no role, so one shadow suffices.
		let default_sketch = SAMPLE_MULTIPLIERS
			.iter()
			.position(|&m| m == INITIAL_MULTIPLIER)
			.unwrap_or(0);
		cache.shadows.push(make_shadow(1.0, default_sketch));
		cache.config = cache
			.shadows
			.iter()
			.position(|s| s.window == INITIAL_WINDOW && s.multiplier == INITIAL_MULTIPLIER)
			.unwrap_or(0);
		cache.sample_rate = rate as u64;
		cache.period = capacity.max(MINI_TARGET_SIZE);
		Some(cache)
	}

	/// Plain W-TinyLFU with a fixed window (no shadows); `capacity >= 1`.
	fn with_window(capacity: usize, window_max: usize) -> Self {
		Self {
			map: HashMap::with_capacity(capacity.saturating_add(1)),
			nodes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			sizes: [0; 3],
			capacity,
			window_max,
			protected_max: protected_for(capacity - window_max),
			sketch: Sketch::new(capacity),
			last_miss: None,
			shadows: Vec::new(),
			shadow_sketches: Vec::new(),
			config: 0,
			sample_rate: 1,
			period: usize::MAX,
			accesses: 0,
			rng: 0x2545_F491_4F6C_DD1D,
		}
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		self.simulate(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; overflow is resolved
	/// by TinyLFU admission between the window victim and the main victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_of(&key);
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		// Temporarily take the sketch so admission can borrow it alongside `self`.
		let sketch = std::mem::take(&mut self.sketch);
		self.insert(key, value, hash, &sketch);
		self.sketch = sketch;
	}

	/// Insert a new key into the window and resolve overflow via TinyLFU
	/// admission driven by `sketch`.
	fn insert(&mut self, key: K, value: V, hash: u64, sketch: &Sketch) {
		let idx = self.alloc(Node {
			key: key.clone(),
			value,
			hash,
			prev: NIL,
			next: NIL,
			queue: WINDOW as u8,
		});
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		let mut candidate = NIL;
		while self.sizes[WINDOW] > self.window_max {
			let c = self.heads[WINDOW];
			self.unlink(c);
			self.push_back(PROBATION, c);
			if candidate == NIL {
				candidate = c;
			}
		}
		while self.map.len() > self.capacity {
			self.evict_one(candidate, sketch);
			candidate = NIL;
		}
	}

	fn alloc(&mut self, node: Node<K, V>) -> usize {
		match self.free.pop() {
			Some(i) => {
				self.nodes[i] = node;
				i
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		}
	}

	fn on_hit(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue as usize;
		self.unlink(idx);
		if queue == WINDOW {
			self.push_back(WINDOW, idx);
		} else {
			self.push_back(PROTECTED, idx);
			self.demote_protected();
		}
	}

	fn demote_protected(&mut self) {
		while self.sizes[PROTECTED] > self.protected_max {
			let c = self.heads[PROTECTED];
			self.unlink(c);
			self.push_back(PROBATION, c);
		}
	}

	fn evict_one(&mut self, candidate: usize, sketch: &Sketch) {
		let mut victim = self.heads[PROBATION];
		if victim == candidate {
			victim = self.heads[PROTECTED];
		}
		let evict = if candidate == NIL {
			if victim == NIL {
				self.heads[WINDOW]
			} else {
				victim
			}
		} else if victim == NIL || !self.admit(candidate, victim, sketch) {
			candidate
		} else {
			victim
		};
		self.unlink(evict);
		self.map.remove(&self.nodes[evict].key);
		self.free.push(evict);
	}

	fn admit(&mut self, candidate: usize, victim: usize, sketch: &Sketch) -> bool {
		let candidate_freq = sketch.frequency(self.nodes[candidate].hash);
		let victim_freq = sketch.frequency(self.nodes[victim].hash);
		if candidate_freq > victim_freq {
			return true;
		}
		if candidate_freq < ADMIT_HASHDOS_THRESHOLD {
			return false;
		}
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		self.rng & 127 == 0
	}

	/// Feed the access to the shadow caches (sampled) and adapt once per period.
	fn simulate(&mut self, hash: u64) {
		if self.shadows.is_empty() {
			return;
		}
		if (hash >> 32) % self.sample_rate == 0 {
			for sketch in self.shadow_sketches.iter_mut() {
				sketch.increment(hash);
			}
			let sketches = &self.shadow_sketches;
			for shadow in self.shadows.iter_mut() {
				if shadow.cache.shadow_access(hash, &sketches[shadow.sketch]) {
					shadow.misses += 1;
				}
			}
		}
		self.accesses += 1;
		if self.accesses >= self.period {
			self.accesses = 0;
			self.adapt();
		}
	}

	/// Switch to the configuration whose shadow has the fewest decayed misses.
	fn adapt(&mut self) {
		for shadow in self.shadows.iter_mut() {
			shadow.score = shadow.score * SCORE_DECAY + shadow.misses as f64;
			shadow.misses = 0;
		}
		let threshold = self.shadows[self.config].score * SWITCH_MARGIN;
		let mut best = self.config;
		for (i, shadow) in self.shadows.iter().enumerate() {
			if shadow.score < threshold && shadow.score < self.shadows[best].score {
				best = i;
			}
		}
		if best != self.config {
			self.config = best;
			let window = self.shadows[best].window;
			let multiplier = self.shadows[best].multiplier;
			self.sketch.sample = sample_for(self.capacity, multiplier);
			self.set_window(window_for(self.capacity, window));
		}
	}

	/// Resize the window. Shrinking demotes window LRU entries into probation;
	/// growing is lazy: new arrivals fill the window while the oversized main
	/// space drains through ordinary probation-LRU eviction.
	fn set_window(&mut self, target: usize) {
		let target = target.clamp(1, self.capacity);
		self.window_max = target;
		self.protected_max = protected_for(self.capacity - target);
		while self.sizes[WINDOW] > self.window_max {
			let c = self.heads[WINDOW];
			self.unlink(c);
			self.push_back(PROBATION, c);
		}
		self.demote_protected();
	}

	/// Remove node from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.heads[queue] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tails[queue] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.sizes[queue] -= 1;
	}

	/// Append node to the MRU end of `queue`.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.tails[queue];
		self.nodes[idx].queue = queue as u8;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail == NIL {
			self.heads[queue] = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.tails[queue] = idx;
		self.sizes[queue] += 1;
	}
}

impl LruCache<u64, ()> {
	/// Shadow access driven by a shared sketch (already incremented for `hash`);
	/// the key is the hash itself. Returns true on a miss.
	fn shadow_access(&mut self, hash: u64, sketch: &Sketch) -> bool {
		if let Some(&idx) = self.map.get(&hash) {
			self.on_hit(idx);
			return false;
		}
		self.insert(hash, (), hash, sketch);
		true
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

	fn check_invariants<K, V>(cache: &LruCache<K, V>) {
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.nodes.len() <= cache.capacity + 1);
		assert_eq!(cache.sizes.iter().sum::<usize>(), cache.map.len());
		assert!(cache.window_max >= 1);
		assert!(cache.sizes[WINDOW] <= cache.window_max);
		assert!(cache.sizes[PROTECTED] <= cache.protected_max);
		assert!(cache.window_max + cache.protected_max <= cache.capacity);
		for shadow in &cache.shadows {
			check_invariants(&shadow.cache);
		}
	}

	#[test]
	fn window_resizing_preserves_invariants() {
		let mut cache = LruCache::new(64);
		let mut x: u64 = 7;
		for step in 0..10_000u64 {
			if step % 500 == 0 {
				let target = (step / 500 % 7) as usize * 10 + 1;
				cache.set_window(target);
				check_invariants(&cache);
			}
			x = x
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let k = (x >> 33) % 200;
			if cache.get(&k).is_none() {
				cache.put(k, k);
				assert_eq!(cache.get(&k).copied(), Some(k));
			}
			check_invariants(&cache);
		}
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
	fn overwrite_does_not_grow() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
		check_invariants(&cache);
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
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		check_invariants(&cache);
	}

	#[test]
	fn eviction_keeps_size_bounded_and_newest_key() {
		let mut cache = LruCache::new(3);
		for k in 1..=10 {
			cache.put(k, k * 10);
			check_invariants(&cache);
		}
		assert_eq!(cache.map.len(), 3);
		assert_eq!(*cache.get(&10).unwrap(), 100);
	}

	#[test]
	fn frequent_key_survives_scan() {
		let mut cache = LruCache::new(10);
		cache.put(0, 0);
		for _ in 0..20 {
			cache.get(&0);
		}
		for k in 1..=100 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			check_invariants(&cache);
		}
		assert!(cache.get(&0).is_some());
	}

	#[test]
	fn randomized_workload_preserves_invariants() {
		for &cap in &[1usize, 2, 3, 7, 50, 200] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 12345;
			for _ in 0..20_000 {
				x = x
					.wrapping_mul(6_364_136_223_846_793_005)
					.wrapping_add(1_442_695_040_888_963_407);
				let r = x >> 33;
				let k = if r & 1 == 0 {
					(r >> 1) % (cap as u64 + 1)
				} else {
					(r >> 1) % (cap as u64 * 8)
				};
				match cache.get(&k).copied() {
					Some(v) => assert_eq!(v, k + 1),
					None => {
						cache.put(k, k + 1);
						assert_eq!(cache.get(&k).copied(), Some(k + 1));
					}
				}
				check_invariants(&cache);
			}
		}
	}

	#[test]
	fn try_new_zero_returns_none() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
