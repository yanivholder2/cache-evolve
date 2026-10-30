// W-TinyLFU cache: a small LRU admission window in front of a segmented-LRU main space
// (probation + protected), guarded by a 4-bit count-min frequency sketch (conservative
// update). The configuration (window size x probation share x admission slack) is tuned
// by racing key-less
// ghost caches, one per candidate configuration, over a spatially sampled copy of the
// access stream and periodically adopting the best recent performer. The admission slack
// spans strict TinyLFU, tie-admitting TinyLFU and unconditional admission (windowed
// SLRU); a window of 100% degenerates to plain LRU. Arena-backed intrusive lists,
// evicted slots reused in place. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

/// Sentinel "null" arena index.
const NIL: usize = usize::MAX;
/// Queue ids: admission window, main probation segment, main protected segment.
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

/// Candidate window sizes (fraction of capacity) raced by the ghost caches; 1.0 is LRU.
const WINDOW_FRACTIONS: [f64; 12] = [
	0.01, 0.02, 0.04, 0.07, 0.11, 0.17, 0.25, 0.37, 0.52, 0.70, 0.85, 1.0,
];
/// Candidate admission slacks raced jointly with the window sizes: the window's LRU
/// candidate enters the main space when `freq(candidate) + slack > freq(victim)`.
/// 0 is strict TinyLFU, 1 also admits ties (leaning to recency), 16 exceeds any 4-bit
/// count and so admits everything (a windowed SLRU).
const ADMIT_SLACKS: [u64; 3] = [0, 1, 16];
/// Candidate probation shares of the main space (percent), raced jointly with the window
/// size and admission slack: 20 is the classic 80% protected SLRU, 50 gives newly
/// admitted entries a longer trial (an ARC-like shift of the recency/frequency balance).
const PROBATION_PERCENTS: [usize; 2] = [20, 50];
/// Sketch aging period (sample size as a multiple of capacity): Caffeine's default.
const SKETCH_PERIOD: usize = 10;
/// Start with Caffeine's defaults: 1% window with strict TinyLFU admission (ghost 0).
const INITIAL_CHOICE: usize = 0;
/// Large caches are spatially sampled so that each ghost holds at most this many keys.
const MAX_GHOST_CAPACITY: usize = 4096;
/// Minimum number of sampled accesses between adaptation decisions.
const MIN_PERIOD: usize = 256;
/// Per-period decay of the ghosts' hit scores (memory of roughly ten periods).
const SCORE_DECAY: f64 = 0.9;
/// Relative score lead a ghost needs before the cache switches to its configuration.
const SWITCH_MARGIN: f64 = 0.002;

const SKETCH_SEEDS: [u64; 4] = [
	0xc3a5_c85c_97cb_3127,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
	0xcbf2_9ce4_8422_2325,
];
const RESET_MASK: u64 = 0x7777_7777_7777_7777;
const ONE_MASK: u64 = 0x1111_1111_1111_1111;

/// Count-min sketch of 4-bit counters (16 per word) with periodic halving (aging).
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u64>,
	mask: usize,
	size: usize,
	sample_size: usize,
}

impl FrequencySketch {
	/// `period` is the aging period as a multiple of capacity: after that many
	/// recorded increments every counter is halved.
	fn new(capacity: usize, period: usize) -> Self {
		let len = capacity.max(16).next_power_of_two();
		Self {
			table: vec![0; len],
			mask: len - 1,
			size: 0,
			sample_size: capacity.saturating_mul(period).max(16),
		}
	}

