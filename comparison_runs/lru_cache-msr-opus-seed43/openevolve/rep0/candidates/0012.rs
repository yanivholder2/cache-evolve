// Adaptive W-TinyLFU cache behind the original LruCache API (HashMap + arena-based lists).
//
// Resident entries are split into three LRU segments threaded through a single arena:
//   * window    (~1% initially) - admits every new key, absorbs recency bursts
//   * probation (~20% of main)  - main-region entries not re-referenced since admission
//   * protected (~80% of main)  - main-region entries re-referenced while resident
// A count-min sketch (saturating at 15, periodically halved) estimates access frequency.
// When full, the window's LRU entry (candidate) enters the main region only if it is estimated
// to be more popular than the main region's victim (TinyLFU admission); otherwise it leaves.
// The victim is the least frequent of the few oldest probation entries, so a warm entry that
// happens to sit at the probation head neither blocks admission nor outlives colder neighbours.
// A hill climber periodically resizes the window to follow recency- vs frequency-biased phases.
// Slots are reused in place on eviction, so no free list is needed. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

// Hill-climber tuning (mirrors Caffeine's defaults).
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;
/// Upper bound on how many of the oldest main-region entries compete to be the victim.
const VICTIM_SAMPLES: usize = 4;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Cached key hash, so frequency lookups during eviction never re-hash the key.
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

/// Head (LRU) / tail (MRU) of one segment's doubly-linked recency list.
#[derive(Debug, Clone, Copy)]
struct List {
	head: usize,
	tail: usize,
	len: usize,
}

impl List {
	const fn new() -> Self {
		Self {
			head: NIL,
			tail: NIL,
			len: 0,
		}
	}
}

