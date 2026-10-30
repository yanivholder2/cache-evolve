use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
/// Minimum counter ceiling (4-bit, as in Caffeine's FrequencySketch). A sketch whose aging
/// period is `factor * capacity` saturates at `max(MAX_COUNT, factor)`: within such a period
/// at most `capacity` keys can reach `factor` hits, so that ceiling never hides the
/// top-`capacity` boundary (TinyLFU's W/C counter cap).
const MAX_COUNT: u8 = 15;
/// Sketch aging periods maintained in parallel: a sketch halves every counter after
/// `factor * capacity` additions. Short periods favour recency, long ones frequency;
/// the miniature simulations pick the period that currently pays best.
const AGING_FACTORS: [usize; 4] = [4, 10, 25, 64];
/// Index of Caffeine's default aging period (10 * capacity) in `AGING_FACTORS`.
const INITIAL_AGING: usize = 1;
/// A warm candidate that loses the admission contest is still admitted with probability
/// 1/128. "Warm" is this estimate for a `MAX_COUNT`-ceiling sketch, scaled with the ceiling.
const ADMIT_RANDOM_THRESHOLD: u8 = 6;
const SEEDS: [u64; SKETCH_DEPTH] = [
	0xc3a5_c85c_97cb_3127,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
	0xcbf2_9ce4_8422_2325,
];

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const PROTECTED_FRACTION: f64 = 0.8;

/// Admission-window shares tried by the miniature simulations (index 1 = the initial 1%).
/// Roughly geometric spacing, densest where W-TinyLFU's optimum usually lies (1%–35%).
const MINI_WINDOW_FRACTIONS: [f64; 12] = [
	0.0, 0.01, 0.02, 0.04, 0.07, 0.11, 0.17, 0.25, 0.35, 0.5, 0.7, 1.0,
];
const INITIAL_CONFIG: usize = 1;
/// Miniature caches are hash-sampled down to roughly this many entries.
const MINI_TARGET_CAPACITY: usize = 8192;
const SAMPLE_SEED: u64 = 0x2545_f491_4f6c_dd1d;
/// Accesses per adaptation epoch: `EPOCH_FACTOR * capacity`, at least `MIN_EPOCH`.
const EPOCH_FACTOR: usize = 4;
const MIN_EPOCH: usize = 8192;
/// Per-epoch decay of the miniature caches' hit scores.
const SCORE_DECAY: f64 = 0.5;
/// Relative score lead a configuration needs before the main cache switches to it.
const SWITCH_MARGIN: f64 = 0.002;

fn hash_of<Q: Hash + ?Sized>(key: &Q) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