	/// Word index and bit shift of the `i`-th counter for `hash`.
	#[inline]
	fn slot(&self, hash: u64, i: usize) -> (usize, u32) {
		let mut h = hash.wrapping_add(SKETCH_SEEDS[i]).wrapping_mul(SKETCH_SEEDS[i]);
		h ^= h >> 32;
		let start = ((hash >> 62) as u32) << 2;
		((h as usize) & self.mask, (start + i as u32) << 2)
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

	/// Conservative update: only the counters holding the current minimum are
	/// incremented, curbing the collision-driven over-estimation of cold keys.
	fn increment(&mut self, hash: u64) {
		let slots = [
			self.slot(hash, 0),
			self.slot(hash, 1),
			self.slot(hash, 2),
			self.slot(hash, 3),
		];
		let min = slots
			.iter()
			.map(|&(index, shift)| (self.table[index] >> shift) & 0xF)
			.min()
			.unwrap_or(15);
		if min >= 15 {
			return;
		}
		for &(index, shift) in &slots {
			if ((self.table[index] >> shift) & 0xF) == min {
				self.table[index] += 1u64 << shift;
			}
		}
		self.size += 1;
		if self.size >= self.sample_size {
			self.reset();
		}
	}

	fn reset(&mut self) {
		let mut odd = 0usize;
		for word in self.table.iter_mut() {
			odd += (*word & ONE_MASK).count_ones() as usize;
			*word = (*word >> 1) & RESET_MASK;
		}
		self.size = self.size.saturating_sub(odd >> 2) >> 1;
	}
}

/// Intrusive doubly-linked list over the arena: head = LRU, tail = MRU.
#[derive(Debug, Clone, Copy)]
struct List {
	head: usize,
	tail: usize,
	len: usize,
}

impl List {
	const EMPTY: List = List { head: NIL, tail: NIL, len: 0 };
}

/// Per-slot policy metadata: key hash, recency links and owning queue.
#[derive(Debug, Clone, Copy)]
struct Link {
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

/// Key-agnostic W-TinyLFU policy over an arena of slots: an LRU admission window in
/// front of a segmented-LRU main space (probation + protected), with TinyLFU admission
/// guarding the main space. Evicted slots are reused in place, so the slot count equals
/// the resident count. Shared by the real cache and its ghost caches.
#[derive(Debug)]
struct Policy {
	links: Vec<Link>,
	lists: [List; 3], // window, probation, protected
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	admit_slack: u64,
	probation_percent: usize,
}

impl Policy {
	fn new(capacity: usize, fraction: f64, admit_slack: u64, probation_percent: usize) -> Self {
		let mut policy = Self {
			links: Vec::with_capacity(capacity),
			lists: [List::EMPTY; 3],
			capacity,
			window_max: 1,
			protected_max: 0,
			admit_slack,
			probation_percent,
		};
		policy.set_window(Self::window_for(capacity, fraction));
		policy
	}

	fn window_for(capacity: usize, fraction: f64) -> usize {
		((capacity as f64 * fraction).round() as usize).clamp(1, capacity)
	}

	/// Resize the admission window; the main space takes the rest, split between
	/// probation (`probation_percent`) and protected.
	fn set_window(&mut self, window_max: usize) {
		self.window_max = window_max.clamp(1, self.capacity);
		let main_max = self.capacity - self.window_max;
		self.protected_max = main_max - main_max * self.probation_percent / 100;
		self.trim_window();
		self.demote_protected();
		// A larger window is refilled from the main space's coldest entries, placed at
		// the window's LRU end (coldest first) so they promptly re-face admission.
		let main_len = self.lists[PROBATION].len + self.lists[PROTECTED].len;
		let count = (self.window_max - self.lists[WINDOW].len).min(main_len.saturating_sub(main_max));
		let mut moved = Vec::with_capacity(count);
		for _ in 0..count {
			let lru = self.main_lru();
			self.unlink(lru);
			moved.push(lru);
		}
		for &idx in moved.iter().rev() {
			self.push_front(idx, WINDOW);
		}
	}

	/// Window/protected hits move to their MRU end; a probation hit graduates to
	/// protected, demoting protected's LRU back to probation when over quota.
	fn on_hit(&mut self, idx: usize) {
		if self.links[idx].queue as usize == WINDOW {
			self.move_to(idx, WINDOW);
		} else {
			self.move_to(idx, PROTECTED);
			self.demote_protected();
		}
	}

