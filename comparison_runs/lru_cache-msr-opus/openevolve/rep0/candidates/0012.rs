// Adaptive W-TinyLFU cache (Caffeine-style), O(1) per operation, std only, no unsafe.
//
// Layout: HashMap<K, slot> + arena of nodes threaded onto three intrusive LRU lists:
//   * Window    - small LRU that admits every new key (captures recency bursts).
//   * Probation - main-region segment for keys that left the window or were demoted.
//   * Protected - main-region segment for keys re-referenced while in probation.
// When a new key arrives at full capacity, the window's LRU entry (candidate) competes
// with the probation LRU entry (victim): a count-min sketch with 4-bit counters and
// periodic halving (TinyLFU) estimates their popularity and the rarer one is evicted.
// The window size and the sketch's aging period are tuned by "mini-simulations": scaled-down
// copies of this very policy replay a hash-sampled subset of the requests. One group fixes
// the window share (at the current aging period), another fixes the aging period (at the
// current window); the settings whose copies recently missed least are applied to the real
// cache (coordinate-wise search). Arena slots are reused in place on eviction.
//
// Tiny caches (capacity < ADAPTIVE_MIN_CAPACITY) have too few slots for meaningful
// frequency statistics; there the window spans the whole cache, which is exactly LRU.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

/// Null link in the arena-based lists.
const NIL: usize = usize::MAX;
/// Below this capacity the cache runs as a plain LRU (window == whole cache).
const ADAPTIVE_MIN_CAPACITY: usize = 16;
/// Share of the main region reserved for the protected segment.
const PROTECTED_PERCENT: f64 = 0.80;
/// Sketch aging periods (sample size per unit of capacity) evaluated by the tuner.
const SAMPLE_MULTIPLIERS: [usize; 4] = [5, 10, 20, 40];
/// Index of the initial aging period (Caffeine's default of 10x capacity).
const DEFAULT_MULTIPLIER: usize = 1;
/// Initial sketch sample size per unit of capacity.
const SAMPLE_MULTIPLIER: usize = SAMPLE_MULTIPLIERS[DEFAULT_MULTIPLIER];
/// Window shares (percent of capacity) evaluated by the mini-simulations, ascending; the
/// first one is also the initial window of an adaptive cache.
const WINDOW_PERCENTS: [usize; 14] = [1, 3, 5, 10, 15, 20, 30, 40, 50, 60, 70, 80, 90, 100];
/// Keys are sampled so that each mini-simulation models roughly this many entries.
const MINI_TARGET_CAPACITY: usize = 1_000;
/// Minimum number of sampled requests between two window decisions.
const MINI_MIN_PERIOD: usize = 128;
/// Per-period decay applied to the mini-simulations' accumulated miss scores.
const MINI_SCORE_DECAY: f64 = 0.8;

/// List identifiers (indices into `LruCache::lists`).
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,    // sketch hash of `key`, cached for admission decisions
	queue: usize, // WINDOW, PROBATION or PROTECTED
	prev: usize,
	next: usize,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize, // LRU end
	tail: usize, // MRU end
	len: usize,
}

impl List {
	const EMPTY: List = List {
		head: NIL,
		tail: NIL,
		len: 0,
	};
}

