// Adaptive W-TinyLFU cache (Caffeine-style) behind the LruCache API.
// Arena-backed doubly-linked lists (window LRU, probation, protected) plus a 4-bit
// count-min frequency sketch for admission and a hill climber that tunes the window size.
// Small capacities degenerate to strict LRU. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
/// Below this capacity the cache behaves as a strict LRU.
const TINYLFU_MIN_CAPACITY: usize = 32;

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
struct Queue {
	head: usize,
	tail: usize,
	len: usize,
}

/// 4-bit count-min sketch with periodic halving (aging), as in Caffeine.
#[derive(Debug)]
struct Sketch {
	table: Vec<u64>,
	mask: usize,
	size: usize,
	sample: usize,
}

const SEEDS: [u64; 4] = [0xc3a5c85c97cb3127, 0xb492b66fbe98f273, 0x9ae16a3b2f90404f, 0xcbf29ce484222325];

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.max(16).next_power_of_two();
		Self { table: vec![0; width], mask: width - 1, size: 0, sample: 10 * capacity.max(16) }
	}

	fn slot(&self, h: u64, i: usize) -> (usize, u32) {
		let mut x = h.wrapping_add(SEEDS[i]).wrapping_mul(SEEDS[i]);
		x = x.wrapping_add(x >> 32);
		let shift = ((((h & 3) << 2) as u32) + i as u32) << 2;
		((x as usize) & self.mask, shift)
	}

	fn frequency(&self, h: u64) -> u32 {
		(0..4)
			.map(|i| {
				let (idx, s) = self.slot(h, i);
				((self.table[idx] >> s) & 0xF) as u32
			})
			.min()
			.unwrap_or(0)
	}

	fn increment(&mut self, h: u64) {
		let mut added = false;
		for i in 0..4 {
			let (idx, s) = self.slot(h, i);
			if (self.table[idx] >> s) & 0xF != 0xF {
				self.table[idx] += 1u64 << s;
				added = true;
			}
		}
		if added {
			self.size += 1;
			if self.size >= self.sample {
				let mut odd = 0usize;
				for c in self.table.iter_mut() {
					odd += (*c & 0x1111_1111_1111_1111).count_ones() as usize;
					*c = (*c >> 1) & 0x7777_7777_7777_7777;
				}
				self.size = self.size.saturating_sub(odd >> 2) / 2;
			}
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // reusable arena slots
	queues: [Queue; 3],     // window, probation, protected (head = LRU, tail = MRU)
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	tinylfu: bool,
	sketch: Sketch,
	last_miss: Option<u64>,
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
		let tinylfu = capacity >= TINYLFU_MIN_CAPACITY;
		let window_max = if tinylfu { (capacity / 100).max(1) } else { capacity };
		let protected_max = if tinylfu { (capacity - window_max) * 4 / 5 } else { 0 };
		let empty = Queue { head: NIL, tail: NIL, len: 0 };
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			queues: [empty; 3],
			capacity,
			window_max,
			protected_max,
			tinylfu,
			sketch: Sketch::new(if tinylfu { capacity } else { 1 }),
			last_miss: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -0.0625 * capacity as f64,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	fn hash_of(key: &K) -> u64 {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		h.finish()
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = self.map.get(key).copied();
		if self.tinylfu {
			let h = Self::hash_of(key);
			self.sketch.increment(h);
			if idx.is_some() {
				self.hits += 1;
			} else {
				self.misses += 1;
				self.last_miss = Some(h);
			}
			self.climb();
		}
		let idx = idx?;
		self.on_access(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys enter the window; evictions keep size <= capacity.
	pub fn put(&mut self, key: K, value: V) {
		let h = if self.tinylfu { Self::hash_of(&key) } else { 0 };
		if self.tinylfu && self.last_miss != Some(h) {
			self.sketch.increment(h);
		}
		self.last_miss = None;
		if let Some(idx) = self.map.get(&key).copied() {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let idx = match self.free.pop() {
			Some(i) => {
				let n = &mut self.nodes[i];
				n.key = key.clone();
				n.value = value;
				n.hash = h;
				i
			}
			None => {
				self.nodes.push(Node { key: key.clone(), value, hash: h, prev: NIL, next: NIL, queue: WINDOW });
				self.nodes.len() - 1
			}
		};
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		self.evict_entries();
		debug_assert!(self.map.len() <= self.capacity);
	}

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
		while self.queues[PROTECTED].len > self.protected_max {
			let h = self.queues[PROTECTED].head;
			self.move_to(h, PROBATION);
		}
	}

	/// Spill window overflow into probation, then evict until size <= capacity,
	/// letting each window candidate compete with the probation victim by frequency.
	fn evict_entries(&mut self) {
		let mut candidates = 0usize;
		while self.queues[WINDOW].len > self.window_max {
			let h = self.queues[WINDOW].head;
			self.move_to(h, PROBATION);
			candidates += 1;
		}
		let mut candidate = if candidates > 0 { self.queues[PROBATION].tail } else { NIL };
		while self.map.len() > self.capacity {
			let mut victim = self.queues[PROBATION].head;
			if victim == NIL {
				victim = self.queues[PROTECTED].head;
			}
			if victim == NIL {
				victim = self.queues[WINDOW].head;
			}
			let evict = if candidate == NIL || candidate == victim {
				candidate = NIL;
				victim
			} else {
				candidates -= 1;
				let prev = self.nodes[candidate].prev;
				let (ch, vh) = (self.nodes[candidate].hash, self.nodes[victim].hash);
				let e = if self.admit(ch, vh) { victim } else { candidate };
				candidate = if candidates > 0 && prev != e { prev } else { NIL };
				e
			};
			self.remove(evict);
		}
	}

	fn admit(&mut self, cand: u64, victim: u64) -> bool {
		let cf = self.sketch.frequency(cand);
		let vf = self.sketch.frequency(victim);
		if cf > vf {
			return true;
		}
		if cf <= 5 {
			return false;
		}
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		(self.rng & 127) == 0
	}

	/// Hill climber: periodically nudges the window size toward a higher hit rate.
	fn climb(&mut self) {
		let total = self.hits + self.misses;
		if total < self.sketch.sample {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= 0.05 {
			let s = 0.0625 * self.capacity as f64;
			if amount >= 0.0 { s } else { -s }
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		let a = amount as i64;
		if a > 0 {
			let q = (a as usize).min(self.capacity - 1 - self.window_max);
			self.window_max += q;
			self.protected_max = (self.capacity - self.window_max) * 4 / 5;
			self.demote_protected();
			for _ in 0..q {
				let mut n = self.queues[PROBATION].head;
				if n == NIL {
					n = self.queues[PROTECTED].head;
				}
				if n == NIL || self.queues[WINDOW].len >= self.window_max {
					break;
				}
				self.move_to(n, WINDOW);
			}
		} else if a < 0 {
			let q = ((-a) as usize).min(self.window_max - 1);
			self.window_max -= q;
			self.protected_max = (self.capacity - self.window_max) * 4 / 5;
			while self.queues[WINDOW].len > self.window_max {
				let h = self.queues[WINDOW].head;
				self.move_to(h, PROBATION);
			}
		}
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	fn move_to(&mut self, idx: usize, q: usize) {
		self.unlink(idx);
		self.push_back(idx, q);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (q, prev, next) = (self.nodes[idx].queue, self.nodes[idx].prev, self.nodes[idx].next);
		if prev == NIL {
			self.queues[q].head = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.queues[q].tail = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.queues[q].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end (tail) of list `q`.
	fn push_back(&mut self, idx: usize, q: usize) {
		let tail = self.queues[q].tail;
		let n = &mut self.nodes[idx];
		n.prev = tail;
		n.next = NIL;
		n.queue = q;
		if tail == NIL {
			self.queues[q].head = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.queues[q].tail = idx;
		self.queues[q].len += 1;
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
	fn large_capacity_bounded_and_put_then_get_hits() {
		let mut cache = LruCache::new(100);
		let mut x: u64 = 12345;
		for i in 0..200_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = if i % 3 == 0 { x % 50 } else { x % 5000 };
			if cache.get(&k).is_none() {
				cache.put(k, i);
				assert!(cache.get(&k).is_some());
			}
			assert!(cache.map.len() <= 100);
			let total: usize = cache.queues.iter().map(|q| q.len).sum();
			assert_eq!(total, cache.map.len());
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