	/// Places a new entry at the window's MRU end, evicting one entry first when full.
	/// Returns the entry's slot and, if an eviction freed that slot, the evicted hash.
	fn insert(&mut self, hash: u64, sketch: &FrequencySketch) -> (usize, Option<u64>) {
		let (idx, evicted) = if self.links.len() >= self.capacity {
			let idx = self.evict(sketch);
			let old = std::mem::replace(&mut self.links[idx].hash, hash);
			(idx, Some(old))
		} else {
			self.links.push(Link {
				hash,
				prev: NIL,
				next: NIL,
				queue: WINDOW as u8,
			});
			(self.links.len() - 1, None)
		};
		self.push_back(idx, WINDOW);
		self.trim_window();
		(idx, evicted)
	}

	fn main_lru(&self) -> usize {
		if self.lists[PROBATION].head != NIL {
			self.lists[PROBATION].head
		} else {
			self.lists[PROTECTED].head
		}
	}

	/// Unlinks exactly one resident entry and returns its slot. When the window is
	/// full its LRU entry must out-score the main space's LRU victim on estimated
	/// frequency (TinyLFU) to be admitted, otherwise it is the one evicted.
	fn evict(&mut self, sketch: &FrequencySketch) -> usize {
		let victim = self.main_lru();
		let window = self.lists[WINDOW];
		let evicted = if window.head != NIL && window.len >= self.window_max {
			let candidate = window.head;
			if victim != NIL
				&& (self.admit_slack > 15
					|| sketch.frequency(self.links[candidate].hash) + self.admit_slack
						> sketch.frequency(self.links[victim].hash))
			{
				self.move_to(candidate, PROBATION);
				victim
			} else {
				candidate
			}
		} else if victim != NIL {
			victim
		} else {
			window.head
		};
		self.unlink(evicted);
		evicted
	}

	/// Remove a slot from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let Link { prev, next, queue, .. } = self.links[idx];
		let q = queue as usize;
		if prev != NIL {
			self.links[prev].next = next;
		} else {
			self.lists[q].head = next;
		}
		if next != NIL {
			self.links[next].prev = prev;
		} else {
			self.lists[q].tail = prev;
		}
		self.lists[q].len -= 1;
		self.links[idx].prev = NIL;
		self.links[idx].next = NIL;
	}

	/// Append a slot to the MRU end (tail) of list `q`.
	fn push_back(&mut self, idx: usize, q: usize) {
		let tail = self.lists[q].tail;
		let link = &mut self.links[idx];
		link.prev = tail;
		link.next = NIL;
		link.queue = q as u8;
		if tail != NIL {
			self.links[tail].next = idx;
		} else {
			self.lists[q].head = idx;
		}
		self.lists[q].tail = idx;
		self.lists[q].len += 1;
	}

	/// Prepend a slot to the LRU end (head) of list `q`.
	fn push_front(&mut self, idx: usize, q: usize) {
		let head = self.lists[q].head;
		let link = &mut self.links[idx];
		link.prev = NIL;
		link.next = head;
		link.queue = q as u8;
		if head != NIL {
			self.links[head].prev = idx;
		} else {
			self.lists[q].tail = idx;
		}
		self.lists[q].head = idx;
		self.lists[q].len += 1;
	}

	fn move_to(&mut self, idx: usize, q: usize) {
		self.unlink(idx);
		self.push_back(idx, q);
	}

	fn trim_window(&mut self) {
		while self.lists[WINDOW].len > self.window_max {
			let lru = self.lists[WINDOW].head;
			self.move_to(lru, PROBATION);
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let lru = self.lists[PROTECTED].head;
			self.move_to(lru, PROBATION);
		}
	}

