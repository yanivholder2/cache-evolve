// Adaptive W-TinyLFU cache (the policy behind Caffeine), kept under the historical
// `LruCache` name because the JNI boundary binds to it.
//
// A small LRU *window* admits every new key. When the window overflows, its LRU entry
// (the candidate) competes with the LRU entry of the segmented-LRU *main* space
// (probation + protected; the victim): a count-min sketch of 4-bit counters, halved
// periodically so it ages, estimates their popularity, and the candidate is admitted
// only if it is more popular. A hill climber resizes the window from the observed hit
// rate, so recency-heavy traces still behave like LRU.
// All three lists thread through one arena of nodes; slots are reused in place on
// eviction. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;
const WARM_CANDIDATE: u8 = 6;

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
	head: usize, // LRU
	tail: usize, // MRU
	len: usize,
}

impl List {
	const fn new() -> Self {
		Self { head: NIL, tail: NIL, len: 0 }
	}
}

/// Count-min sketch (4 rows, saturating 4-bit counters, conservative update) with
/// periodic halving so that stale popularity decays.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample_size: usize,
}

fn mix(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.saturating_mul(4).clamp(64, 1 << 24).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(64),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			let h = mix(hash.wrapping_add((row as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15)));
			*slot = row * self.width + (h as usize & (self.width - 1));
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&i| self.table[i]).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let slots = self.slots(hash);
		let min = slots.iter().map(|&i| self.table[i]).min().unwrap_or(0);
		if min >= 15 {
			return;
		}
		for &i in &slots {
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
	last_miss: Option<u64>,
	hits_in_sample: usize,
	misses_in_sample: usize,
	sample_size: usize,
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
		let window_max = (capacity / 100).max(1);
		let protected_max = (capacity - window_max) * 4 / 5;
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
			last_miss: None,
			hits_in_sample: 0,
			misses_in_sample: 0,
			sample_size: capacity.saturating_mul(10).max(64),
			prev_hit_rate: 0.0,
			step: -STEP_PERCENT * capacity as f64,
			rng: 0x2545_f491_4f6c_dd1d,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		match self.map.get(key).copied() {
			Some(idx) => {
				self.last_miss = None;
				self.hits_in_sample += 1;
				self.on_access(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				// The driver normally follows a miss with a put of the same key; remember
				// it so that this access is not counted twice in the sketch.
				self.last_miss = Some(hash);
				self.misses_in_sample += 1;
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; at capacity exactly one entry
	/// is evicted (window candidate or main victim, decided by TinyLFU admission) and its
	/// arena slot is reused.
	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_of(&key);
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		// Key already exists: update value in place and treat it as an access
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
				queue: WINDOW,
			});
			self.nodes.len() - 1
		};
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		self.drain_window();
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Frees one slot (unlinked and removed from the map) and returns its index.
	fn evict(&mut self) -> usize {
		let victim = if self.probation.head != NIL {
			self.probation.head
		} else {
			self.protected.head
		};
		let idx = if self.window.len >= self.window_max && self.window.head != NIL {
			// The window is about to overflow: its LRU entry must win admission into main.
			let candidate = self.window.head;
			if victim != NIL && self.admit(candidate, victim) {
				self.move_to(candidate, PROBATION);
				victim
			} else {
				candidate
			}
		} else if victim != NIL {
			victim
		} else {
			self.window.head
		};
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		idx
	}

	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let cf = self.sketch.frequency(self.nodes[candidate].hash);
		let vf = self.sketch.frequency(self.nodes[victim].hash);
		if cf > vf {
			return true;
		}
		if cf >= WARM_CANDIDATE {
			// Occasionally let a warm candidate in so a stale main space can turn over.
			self.rng ^= self.rng << 13;
			self.rng ^= self.rng >> 7;
			self.rng ^= self.rng << 17;
			return self.rng & 127 == 0;
		}
		false
	}

	fn on_access(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	fn drain_window(&mut self) {
		while self.window.len > self.window_max {
			let head = self.window.head;
			self.move_to(head, PROBATION);
		}
	}

	fn demote_protected(&mut self) {
		while self.protected.len > self.protected_max {
			let head = self.protected.head;
			self.move_to(head, PROBATION);
		}
	}

	/// Hill climbing on the window size, driven by the hit rate of each sample period.
	fn climb(&mut self) {
		let total = self.hits_in_sample + self.misses_in_sample;
		if total < self.sample_size {
			return;
		}
		let hit_rate = self.hits_in_sample as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= RESTART_THRESHOLD {
			STEP_PERCENT * self.capacity as f64 * amount.signum()
		} else {
			STEP_DECAY * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits_in_sample = 0;
		self.misses_in_sample = 0;

		let amount = amount as i64;
		if amount > 0 {
			let quota = (amount as usize).min(self.protected_max);
			self.protected_max -= quota;
			self.window_max += quota;
			self.demote_protected();
		} else if amount < 0 {
			let quota = (amount.unsigned_abs() as usize).min(self.window_max - 1);
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

	/// Remove node from the list it currently belongs to.
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

	/// Append node to the MRU end (tail) of `queue`.
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

	/// Structural invariants: lists are consistent, cover every resident key exactly
	/// once, and the cache never exceeds its capacity.
	fn check<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		assert!(c.map.len() <= c.capacity);
		assert_eq!(c.map.len(), c.nodes.len());
		let mut seen = 0;
		for (queue, list) in [(WINDOW, c.window), (PROBATION, c.probation), (PROTECTED, c.protected)] {
			let mut prev = NIL;
			let mut cur = list.head;
			let mut n = 0;
			while cur != NIL {
				let node = &c.nodes[cur];
				assert_eq!(node.queue, queue);
				assert_eq!(node.prev, prev);
				assert_eq!(c.map.get(&node.key), Some(&cur));
				prev = cur;
				cur = node.next;
				n += 1;
			}
			assert_eq!(prev, list.tail);
			assert_eq!(n, list.len);
			seen += n;
		}
		assert_eq!(seen, c.map.len());
		assert!(c.window_max >= 1);
		assert!(c.window.len <= c.window_max);
		assert!(c.protected.len <= c.protected_max);
	}

	#[test]
	fn fill_to_capacity_keeps_everything() {
		let mut cache = LruCache::new(50);
		for k in 0..50 {
			cache.put(k, k * 10);
		}
		for k in 0..50 {
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
		check(&cache);
	}

	#[test]
	fn new_key_is_resident_after_put() {
		for cap in [1, 2, 3, 7, 100] {
			let mut cache = LruCache::new(cap);
			for k in 0..1_000i64 {
				let key = k % 37 * 13 + k / 37;
				cache.put(key, k);
				assert_eq!(*cache.get(&key).unwrap(), k);
			}
			check(&cache);
		}
	}

	#[test]
	fn eviction_bounds_size() {
		let mut cache = LruCache::new(3);
		for k in 0..10 {
			cache.put(k, k);
			check(&cache);
		}
		let resident = (0..10).filter(|k| cache.map.contains_key(k)).count();
		assert_eq!(resident, 3);
	}

	#[test]
	fn frequent_key_survives_scan() {
		let mut cache = LruCache::new(10);
		cache.put(0, 0);
		for _ in 0..5 {
			assert!(cache.get(&0).is_some());
		}
		for k in 1..400 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			if k % 12 == 0 {
				assert_eq!(*cache.get(&0).unwrap(), 0);
			}
		}
		check(&cache);
	}

	#[test]
	fn hot_set_beats_lru_under_scan_pollution() {
		// Hot keys 0..8 interleaved with one-hit wonders: strict LRU with capacity 10
		// never hits (reuse distance 23); W-TinyLFU must keep the hot set resident.
		let mut cache = LruCache::new(10);
		let mut hits = 0;
		let mut next_cold = 1_000;
		for round in 0..500 {
			for k in 0..8 {
				if cache.get(&k).is_some() {
					hits += 1;
				} else {
					cache.put(k, k);
				}
				for _ in 0..2 {
					if cache.get(&next_cold).is_none() {
						cache.put(next_cold, next_cold);
					}
					next_cold += 1;
				}
			}
			if round % 50 == 0 {
				check(&cache);
			}
		}
		assert!(hits > 2_000, "hits = {hits}");
	}

	#[test]
	fn randomized_operations_keep_invariants_and_values() {
		let mut cache = LruCache::new(17);
		let mut shadow = HashMap::new();
		let mut x: u64 = 88_172_645_463_325_252;
		for step in 0..50_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = (x % 64) as i64;
			if x & 0x300 == 0 {
				cache.put(key, step);
				shadow.insert(key, step);
				assert_eq!(*cache.get(&key).unwrap(), step);
			} else if let Some(v) = cache.get(&key) {
				assert_eq!(Some(v), shadow.get(&key));
			} else {
				cache.put(key, step);
				shadow.insert(key, step);
			}
			if step % 997 == 0 {
				check(&cache);
			}
		}
		check(&cache);
	}

	#[test]
	fn hill_climber_keeps_window_in_bounds() {
		let mut cache = LruCache::new(20);
		for k in 0..20_000i64 {
			// Phase change: frequency-friendly loop, then pure recency stream.
			let key = if k < 10_000 { (k * 7919) % 30 } else { k / 3 };
			if cache.get(&key).is_none() {
				cache.put(key, k);
			}
			assert!(cache.window_max >= 1 && cache.window_max + cache.protected_max <= 20);
		}
		check(&cache);
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
		// W-TinyLFU: window candidate 2 replaces main victim 1 only when more popular.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&2);
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
		cache.put(4, 40); // cold window candidate 3 is rejected
		cache.get(&4);
		cache.get(&4);
		cache.put(5, 50); // popular candidate 4 evicts victim 1
		cache.get(&5);
		cache.get(&5);
		cache.put(6, 60); // popular candidate 5 evicts victim 2

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
		cache.get(&3); // window candidate 3 now outranks probation victim 2
		cache.put(4, 40); // should evict key 2, not the hot key 1
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
		cache.get(&3); // window candidate 3 now outranks probation victim 1
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
		cache.put(3, 30); // cold candidate 2 rejected -> slot 1 reused for key 3
		cache.get(&3); // key 3 now outranks key 1
		cache.put(4, 40); // 3 admitted, victim 1 evicted -> slot 0 reused for key 4
		cache.get(&4);
		cache.get(&4); // key 4 now outranks key 3
		cache.put(5, 50); // 4 admitted, victim 3 evicted -> slot 1 reused AGAIN for key 5
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
