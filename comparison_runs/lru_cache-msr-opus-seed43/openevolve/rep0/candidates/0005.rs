// W-TinyLFU cache (Caffeine-style) behind the LruCache API.
// - New keys enter a small LRU admission window.
// - The main region is a segmented LRU (probation + protected). A window overflow
//   candidate only replaces the probation LRU victim when a 4-bit count-min sketch
//   estimates it is accessed more often (TinyLFU admission) -> scan/one-hit resistance.
// - The window/main split is tuned online with Caffeine's hill climber, so
//   recency-biased traces still get LRU-like behaviour.
// - Tiny caches (capacity < ADAPTIVE_MIN_CAPACITY) degrade to exact LRU.
// Arena of nodes indexed by usize threaded by three intrusive doubly-linked lists.
// O(1) per operation, no unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
const ADAPTIVE_MIN_CAPACITY: usize = 16;
const PROTECTED_PERCENT: f64 = 0.80;
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

fn hash_of<T: Hash + ?Sized>(item: &T) -> u64 {
	let mut h = DefaultHasher::new();
	item.hash(&mut h);
	h.finish()
}

const SEEDS: [u64; 4] = [
	0xc3a5_c85c_97cb_3127,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
	0xcbf2_9ce4_8422_2325,
];
const RESET_MASK: u64 = 0x7777_7777_7777_7777;
const ONE_MASK: u64 = 0x1111_1111_1111_1111;

