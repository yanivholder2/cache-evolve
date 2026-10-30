// W-TinyLFU cache (Caffeine-style) behind the LruCache API.
// A small LRU admission window feeds a segmented-LRU main space (probation + protected).
// Once full, the window's LRU candidate must beat the probation LRU victim on estimated
// frequency (4-row Count-Min sketch with periodic halving) to be admitted; otherwise it is
// dropped, so one-hit wonders and scans cannot flush the frequently used working set.
// A hill climber retunes the window/main split from the sampled hit rate, so recency-heavy
// phases still get a large LRU window. One arena, three intrusive lists, std only.

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

/// Count-Min sketch: 4 rows of saturating (max 15) counters, halved every `sample` additions.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample: usize,
}

const SEEDS: [u64; 4] = [
	0x97cb_3127_c2d5_e8f1,
	0xb492_b66f_be98_f273,
	0x9ae1_6a3b_2f90_404f,
	0xcbf2_9ce4_8422_2325,
];

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = (capacity.max(16) * 4).next_power_of_two();
		Self { table: vec![0; width * 4], width, additions: 0, sample: capacity.max(16) * 10 }
	}

	#[inline]
	fn index(&self, h: u64, row: usize) -> usize {
		let mut x = (h ^ SEEDS[row]).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		x ^= x >> 32;
		row * self.width + ((x as usize) & (self.width - 1))
	}

	fn frequency(&self, h: u64) -> u8 {
		(0..4).map(|r| self.table[self.index(h, r)]).min().unwrap_or(0)
	}

	fn increment(&mut self, h: u64) {
		let mut added = false;
		for r in 0..4 {
			let i = self.index(h, r);
			if self.table[i] < 15 {
				self.table[i] += 1;
				added = true;
			}
		}
		if added {
			self.additions += 1;
			if self.additions >= self.sample {
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
	free: Vec<usize>,       // reusable arena slots
	lists: [List; 3],       // window, probation, protected
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	pending: Option<u64>, // hash of the last missed get (avoids double counting on put)
	hits: usize,
	misses: usize,
	prev_hit_rate: f64,
	step: f64,
	rng: u64,
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut s = DefaultHasher::new();
	key.hash(&mut s);
	s.finish()
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
		let main = (capacity as f64 * 0.99) as usize;
		let window_max = (capacity - main).max(1);
		let main = capacity - window_max;
		let empty = List { head: NIL, tail: NIL, len: 0 };
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [empty; 3],
			capacity,
			window_max,
			protected_max: (main as f64 * 0.8) as usize,
			sketch: Sketch::new(capacity),
			pending: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -0.0625 * capacity as f64,
			rng: 0x2545_f491_4f6c_dd1d,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access in the sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = hash_of(key);
		self.sketch.increment(h);
		let found = self.map.get(key).copied();
		let full = self.map.len() >= self.capacity;
		match found {
			Some(idx) => {
				self.pending = None;
				if full {
					self.hits += 1;
				}
				self.on_access(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				self.pending = Some(h);
				if full {
					self.misses += 1;
				}
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. A new key enters the window; overflow is resolved by
	/// frequency-based admission between the window candidate and the main victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let h = self.nodes[idx].hash;
			self.sketch.increment(h);
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let h = hash_of(&key);
		if self.pending != Some(h) {
			self.sketch.increment(h);
		}
		self.pending = None;
		let node = Node { key: key.clone(), value, hash: h, prev: NIL, next: NIL, queue: WINDOW };
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
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		self.evict();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn on_access(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		self.unlink(idx);
		if q == PROBATION {
			self.push_back(PROTECTED, idx);
			self.demote_protected();
		} else {
			self.push_back(q, idx);
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED].len > self.protected_max {
			let h = self.lists[PROTECTED].head;
			self.unlink(h);
			self.push_back(PROBATION, h);
		}
	}

	fn evict(&mut self) {
		while self.lists[WINDOW].len > self.window_max {
			let cand = self.lists[WINDOW].head;
			if self.map.len() <= self.capacity {
				self.unlink(cand);
				self.push_back(PROBATION, cand);
				continue;
			}
			let victim = if self.lists[PROBATION].head != NIL {
				self.lists[PROBATION].head
			} else {
				self.lists[PROTECTED].head
			};
			let admitted = victim != NIL && {
				let (ch, vh) = (self.nodes[cand].hash, self.nodes[victim].hash);
				self.admit(ch, vh)
			};
			if admitted {
				self.remove(victim);
				self.unlink(cand);
				self.push_back(PROBATION, cand);
			} else {
				self.remove(cand);
			}
		}
		while self.map.len() > self.capacity {
			let victim = [PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.lists[q].head)
				.find(|&i| i != NIL)
				.expect("over-capacity cache is non-empty");
			self.remove(victim);
		}
	}

	fn admit(&mut self, cand: u64, victim: u64) -> bool {
		let cf = self.sketch.frequency(cand);
		let vf = self.sketch.frequency(victim);
		if cf > vf {
			return true;
		}
		if cf <= 6 {
			return false;
		}
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		self.rng & 127 == 0
	}

	fn remove(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Hill-climb the window size toward a higher sampled hit rate.
	fn climb(&mut self) {
		let requests = self.hits + self.misses;
		if requests < self.sketch.sample {
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
			self.increase_window(adj as usize);
		} else if adj < 0 {
			self.decrease_window((-adj) as usize);
		}
	}

	fn increase_window(&mut self, amount: usize) {
		let mut quota = amount.min(self.protected_max);
		self.protected_max -= quota;
		self.window_max += quota;
		self.demote_protected();
		while quota > 0 {
			let cand = if self.lists[PROBATION].head != NIL {
				self.lists[PROBATION].head
			} else {
				self.lists[PROTECTED].head
			};
			if cand == NIL {
				break;
			}
			self.unlink(cand);
			self.push_back(WINDOW, cand);
			quota -= 1;
		}
		self.protected_max += quota;
		self.window_max -= quota;
	}

	fn decrease_window(&mut self, amount: usize) {
		let mut quota = amount.min(self.window_max.saturating_sub(1));
		self.protected_max += quota;
		self.window_max -= quota;
		while quota > 0 {
			let cand = self.lists[WINDOW].head;
			if cand == NIL {
				break;
			}
			self.unlink(cand);
			self.push_back(PROBATION, cand);
			quota -= 1;
		}
		self.protected_max -= quota;
		self.window_max += quota;
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = (self.nodes[idx].prev, self.nodes[idx].next, self.nodes[idx].queue);
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

	/// Append node to the MRU end of list `q`.
	fn push_back(&mut self, q: usize, idx: usize) {
		let tail = self.lists[q].tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = NIL;
		self.nodes[idx].queue = q;
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.lists[q].head = idx;
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
	fn eviction_respects_capacity() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // one older key must go; the newest key stays resident

		assert_eq!(cache.map.len(), 2);
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
	fn eviction_chain_spares_frequent_keys() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..2 {
			cache.get(&1);
			cache.get(&2);
		}
		cache.put(4, 40); // one-hit wonders 3, 4, 5 are rejected in turn
		cache.put(5, 50);
		cache.put(6, 60);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&6).unwrap(), 60);
		assert!(cache.get(&3).is_none());
		assert!(cache.get(&4).is_none());
		assert!(cache.get(&5).is_none());
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
		cache.put(4, 40); // the hot key 1 must not be the one evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(cache.map.len(), 3);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // 2 is protected; window candidate 3 duels probation's 1
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(cache.map.len(), 3);
	}

	#[test]
	fn slot_reuse_keeps_arena_bounded() {
		let mut cache = LruCache::new(2);
		for k in 0..100i64 {
			cache.put(k, k * 10);
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
		assert!(cache.nodes.len() <= 3);
		assert_eq!(cache.map.len(), 2);
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

	fn check_lists<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		let total: usize = c.lists.iter().map(|l| l.len).sum();
		assert_eq!(total, c.map.len());
		assert!(c.map.len() <= c.capacity);
		for (q, l) in c.lists.iter().enumerate() {
			let (mut n, mut i, mut prev) = (0usize, l.head, NIL);
			while i != NIL {
				assert_eq!(c.nodes[i].queue, q);
				assert_eq!(c.nodes[i].prev, prev);
				assert_eq!(c.map.get(&c.nodes[i].key), Some(&i));
				prev = i;
				i = c.nodes[i].next;
				n += 1;
			}
			assert_eq!(prev, l.tail);
			assert_eq!(n, l.len);
		}
	}

	#[test]
	fn newest_key_always_resident() {
		for cap in 1..20 {
			let mut cache = LruCache::new(cap);
			for k in 0..500i64 {
				let key = (k * 7919) % 97;
				if cache.get(&key).is_none() {
					cache.put(key, key * 2);
				}
				assert_eq!(*cache.get(&key).unwrap(), key * 2);
				check_lists(&cache);
			}
		}
	}

	#[test]
	fn frequent_key_survives_scan() {
		let mut cache = LruCache::new(10);
		for _ in 0..20 {
			if cache.get(&0).is_none() {
				cache.put(0, 0);
			}
		}
		for k in 1..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		assert_eq!(*cache.get(&0).unwrap(), 0);
	}

	#[test]
	fn stress_invariants_and_values() {
		let mut cache = LruCache::new(64);
		let mut x: u64 = 12345;
		let mut truth: HashMap<u64, u64> = HashMap::new();
		for step in 0..200_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = if step % 3 == 0 { x % 50 } else { x % 1000 };
			match cache.get(&key).copied() {
				Some(v) => assert_eq!(Some(&v), truth.get(&key)),
				None => {
					cache.put(key, step);
					truth.insert(key, step);
				}
			}
			assert!(cache.map.len() <= 64);
			if step % 997 == 0 {
				check_lists(&cache);
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
