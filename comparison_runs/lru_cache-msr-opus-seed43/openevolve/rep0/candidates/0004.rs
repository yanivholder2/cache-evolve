// Adaptive W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// Layout: a small LRU *window* admits every new key; keys leaving the window must win a
// frequency duel (TinyLFU count-min sketch) against the LRU victim of the *main* segmented
// LRU (probation + protected) to stay resident. A hill climber periodically resizes the
// window to track whether the workload favours recency or frequency.
// Nodes live in a Vec arena linked by indices; freed slots are recycled via a free list.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize, // LRU
	tail: usize, // MRU
	len: usize,
}

const EMPTY_LIST: List = List { head: NIL, tail: NIL, len: 0 };

/// 4-bit count-min sketch with periodic aging (halving), as in Caffeine's FrequencySketch.
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u64>,
	mask: usize,
	size: usize,
	sample_size: usize,
}

const SEEDS: [u64; 4] = [0xc3a5c85c97cb3127, 0xb492b66fbe98f273, 0x9ae16a3b2f90404f, 0xcbf29ce484222325];

impl FrequencySketch {
	fn new(capacity: usize) -> Self {
		let len = capacity.max(16).next_power_of_two();
		Self { table: vec![0; len], mask: len - 1, size: 0, sample_size: 10 * capacity.max(16) }
	}

	fn index(&self, hash: u64, i: usize) -> usize {
		let mut h = hash.wrapping_add(SEEDS[i]).wrapping_mul(SEEDS[i]);
		h = h.wrapping_add(h >> 32);
		(h as usize) & self.mask
	}

	fn frequency(&self, hash: u64) -> u64 {
		let start = ((hash & 3) << 2) as usize;
		(0..4)
			.map(|i| (self.table[self.index(hash, i)] >> ((start + i) << 2)) & 0xF)
			.min()
			.unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let start = ((hash & 3) << 2) as usize;
		let mut added = false;
		for i in 0..4 {
			let idx = self.index(hash, i);
			let shift = (start + i) << 2;
			if (self.table[idx] >> shift) & 0xF != 0xF {
				self.table[idx] += 1u64 << shift;
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
		for slot in self.table.iter_mut() {
			odd += (*slot & 0x1111_1111_1111_1111).count_ones() as usize;
			*slot = (*slot >> 1) & 0x7777_7777_7777_7777;
		}
		self.size = self.size.saturating_sub(odd >> 2) >> 1;
	}
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	let mut x = h.finish();
	x = (x ^ (x >> 33)).wrapping_mul(0xff51afd7ed558ccd);
	x ^ (x >> 33)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: FrequencySketch,
	last_miss: Option<u64>,
	// hill climber state
	hits: usize,
	misses: usize,
	sample_size: usize,
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
		// Tiny caches degrade to exact LRU (window = everything, no climbing): sketch noise
		// dominates at that size and strict recency is the expected contract there.
		let small = capacity <= 16;
		let window_max = if small { capacity } else { (capacity - (capacity as f64 * 0.99) as usize).max(1) };
		let protected_max = if small { 0 } else { ((capacity - window_max) as f64 * 0.8) as usize };
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max,
			sketch: FrequencySketch::new(capacity),
			last_miss: None,
			hits: 0,
			misses: 0,
			sample_size: if small { usize::MAX } else { 10 * capacity },
			prev_hit_rate: 0.0,
			step: -0.0625 * capacity as f64,
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
				self.hits += 1;
				self.on_hit(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				self.misses += 1;
				self.last_miss = Some(hash);
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; overflow is resolved by TinyLFU admission.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_of(&key);
		if self.last_miss != Some(hash) {
			self.sketch.increment(hash);
		}
		self.last_miss = None;
		let node = Node { key: key.clone(), value, hash, prev: NIL, next: NIL, queue: WINDOW };
		let idx = match self.free.pop() {
			Some(i) => {
				self.nodes[i] = node;
				i
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		self.evict();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			PROBATION => {
				self.unlink(idx);
				self.push_back(PROTECTED, idx);
				self.demote_protected();
			}
			q => {
				self.unlink(idx);
				self.push_back(q, idx);
			}
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let idx = self.lists[PROTECTED].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn evict(&mut self) {
		// Drain window overflow into main; when over capacity each candidate duels main's LRU victim.
		while self.lists[WINDOW].len > self.window_max {
			let cand = self.lists[WINDOW].head;
			if self.map.len() > self.capacity {
				let victim = if self.lists[PROBATION].len > 0 {
					self.lists[PROBATION].head
				} else {
					self.lists[PROTECTED].head
				};
				if victim == NIL
					|| self.sketch.frequency(self.nodes[cand].hash) <= self.sketch.frequency(self.nodes[victim].hash)
				{
					self.remove(cand);
					continue;
				}
				self.remove(victim);
			}
			self.unlink(cand);
			self.push_back(PROBATION, cand);
		}
		while self.map.len() > self.capacity {
			let victim = [PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.lists[q].head)
				.find(|&h| h != NIL)
				.expect("over-capacity cache is non-empty");
			self.remove(victim);
		}
	}

	/// Caffeine's hill climber: grow or shrink the window in the direction that improved hit rate.
	fn climb(&mut self) {
		let requests = self.hits + self.misses;
		if requests < self.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * amount.signum()
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		let adj = amount as i64;
		if adj > 0 {
			let q = (adj as usize).min(self.protected_max);
			self.protected_max -= q;
			self.window_max += q;
			self.demote_protected();
		} else if adj < 0 {
			let q = ((-adj) as usize).min(self.window_max - 1);
			self.protected_max += q;
			self.window_max -= q;
		}
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Remove node from whichever list it is in.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = (self.nodes[idx].prev, self.nodes[idx].next, self.nodes[idx].queue);
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

	/// Append node to the MRU end of list `q`.
	fn push_back(&mut self, q: usize, idx: usize) {
		let tail = self.lists[q].tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		self.nodes[idx].queue = q;
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

	fn check_links<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		let mut total = 0;
		for q in 0..3 {
			let mut n = 0;
			let mut cur = c.lists[q].head;
			let mut prev = NIL;
			while cur != NIL {
				assert_eq!(c.nodes[cur].queue, q);
				assert_eq!(c.nodes[cur].prev, prev);
				prev = cur;
				cur = c.nodes[cur].next;
				n += 1;
			}
			assert_eq!(prev, c.lists[q].tail);
			assert_eq!(n, c.lists[q].len);
			total += n;
		}
		assert_eq!(total, c.map.len());
		assert!(c.map.len() <= c.capacity);
	}

	#[test]
	fn newest_key_always_resident() {
		for cap in [1usize, 2, 3, 7, 17, 100, 1000] {
			let mut cache = LruCache::new(cap);
			for k in 0..2000i64 {
				let key = (k * 7919) % 613;
				if cache.get(&key).is_none() {
					cache.put(key, k);
				}
				assert!(cache.get(&key).is_some());
				check_links(&cache);
			}
		}
	}

	#[test]
	fn capacity_is_respected_under_random_load() {
		let mut cache = LruCache::new(50);
		let mut x: u64 = 1;
		for _ in 0..100_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = (x % 1000) as i64;
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
			assert!(cache.map.len() <= 50);
		}
		check_links(&cache);
	}

	#[test]
	fn frequent_keys_survive_scan() {
		let mut cache = LruCache::new(100);
		for _ in 0..20 {
			for k in 0..50i64 {
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
		}
		for k in 1000..5000i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let hot = (0..50i64).filter(|k| cache.get(k).is_some()).count();
		assert!(hot >= 45, "hot keys retained: {hot}");
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