	#[cfg(test)]
	fn check_integrity(&self) {
		let mut total = 0;
		for q in 0..3 {
			let list = self.lists[q];
			let mut prev = NIL;
			let mut cur = list.head;
			let mut count = 0;
			while cur != NIL {
				let link = self.links[cur];
				assert_eq!(link.queue as usize, q);
				assert_eq!(link.prev, prev);
				prev = cur;
				cur = link.next;
				count += 1;
				assert!(count <= self.capacity, "cycle in list {q}");
			}
			assert_eq!(list.tail, prev);
			assert_eq!(list.len, count);
			total += count;
		}
		assert_eq!(total, self.links.len());
		assert!(self.links.len() <= self.capacity);
		assert!(self.lists[WINDOW].len <= self.window_max);
		assert!(self.lists[PROTECTED].len <= self.protected_max);
		assert!(self.window_max >= 1 && self.window_max <= self.capacity);
	}
}

/// Pass-through hasher for maps keyed by an already well-mixed 64-bit hash.
#[derive(Debug, Default)]
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

/// Key-less shadow cache with a fixed configuration (window size, admission slack):
/// replays the (sampled) access stream to measure how that configuration performs.
#[derive(Debug)]
struct Ghost {
	policy: Policy,
	index: HashIndex,
	fraction: f64,
	hits: u64,
	score: f64,
}

impl Ghost {
	fn new(capacity: usize, fraction: f64, admit_slack: u64, probation_percent: usize) -> Self {
		Self {
			policy: Policy::new(capacity, fraction, admit_slack, probation_percent),
			index: HashIndex::with_capacity_and_hasher(capacity, BuildHasherDefault::default()),
			fraction,
			hits: 0,
			score: 0.0,
		}
	}

