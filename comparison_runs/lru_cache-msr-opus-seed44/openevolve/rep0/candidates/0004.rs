// Adaptive W-TinyLFU cache (the policy Caffeine itself uses), built on an arena of nodes
// threaded through three intrusive doubly-linked lists:
//   * window    - small LRU absorbing new arrivals (recency / bursts),
//   * probation - main-region segment holding entries on trial,
//   * protected - main-region segment for entries re-referenced while on probation.
// When the window overflows, its LRU entry becomes an admission candidate that must beat the
// probation LRU victim on estimated frequency (4-bit count-min sketch with periodic aging).
// A hill climber resizes the window from the sampled hit rate, so recency-heavy traces drift
// towards LRU and frequency-heavy traces towards LFU. Tiny caches (< ADAPTIVE_MIN_CAPACITY)
// keep the window spanning the whole cache, which is exactly strict LRU.
// Arena slots are reused in place on eviction. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

const ADAPTIVE_MIN_CAPACITY: usize = 64;
const PERCENT_MAIN: f64 = 0.99;
const PERCENT_MAIN_PROTECTED: f64 = 0.80;
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY_RATE: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;
const ADMIT_HASHDOS_THRESHOLD: u64 = 6;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
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

/// 4-bit count-min sketch (16 counters per u64 word) with periodic halving ("aging").
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u64>,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	fn new(capacity: usize) -> Self {
		let len = capacity.max(16).next_power_of_two();
		Self {
			table: vec![0u64; len],
			mask: len - 1,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(160),
		}
	}

	/// (word index, bit shift) of the `i`-th counter for `hash`.
	#[inline]
	fn slot(&self, hash: u64, i: u64) -> (usize, u32) {
		let mut z = hash ^ (i + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^= z >> 31;
		((z as usize) & self.mask, ((z >> 58) & 0x3C) as u32)
	}

	fn frequency(&self, hash: u64) -> u64 {
		let mut freq = 15u64;
		for i in 0..4u64 {
			let (w, s) = self.slot(hash, i);
			freq = freq.min((self.table[w] >> s) & 0xF);
		}
		freq
	}

	fn increment(&mut self, hash: u64) {
		let mut added = false;
		for i in 0..4u64 {
			let (w, s) = self.slot(hash, i);
			if ((self.table[w] >> s) & 0xF) < 15 {
				self.table[w] += 1u64 << s;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				for w in self.table.iter_mut() {
					*w = (*w >> 1) & 0x7777_7777_7777_7777;
				}
				self.additions /= 2;
			}
		}
	}
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	lists: [List; 3],       // indexed by WINDOW / PROBATION / PROTECTED
	capacity: usize,
	max_window: usize,
	max_protected: usize,
	sketch: FrequencySketch,
	adaptive: bool,
	last_miss: Option<u64>, // hash of last missed key (its access is already recorded)
	hits_in_sample: usize,
	misses_in_sample: usize,
	prev_hit_rate: f64,
	step_size: f64,
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
		let adaptive = capacity >= ADAPTIVE_MIN_CAPACITY;
		let (max_window, max_protected) = if adaptive {
			let window = (capacity - (PERCENT_MAIN * capacity as f64) as usize).max(1);
			let protected = (PERCENT_MAIN_PROTECTED * (capacity - window) as f64) as usize;
			(window, protected)
		} else {
			(capacity, 0) // window spans the whole cache: strict LRU
		};
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::EMPTY; 3],
			capacity,
			max_window,
			max_protected,
			sketch: FrequencySketch::new(if adaptive { capacity } else { 1 }),
			adaptive,
			last_miss: None,
			hits_in_sample: 0,
			misses_in_sample: 0,
			prev_hit_rate: 0.0,
			step_size: -STEP_PERCENT * capacity as f64,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it within its region.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.sketch.increment(self.nodes[idx].hash);
				self.last_miss = None;
				self.hits_in_sample += 1;
				self.on_hit(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				let hash = hash_of(key);
				self.sketch.increment(hash);
				self.last_miss = Some(hash);
				self.misses_in_sample += 1;
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys always enter the window (so they are immediately
	/// retrievable); when full, the window's LRU entry competes with the main-region victim
	/// and the less frequently used of the two is evicted.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_of(&key);
		// A preceding miss on this key already recorded the access in the sketch.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		let idx = match self.make_room() {
			Some(slot) => {
				let node = &mut self.nodes[slot];
				node.key = key.clone();
				node.value = value;
				node.hash = hash;
				slot
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
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Makes room for one new window entry. Returns the arena slot freed by an eviction, if any.
	fn make_room(&mut self) -> Option<usize> {
		// Window overflow: its LRU entries move to probation as admission candidates.
		let mut candidate = NIL;
		while self.lists[WINDOW as usize].len >= self.max_window && self.lists[WINDOW as usize].len > 0 {
			let c = self.lists[WINDOW as usize].head;
			self.move_to(c, PROBATION);
			if candidate == NIL {
				candidate = c;
			}
		}
		if self.map.len() < self.capacity {
			return None;
		}
		let victim = self.pick_victim(candidate);
		let evict = if candidate == NIL {
			victim
		} else if victim == NIL || !self.admit(candidate, victim) {
			candidate
		} else {
			victim
		};
		debug_assert!(evict != NIL, "full cache must yield an eviction");
		self.unlink(evict);
		self.map.remove(&self.nodes[evict].key);
		Some(evict)
	}

	/// Main-region victim: probation LRU, else protected LRU (never the candidate itself).
	fn pick_victim(&self, candidate: usize) -> usize {
		let probation = self.lists[PROBATION as usize].head;
		if probation != NIL && probation != candidate {
			return probation;
		}
		let protected = self.lists[PROTECTED as usize].head;
		if protected != NIL || candidate != NIL {
			return protected;
		}
		self.lists[WINDOW as usize].head
	}

	/// TinyLFU admission: does `candidate` deserve to replace `victim`?
	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let candidate_freq = self.sketch.frequency(self.nodes[candidate].hash);
		let victim_freq = self.sketch.frequency(self.nodes[victim].hash);
		if candidate_freq > victim_freq {
			return true;
		}
		if candidate_freq >= ADMIT_HASHDOS_THRESHOLD {
			// Occasionally admit a warm candidate so an entrenched victim cannot block forever.
			self.rng ^= self.rng << 13;
			self.rng ^= self.rng >> 7;
			self.rng ^= self.rng << 17;
			return self.rng & 127 == 0;
		}
		false
	}

	/// Window/protected hits move to MRU; probation hits are promoted to protected.
	fn on_hit(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		if queue == PROBATION {
			self.move_to(idx, PROTECTED);
			self.demote_protected();
		} else {
			self.move_to(idx, queue);
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED as usize].len > self.max_protected {
			let head = self.lists[PROTECTED as usize].head;
			self.move_to(head, PROBATION);
		}
	}

	/// Hill climber: every sample period, step the window size in the direction that
	/// improved the hit rate; step size decays unless the hit rate shifts sharply.
	fn climb(&mut self) {
		if !self.adaptive {
			return;
		}
		if self.map.len() < self.capacity / 2 {
			// Still warming up: hit rates are not yet representative.
			self.hits_in_sample = 0;
			self.misses_in_sample = 0;
			self.prev_hit_rate = 0.0;
			return;
		}
		let requests = self.hits_in_sample + self.misses_in_sample;
		if requests < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits_in_sample as f64 / requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step_size } else { -self.step_size };
		self.step_size = if change.abs() >= RESTART_THRESHOLD {
			let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
			STEP_PERCENT * self.capacity as f64 * sign
		} else {
			STEP_DECAY_RATE * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits_in_sample = 0;
		self.misses_in_sample = 0;
		let adjustment = amount as i64;
		if adjustment > 0 {
			self.increase_window(adjustment as usize);
		} else if adjustment < 0 {
			self.decrease_window((-adjustment) as usize);
		}
	}

	/// Grows the window (taken from the protected budget), transferring main's LRU entries.
	fn increase_window(&mut self, amount: usize) {
		let quota = amount.min(self.max_protected);
		self.max_protected -= quota;
		self.max_window += quota;
		self.demote_protected();
		let mut moved = 0;
		while moved < quota {
			let mut idx = self.lists[PROBATION as usize].head;
			if idx == NIL {
				idx = self.lists[PROTECTED as usize].head;
			}
			if idx == NIL {
				break;
			}
			self.move_to(idx, WINDOW);
			moved += 1;
		}
		self.max_protected += quota - moved;
		self.max_window -= quota - moved;
	}

	/// Shrinks the window (giving budget to protected), transferring window LRU entries to probation.
	fn decrease_window(&mut self, amount: usize) {
		let quota = amount.min(self.max_window.saturating_sub(1));
		self.max_window -= quota;
		self.max_protected += quota;
		let mut moved = 0;
		while moved < quota {
			let idx = self.lists[WINDOW as usize].head;
			if idx == NIL {
				break;
			}
			self.move_to(idx, PROBATION);
			moved += 1;
		}
		self.max_window += quota - moved;
		self.max_protected -= quota - moved;
	}

	fn move_to(&mut self, idx: usize, queue: u8) {
		self.unlink(idx);
		self.push_back(idx, queue);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if prev == NIL {
			self.lists[q].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.lists[q].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lists[q].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end of list `queue`.
	fn push_back(&mut self, idx: usize, queue: u8) {
		let q = queue as usize;
		let tail = self.lists[q].tail;
		{
			let node = &mut self.nodes[idx];
			node.queue = queue;
			node.prev = tail;
			node.next = NIL;
		}
		if tail == NIL {
			self.lists[q].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.lists[q].tail = idx;
		self.lists[q].len += 1;
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

	fn check_structure(cache: &LruCache<i64, i64>) {
		let total: usize = cache.lists.iter().map(|l| l.len).sum();
		assert_eq!(total, cache.map.len());
		assert!(cache.map.len() <= cache.capacity);
		for (key, &idx) in &cache.map {
			assert_eq!(cache.nodes[idx].key, *key);
		}
	}

	#[test]
	fn adaptive_mode_respects_capacity_and_put_get() {
		let mut cache = LruCache::new(200);
		let mut seed = 12345u64;
		for i in 0..200_000i64 {
			seed = seed
				.wrapping_mul(6364136223846793005)
				.wrapping_add(1442695040888963407);
			let r = (seed >> 33) as i64;
			// Hot set + long tail, with a one-pass scan in the middle.
			let key = if (50_000..80_000).contains(&i) {
				1_000_000 + i
			} else if r % 4 == 0 {
				r % 5000
			} else {
				r % 150
			};
			if cache.get(&key).is_none() {
				cache.put(key, key * 2);
				assert_eq!(*cache.get(&key).unwrap(), key * 2);
			}
			if i % 10_000 == 0 {
				check_structure(&cache);
			}
		}
		check_structure(&cache);
	}
}