/// Count-min sketch with conservative update and periodic aging (halving).
struct Sketch {
	table: Vec<u8>,
	width: usize,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl Debug for Sketch {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Sketch")
			.field("width", &self.width)
			.field("additions", &self.additions)
			.field("sample_size", &self.sample_size)
			.finish()
	}
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.clamp(16, 1 << 20).next_power_of_two() * 4;
		Self {
			table: vec![0; width * 4],
			width,
			mask: width - 1,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(64),
		}
	}

	fn indexes(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		let mut h = hash;
		for (row, slot) in out.iter_mut().enumerate() {
			h = h.wrapping_add(0x9E37_79B9_7F4A_7C15);
			let mut z = h;
			z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
			z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
			z ^= z >> 31;
			*slot = row * self.width + ((z as usize) & self.mask);
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.indexes(hash).iter().map(|&i| self.table[i]).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let idx = self.indexes(hash);
		let min = idx.iter().map(|&i| self.table[i]).min().unwrap_or(0);
		if min >= 15 {
			return;
		}
		for &i in &idx {
			if self.table[i] == min {
				self.table[i] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample_size {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	window: List,
	probation: List,
	protected: List,
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	/// Hash of the last key that missed in `get`, so the follow-up `put` is not double counted.
	pending_miss: Option<u64>,
	hits: usize,
	misses: usize,
	prev_hit_rate: f64,
	step: f64,
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
		let main = ((capacity as u128 * 99) / 100) as usize;
		let window_max = capacity - main; // >= 1 for any capacity >= 1
		let protected_max = ((main as u128 * 80) / 100) as usize;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			window: List::new(),
			probation: List::new(),
			protected: List::new(),
			capacity,
			window_max,
			protected_max,
			sketch: Sketch::new(capacity),
			pending_miss: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -STEP_PERCENT * capacity as f64,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access for the policy.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = Self::hash_key(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits += 1;
				self.on_hit(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				self.misses += 1;
				self.pending_miss = Some(hash);
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. A new key always enters the window; when the cache is full the
	/// TinyLFU admission duel decides whether the window's LRU entry or the main LRU entry leaves.
	pub fn put(&mut self, key: K, value: V) {
		let hash = Self::hash_key(&key);
		if self.pending_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		// Key already exists: update value in place and treat it as an access
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let idx = if self.nodes.len() < self.capacity {
			// Arena has room: allocate a new slot
			self.nodes.push(Node {
				key: key.clone(),
				value,
				hash,
				prev: NIL,
				next: NIL,
				queue: WINDOW,
			});
			self.nodes.len() - 1
		} else {
			// Full: pick the entry to evict and reuse its slot for the new key
			let slot = self.select_victim();
			self.unlink(slot);
			let old_key = std::mem::replace(&mut self.nodes[slot].key, key.clone());
			self.map.remove(&old_key);
			self.nodes[slot].value = value;
			self.nodes[slot].hash = hash;
			slot
		};
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		// Window overflow flows into probation (only happens while there is spare room)
		self.drain_window();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn hash_key(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Main-region victim: among the few least-recently-used entries of probation (or protected,
	/// when probation is empty) the one with the lowest estimated frequency; ties keep LRU order.
	/// Returns the victim's index (NIL if the main region is empty) and its frequency.
	fn main_victim(&self) -> (usize, u8) {
		let list = if self.probation.head != NIL {
			self.probation
		} else {
			self.protected
		};
		if list.head == NIL {
			return (NIL, 0);
		}
		// Small lists keep strict LRU; large ones sample a handful of nearly-equally-old entries
		let samples = (list.len / 8).clamp(1, VICTIM_SAMPLES);
		let mut best = list.head;
		let mut best_freq = self.sketch.frequency(self.nodes[best].hash);
		let mut cur = self.nodes[best].next;
		for _ in 1..samples {
			if cur == NIL || best_freq == 0 {
				break;
			}
			let freq = self.sketch.frequency(self.nodes[cur].hash);
			if freq < best_freq {
				best = cur;
				best_freq = freq;
			}
			cur = self.nodes[cur].next;
		}
		(best, best_freq)
	}

	/// Choose which resident entry to evict when the cache is full.
	fn select_victim(&mut self) -> usize {
		let (main_victim, victim_freq) = self.main_victim();
		if self.window.len < self.window_max || self.window.head == NIL {
			// Window is growing toward an enlarged target: shrink the main region instead
			return if main_victim != NIL {
				main_victim
			} else {
				self.window.head
			};
		}
		let candidate = self.window.head;
		if main_victim == NIL {
			return candidate;
		}
		if self.admit(candidate, victim_freq) {
			self.move_to(candidate, PROBATION);
			main_victim
		} else {
			candidate
		}
	}

	/// TinyLFU admission: the candidate wins only if it is estimated to be more popular.
	fn admit(&mut self, candidate: usize, victim_freq: u8) -> bool {
		let cand_freq = self.sketch.frequency(self.nodes[candidate].hash);
		if cand_freq > victim_freq {
			return true;
		}
		if cand_freq >= 6 {
			// Occasionally admit a warm candidate so a hot victim cannot block it forever
			self.rng ^= self.rng << 13;
			self.rng ^= self.rng >> 7;
			self.rng ^= self.rng << 17;
			return self.rng & 127 == 0;
		}
		false
	}

	/// Reorder a node after it was accessed.
	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.drain_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	fn drain_window(&mut self) {
		while self.window.len > self.window_max {
			let lru = self.window.head;
			self.move_to(lru, PROBATION);
		}
	}

	fn drain_protected(&mut self) {
		while self.protected.len > self.protected_max {
			let lru = self.protected.head;
			self.move_to(lru, PROBATION);
		}
	}

	/// Hill climber: every sample period, step the window size in the direction that
	/// improved the hit rate, decaying the step size as it converges.
	fn climb(&mut self) {
		let total = self.hits + self.misses;
		if total < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= RESTART_THRESHOLD {
			(STEP_PERCENT * self.capacity as f64).copysign(amount)
		} else {
			STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		self.resize_window(amount as isize);
	}

	fn resize_window(&mut self, delta: isize) {
		if delta > 0 {
			let quota = (delta as usize).min(self.protected_max);
			self.window_max += quota;
			self.protected_max -= quota;
			self.drain_protected();
		} else if delta < 0 {
			let quota = delta.unsigned_abs().min(self.window_max - 1);
			self.window_max -= quota;
			self.protected_max += quota;
			self.drain_window();
		}
	}

	fn list_mut(&mut self, queue: u8) -> &mut List {
		match queue {
			WINDOW => &mut self.window,
			PROBATION => &mut self.probation,
			_ => &mut self.protected,
		}
	}

	fn move_to(&mut self, idx: usize, queue: u8) {
		self.unlink(idx);
		self.push_back(idx, queue);
	}

	/// Remove node from its current segment's list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, queue) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.list_mut(queue).head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.list_mut(queue).tail = prev;
		}
		self.list_mut(queue).len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end (tail) of the given segment.
	fn push_back(&mut self, idx: usize, queue: u8) {
		let tail = self.list_mut(queue).tail;
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.queue = queue;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.list_mut(queue).head = idx;
		}
		let list = self.list_mut(queue);
		list.tail = idx;
		list.len += 1;
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
	fn cold_newcomer_cannot_displace_popular_key() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		for _ in 0..3 {
			cache.get(&1);
		}
		cache.put(2, 20);
		cache.put(3, 30); // window candidate 2 loses the admission duel against popular key 1

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_consistent(&cache);
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
		for k in 1..=6 {
			cache.put(k, k * 10);
			assert_consistent(&cache);
		}
		// Exactly `capacity` keys survive and the newest key is always resident
		assert_eq!((1..=6).filter(|k| cache.map.contains_key(k)).count(), 3);
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
	fn repeated_get_protects_key() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		for k in 4..20 {
			cache.put(k, k * 10); // a stream of one-hit wonders
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&19).unwrap(), 190);
		assert_consistent(&cache);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probation
		cache.put(2, 20); // probation
		cache.put(3, 30); // window
		cache.get(&2); // promote 2 from probation to protected
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40);
		cache.put(5, 50);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_consistent(&cache);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // reuses an evicted slot
		cache.get(&3);
		cache.put(4, 40);
		cache.put(5, 50);
		assert_eq!(cache.nodes.len(), 2); // arena never grows past capacity
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_consistent(&cache);
	}

	#[test]
	fn randomized_ops_keep_structure_consistent() {
		let mut cache = LruCache::new(64);
		let mut x: u64 = 0x1234_5678_9ABC_DEF0;
		for i in 0..50_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			// skewed key distribution mixing a hot set and a long tail
			let key = if x % 4 == 0 { x % 1000 } else { x % 40 };
			if cache.get(&key).is_none() {
				cache.put(key, key + 1);
				assert_eq!(*cache.get(&key).unwrap(), key + 1);
			}
			if i % 997 == 0 {
				assert_consistent(&cache);
			}
		}
		assert_consistent(&cache);
		assert_eq!(cache.map.len(), 64);
	}

	#[test]
	fn frequent_keys_survive_scan() {
		let mut cache = LruCache::new(100);
		for _ in 0..5 {
			for k in 0..50u64 {
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
		}
		for k in 1_000..5_000u64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let survivors = (0..50u64).filter(|k| cache.map.contains_key(k)).count();
		assert!(survivors >= 45, "only {survivors} hot keys survived the scan");
	}

	#[test]
	fn sampled_victim_spares_warm_probation_head() {
		let mut cache = LruCache::new(100);
		// Key 0 is warm before insertion, so it lands at the probation head with frequency 5
		for _ in 0..5 {
			assert!(cache.get(&0u64).is_none());
		}
		cache.put(0u64, 0u64);
		for k in 1..99u64 {
			cache.put(k, k);
		}
		// Key 99 becomes the window entry after being requested twice
		assert!(cache.get(&99).is_none());
		assert!(cache.get(&99).is_none());
		cache.put(99, 99);
		// Candidate 99 beats a cold neighbour of the head instead of losing to warm key 0
		cache.put(1_000, 1_000);
		assert!(cache.map.contains_key(&0));
		assert!(cache.map.contains_key(&99));
		assert!(cache.map.contains_key(&1_000));
		assert_eq!((1..4u64).filter(|k| !cache.map.contains_key(k)).count(), 1);
		assert_consistent(&cache);
	}

	fn assert_consistent<K, V>(cache: &LruCache<K, V>)
	where
		K: Hash + Eq + Clone + Debug,
		V: Debug,
	{
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.nodes.len() <= cache.capacity);
		assert_eq!(
			cache.window.len + cache.probation.len + cache.protected.len,
			cache.map.len()
		);
		for (key, &idx) in &cache.map {
			assert!(cache.nodes[idx].key == *key);
		}
		for (list, queue) in [
			(cache.window, WINDOW),
			(cache.probation, PROBATION),
			(cache.protected, PROTECTED),
		] {
			let mut prev = NIL;
			let mut cur = list.head;
			let mut count = 0;
			while cur != NIL {
				assert_eq!(cache.nodes[cur].queue, queue);
				assert_eq!(cache.nodes[cur].prev, prev);
				prev = cur;
				cur = cache.nodes[cur].next;
				count += 1;
			}
			assert_eq!(prev, list.tail);
			assert_eq!(count, list.len);
		}
		assert!(cache.window_max >= 1);
		assert!(cache.protected.len <= cache.protected_max);
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