	fn access(&mut self, hash: u64, sketch: &FrequencySketch) {
		if let Some(&idx) = self.index.get(&hash) {
			self.hits += 1;
			self.policy.on_hit(idx);
		} else {
			let (idx, evicted) = self.policy.insert(hash, sketch);
			if let Some(old) = evicted {
				self.index.remove(&old);
			}
			self.index.insert(hash, idx);
		}
	}
}

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> slot
	nodes: Vec<Node<K, V>>, // slot -> entry, aligned with the policy's slots
	policy: Policy,
	sketch: FrequencySketch,
	ghosts: Vec<Ghost>,             // one per candidate (window size x admission slack)
	choice: usize,                  // ghost whose configuration the cache runs
	sample_shift: u32,
	sample_period: u64,
	sampled: u64,
	last_get_hash: Option<u64>,
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
		let mut sample_shift = 0u32;
		while (capacity >> sample_shift) > MAX_GHOST_CAPACITY {
			sample_shift += 1;
		}
		let ghost_capacity = capacity >> sample_shift;
		let mut ghosts = Vec::with_capacity(
			WINDOW_FRACTIONS.len() * PROBATION_PERCENTS.len() * ADMIT_SLACKS.len(),
		);
		for &fraction in WINDOW_FRACTIONS.iter() {
			if fraction >= 1.0 {
				// A 100% window never admits into a main space (plain LRU): race it once.
				ghosts.push(Ghost::new(
					ghost_capacity,
					fraction,
					ADMIT_SLACKS[0],
					PROBATION_PERCENTS[0],
				));
				continue;
			}
			for &percent in PROBATION_PERCENTS.iter() {
				for &slack in ADMIT_SLACKS.iter() {
					ghosts.push(Ghost::new(ghost_capacity, fraction, slack, percent));
				}
			}
		}
		let initial = &ghosts[INITIAL_CHOICE];
		let policy = Policy::new(
			capacity,
			initial.fraction,
			initial.policy.admit_slack,
			initial.policy.probation_percent,
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			policy,
			sketch: FrequencySketch::new(capacity, SKETCH_PERIOD),
			ghosts,
			choice: INITIAL_CHOICE,
			sample_shift,
			sample_period: ghost_capacity.max(MIN_PERIOD) as u64,
			sampled: 0,
			last_get_hash: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. Every lookup is recorded in the
	/// frequency sketches; a hit is promoted within its segment (probation -> protected).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = Self::hash_key(key);
		self.record_access(hash);
		self.last_get_hash = Some(hash);
		let idx = *self.map.get(key)?;
		self.policy.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	fn hash_key(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Records one access in every frequency sketch and replays it through the ghost
	/// caches (spatially sampled for large capacities); every period the configuration
	/// of the best recently performing ghost is adopted.
	fn record_access(&mut self, hash: u64) {
		self.sketch.increment(hash);
		if !self.is_sampled(hash) {
			return;
		}
		for ghost in &mut self.ghosts {
			ghost.access(hash, &self.sketch);
		}
		self.sampled += 1;
		if self.sampled % self.sample_period == 0 {
			self.adapt();
		}
	}

	fn is_sampled(&self, hash: u64) -> bool {
		if self.sample_shift == 0 {
			return true;
		}
		let mixed = (hash ^ (hash >> 29)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
		(mixed >> (64 - self.sample_shift)) == 0
	}

	fn adapt(&mut self) {
		for ghost in &mut self.ghosts {
			ghost.score = ghost.score * SCORE_DECAY + ghost.hits as f64;
			ghost.hits = 0;
		}
		let current = self.ghosts[self.choice].score;
		let mut best = self.choice;
		for (i, ghost) in self.ghosts.iter().enumerate() {
			if ghost.score > self.ghosts[best].score {
				best = i;
			}
		}
		if best != self.choice && self.ghosts[best].score > current * (1.0 + SWITCH_MARGIN) {
			self.choice = best;
			let (fraction, slack, percent) = {
				let ghost = &self.ghosts[best];
				(ghost.fraction, ghost.policy.admit_slack, ghost.policy.probation_percent)
			};
			self.policy.admit_slack = slack;
			let window = Policy::window_for(self.policy.capacity, fraction);
			if window != self.policy.window_max || percent != self.policy.probation_percent {
				self.policy.probation_percent = percent;
				self.policy.set_window(window);
			}
		}
	}

	/// Inserts or updates `key`. A new key enters the admission window; when the cache
	/// is full the policy evicts exactly one entry (TinyLFU duel between the window's
	/// LRU candidate and the main space's LRU victim) and its slot is reused.
	pub fn put(&mut self, key: K, value: V) {
		let hash = Self::hash_key(&key);
		// A miss is normally `get` then `put` of the same key: count the access once.
		if self.last_get_hash.take() != Some(hash) {
			self.record_access(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.policy.on_hit(idx);
			return;
		}
		let (idx, evicted) = self.policy.insert(hash, &self.sketch);
		if evicted.is_some() {
			let node = &mut self.nodes[idx];
			self.map.remove(&node.key);
			node.key = key.clone();
			node.value = value;
		} else {
			debug_assert_eq!(idx, self.nodes.len());
			self.nodes.push(Node {
				key: key.clone(),
				value,
			});
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.policy.capacity);
	}

	#[cfg(test)]
	fn check_integrity(&self) {
		self.policy.check_integrity();
		assert_eq!(self.nodes.len(), self.policy.links.len());
		assert_eq!(self.map.len(), self.nodes.len());
		for (key, &idx) in &self.map {
			assert_eq!(&self.nodes[idx].key, key);
			assert_eq!(self.policy.links[idx].hash, Self::hash_key(key));
		}
		assert!(self.choice < self.ghosts.len());
		assert_eq!(self.policy.admit_slack, self.ghosts[self.choice].policy.admit_slack);
		assert_eq!(
			self.policy.probation_percent,
			self.ghosts[self.choice].policy.probation_percent
		);
		for ghost in &self.ghosts {
			ghost.policy.check_integrity();
			assert_eq!(ghost.index.len(), ghost.policy.links.len());
			for (&hash, &idx) in &ghost.index {
				assert_eq!(ghost.policy.links[idx].hash, hash);
			}
		}
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
	fn newest_key_is_always_resident() {
		let mut cache = LruCache::new(3);
		for k in 0..200 {
			cache.put(k, k * 10);
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
			cache.check_integrity();
		}
	}

	#[test]
	fn frequent_key_survives_scan() {
		// Plain LRU would lose key 1 here; TinyLFU admission keeps it.
		let mut cache = LruCache::new(100);
		for _ in 0..30 {
			if cache.get(&1).is_none() {
				cache.put(1, 1);
			}
		}
		for k in 1000..1300 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&1).unwrap(), 1);
	}

	#[test]
	fn structure_stays_consistent_under_mixed_workload() {
		let mut cache = LruCache::new(64);
		let mut state: u64 = 0x2545_f491_4f6c_dd1d;
		for i in 0..40_000u64 {
			state = state
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let r = state >> 33;
			// Alternate hot-set and long-tail phases so the climber moves both ways.
			let key = if (i / 5_000) % 2 == 0 && r % 4 != 0 { r % 48 } else { r % 4_000 };
			if cache.get(&key).is_none() {
				cache.put(key, i);
			}
			cache.check_integrity();
		}
	}

	#[test]
	fn sampled_ghosts_stay_consistent() {
		// Capacity above MAX_GHOST_CAPACITY exercises the spatially sampled ghosts.
		let mut cache = LruCache::new(5_000);
		assert!(cache.sample_shift > 0);
		let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
		for i in 0..100_000u64 {
			state = state
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let r = state >> 33;
			let key = if r % 3 == 0 { r % 2_000 } else { (i / 7) % 30_000 };
			if cache.get(&key).is_none() {
				cache.put(key, i);
			}
		}
		cache.check_integrity();
	}

	#[test]
	fn admission_slack_ghosts_stay_consistent() {
		// Every admission mode (strict, tie-admitting, unconditional) keeps the segment
		// structure intact.
		let mut sketch = FrequencySketch::new(32, SKETCH_PERIOD);
		for (&slack, &percent) in ADMIT_SLACKS.iter().zip(PROBATION_PERCENTS.iter().cycle()) {
			let mut ghost = Ghost::new(32, 0.25, slack, percent);
			let mut state: u64 = 0x1234_5678_9abc_def1;
			for _ in 0..10_000 {
				state = state
					.wrapping_mul(6_364_136_223_846_793_005)
					.wrapping_add(1_442_695_040_888_963_407);
				let hash = (state >> 33) % 100;
				sketch.increment(hash);
				ghost.access(hash, &sketch);
				ghost.policy.check_integrity();
			}
			assert_eq!(ghost.index.len(), ghost.policy.links.len());
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
		cache.get(&2); // key 2 is reused, key 1 is not
		cache.put(3, 30); // evicts key 1 (the colder entry)

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
		cache.put(4, 40); // one-hit wonders compete for admission
		cache.put(5, 50);
		cache.put(6, 60);

		// Newest key is resident and the cache stays exactly at capacity.
		assert_eq!(*cache.get(&6).unwrap(), 60);
		let mut resident = 0;
		for k in 1..=5 {
			if cache.get(&k).is_some() {
				resident += 1;
			}
		}
		assert_eq!(resident, 2);
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
		cache.put(4, 40); // the hot key must survive the eviction
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // promoted key 2 is protected from eviction
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to MRU
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // the evicted entry's slot is reused for key 5
		assert_eq!(*cache.get(&5).unwrap(), 50);
		let mut resident = 0;
		for k in 1..=4 {
			if cache.get(&k).is_some() {
				resident += 1;
			}
		}
		assert_eq!(resident, 1);
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