fn mix64(mut x: u64) -> u64 {
	x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	x ^ (x >> 31)
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

/// W-TinyLFU cache (window LRU + segmented-LRU main, TinyLFU admission) whose
/// window size is chosen by miniature simulations of alternative window sizes.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	policy: Policy,
	capacity: usize,
	/// One frequency sketch per entry of `AGING_FACTORS`, all fed the full access stream.
	sketches: Vec<CountMinSketch>,
	/// Index of the sketch (aging period) the main cache's admission contest consults.
	aging: usize,
	pending_miss: Option<u64>,
	minis: Vec<Mini>,
	sample_rate: u64,
	epoch_len: usize,
	epoch_accesses: usize,
	current: usize,
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
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		let sample_rate = (capacity / MINI_TARGET_CAPACITY).max(1);
		let mini_capacity = (capacity / sample_rate).max(1);
		// One miniature simulation per (aging period, window share) pair.
		let mut minis: Vec<Mini> =
			Vec::with_capacity(AGING_FACTORS.len() * MINI_WINDOW_FRACTIONS.len());
		for aging in 0..AGING_FACTORS.len() {
			for &fraction in MINI_WINDOW_FRACTIONS.iter() {
				let i = minis.len();
				let seed = SEEDS[i % SKETCH_DEPTH] ^ i as u64;
				minis.push(Mini::new(mini_capacity, fraction, aging, seed));
			}
		}
		let sketches: Vec<CountMinSketch> = AGING_FACTORS
			.iter()
			.map(|&factor| CountMinSketch::new(width, factor, capacity))
			.collect();
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			policy: Policy::new(capacity, capacity / 100, 0x9E37_79B9_7F4A_7C15),
			capacity,
			sketches,
			aging: INITIAL_AGING,
			pending_miss: None,
			minis,
			sample_rate: sample_rate as u64,
			epoch_len: EPOCH_FACTOR.saturating_mul(capacity).max(MIN_EPOCH),
			epoch_accesses: 0,
			current: INITIAL_AGING * MINI_WINDOW_FRACTIONS.len() + INITIAL_CONFIG,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u8>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		// Sum over the per-aging-period sketches (all sized identically at construction).
		self.sketches.iter().map(CountMinSketch::heap_bytes).sum()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.record_access(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.pending_miss = None;
				self.policy.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.pending_miss = Some(hash);
				None
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_of(&key);
		// A miss is reported as `get` followed by `put`: count it as one access.
		if self.pending_miss.take() != Some(hash) {
			self.record_access(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.policy.on_hit(idx);
			return;
		}
		let (idx, evicted) = self.policy.insert(hash, &self.sketches[self.aging]);
		let node = Node { key, value };
		if idx < self.nodes.len() {
			self.nodes[idx] = node;
		} else {
			self.nodes.push(node);
		}
		if evicted == Some(idx) {
			// The newcomer lost the admission contest; residency is unchanged.
			return;
		}
		if let Some(slot) = evicted {
			self.map.remove(&self.nodes[slot].key);
		}
		let key = self.nodes[idx].key.clone();
		self.map.insert(key, idx);
	}

	/// Counts one logical access: feeds the sketch and the miniature simulations and,
	/// at each epoch boundary, re-tunes the admission window from their hit scores.
	fn record_access(&mut self, hash: u64) {
		for sketch in self.sketches.iter_mut() {
			sketch.increment(hash);
		}
		if self.sample_rate == 1 || mix64(hash ^ SAMPLE_SEED) % self.sample_rate == 0 {
			let sketches = &self.sketches;
			for mini in self.minis.iter_mut() {
				let aging = mini.aging;
				mini.access(hash, &sketches[aging]);
			}
		}
		self.epoch_accesses += 1;
		if self.epoch_accesses >= self.epoch_len {
			self.epoch_accesses = 0;
			self.adapt();
		}
	}

	/// Adopt the window share of the best-scoring miniature simulation.
	fn adapt(&mut self) {
		for mini in self.minis.iter_mut() {
			mini.score = mini.score * SCORE_DECAY + mini.hits as f64;
			mini.hits = 0;
		}
		let mut best = self.current;
		for (i, mini) in self.minis.iter().enumerate() {
			if mini.score > self.minis[best].score {
				best = i;
			}
		}
		if best != self.current
			&& self.minis[best].score > self.minis[self.current].score * (1.0 + SWITCH_MARGIN)
		{
			self.current = best;
			// Every sketch has seen the full stream, so switching aging period is free.
			self.aging = self.minis[best].aging;
		}
		let fraction = self.refined_fraction(self.current);
		let window = (self.capacity as f64 * fraction).round() as usize;
		if window != self.policy.window_cap {
			self.policy.set_window(window);
		}
	}

	/// Window share for configuration `config`, refined by a parabola through the scores
	/// of its neighbouring window shares (same aging period), so the main cache is not
	/// limited to the coarse grid of simulated shares. The vertex is clamped to half a grid
	/// step and only used when the three scores form a peak.
	fn refined_fraction(&self, config: usize) -> f64 {
		let n = MINI_WINDOW_FRACTIONS.len();
		let j = config % n;
		let f = MINI_WINDOW_FRACTIONS[j];
		if j == 0 || j + 1 == n {
			return f;
		}
		let left = self.minis[config - 1].score;
		let mid = self.minis[config].score;
		let right = self.minis[config + 1].score;
		let curvature = left - 2.0 * mid + right;
		if !(curvature < 0.0) {
			return f;
		}
		let offset = ((left - right) / (2.0 * curvature)).clamp(-0.5, 0.5);
		let refined = if offset >= 0.0 {
			f + offset * (MINI_WINDOW_FRACTIONS[j + 1] - f)
		} else {
			f + offset * (f - MINI_WINDOW_FRACTIONS[j - 1])
		};
		refined.clamp(0.0, 1.0)
	}
}

#[derive(Debug, Clone, Copy)]
struct Link {
	prev: usize,
	next: usize,
	segment: u8,
}

/// One intrusive LRU list (head = LRU, tail = MRU) threaded through the arena.
#[derive(Debug, Clone, Copy)]
struct Segment {
	head: usize,
	tail: usize,
	len: usize,
}

impl Segment {
	const EMPTY: Segment = Segment {
		head: NIL,
		tail: NIL,
		len: 0,
	};
}

/// W-TinyLFU residency over arena slots: an admission-window LRU in front of a
/// segmented-LRU main space (probation + protected). Window overflow contests the
/// probation victim through the frequency sketch.
#[derive(Debug)]
struct Policy {
	links: Vec<Link>,
	hashes: Vec<u64>,
	free: Vec<usize>,
	lists: [Segment; 3],
	len: usize,
	capacity: usize,
	window_cap: usize,
	protected_cap: usize,
	rng: u64,
}

impl Policy {
	fn new(capacity: usize, window_cap: usize, seed: u64) -> Self {
		let mut policy = Self {
			links: Vec::with_capacity(capacity + 1),
			hashes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [Segment::EMPTY; 3],
			len: 0,
			capacity,
			window_cap: 0,
			protected_cap: 0,
			rng: seed | 1,
		};
		policy.set_window(window_cap);
		policy
	}

	/// Resize the admission window (main space takes the rest); entries are moved
	/// between segments, never evicted.
	fn set_window(&mut self, window_cap: usize) {
		self.window_cap = window_cap.min(self.capacity);
		self.protected_cap =
			((self.capacity - self.window_cap) as f64 * PROTECTED_FRACTION) as usize;
		while self.lists[WINDOW as usize].len > self.window_cap {
			let head = self.lists[WINDOW as usize].head;
			self.move_to(head, PROBATION);
		}
		self.demote_protected_overflow();
		while self.lists[WINDOW as usize].len < self.window_cap {
			let mut idx = self.lists[PROBATION as usize].head;
			if idx == NIL {
				idx = self.lists[PROTECTED as usize].head;
			}
			if idx == NIL {
				break;
			}
			self.move_to(idx, WINDOW);
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.lists[PROTECTED as usize].len > self.protected_cap {
			let head = self.lists[PROTECTED as usize].head;
			self.move_to(head, PROBATION);
		}
	}

	fn on_hit(&mut self, idx: usize) {
		let seg = self.links[idx].segment;
		if seg == PROBATION {
			self.move_to(idx, PROTECTED);
			self.demote_protected_overflow();
		} else {
			self.move_to(idx, seg);
		}
	}

	/// Insert `hash` at the window MRU and restore the capacity bound. Returns the new
	/// slot and the slot evicted to make room (possibly the newcomer itself).
	fn insert(&mut self, hash: u64, sketch: &CountMinSketch) -> (usize, Option<usize>) {
		let idx = match self.free.pop() {
			Some(slot) => {
				self.hashes[slot] = hash;
				slot
			}
			None => {
				self.links.push(Link {
					prev: NIL,
					next: NIL,
					segment: WINDOW,
				});
				self.hashes.push(hash);
				self.links.len() - 1
			}
		};
		self.push_back(idx, WINDOW);
		self.len += 1;
		let mut candidate = NIL;
		while self.lists[WINDOW as usize].len > self.window_cap {
			let head = self.lists[WINDOW as usize].head;
			self.move_to(head, PROBATION);
			candidate = head;
		}
		if self.len <= self.capacity {
			return (idx, None);
		}
		let victim = [PROBATION, PROTECTED, WINDOW]
			.iter()
			.map(|&seg| self.lists[seg as usize].head)
			.find(|&head| head != NIL && head != candidate)
			.unwrap_or(NIL);
		let evicted = if victim == NIL {
			candidate
		} else if candidate == NIL || self.admit(candidate, victim, sketch) {
			victim
		} else {
			candidate
		};
		self.unlink(evicted);
		self.free.push(evicted);
		self.len -= 1;
		(idx, Some(evicted))
	}

	fn admit(&mut self, candidate: usize, victim: usize, sketch: &CountMinSketch) -> bool {
		let candidate_freq = sketch.estimate(self.hashes[candidate]);
		let victim_freq = sketch.estimate(self.hashes[victim]);
		if candidate_freq > victim_freq {
			true
		} else if candidate_freq >= sketch.warm {
			(self.next_random() & 127) == 0
		} else {
			false
		}
	}

	fn next_random(&mut self) -> u64 {
		let mut x = self.rng;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.rng = x;
		x
	}

	fn move_to(&mut self, idx: usize, seg: u8) {
		self.unlink(idx);
		self.push_back(idx, seg);
	}

	fn unlink(&mut self, idx: usize) {
		let Link { prev, next, segment } = self.links[idx];
		let list = segment as usize;
		if prev != NIL {
			self.links[prev].next = next;
		} else {
			self.lists[list].head = next;
		}
		if next != NIL {
			self.links[next].prev = prev;
		} else {
			self.lists[list].tail = prev;
		}
		self.lists[list].len -= 1;
		self.links[idx].prev = NIL;
		self.links[idx].next = NIL;
	}

	fn push_back(&mut self, idx: usize, seg: u8) {
		let list = seg as usize;
		let tail = self.lists[list].tail;
		self.links[idx] = Link {
			prev: tail,
			next: NIL,
			segment: seg,
		};
		if tail != NIL {
			self.links[tail].next = idx;
		} else {
			self.lists[list].head = idx;
		}
		self.lists[list].tail = idx;
		self.lists[list].len += 1;
	}
}

/// Hasher for keys that are already well-mixed 64-bit hashes.
#[derive(Debug, Default, Clone, Copy)]
struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
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

type HashIndex = HashMap<u64, usize, BuildHasherDefault<IdentityHasher>>;

/// A miniature W-TinyLFU simulation over a hash-sampled slice of the key space with a
/// fixed window share. It shares the main cache's sketch (frequencies depend only on
/// the access stream, not on residency).
#[derive(Debug)]
struct Mini {
	policy: Policy,
	map: HashIndex,
	fraction: f64,
	/// Index into `AGING_FACTORS` of the sketch this simulation admits through.
	aging: usize,
	hits: u64,
	score: f64,
}

impl Mini {
	fn new(capacity: usize, fraction: f64, aging: usize, seed: u64) -> Self {
		let window = (capacity as f64 * fraction).round() as usize;
		Self {
			policy: Policy::new(capacity, window, seed),
			map: HashIndex::with_capacity_and_hasher(capacity + 1, BuildHasherDefault::default()),
			fraction,
			aging,
			hits: 0,
			score: 0.0,
		}
	}

	fn access(&mut self, hash: u64, sketch: &CountMinSketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.hits += 1;
			self.policy.on_hit(idx);
			return;
		}
		let (idx, evicted) = self.policy.insert(hash, sketch);
		self.map.insert(hash, idx);
		if let Some(slot) = evicted {
			let victim = self.policy.hashes[slot];
			self.map.remove(&victim);
		}
	}
}

/// Count-min sketch of 4-bit counters with conservative update and periodic halving.
#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	counts: Vec<u8>,
	additions: usize,
	sample_size: usize,
	/// Saturation ceiling of every counter (TinyLFU's W/C cap, at least `MAX_COUNT`).
	max_count: u8,
	/// Estimate from which a losing candidate still gets the 1/128 random admission.
	warm: u8,
}

