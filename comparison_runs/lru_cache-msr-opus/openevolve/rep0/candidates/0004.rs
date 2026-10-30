// W-TinyLFU cache: a small LRU admission window in front of a segmented-LRU main space
// (probation + protected), guarded by a 4-bit count-min frequency sketch, with
// Caffeine-style hill climbing of the window size. Arena-backed intrusive lists,
// evicted slots reused in place. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

/// Sentinel "null" arena index.
const NIL: usize = usize::MAX;
/// Queue ids: admission window, main probation segment, main protected segment.
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

/// Hill-climber tuning (Caffeine's constants).
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;

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
	fn new(capacity: usize) -> Self {
		let len = capacity.max(16).next_power_of_two();
		Self {
			table: vec![0; len],
			mask: len - 1,
			size: 0,
			sample_size: capacity.saturating_mul(10),
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

	fn increment(&mut self, hash: u64) {
		let mut added = false;
		for i in 0..4 {
			let (index, shift) = self.slot(hash, i);
			if ((self.table[index] >> shift) & 0xF) < 15 {
				self.table[index] += 1u64 << shift;
				added = true;
			}
		}
		if added {
			self.size += 1;
			if self.size >= self.sample_size {
				self.reset();
			}
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

pub mod lru_cache_jni;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> arena index
	nodes: Vec<Node<K, V>>, // arena
	lists: [List; 3],       // window, probation, protected
	sketch: FrequencySketch,
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	last_get_hash: Option<u64>,
	sample_hits: usize,
	sample_misses: usize,
	prev_hit_rate: f64,
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
		log::info!("LruCache.try_new: capacity={capacity}");
		let window_max = (capacity / 100).max(1);
		let main_max = capacity - window_max;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::EMPTY; 3],
			sketch: FrequencySketch::new(capacity),
			capacity,
			window_max,
			protected_max: main_max - main_max / 5,
			last_get_hash: None,
			sample_hits: 0,
			sample_misses: 0,
			prev_hit_rate: 0.0,
			step_size: -STEP_PERCENT * capacity as f64,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. Every lookup is recorded in the
	/// frequency sketch; a hit is promoted within its segment (probation -> protected).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = Self::hash_key(key);
		self.sketch.increment(hash);
		self.last_get_hash = Some(hash);
		let found = self.map.get(key).copied();
		self.record_sample(found.is_some());
		let idx = found?;
		self.on_access(idx);
		Some(&self.nodes[idx].value)
	}

	fn hash_key(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Window/protected hits move to their MRU end; a probation hit graduates to
	/// protected, demoting protected's LRU back to probation when over quota.
	fn on_access(&mut self, idx: usize) {
		match self.nodes[idx].queue as usize {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	/// Hill climber: sample the hit rate and resize the window toward better hit rates.
	fn record_sample(&mut self, hit: bool) {
		if self.map.len() < self.capacity {
			return;
		}
		if hit {
			self.sample_hits += 1;
		} else {
			self.sample_misses += 1;
		}
		let requests = self.sample_hits + self.sample_misses;
		if requests < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.sample_hits as f64 / requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step_size } else { -self.step_size };
		self.step_size = if change.abs() >= RESTART_THRESHOLD {
			let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
			STEP_PERCENT * self.capacity as f64 * sign
		} else {
			STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.sample_hits = 0;
		self.sample_misses = 0;
		let adjust = amount as i64;
		if adjust > 0 {
			self.increase_window(adjust as usize);
		} else if adjust < 0 {
			self.decrease_window(adjust.unsigned_abs() as usize);
		}
	}

	/// Inserts or updates `key`. A new key enters the admission window; when the cache
	/// is full, the window's LRU entry must out-score the main space's LRU victim on
	/// estimated frequency (TinyLFU) to be admitted, otherwise it is the one evicted.
	pub fn put(&mut self, key: K, value: V) {
		let hash = Self::hash_key(&key);
		// A miss is normally `get` then `put` of the same key: count the access once.
		if self.last_get_hash.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let idx = if self.map.len() >= self.capacity {
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
				prev: NIL,
				next: NIL,
				queue: WINDOW as u8,
			});
			self.nodes.len() - 1
		};
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		while self.lists[WINDOW].len > self.window_max {
			let lru = self.lists[WINDOW].head;
			self.move_to(lru, PROBATION);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Frees exactly one resident entry and returns its (unlinked) arena slot.
	fn evict(&mut self) -> usize {
		let victim = if self.lists[PROBATION].head != NIL {
			self.lists[PROBATION].head
		} else {
			self.lists[PROTECTED].head
		};
		let window = self.lists[WINDOW];
		let evicted = if window.head != NIL && window.len >= self.window_max {
			// Window is about to overflow: its LRU entry duels the main-space victim.
			let candidate = window.head;
			if victim != NIL && self.admit(candidate, victim) {
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
		self.map.remove(&self.nodes[evicted].key);
		evicted
	}

	fn admit(&self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.frequency(self.nodes[candidate].hash);
		let victim_freq = self.sketch.frequency(self.nodes[victim].hash);
		candidate_freq > victim_freq
	}

	/// Remove node from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.lists[q].head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.lists[q].tail = prev;
		}
		self.lists[q].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end (tail) of list `q`.
	fn push_back(&mut self, idx: usize, q: usize) {
		let tail = self.lists[q].tail;
		{
			let node = &mut self.nodes[idx];
			node.prev = tail;
			node.next = NIL;
			node.queue = q as u8;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.lists[q].head = idx;
		}
		self.lists[q].tail = idx;
		self.lists[q].len += 1;
	}

	fn move_to(&mut self, idx: usize, q: usize) {
		self.unlink(idx);
		self.push_back(idx, q);
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let lru = self.lists[PROTECTED].head;
			self.move_to(lru, PROBATION);
		}
	}

	/// Grow the window at the expense of the protected segment, pulling main-space
	/// LRU entries into the window.
	fn increase_window(&mut self, amount: usize) {
		let quota = amount.min(self.protected_max);
		self.protected_max -= quota;
		self.window_max += quota;
		self.demote_protected();
		for _ in 0..quota {
			let lru = if self.lists[PROBATION].head != NIL {
				self.lists[PROBATION].head
			} else {
				self.lists[PROTECTED].head
			};
			if lru == NIL {
				break;
			}
			self.move_to(lru, WINDOW);
		}
	}

	/// Shrink the window (never below one entry), handing its LRU entries to probation.
	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.window_max - 1);
		self.window_max -= quota;
		self.protected_max += quota;
		while self.lists[WINDOW].len > self.window_max {
			let lru = self.lists[WINDOW].head;
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
				let node = &self.nodes[cur];
				assert_eq!(node.queue as usize, q);
				assert_eq!(node.prev, prev);
				assert_eq!(self.map.get(&node.key), Some(&cur));
				prev = cur;
				cur = node.next;
				count += 1;
				assert!(count <= self.capacity, "cycle in list {q}");
			}
			assert_eq!(list.tail, prev);
			assert_eq!(list.len, count);
			total += count;
		}
		assert_eq!(total, self.map.len());
		assert_eq!(self.nodes.len(), self.map.len());
		assert!(self.map.len() <= self.capacity);
		assert!(self.lists[WINDOW].len <= self.window_max);
		assert!(self.lists[PROTECTED].len <= self.protected_max);
		assert!(self.window_max >= 1 && self.window_max + self.protected_max <= self.capacity);
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