/// 4-bit count-min sketch with periodic aging (halving), as in Caffeine.
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u64>,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	fn new(capacity: usize) -> Self {
		let len = capacity.clamp(8, 1 << 26).next_power_of_two();
		Self {
			table: vec![0; len],
			mask: len - 1,
			additions: 0,
			sample_size: capacity.max(8).saturating_mul(10),
		}
	}

	fn index_of(&self, hash: u64, i: usize) -> usize {
		let mut h = hash.wrapping_add(SEEDS[i]).wrapping_mul(SEEDS[i]);
		h = h.wrapping_add(h >> 32);
		(h as usize) & self.mask
	}

	fn frequency(&self, hash: u64) -> u64 {
		let start = ((hash & 3) << 2) as usize;
		let mut freq = 15u64;
		for i in 0..4 {
			let shift = (start + i) << 2;
			let count = (self.table[self.index_of(hash, i)] >> shift) & 0xF;
			freq = freq.min(count);
		}
		freq
	}

	fn increment(&mut self, hash: u64) {
		let start = ((hash & 3) << 2) as usize;
		let mut added = false;
		for i in 0..4 {
			let idx = self.index_of(hash, i);
			let shift = (start + i) << 2;
			let mask = 0xFu64 << shift;
			if self.table[idx] & mask != mask {
				self.table[idx] += 1u64 << shift;
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

	fn reset(&mut self) {
		let mut odd = 0usize;
		for slot in self.table.iter_mut() {
			odd += (*slot & ONE_MASK).count_ones() as usize;
			*slot = (*slot >> 1) & RESET_MASK;
		}
		self.additions = (self.additions >> 1).saturating_sub(odd >> 2);
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // evicted slots ready for reuse
	heads: [usize; 3],      // LRU end of window / probation / protected
	tails: [usize; 3],      // MRU end of window / probation / protected
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: FrequencySketch,
	last_miss: Option<u64>, // hash of the last get() miss (avoid double counting the follow-up put)
	rng: u64,
	adaptive: bool,
	filled: bool,
	hits: usize,
	misses: usize,
	prev_hit_rate: f64,
	step: f64,
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
		let adaptive = capacity >= ADAPTIVE_MIN_CAPACITY;
		// Tiny caches: the window spans everything -> exact LRU.
		let window_max = if adaptive { (capacity / 100).max(1) } else { capacity };
		let protected_max = ((capacity - window_max) as f64 * PROTECTED_PERCENT) as usize;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max,
			sketch: FrequencySketch::new(capacity),
			last_miss: None,
			rng: 0x9E37_79B9_7F4A_7C15,
			adaptive,
			filled: false,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -STEP_PERCENT * capacity as f64,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.last_miss = None;
				self.on_access(idx);
				self.record(true);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
				self.record(false);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the admission window; overflow is
	/// resolved by TinyLFU admission against the main region's LRU victim.
	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_of(&key);
		// A put right after a get-miss of the same key is the same access: count once.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let idx = match self.free.pop() {
			Some(idx) => {
				let node = &mut self.nodes[idx];
				node.key = key.clone();
				node.value = value;
				node.hash = hash;
				idx
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					hash,
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
		if self.map.len() >= self.capacity {
			self.filled = true;
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Promote a node within its segment (probation hits graduate to protected).
	fn on_access(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
			q => self.move_to(idx, q),
		}
	}

	fn demote_protected(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			let idx = self.heads[PROTECTED];
			self.move_to(idx, PROBATION);
		}
	}

	/// Restore window_len <= window_max and main_len <= capacity - window_max.
	fn evict(&mut self) {
		while self.lens[WINDOW] > self.window_max {
			let candidate = self.heads[WINDOW];
			let main_len = self.lens[PROBATION] + self.lens[PROTECTED];
			if main_len + self.window_max < self.capacity {
				// Main region still has room: admit unconditionally.
				self.move_to(candidate, PROBATION);
				continue;
			}
			let victim = if self.heads[PROBATION] != NIL {
				self.heads[PROBATION]
			} else {
				self.heads[PROTECTED]
			};
			if victim != NIL && self.admit(candidate, victim) {
				self.remove(victim);
				self.move_to(candidate, PROBATION);
			} else {
				self.remove(candidate);
			}
		}
		// Safety net (unreachable while the segment invariants hold).
		while self.map.len() > self.capacity {
			let mut victim = NIL;
			for q in [PROBATION, PROTECTED, WINDOW] {
				if self.heads[q] != NIL {
					victim = self.heads[q];
					break;
				}
			}
			if victim == NIL {
				break;
			}
			self.remove(victim);
		}
	}

	/// TinyLFU admission: the candidate must be estimated more popular than the victim.
	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let cand_freq = self.sketch.frequency(self.nodes[candidate].hash);
		let victim_freq = self.sketch.frequency(self.nodes[victim].hash);
		if cand_freq > victim_freq {
			return true;
		}
		if cand_freq <= 5 {
			return false;
		}
		// Occasionally admit a warm candidate so a hot victim can't block forever.
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		(self.rng & 127) == 0
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Hill-climbing sample bookkeeping (only once the cache has filled).
	fn record(&mut self, hit: bool) {
		if !self.adaptive || !self.filled {
			return;
		}
		if hit {
			self.hits += 1;
		} else {
			self.misses += 1;
		}
		if self.hits + self.misses >= self.sketch.sample_size {
			self.climb();
		}
	}

	fn climb(&mut self) {
		let total = self.hits + self.misses;
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
		self.step = if change.abs() >= RESTART_THRESHOLD {
			STEP_PERCENT * self.capacity as f64 * sign
		} else {
			STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		let adjustment = amount as i64;
		if adjustment > 0 {
			self.increase_window(adjustment as usize);
		} else if adjustment < 0 {
			self.decrease_window(adjustment.unsigned_abs() as usize);
		}
	}

	/// Grow the window by shrinking the protected segment, moving main LRU entries
	/// into the window (no evictions).
	fn increase_window(&mut self, amount: usize) {
		let quota = amount.min(self.protected_max);
		if quota == 0 {
			return;
		}
		self.protected_max -= quota;
		self.demote_protected();
		let mut moved = 0;
		while moved < quota {
			let idx = if self.heads[PROBATION] != NIL {
				self.heads[PROBATION]
			} else {
				self.heads[PROTECTED]
			};
			if idx == NIL {
				break;
			}
			self.move_to(idx, WINDOW);
			moved += 1;
		}
		self.protected_max += quota - moved;
		self.window_max += moved;
	}

	/// Shrink the window (min 1), moving its LRU entries into probation.
	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_max.saturating_sub(1));
		let mut moved = 0;
		while moved < quota {
			let idx = self.heads[WINDOW];
			if idx == NIL {
				break;
			}
			self.move_to(idx, PROBATION);
			moved += 1;
		}
		self.window_max -= moved;
		self.protected_max += moved;
	}

	fn move_to(&mut self, idx: usize, queue: usize) {
		self.unlink(idx);
		self.push_back(queue, idx);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.heads[q] = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.tails[q] = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
		self.lens[q] -= 1;
	}

	/// Append node to the MRU end of list `queue`.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.tails[queue];
		self.nodes[idx].queue = queue;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.heads[queue] = idx;
		}
		self.tails[queue] = idx;
		self.lens[queue] += 1;
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

	#[test]
	fn large_cache_bounds_and_newest_resident() {
		let mut cache = LruCache::new(100);
		for k in 0..10_000i64 {
			if cache.get(&(k % 731)).is_none() {
				cache.put(k % 731, k);
			}
			cache.put(k, k * 2);
			assert_eq!(*cache.get(&k).unwrap(), k * 2);
			assert!(cache.map.len() <= 100);
			assert_eq!(cache.lens.iter().sum::<usize>(), cache.map.len());
		}
	}

	#[test]
	fn frequent_keys_survive_scan() {
		let mut cache = LruCache::new(100);
		for _ in 0..5 {
			for k in 0..50i64 {
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
		}
		for k in 1000..3000i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let survivors = (0..50i64).filter(|k| cache.get(k).is_some()).count();
		assert!(survivors >= 45, "only {survivors} hot keys survived the scan");
	}
}
