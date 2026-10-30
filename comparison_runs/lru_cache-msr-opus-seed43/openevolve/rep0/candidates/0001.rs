// Adaptive W-TinyLFU cache (window LRU + segmented main LRU + count-min frequency sketch + hill climber),
// using a HashMap + arena-based doubly-linked lists. Arena slots are reused in place on eviction.
// Small capacities degenerate to strict LRU (the window spans the whole cache). No unsafe, std only.

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
const ADAPTIVE_MIN_CAPACITY: usize = 16;

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
	head: usize, // LRU
	tail: usize, // MRU
	len: usize,
}

/// Count-min sketch of 4-bit (saturating at 15) counters with periodic halving (aging).
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = (capacity.max(8) * 8).next_power_of_two();
		Self { table: vec![0; width], mask: width - 1, additions: 0, sample_size: capacity.max(1) * 10 }
	}

	#[inline]
	fn index(&self, hash: u64, i: u64) -> usize {
		let mut x = hash.wrapping_add(i.wrapping_mul(0x9E37_79B9_7F4A_7C15));
		x = (x ^ (x >> 31)).wrapping_mul(0xD6E8_FEB8_6659_FD93);
		x ^= x >> 32;
		(x as usize) & self.mask
	}

	fn frequency(&self, hash: u64) -> u8 {
		(0..4).map(|i| self.table[self.index(hash, i)]).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let mut added = false;
		for i in 0..4 {
			let j = self.index(hash, i);
			if self.table[j] < 15 {
				self.table[j] += 1;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample_size {
				self.table.iter_mut().for_each(|c| *c >>= 1);
				self.additions /= 2;
			}
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	queues: [Queue; 3],     // window, probation, protected
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	adaptive: bool,
	sketch: Sketch,
	last_miss: Option<u64>, // hash of the last get-miss (already recorded in the sketch)
	hits: usize,
	misses: usize,
	step: f64,
	prev_hit_rate: f64,
	rng: u64,
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
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
		let window_max = if adaptive { (capacity / 100).max(1) } else { capacity };
		let protected_max = if adaptive { (capacity - window_max) * 4 / 5 } else { 0 };
		let empty = Queue { head: NIL, tail: NIL, len: 0 };
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			queues: [empty; 3],
			capacity,
			window_max,
			protected_max,
			adaptive,
			sketch: Sketch::new(if adaptive { capacity } else { 1 }),
			last_miss: None,
			hits: 0,
			misses: 0,
			step: -0.0625 * capacity as f64,
			prev_hit_rate: 0.0,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		if self.adaptive {
			let h = hash_key(key);
			self.sketch.increment(h);
			let full = self.map.len() >= self.capacity;
			match found {
				Some(_) => {
					self.last_miss = None;
					self.hits += full as usize;
				}
				None => {
					self.last_miss = Some(h);
					self.misses += full as usize;
				}
			}
		}
		let idx = found?;
		self.on_hit(idx);
		self.climb();
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts an entry chosen by the W-TinyLFU policy.
	pub fn put(&mut self, key: K, value: V) {
		let h = if self.adaptive { hash_key(&key) } else { 0 };
		let recorded = self.adaptive && self.last_miss.take() == Some(h);
		if self.adaptive && !recorded {
			self.sketch.increment(h);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		if self.adaptive && !recorded && self.map.len() >= self.capacity {
			self.misses += 1;
		}
		let idx = if self.nodes.len() < self.capacity {
			self.nodes.push(Node { key: key.clone(), value, hash: h, prev: NIL, next: NIL, queue: WINDOW });
			self.nodes.len() - 1
		} else {
			let slot = self.evict();
			let node = &mut self.nodes[slot];
			node.key = key.clone();
			node.value = value;
			node.hash = h;
			slot
		};
		self.attach(idx, WINDOW);
		self.map.insert(key, idx);
		while self.queues[WINDOW].len > self.window_max {
			let head = self.queues[WINDOW].head;
			self.move_to(head, PROBATION);
		}
		self.climb();
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Removes one entry, returning its (detached) arena slot for reuse.
	fn evict(&mut self) -> usize {
		let main_victim = [PROBATION, PROTECTED].iter().map(|&q| self.queues[q].head).find(|&i| i != NIL);
		let candidate = self.queues[WINDOW].head;
		let victim = match main_victim {
			Some(v) if candidate == NIL || self.queues[WINDOW].len < self.window_max => v,
			Some(v) => {
				if self.admit(candidate, v) {
					self.move_to(candidate, PROBATION);
					v
				} else {
					candidate
				}
			}
			None => candidate,
		};
		self.detach(victim);
		self.map.remove(&self.nodes[victim].key);
		victim
	}

	/// TinyLFU admission: the window candidate replaces the main victim only if it is more frequent.
	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let cf = self.sketch.frequency(self.nodes[candidate].hash);
		let vf = self.sketch.frequency(self.nodes[victim].hash);
		if cf > vf {
			return true;
		}
		if cf < 6 {
			return false;
		}
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		(self.rng & 127) == 0
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].queue {
			WINDOW => self.move_to(idx, WINDOW),
			_ => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
		}
	}

	fn demote_protected(&mut self) {
		while self.queues[PROTECTED].len > self.protected_max {
			let head = self.queues[PROTECTED].head;
			self.move_to(head, PROBATION);
		}
	}

	/// Hill climber: periodically resize the window toward a higher sampled hit rate.
	fn climb(&mut self) {
		let total = self.hits + self.misses;
		if !self.adaptive || total < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * sign
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		let k = amount.abs() as usize;
		if amount > 0.0 {
			let k = k.min(self.protected_max);
			self.protected_max -= k;
			self.window_max += k;
			for _ in 0..k {
				let src = [PROBATION, PROTECTED].iter().map(|&q| self.queues[q].head).find(|&i| i != NIL);
				match src {
					Some(i) => self.move_to(i, WINDOW),
					None => break,
				}
			}
		} else {
			let k = k.min(self.window_max - 1);
			self.window_max -= k;
			self.protected_max += k;
			while self.queues[WINDOW].len > self.window_max {
				let head = self.queues[WINDOW].head;
				self.move_to(head, PROBATION);
			}
		}
		self.demote_protected();
	}

	fn move_to(&mut self, idx: usize, queue: usize) {
		self.detach(idx);
		self.attach(idx, queue);
	}

	/// Remove node from its current queue.
	fn detach(&mut self, idx: usize) {
		let (prev, next, queue) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		match prev {
			NIL => self.queues[queue].head = next,
			p => self.nodes[p].next = next,
		}
		match next {
			NIL => self.queues[queue].tail = prev,
			n => self.nodes[n].prev = prev,
		}
		self.queues[queue].len -= 1;
	}

	/// Append node to the MRU end of `queue`.
	fn attach(&mut self, idx: usize, queue: usize) {
		let tail = self.queues[queue].tail;
		let node = &mut self.nodes[idx];
		node.prev = tail;
		node.next = NIL;
		node.queue = queue;
		match tail {
			NIL => self.queues[queue].head = idx,
			t => self.nodes[t].next = idx,
		}
		self.queues[queue].tail = idx;
		self.queues[queue].len += 1;
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
	fn adaptive_contract_holds() {
		let cap = 100;
		let mut cache = LruCache::new(cap);
		let mut x: u64 = 7;
		for i in 0..200_000i64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = if i % 3 == 0 { (x % 50) as i64 } else { (x % 5000) as i64 };
			if cache.get(&k).is_none() {
				cache.put(k, k * 2);
				assert_eq!(*cache.get(&k).unwrap(), k * 2, "freshly put key must be resident");
			}
			assert!(cache.map.len() <= cap);
		}
		for (k, &i) in cache.map.iter() {
			assert_eq!(*k, cache.nodes[i].key);
		}
		let total: usize = cache.queues.iter().map(|q| q.len).sum();
		assert_eq!(total, cache.map.len());
	}

	#[test]
	fn frequent_keys_survive_scan() {
		let mut cache = LruCache::new(64);
		for _ in 0..20 {
			for k in 0..32i64 {
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
		}
		for k in 1000..10_000i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let survivors = (0..32i64).filter(|k| cache.get(k).is_some()).count();
		assert!(survivors >= 28, "hot set should resist a one-pass scan, got {survivors}");
	}
}
