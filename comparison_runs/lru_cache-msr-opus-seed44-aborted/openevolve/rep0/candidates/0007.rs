// Adaptive W-TinyLFU cache (HashMap + arena-backed doubly-linked lists).
//
// Layout (all lists live in one arena, slots are reused in place on eviction):
//   * Window    — small LRU admitting every new key (recency buffer).
//   * Probation — main-region SLRU segment for entries not yet re-referenced.
//   * Protected — main-region SLRU segment for entries hit while in main.
// A count-min frequency sketch (4-bit saturating counters, periodic halving)
// decides whether the window's LRU victim may displace the main region's
// victim (TinyLFU admission). The window size is chosen by "mini-sim":
// a set of small fixed-window W-TinyLFU simulations (0% .. 100% window, the
// latter being pure LRU) replay a hash-sampled subset of the keys; every
// period the real cache adopts the window of the simulation that missed
// least. All simulations see the identical sample, so the comparison is
// paired and low-noise, and the cache can jump straight to the right size.
// Very small caches (< ADAPTIVE_MIN_CAPACITY) degrade to pure LRU
// (window == capacity), where a sketch carries no useful signal.
// No unsafe, no external crates.

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
const ADAPTIVE_MIN_CAPACITY: usize = 16;
/// Candidate window sizes (fraction of capacity) evaluated by the simulations.
const WINDOW_FRACTIONS: [f64; 15] = [
	0.0, 0.01, 0.03, 0.06, 0.1, 0.15, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9, 1.0,
];
/// Index into WINDOW_FRACTIONS used until evidence says otherwise (1%).
const DEFAULT_FRACTION: usize = 1;
/// Target capacity of each simulation; larger caches are key-sampled down to it.
const MINI_TARGET: usize = 512;
/// Sampled accesses per adaptation period, as a multiple of simulation capacity.
const PERIOD_FACTOR: usize = 2;
/// Exponential decay applied to the simulated miss counts every period.
const MISS_DECAY: f64 = 0.75;
/// Relative miss advantage required before switching window size.
const SWITCH_MARGIN: f64 = 0.002;
const SAMPLE_SALT: u64 = 0x5851_F42D_4C95_7F2D;

#[inline]
fn window_for(capacity: usize, fraction: f64) -> usize {
	((capacity as f64 * fraction).round() as usize).clamp(1, capacity)
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

/// Count-min sketch with 4 rows of saturating 4-bit counters and aging.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	mask: usize,
	width: usize,
	additions: usize,
	sample_size: usize,
}

const SEEDS: [u64; 4] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0xD6E8_FEB8_6659_FD93,
];

#[inline]
fn mix(mut x: u64) -> u64 {
	x ^= x >> 30;
	x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
	x ^= x >> 27;
	x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
	x ^ (x >> 31)
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.saturating_mul(2).clamp(64, 1 << 26).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			mask: width - 1,
			width,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(10),
		}
	}

	#[inline]
	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			*slot = row * self.width + (mix(hash.wrapping_add(SEEDS[row])) as usize & self.mask);
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&i| self.table[i]).min().unwrap_or(0)
	}

	/// Conservative update: only the minimal counters are incremented.
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
	head: [usize; 3],       // per-queue LRU end
	tail: [usize; 3],       // per-queue MRU end
	len: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	adaptive: bool,
	sketch: Sketch,
	minis: Vec<LruCache<u64, ()>>, // fixed-window simulations on sampled keys
	mini_misses: Vec<f64>,         // decayed miss counts per simulation
	sample_rate: u64,
	sampled: usize,
	period: usize,
	current: usize, // index into WINDOW_FRACTIONS currently in effect
}

