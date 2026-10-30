// Adaptive W-TinyLFU cache (Caffeine-style), O(1) per operation, std only, no unsafe.
//
// Layout: HashMap<K, slot> + arena of nodes threaded onto three intrusive LRU lists:
//   * Window    - small LRU that admits every new key (captures recency bursts).
//   * Probation - main-region segment for keys that left the window or were demoted.
//   * Protected - main-region segment for keys re-referenced while in probation.
// When a new key arrives at full capacity, the window's LRU entry (candidate) competes
// with the probation LRU entry (victim): a count-min sketch with 4-bit counters and
// periodic halving (TinyLFU) estimates their popularity and the rarer one is evicted.
// A hill climber periodically resizes the window to track whether the workload currently
// rewards recency or frequency. Arena slots are reused in place on eviction.
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
/// Initial share of the capacity given to the main (probation + protected) region.
const MAIN_PERCENT: f64 = 0.99;
/// Share of the main region reserved for the protected segment.
const PROTECTED_PERCENT: f64 = 0.80;
/// Hill-climber tuning (Caffeine defaults).
const CLIMBER_STEP_PERCENT: f64 = 0.0625;
const CLIMBER_STEP_DECAY: f64 = 0.98;
const CLIMBER_RESTART_THRESHOLD: f64 = 0.05;
/// Sketch sample size (and climber sample period) per unit of capacity.
const SAMPLE_MULTIPLIER: usize = 10;

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

	fn increment(&mut self, hash: u64) {
		let mut added = false;
		for i in 0..4 {
			let (index, shift) = self.slot(hash, i);
			let mask = 0xF_u64 << shift;
			if self.table[index] & mask != mask {
				self.table[index] += 1_u64 << shift;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.reset();
			}
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
	adaptive: bool,
	last_miss: Option<u64>, // hash of the key the last get() missed
	hits_in_sample: usize,
	misses_in_sample: usize,
	previous_hit_rate: f64,
	step_size: f64,
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
			capacity
				.saturating_sub((capacity as f64 * MAIN_PERCENT) as usize)
				.max(1)
		} else {
			capacity
		};
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		log::info!(
			"LruCache.try_new: capacity={capacity} window={window_max} protected={protected_max}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::EMPTY; 3],
			capacity,
			window_max,
			protected_max,
			sketch: FrequencySketch::new(capacity),
			adaptive,
			last_miss: None,
			hits_in_sample: 0,
			misses_in_sample: 0,
			previous_hit_rate: 0.0,
			step_size: -CLIMBER_STEP_PERCENT * capacity as f64,
		})
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
				self.sketch.increment(self.nodes[idx].hash);
				self.last_miss = None;
				self.hits_in_sample += 1;
				self.on_access(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				let hash = Self::hash_of(key);
				self.sketch.increment(hash);
				self.last_miss = Some(hash);
				self.misses_in_sample += 1;
				self.climb();
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

	/// Hill climber: every sample period, step the window size in the direction that last
	/// improved the hit rate; restart with a large step when the hit rate shifts sharply.
	fn climb(&mut self) {
		if !self.adaptive {
			return;
		}
		if self.map.len() < self.capacity / 2 {
			// Warm-up: hit rates are not representative yet.
			self.hits_in_sample = 0;
			self.misses_in_sample = 0;
			return;
		}
		let requests = self.hits_in_sample + self.misses_in_sample;
		if requests < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits_in_sample as f64 / requests as f64;
		let change = hit_rate - self.previous_hit_rate;
		let amount = if change >= 0.0 {
			self.step_size
		} else {
			-self.step_size
		};
		self.step_size = if change.abs() >= CLIMBER_RESTART_THRESHOLD {
			let direction = if amount >= 0.0 { 1.0 } else { -1.0 };
			CLIMBER_STEP_PERCENT * self.capacity as f64 * direction
		} else {
			CLIMBER_STEP_DECAY * amount
		};
		self.previous_hit_rate = hit_rate;
		self.hits_in_sample = 0;
		self.misses_in_sample = 0;
		let adjustment = amount as i64;
		if adjustment > 0 {
			self.increase_window(adjustment as usize);
		} else if adjustment < 0 {
			self.decrease_window(adjustment.unsigned_abs() as usize);
		}
	}

	/// Grow the window by up to `amount` slots taken from the protected budget. The window
	/// then fills with new arrivals while the main region shrinks through normal eviction.
	fn increase_window(&mut self, amount: usize) {
		let quota = amount.min(self.protected_max);
		self.protected_max -= quota;
		self.window_max += quota;
		self.demote_protected_overflow();
	}

	/// Shrink the window by up to `amount` slots (never below one), handing the budget to
	/// the protected segment; overflowing window entries spill into probation.
	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_max.saturating_sub(1));
		self.window_max -= quota;
		self.protected_max += quota;
		while self.lists[WINDOW].len > self.window_max {
			let lru = self.lists[WINDOW].head;
			self.move_to(lru, PROBATION);
		}
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
