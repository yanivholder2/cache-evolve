// W-TinyLFU cache behind the LruCache API (O(1) per operation, no unsafe, std only).
//
// * Window: small LRU absorbing new keys (captures recency / bursts).
// * Main: segmented LRU = probation + protected (80% of main).
// * Admission: an entry leaving the window enters main only if a 4-bit count-min
//   sketch (periodically halved = aging) estimates it more popular than main's victim.
// * Window size adapts online via Caffeine-style hill climbing on sampled hit rate.
// Tiny caches (< MIN_ADAPTIVE_CAPACITY) degenerate to strict LRU (window = whole cache).

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const MIN_ADAPTIVE_CAPACITY: usize = 16;
const STEP_PERCENT: f64 = 0.0625;
const STEP_DECAY: f64 = 0.98;
const RESTART_THRESHOLD: f64 = 0.05;

#[inline]
fn mix64(mut x: u64) -> u64 {
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	x ^ (x >> 33)
}

/// Fast multiplicative hasher with a strong finalizer (used for map and sketch).
#[derive(Debug, Default, Clone, Copy)]
struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(0x517c_c1b7_2722_0a95);
	}
}

impl Hasher for FxHasher {
	fn write(&mut self, bytes: &[u8]) {
		for chunk in bytes.chunks(8) {
			let mut buf = [0u8; 8];
			buf[..chunk.len()].copy_from_slice(chunk);
			self.add(u64::from_le_bytes(buf));
		}
	}
	fn write_u8(&mut self, n: u8) {
		self.add(n as u64);
	}
	fn write_u16(&mut self, n: u16) {
		self.add(n as u64);
	}
	fn write_u32(&mut self, n: u32) {
		self.add(n as u64);
	}
	fn write_u64(&mut self, n: u64) {
		self.add(n);
	}
	fn write_usize(&mut self, n: usize) {
		self.add(n as u64);
	}
	fn finish(&self) -> u64 {
		mix64(self.hash)
	}
}

#[inline]
fn hash_key<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut h = FxHasher::default();
	key.hash(&mut h);
	h.finish()
}