impl CountMinSketch {
	/// A sketch that halves its counters after `aging_factor * capacity` additions.
	fn new(width: usize, aging_factor: usize, capacity: usize) -> Self {
		let max_count = aging_factor.clamp(MAX_COUNT as usize, u8::MAX as usize);
		let warm = (ADMIT_RANDOM_THRESHOLD as usize * max_count / MAX_COUNT as usize).max(1);
		Self {
			width,
			counts: vec![0; width * SKETCH_DEPTH],
			additions: 0,
			sample_size: aging_factor.saturating_mul(capacity).max(1),
			max_count: max_count as u8,
			warm: warm as u8,
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.counts.len() * std::mem::size_of::<u8>()
	}

	fn index(&self, hash: u64, row: usize) -> usize {
		row * self.width + ((mix64(hash ^ SEEDS[row]) as usize) & (self.width - 1))
	}

	/// Conservative update: raise only the counters currently holding the minimum.
	fn increment(&mut self, hash: u64) {
		let mut slots = [0usize; SKETCH_DEPTH];
		let mut min = self.max_count;
		for (row, slot) in slots.iter_mut().enumerate() {
			*slot = self.index(hash, row);
			min = min.min(self.counts[*slot]);
		}
		if min >= self.max_count {
			return;
		}
		for &slot in slots.iter() {
			if self.counts[slot] == min {
				self.counts[slot] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	fn estimate(&self, hash: u64) -> u8 {
		(0..SKETCH_DEPTH)
			.map(|row| self.counts[self.index(hash, row)])
			.min()
			.unwrap_or(0)
	}

	/// Age the history: halve every counter so stale popularity decays.
	fn reset(&mut self) {
		for c in self.counts.iter_mut() {
			*c >>= 1;
		}
		self.additions /= 2;
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
}