/// Count-min sketch with 4-bit counters (16 per u64 word) and periodic halving ("aging").
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u64>,
	mask: u64,
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	const SEEDS: [u64; 4] = [
		0xc3a5_c85c_97cb_3127,
		0xb492_b66f_be98_f273,
		0x9ae1_6a3b_2f90_404f,
		0xcbf2_9ce4_8422_2325,
	];
	const RESET_MASK: u64 = 0x7777_7777_7777_7777;
	const ONE_MASK: u64 = 0x1111_1111_1111_1111;

	fn new(capacity: usize) -> Self {
		let width = capacity.clamp(8, 1 << 26).next_power_of_two();
		Self {
			table: vec![0; width],
			mask: (width - 1) as u64,
			additions: 0,
			sample_size: capacity.saturating_mul(SAMPLE_MULTIPLIER).max(16),
		}
	}

	/// (word index, bit shift) of the counter for `hash` in row `i`.
	#[inline]
	fn slot(&self, hash: u64, i: usize) -> (usize, u64) {
		let seed = Self::SEEDS[i];
		let mut h = hash.wrapping_add(seed).wrapping_mul(seed);
		h = h.wrapping_add(h >> 32);
		let index = (h & self.mask) as usize;
		let counter = ((hash & 3) << 2) + i as u64; // 0..16 within the word
		(index, counter << 2)
	}

	fn frequency(&self, hash: u64) -> u64 {
		(0..4)
			.map(|i| {
				let (index, shift) = self.slot(hash, i);
				(self.table[index] >> shift) & 0xF
			})
			.min()
			.unwrap_or(0)
	}

	/// Conservative ("minimal") increment: only the counters holding the key's current
	/// estimate are raised, so collisions inflate other keys' estimates far less.
	fn increment(&mut self, hash: u64) {
		let mut slots = [(0_usize, 0_u64); 4];
		let mut min = 0xF_u64;
		for (i, slot) in slots.iter_mut().enumerate() {
			*slot = self.slot(hash, i);
			min = min.min((self.table[slot.0] >> slot.1) & 0xF);
		}
		if min == 0xF {
			return; // saturated
		}
		for &(index, shift) in slots.iter() {
			if (self.table[index] >> shift) & 0xF == min {
				self.table[index] += 1_u64 << shift;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			self.reset();
		}
	}

	/// Changes the aging period; an overdue sketch ages once now rather than repeatedly.
	fn set_sample_size(&mut self, sample_size: usize) {
		let sample_size = sample_size.max(16);
		if sample_size == self.sample_size {
			return;
		}
		self.sample_size = sample_size;
		if self.additions >= sample_size {
			self.reset();
			self.additions = self.additions.min(sample_size / 2);
		}
	}

	/// Halves every counter so that stale popularity decays over time.
	fn reset(&mut self) {
		let mut odd = 0usize;
		for word in self.table.iter_mut() {
			odd += (*word & Self::ONE_MASK).count_ones() as usize;
			*word = (*word >> 1) & Self::RESET_MASK;
		}
		self.additions = self.additions.saturating_sub(odd >> 2) >> 1;
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	lists: [List; 3],       // WINDOW, PROBATION, PROTECTED
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: FrequencySketch,
	last_miss: Option<u64>,    // hash of the key the last get() missed
	sim: Option<Box<MiniSim>>, // window tuner; None for tiny caches and for the minis
}

/// Window size for `percent` of `capacity`: at least one slot, at most the whole cache.
fn window_for(capacity: usize, percent: usize) -> usize {
	(capacity / 100 * percent + capacity % 100 * percent / 100).clamp(1, capacity)
}

/// Protected budget for a given window size: PROTECTED_PERCENT of the main region.
fn protected_for(capacity: usize, window: usize) -> usize {
	((capacity - window) as f64 * PROTECTED_PERCENT) as usize
}

/// Index of the lowest score; `current` is kept unless another score is strictly lower.
fn argmin_from(scores: &[f64], current: usize) -> usize {
	let mut best = current;
	for (i, &score) in scores.iter().enumerate() {
		if score < scores[best] {
			best = i;
		}
	}
	best
}

/// Policy tuner: scaled-down W-TinyLFU caches replay a hash-sampled subset of the requests.
/// The first `windows.len()` minis each fix one window share (all at the current aging
/// period); the remaining minis each fix one aging period (all at the current window).
/// Every period the best of each group (older periods decayed) is reported as the
/// (window, sample multiplier) the real cache should use, and fed back to the other group.
#[derive(Debug)]
struct MiniSim {
	minis: Vec<LruCache<u64, ()>>,
	windows: Vec<usize>,      // real-cache window size modelled by each window mini
	mini_windows: Vec<usize>, // the same window shares in mini-cache slots
	misses: Vec<u64>,         // misses per mini in the current period
	scores: Vec<f64>,         // decayed miss totals per mini
	rate: u64,                // a key is sampled when (hash >> 32) % rate == 0
	samples: usize,           // sampled requests in the current period
	period: usize,
	best_window: usize,     // index into `windows`
	best_multiplier: usize, // index into SAMPLE_MULTIPLIERS
}

impl MiniSim {
	fn new(capacity: usize) -> Self {
		let rate = (capacity / MINI_TARGET_CAPACITY).max(1);
		let mini_capacity = (capacity / rate).max(1);
		let total = WINDOW_PERCENTS.len() + SAMPLE_MULTIPLIERS.len();
		let mut minis = Vec::with_capacity(total);
		let mut windows: Vec<usize> = Vec::with_capacity(WINDOW_PERCENTS.len());
		let mut mini_windows: Vec<usize> = Vec::with_capacity(WINDOW_PERCENTS.len());
		for &percent in WINDOW_PERCENTS.iter() {
			let window = window_for(capacity, percent);
			if windows.last() == Some(&window) {
				continue;
			}
			windows.push(window);
			let mini_window = window_for(mini_capacity, percent);
			mini_windows.push(mini_window);
			minis.push(LruCache::<u64, ()>::with_window(mini_capacity, mini_window));
		}
		for &multiplier in SAMPLE_MULTIPLIERS.iter() {
			let mut mini = LruCache::<u64, ()>::with_window(mini_capacity, mini_windows[0]);
			mini.set_sample_multiplier(multiplier);
			minis.push(mini);
		}
		let count = minis.len();
		Self {
			minis,
			windows,
			mini_windows,
			misses: vec![0; count],
			scores: vec![0.0; count],
			rate: rate as u64,
			samples: 0,
			period: mini_capacity.max(MINI_MIN_PERIOD),
			best_window: 0,
			best_multiplier: DEFAULT_MULTIPLIER,
		}
	}

	/// Replays one request (if its key is sampled) on every mini cache, emulating a get
	/// followed by a put on miss. Returns the preferred (window size, sample multiplier)
	/// when a period ends.
	fn record(&mut self, hash: u64) -> Option<(usize, usize)> {
		if (hash >> 32) % self.rate != 0 {
			return None;
		}
		for (mini, misses) in self.minis.iter_mut().zip(self.misses.iter_mut()) {
			if mini.get(&hash).is_none() {
				mini.put(hash, ());
				*misses += 1;
			}
		}
		self.samples += 1;
		if self.samples < self.period {
			return None;
		}
		self.samples = 0;
		for i in 0..self.minis.len() {
			self.scores[i] = self.scores[i] * MINI_SCORE_DECAY + self.misses[i] as f64;
			self.misses[i] = 0;
		}
		let split = self.windows.len();
		let best_window = argmin_from(&self.scores[..split], self.best_window);
		let best_multiplier = argmin_from(&self.scores[split..], self.best_multiplier);
		if best_window != self.best_window {
			self.best_window = best_window;
			let mini_window = self.mini_windows[best_window];
			for mini in self.minis[split..].iter_mut() {
				mini.set_window(mini_window);
			}
		}
		if best_multiplier != self.best_multiplier {
			self.best_multiplier = best_multiplier;
			let multiplier = SAMPLE_MULTIPLIERS[best_multiplier];
			for mini in self.minis[..split].iter_mut() {
				mini.set_sample_multiplier(multiplier);
			}
		}
		Some((
			self.windows[best_window],
			SAMPLE_MULTIPLIERS[best_multiplier],
		))
	}
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
		let adaptive = capacity >= ADAPTIVE_MIN_CAPACITY;
		let window_max = if adaptive {
			window_for(capacity, WINDOW_PERCENTS[0])
		} else {
			capacity
		};
		let mut cache = Self::with_window(capacity, window_max);
		if adaptive {
			cache.sim = Some(Box::new(MiniSim::new(capacity)));
		}
		let protected_max = cache.protected_max;
		log::info!(
			"LruCache.try_new: capacity={capacity} window={window_max} protected={protected_max}"
		);
		Some(cache)
	}

	/// A cache with a fixed window/main split and no window tuner (also used for the minis).
	fn with_window(capacity: usize, window_max: usize) -> Self {
		let window_max = window_max.clamp(1, capacity);
		Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::EMPTY; 3],
			capacity,
			window_max,
			protected_max: protected_for(capacity, window_max),
			sketch: FrequencySketch::new(capacity),
			last_miss: None,
			sim: None,
		}
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. Every lookup (hit or miss) is recorded in
	/// the frequency sketch; a hit also refreshes the entry's position in its segment.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				let hash = self.nodes[idx].hash;
				self.sketch.increment(hash);
				self.last_miss = None;
				self.on_access(idx);
				self.observe(hash);
				Some(&self.nodes[idx].value)
			}
			None => {
				let hash = Self::hash_of(key);
				self.sketch.increment(hash);
				self.last_miss = Some(hash);
				self.observe(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys always enter the window; at capacity the window's
	/// LRU entry must out-score the probation LRU entry (TinyLFU) to stay in the cache.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in place and refresh its position.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let hash = Self::hash_of(&key);
		// A get-miss followed by a put of the same key is one logical access: count it once.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
			self.observe(hash);
		}
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: the policy frees one slot, which is reused for the new entry.
			let slot = self.evict();
			let node = &mut self.nodes[slot];
			node.key = key.clone();
			node.value = value;
			node.hash = hash;
			slot
		} else {
			self.nodes.push(Node {
				key: key.clone(),
				value,
				hash,
				queue: WINDOW,
				prev: NIL,
				next: NIL,
			});
			self.nodes.len() - 1
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		// While the cache is still filling, window overflow simply spills into probation.
		if self.lists[WINDOW].len > self.window_max {
			let spill = self.lists[WINDOW].head;
			self.move_to(spill, PROBATION);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Frees one arena slot at full capacity and returns it (unlinked and unmapped).
	fn evict(&mut self) -> usize {
		// The incoming key needs a window slot: a full window hands its LRU entry to the
		// probation MRU end, where it becomes the admission candidate.
		let mut candidate = NIL;
		if self.lists[WINDOW].len >= self.window_max && self.lists[WINDOW].head != NIL {
			candidate = self.lists[WINDOW].head;
			self.move_to(candidate, PROBATION);
		}
		let victim = [PROBATION, PROTECTED, WINDOW]
			.iter()
			.map(|&q| self.lists[q].head)
			.find(|&head| head != NIL)
			.expect("a full cache has an entry to evict");
		let evicted = if candidate == NIL || candidate == victim {
			victim
		} else if self.sketch.frequency(self.nodes[candidate].hash)
			> self.sketch.frequency(self.nodes[victim].hash)
		{
			victim // candidate is more popular: admit it
		} else {
			candidate
		};
		self.unlink(evicted);
		self.map.remove(&self.nodes[evicted].key);
		evicted
	}

	/// Policy reaction to a hit: probation entries graduate to protected, others refresh.
	fn on_access(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		if queue == PROBATION {
			self.move_to(idx, PROTECTED);
			self.demote_protected_overflow();
		} else {
			self.move_to(idx, queue);
		}
	}

	/// Demote protected LRU entries back to probation until the segment fits its budget.
	fn demote_protected_overflow(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let lru = self.lists[PROTECTED].head;
			self.move_to(lru, PROBATION);
		}
	}

	/// Feeds one logical request to the policy tuner and applies the window size and
	/// sketch aging period it settles on.
	fn observe(&mut self, hash: u64) {
		let target = self.sim.as_mut().and_then(|sim| sim.record(hash));
		if let Some((window, multiplier)) = target {
			if window != self.window_max {
				self.set_window(window);
			}
			self.set_sample_multiplier(multiplier);
		}
	}

	/// Sets the sketch's aging period to `multiplier` times the capacity.
	fn set_sample_multiplier(&mut self, multiplier: usize) {
		self.sketch
			.set_sample_size(self.capacity.saturating_mul(multiplier));
	}

	/// Resize the window to `window` slots (clamped to 1..=capacity); the main region gets
	/// the rest, PROTECTED_PERCENT of it protected. Overflowing entries move to probation;
	/// a grown window fills with new arrivals while the main region shrinks via eviction.
	fn set_window(&mut self, window: usize) {
		self.window_max = window.clamp(1, self.capacity);
		self.protected_max = protected_for(self.capacity, self.window_max);
		self.demote_protected_overflow();
		while self.lists[WINDOW].len > self.window_max {
			let lru = self.lists[WINDOW].head;
			self.move_to(lru, PROBATION);
		}
	}

	#[cfg(test)]
	fn increase_window(&mut self, amount: usize) {
		self.set_window(self.window_max.saturating_add(amount));
	}

	#[cfg(test)]
	fn decrease_window(&mut self, amount: usize) {
		self.set_window(self.window_max.saturating_sub(amount));
	}

	/// Deterministic 64-bit hash used by the frequency sketch.
	fn hash_of(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Move a node to the MRU end of `queue` (which may be its current list).
	fn move_to(&mut self, idx: usize, queue: usize) {
		self.unlink(idx);
		self.push_back(queue, idx);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, queue) = {
			let node = &self.nodes[idx];
			(node.prev, node.next, node.queue)
		};
		if prev == NIL {
			self.lists[queue].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.lists[queue].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lists[queue].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the tail (MRU end) of `queue`.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.lists[queue].tail;
		{
			let node = &mut self.nodes[idx];
			node.queue = queue;
			node.prev = tail;
			node.next = NIL;
		}
		if tail == NIL {
			self.lists[queue].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.lists[queue].tail = idx;
		self.lists[queue].len += 1;
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

	/// Structural invariants of the three intrusive lists and the size budgets.
	fn assert_consistent(cache: &LruCache<u64, u64>) {
		assert!(cache.map.len() <= cache.capacity);
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.window_max >= 1);
		assert!(cache.window_max + cache.protected_max <= cache.capacity);
		assert!(cache.lists[WINDOW].len <= cache.window_max);
		assert!(cache.lists[PROTECTED].len <= cache.protected_max);
		let mut total = 0;
		for (queue, list) in cache.lists.iter().enumerate() {
			let (mut idx, mut prev, mut len) = (list.head, NIL, 0);
			while idx != NIL {
				assert_eq!(cache.nodes[idx].queue, queue);
				assert_eq!(cache.nodes[idx].prev, prev);
				prev = idx;
				idx = cache.nodes[idx].next;
				len += 1;
			}
			assert_eq!(prev, list.tail);
			assert_eq!(len, list.len);
			total += len;
		}
		assert_eq!(total, cache.map.len());
		for (key, &idx) in &cache.map {
			assert_eq!(cache.nodes[idx].key, *key);
		}
	}

	/// Skewed pseudo-random workload driven like the simulator: get, then put on a miss.
	fn drive(cache: &mut LruCache<u64, u64>, ops: usize, seed: u64) -> usize {
		let mut state = seed;
		let mut hits = 0;
		for i in 0..ops {
			state = state
				.wrapping_mul(6364136223846793005)
				.wrapping_add(1442695040888963407);
			let r = state >> 33;
			let key = if (r >> 24) & 1 == 0 {
				r % 64
			} else {
				1_000 + r % 5_000
			};
			if cache.get(&key).is_some() {
				hits += 1;
			} else {
				cache.put(key, key * 2);
			}
			if i % 997 == 0 {
				assert_consistent(cache);
			}
		}
		hits
	}

	#[test]
	fn frequent_keys_survive_a_scan() {
		let mut cache = LruCache::<u64, u64>::new(100);
		for _ in 0..10 {
			for key in 0..50 {
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
			}
		}
		for key in 1_000..1_300 {
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
		}
		assert_consistent(&cache);
		let survivors = (0..50).filter(|key| cache.get(key).is_some()).count();
		assert!(survivors >= 45, "only {survivors} hot keys survived the scan");
	}

	#[test]
	fn newest_key_is_always_resident() {
		let mut cache = LruCache::<u64, u64>::new(64);
		for key in 0..2_000 {
			cache.put(key, key + 1);
			assert_eq!(cache.get(&key), Some(&(key + 1)));
		}
		assert_consistent(&cache);
	}

	#[test]
	fn adaptive_invariants_hold_under_churn() {
		for &capacity in &[16_usize, 100, 128, 1_000] {
			let mut cache = LruCache::<u64, u64>::new(capacity);
			let hits = drive(&mut cache, 60_000, capacity as u64);
			assert!(hits > 0);
			assert_consistent(&cache);
		}
	}

	#[test]
	fn window_resizing_preserves_invariants() {
		let mut cache = LruCache::<u64, u64>::new(200);
		drive(&mut cache, 5_000, 7);
		cache.increase_window(50);
		assert_consistent(&cache);
		drive(&mut cache, 5_000, 11);
		cache.decrease_window(1_000);
		assert_eq!(cache.window_max, 1);
		assert_consistent(&cache);
		drive(&mut cache, 5_000, 13);
		assert_consistent(&cache);
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
		// Should still have room for a second key without eviction
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 most recent
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
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
		cache.put(2, 20); // cache full
		cache.put(1, 11); // overwrite, must NOT evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		cache.put(4, 40); // should evict key 2 (LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to MRU
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts 3 (slot 0) -> slot 0 reused AGAIN for key 5
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify list is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; 3 is MRU (tail)
		cache.put(3, 31); // overwrite MRU
										// Eviction should take 1 (LRU)
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