/// 4-row count-min sketch of saturating 4-bit counters with periodic halving.
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u8>,
	width: usize,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl FrequencySketch {
	fn new(capacity: usize) -> Self {
		let width = capacity
			.max(16)
			.next_power_of_two()
			.saturating_mul(4)
			.min(1 << 22);
		Self {
			table: vec![0; width * 4],
			width,
			mask: width - 1,
			additions: 0,
			sample_size: capacity.max(16).saturating_mul(10),
		}
	}

	#[inline]
	fn slots(&self, hash: u64) -> [usize; 4] {
		let step = mix64(hash ^ 0x9e37_79b9_7f4a_7c15) | 1;
		let mut out = [0usize; 4];
		for (i, slot) in out.iter_mut().enumerate() {
			let h = hash.wrapping_add(step.wrapping_mul(i as u64));
			*slot = i * self.width + ((h as usize) & self.mask);
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&s| self.table[s]).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let slots = self.slots(hash);
		let min = slots.iter().map(|&s| self.table[s]).min().unwrap_or(0);
		if min >= 15 {
			return;
		}
		// Conservative update: only bump the counters that hold the minimum.
		for &s in &slots {
			if self.table[s] == min {
				self.table[s] += 1;
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

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	region: u8,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize, BuildHasherDefault<FxHasher>>, // key -> node index
	nodes: Vec<Node<K, V>>,                               // arena
	free: Vec<usize>,                                     // freed arena slots
	lists: [List; 3],                                     // window, probation, protected
	capacity: usize,
	adaptive: bool,
	window_max: usize,
	protected_max: usize,
	window_target: f64,
	sketch: FrequencySketch,
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
		let slots = capacity.saturating_add(1);
		let mut cache = Self {
			map: HashMap::with_capacity_and_hasher(slots, BuildHasherDefault::default()),
			nodes: Vec::with_capacity(slots),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			adaptive: capacity >= MIN_ADAPTIVE_CAPACITY,
			window_max: capacity,
			protected_max: 0,
			window_target: capacity as f64,
			sketch: FrequencySketch::new(capacity),
			hits: 0,
			misses: 0,
			sample_size: capacity.saturating_mul(10),
			prev_hit_rate: 0.0,
			step: -STEP_PERCENT * capacity as f64,
		};
		if cache.adaptive {
			cache.set_window(capacity as f64 * 0.01);
		}
		Some(cache)
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present (recording the access in the sketch).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		if self.adaptive {
			self.sketch.increment(hash_key(key));
		}
		let idx = match self.map.get(key) {
			Some(&i) => i,
			None => {
				self.misses += 1;
				self.climb();
				return None;
			}
		};
		self.hits += 1;
		self.on_hit(idx);
		self.climb();
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys enter the window; overflow goes through admission.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let idx = self.alloc(key.clone(), value);
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		self.maintain();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		if let Some(idx) = self.free.pop() {
			let n = &mut self.nodes[idx];
			n.key = key;
			n.value = value;
			idx
		} else {
			self.nodes.push(Node {
				key,
				value,
				prev: NIL,
				next: NIL,
				region: WINDOW,
			});
			self.nodes.len() - 1
		}
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.rebalance_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	fn rebalance_protected(&mut self) {
		while self.lists[PROTECTED as usize].len > self.protected_max {
			let h = self.lists[PROTECTED as usize].head;
			self.move_to(h, PROBATION);
		}
	}

	/// Restore size invariants after an insertion.
	fn maintain(&mut self) {
		while self.lists[WINDOW as usize].len > self.window_max {
			let cand = self.lists[WINDOW as usize].head;
			self.move_to(cand, PROBATION);
			if self.map.len() > self.capacity {
				self.admit(cand);
			}
		}
		while self.map.len() > self.capacity {
			let victim = [PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&r| self.lists[r as usize].head)
				.find(|&h| h != NIL);
			match victim {
				Some(v) => self.evict(v),
				None => break,
			}
		}
	}

	/// TinyLFU admission: candidate (probation MRU) vs main's LRU victim.
	fn admit(&mut self, cand: usize) {
		let mut victim = self.lists[PROBATION as usize].head;
		if victim == cand {
			victim = self.lists[PROTECTED as usize].head;
		}
		if victim == NIL {
			self.evict(cand);
			return;
		}
		let cf = self.sketch.frequency(hash_key(&self.nodes[cand].key));
		let vf = self.sketch.frequency(hash_key(&self.nodes[victim].key));
		if cf > vf {
			self.evict(victim);
		} else {
			self.evict(cand);
		}
	}

	fn evict(&mut self, idx: usize) {
		self.map.remove(&self.nodes[idx].key);
		self.unlink(idx);
		self.free.push(idx);
	}

	fn set_window(&mut self, target: f64) {
		let t = target.clamp(1.0, (self.capacity - 1) as f64);
		self.window_target = t;
		self.window_max = (t.round() as usize).clamp(1, self.capacity - 1);
		self.protected_max = (self.capacity - self.window_max) * 4 / 5;
		while self.lists[WINDOW as usize].len > self.window_max {
			let h = self.lists[WINDOW as usize].head;
			self.move_to(h, PROBATION);
		}
		self.rebalance_protected();
	}

	/// Hill climbing on the window size using sampled hit rate.
	fn climb(&mut self) {
		if !self.adaptive {
			return;
		}
		let total = self.hits + self.misses;
		if total < self.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
		self.step = if change.abs() >= RESTART_THRESHOLD {
			STEP_PERCENT * self.capacity as f64 * sign
		} else {
			amount * STEP_DECAY
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		let target = self.window_target + amount;
		self.set_window(target);
	}

	fn move_to(&mut self, idx: usize, region: u8) {
		self.unlink(idx);
		self.push_back(idx, region);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, r) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.region as usize)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.lists[r].head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.lists[r].tail = prev;
		}
		self.lists[r].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end of `region`'s list.
	fn push_back(&mut self, idx: usize, region: u8) {
		let r = region as usize;
		let tail = self.lists[r].tail;
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.region = region;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.lists[r].head = idx;
		}
		self.lists[r].tail = idx;
		self.lists[r].len += 1;
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
	fn adaptive_mode_keeps_contract() {
		let mut cache = LruCache::new(64);
		let mut x: u64 = 12345;
		for _ in 0..100_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = (x % 300) as i64;
			match cache.get(&k) {
				Some(v) => assert_eq!(*v, k * 2),
				None => cache.put(k, k * 2),
			}
			assert!(cache.map.len() <= 64);
		}
		cache.put(7, 1);
		assert_eq!(*cache.get(&7).unwrap(), 1);
	}
}