#[inline]
fn hash_of<K: Hash>(key: &K) -> u64 {
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
		if capacity < ADAPTIVE_MIN_CAPACITY {
			return Some(Self::build(capacity, capacity, false));
		}
		let mut cache = Self::build(
			capacity,
			window_for(capacity, WINDOW_FRACTIONS[DEFAULT_FRACTION]),
			true,
		);
		let rate = (capacity / MINI_TARGET).max(1);
		let mini_cap = (capacity / rate).max(1);
		cache.minis = WINDOW_FRACTIONS
			.iter()
			.map(|&f| LruCache::<u64, ()>::build(mini_cap, window_for(mini_cap, f), true))
			.collect();
		cache.mini_misses = vec![0.0; WINDOW_FRACTIONS.len()];
		cache.sample_rate = rate as u64;
		cache.period = mini_cap * PERIOD_FACTOR;
		Some(cache)
	}

	/// Builds an empty cache with a fixed window; `tinylfu` enables admission.
	fn build(capacity: usize, window_max: usize, tinylfu: bool) -> Self {
		Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: [NIL; 3],
			tail: [NIL; 3],
			len: [0; 3],
			capacity,
			window_max,
			protected_max: (capacity - window_max) * 4 / 5,
			adaptive: tinylfu,
			sketch: Sketch::new(if tinylfu { capacity } else { 1 }),
			minis: Vec::new(),
			mini_misses: Vec::new(),
			sample_rate: 1,
			sampled: 0,
			period: 0,
			current: DEFAULT_FRACTION,
		}
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		if self.adaptive {
			let hash = match found {
				Some(i) => self.nodes[i].hash,
				None => hash_of(key),
			};
			self.sketch.increment(hash);
			if !self.minis.is_empty() {
				self.simulate(hash);
			}
		}
		let idx = found?;
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. A new key always enters the window, so it is
	/// resident immediately after `put`; at capacity one other entry is evicted.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = if self.adaptive { hash_of(&key) } else { 0 };
		let idx = if self.nodes.len() < self.capacity {
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
			let victim = self.select_victim();
			self.unlink(victim);
			let old = std::mem::replace(&mut self.nodes[victim].key, key.clone());
			self.map.remove(&old);
			self.nodes[victim].value = value;
			self.nodes[victim].hash = hash;
			victim
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		while self.len[WINDOW] > self.window_max {
			self.transfer(self.head[WINDOW], PROBATION);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Chooses the entry to evict when the cache is full and a new key arrives.
	fn select_victim(&mut self) -> usize {
		let main_victim = if self.head[PROBATION] != NIL {
			self.head[PROBATION]
		} else {
			self.head[PROTECTED]
		};
		if self.len[WINDOW] < self.window_max {
			// Window has room for the newcomer; the main region is over its share.
			return if main_victim != NIL { main_victim } else { self.head[WINDOW] };
		}
		let candidate = self.head[WINDOW];
		if main_victim == NIL {
			return candidate;
		}
		// TinyLFU admission: the window's victim enters main only if it is
		// estimated to be more popular than main's victim.
		let cf = self.sketch.frequency(self.nodes[candidate].hash);
		let vf = self.sketch.frequency(self.nodes[main_victim].hash);
		if cf > vf {
			self.transfer(candidate, PROBATION);
			main_victim
		} else {
			candidate
		}
	}

	fn on_hit(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		if q == PROBATION {
			// Re-referenced in main: promote to the protected segment.
			self.transfer(idx, PROTECTED);
			self.demote_protected();
		} else {
			self.transfer(idx, q);
		}
	}

	fn demote_protected(&mut self) {
		while self.len[PROTECTED] > self.protected_max {
			self.transfer(self.head[PROTECTED], PROBATION);
		}
	}

	/// Replays a key-sampled access on every fixed-window simulation and
	/// periodically adopts the window size that missed least.
	fn simulate(&mut self, hash: u64) {
		if self.sample_rate > 1 && mix(hash ^ SAMPLE_SALT) % self.sample_rate != 0 {
			return;
		}
		for (mini, misses) in self.minis.iter_mut().zip(self.mini_misses.iter_mut()) {
			if mini.get(&hash).is_none() {
				mini.put(hash, ());
				*misses += 1.0;
			}
		}
		self.sampled += 1;
		if self.sampled >= self.period {
			self.sampled = 0;
			self.adapt();
		}
	}

	fn adapt(&mut self) {
		let mut best = self.current;
		for (i, &m) in self.mini_misses.iter().enumerate() {
			if m < self.mini_misses[best] {
				best = i;
			}
		}
		if self.mini_misses[best] < self.mini_misses[self.current] * (1.0 - SWITCH_MARGIN) {
			self.current = best;
			self.set_window(window_for(self.capacity, WINDOW_FRACTIONS[best]));
		}
		for m in self.mini_misses.iter_mut() {
			*m *= MISS_DECAY;
		}
	}

	/// Resizes the window (and the protected share of main) in place. A grown
	/// window fills from new arrivals, which evict main's victims meanwhile.
	fn set_window(&mut self, window: usize) {
		self.window_max = window.clamp(1, self.capacity);
		self.protected_max = (self.capacity - self.window_max) * 4 / 5;
		self.demote_protected();
		while self.len[WINDOW] > self.window_max {
			self.transfer(self.head[WINDOW], PROBATION);
		}
	}

	/// Move node to the MRU end of queue `q`.
	#[inline]
	fn transfer(&mut self, idx: usize, q: usize) {
		self.unlink(idx);
		self.push_back(q, idx);
	}

	/// Remove node from its current queue.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else if self.head[q] == idx {
			self.head[q] = next;
		} else {
			return; // not linked
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.tail[q] = prev;
		}
		self.len[q] -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end of queue `q`.
	fn push_back(&mut self, q: usize, idx: usize) {
		let t = self.tail[q];
		{
			let n = &mut self.nodes[idx];
			n.prev = t;
			n.next = NIL;
			n.queue = q;
		}
		if t != NIL {
			self.nodes[t].next = idx;
		} else {
			self.head[q] = idx;
		}
		self.tail[q] = idx;
		self.len[q] += 1;
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

	/// Structural invariants: list links, per-queue lengths, map coherence, bounds.
	fn check<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		let mut total = 0;
		for q in 0..3 {
			let mut n = 0;
			let mut prev = NIL;
			let mut cur = c.head[q];
			while cur != NIL {
				let node = &c.nodes[cur];
				assert_eq!(node.queue, q);
				assert_eq!(node.prev, prev);
				assert_eq!(c.map.get(&node.key), Some(&cur));
				prev = cur;
				cur = node.next;
				n += 1;
			}
			assert_eq!(c.tail[q], prev);
			assert_eq!(c.len[q], n);
			total += n;
		}
		assert_eq!(total, c.map.len());
		assert!(c.map.len() <= c.capacity);
		assert!(c.window_max >= 1 && c.window_max <= c.capacity);
		assert!(c.len[WINDOW] <= c.window_max);
		assert!(c.len[PROTECTED] <= c.protected_max);
		for m in &c.minis {
			check(m);
		}
	}

	#[test]
	fn adaptive_put_then_get_and_bounds() {
		let mut cache = LruCache::new(100);
		let mut x: u64 = 12345;
		for i in 0..200_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = if i % 3 == 0 { x % 5000 } else { x % 150 };
			if cache.get(&k).is_none() {
				cache.put(k, k + 1);
				assert_eq!(*cache.get(&k).unwrap(), k + 1);
			}
			if i % 997 == 0 {
				check(&cache);
			}
		}
		check(&cache);
		assert_eq!(cache.map.len(), 100);
	}

	#[test]
	fn adaptive_resists_scans() {
		let mut cache = LruCache::new(64);
		for _ in 0..20 {
			for k in 0..32u64 {
				if cache.get(&k).is_none() {
					cache.put(k, k);
				}
			}
		}
		for k in 1000..1300u64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let hot = (0..32u64).filter(|k| cache.get(k).is_some()).count();
		assert!(hot >= 30, "hot set evicted by scan: {hot}");
		check(&cache);
	}

	#[test]
	fn adaptive_overwrite_keeps_size() {
		let mut cache = LruCache::new(32);
		for k in 0..32 {
			cache.put(k, k);
		}
		for k in 0..32 {
			cache.put(k, k * 2);
		}
		assert_eq!(cache.map.len(), 32);
		for k in 0..32 {
			assert_eq!(*cache.get(&k).unwrap(), k * 2);
		}
		check(&cache);
	}
}
