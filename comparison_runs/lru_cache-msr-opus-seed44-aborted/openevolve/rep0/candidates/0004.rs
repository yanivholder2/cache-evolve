// W-TinyLFU cache (Caffeine-style) behind the LruCache name.
// - Admission window (LRU, starts at 1% of capacity) absorbs recency bursts.
// - Main space = SLRU (probation + protected 80%) guarded by a TinyLFU admission filter
//   (4-bit count-min sketch, conservative update, periodic halving).
// - Hill climber adapts the window/main split from the observed hit rate.
// For tiny capacities (< SMALL_CAPACITY) the window spans the whole cache => strict LRU.
// Arena of nodes with intrusive doubly-linked lists; O(1) operations. No unsafe, std only.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
/// Below this capacity the cache behaves as strict LRU.
const SMALL_CAPACITY: usize = 32;

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

const EMPTY_LIST: List = List {
	head: NIL,
	tail: NIL,
	len: 0,
};

/// 4-bit-style count-min sketch (4 rows, u8 counters saturating at 15) with aging.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity
			.max(16)
			.checked_next_power_of_two()
			.unwrap_or(1 << 22)
			.saturating_mul(4)
			.min(1 << 22);
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample: capacity.saturating_mul(10).max(160),
		}
	}

	#[inline]
	fn slot(&self, hash: u64, row: usize) -> usize {
		const SEEDS: [u64; 4] = [
			0x9E37_79B9_7F4A_7C15,
			0xC2B2_AE3D_27D4_EB4F,
			0x1656_67B1_9E37_79F9,
			0x27D4_EB2F_1656_67C5,
		];
		let mut x = (hash ^ SEEDS[row]).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		x ^= x >> 29;
		x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
		x ^= x >> 32;
		row * self.width + ((x as usize) & (self.width - 1))
	}

	fn frequency(&self, hash: u64) -> u8 {
		(0..4)
			.map(|r| self.table[self.slot(hash, r)])
			.min()
			.unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let idx = [
			self.slot(hash, 0),
			self.slot(hash, 1),
			self.slot(hash, 2),
			self.slot(hash, 3),
		];
		let min = idx.iter().map(|&i| self.table[i]).min().unwrap_or(0);
		if min < 15 {
			// Conservative update: only raise the counters at the current minimum.
			for &i in idx.iter() {
				if self.table[i] == min {
					self.table[i] += 1;
				}
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
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
	adaptive: bool,
	last_miss: Option<u64>, // hash of the key that just missed in get (avoid double counting)
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
		let adaptive = capacity >= SMALL_CAPACITY;
		let (window_max, protected_max) = if adaptive {
			let window = (capacity / 100).max(1);
			(window, (capacity - window) * 4 / 5)
		} else {
			// Whole cache is the window: exact LRU semantics.
			(capacity, 0)
		};
		Some(Self {
			map: HashMap::with_capacity(capacity.saturating_add(1)),
			nodes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max,
			sketch: Sketch::new(capacity),
			adaptive,
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

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits += 1;
				self.on_access(idx);
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

	/// Inserts or updates `key`. New keys enter the admission window; overflow is
	/// resolved by the TinyLFU admission filter against the main-space victim.
	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		// Count the access unless the preceding get-miss already counted it.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let idx = match self.free.pop() {
			Some(i) => {
				let n = &mut self.nodes[i];
				n.key = key.clone();
				n.value = value;
				n.hash = hash;
				i
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
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Promote on hit: window/protected -> MRU of same queue; probation -> protected.
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

	fn main_victim(&self) -> usize {
		if self.lists[PROBATION].head != NIL {
			self.lists[PROBATION].head
		} else {
			self.lists[PROTECTED].head
		}
	}

	/// Drop an (already unlinked) node from the cache and recycle its slot.
	fn discard(&mut self, idx: usize) {
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	fn evict(&mut self) {
		// Window overflow: candidates move to main, competing with the main victim when full.
		while self.lists[WINDOW].len > self.window_max {
			let cand = self.lists[WINDOW].head;
			self.unlink(cand);
			if self.map.len() <= self.capacity {
				self.push_back(PROBATION, cand);
				continue;
			}
			let victim = self.main_victim();
			let admit = victim != NIL && {
				let (c, v) = (self.nodes[cand].hash, self.nodes[victim].hash);
				self.admit(c, v)
			};
			if admit {
				self.unlink(victim);
				self.discard(victim);
				self.push_back(PROBATION, cand);
			} else {
				self.discard(cand);
			}
		}
		// Window below its target (it is growing): evict straight from main.
		while self.map.len() > self.capacity {
			let mut v = self.main_victim();
			if v == NIL {
				v = self.lists[WINDOW].head;
			}
			self.unlink(v);
			self.discard(v);
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
		// Occasionally admit warm candidates to avoid getting stuck (Caffeine's hash-flood guard).
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		self.rng & 127 == 0
	}

	/// Hill-climbing adaptation of the window / protected split (Caffeine parameters).
	fn climb(&mut self) {
		if !self.adaptive {
			return;
		}
		let total = self.hits + self.misses;
		if total < self.sketch.sample {
			return;
		}
		let rate = self.hits as f64 / total as f64;
		let change = rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * amount.signum()
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = rate;
		self.hits = 0;
		self.misses = 0;
		if amount > 0.0 {
			let quota = (amount as usize).min(self.protected_max);
			self.protected_max -= quota;
			self.window_max += quota;
			self.demote_protected();
		} else if amount < 0.0 {
			let quota = ((-amount) as usize).min(self.window_max.saturating_sub(1));
			self.protected_max += quota;
			self.window_max -= quota;
			while self.lists[WINDOW].len > self.window_max {
				let h = self.lists[WINDOW].head;
				self.unlink(h);
				self.push_back(PROBATION, h);
			}
		}
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
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
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.queue = q;
		}
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
	fn large_capacity_bound_and_recent_put_visible() {
		let mut cache = LruCache::new(100);
		for i in 0..20_000i64 {
			let k = (i * 7919) % 1000;
			if cache.get(&k).is_none() {
				cache.put(k, i);
				assert!(cache.get(&k).is_some());
			}
			assert!(cache.map.len() <= 100);
		}
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
		for k in 1000..1300i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let hits = (0..50i64).filter(|k| cache.get(k).is_some()).count();
		assert!(hits >= 45, "hot keys were flushed by a scan: {hits}");
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
