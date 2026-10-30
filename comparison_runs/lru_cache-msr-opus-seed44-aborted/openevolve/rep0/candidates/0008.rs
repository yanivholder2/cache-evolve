// W-TinyLFU cache behind the classic `LruCache` API (O(1) per operation, std only).
//
// * Window: a small LRU that admits every new key (keeps recency-friendly workloads fast).
// * Main: a segmented LRU (probation + protected) holding keys that proved their worth.
// * Admission: when the window overflows, its LRU entry competes with the main-space victim;
//   the one with the higher estimated recent frequency (count-min sketch with periodic
//   halving) stays resident. This makes the cache scan resistant.
// * Adaptive window (ARC-style): ghost histories remember keys the window recently rejected and
//   keys the main space recently evicted. A miss on a window ghost means a larger window would
//   have hit (grow it); a miss on a main ghost means the main space deserved the room (shrink).
//   With equal ghost horizons the split settles where both marginal gains balance.
//
// All nodes live in one arena (Vec); an eviction frees exactly one slot which the new key reuses.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

fn mix(mut x: u64) -> u64 {
	x ^= x >> 30;
	x = x.wrapping_mul(0xbf58_476d_1ce4_e5b9);
	x ^= x >> 27;
	x = x.wrapping_mul(0x94d0_49bb_1331_11eb);
	x ^ (x >> 31)
}

/// Count-min sketch (4 rows, counters saturating at 15, conservative update) with periodic
/// halving so that estimates reflect recent popularity.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	period: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.saturating_mul(4).clamp(64, 1 << 22).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			period: capacity.saturating_mul(10).max(32),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			let h = mix(hash ^ (row as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15));
			*slot = row * self.width + (h as usize & (self.width - 1));
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
		for &s in &slots {
			if self.table[s] == min {
				self.table[s] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.period {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}
}

/// Ghost history of one queue: hashes of keys that recently left it, stamped with the queue's
/// departure clock. A miss on a key that left within the last `horizon` departures would have
/// been a hit had that queue been `horizon` entries larger.
#[derive(Debug)]
struct Ghost {
	stamps: HashMap<u64, u64>,
	fifo: VecDeque<(u64, u64)>,
	clock: u64,
	horizon: u64,
}

impl Ghost {
	fn new(horizon: usize) -> Self {
		Self {
			stamps: HashMap::new(),
			fifo: VecDeque::new(),
			clock: 0,
			horizon: horizon.max(1) as u64,
		}
	}

	/// One entry left the queue; remember it if it left the cache entirely (`Some(hash)`).
	fn tick(&mut self, hash: Option<u64>) {
		self.clock += 1;
		if let Some(h) = hash {
			self.stamps.insert(h, self.clock);
			self.fifo.push_back((h, self.clock));
		}
		while let Some(&(h, s)) = self.fifo.front() {
			if self.clock - s < self.horizon {
				break;
			}
			self.fifo.pop_front();
			if self.stamps.get(&h) == Some(&s) {
				self.stamps.remove(&h);
			}
		}
	}

	/// True (and forgets the key) if it left the queue within the horizon.
	fn take(&mut self, hash: u64) -> bool {
		match self.stamps.remove(&hash) {
			Some(s) => self.clock - s < self.horizon,
			None => false,
		}
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	queue: usize,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	heads: [usize; 3],      // per-queue LRU end
	tails: [usize; 3],      // per-queue MRU end
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	last_miss: Option<u64>, // hash of the key whose get just missed (avoid double counting)
	window_ghost: Ghost,    // keys the admission filter rejected from the window
	main_ghost: Ghost,      // keys evicted from the main space
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max: (capacity - window_max) * 4 / 5,
			sketch: Sketch::new(capacity),
			last_miss: None,
			window_ghost: Ghost::new(capacity / 4),
			main_ghost: Ghost::new(capacity / 4),
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
		let idx = match found {
			Some(idx) => idx,
			None => {
				self.last_miss = Some(hash);
				self.adapt(hash);
				return None;
			}
		};
		self.last_miss = None;
		self.on_access(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. A new key always enters the window; if the cache is full,
	/// exactly one resident entry (never the new key) is evicted and its slot reused.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(idx) = self.map.get(&key).copied() {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let hash = hash_key(&key);
		if self.last_miss != Some(hash) {
			// Not preceded by a missing get: count this insert as the access (and the miss).
			self.sketch.increment(hash);
			self.adapt(hash);
		}
		self.last_miss = None;
		let idx = if self.nodes.len() >= self.capacity {
			let slot = self.evict();
			let old = std::mem::replace(&mut self.nodes[slot].key, key.clone());
			self.map.remove(&old);
			self.nodes[slot].value = value;
			slot
		} else {
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				queue: WINDOW,
			});
			self.nodes.len() - 1
		};
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		self.trim_window();
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Hit handling: window entries stay in the window; main entries move to protected.
	fn on_access(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		self.unlink(idx);
		if queue == WINDOW {
			self.push_back(idx, WINDOW);
		} else {
			self.push_back(idx, PROTECTED);
			self.trim_protected();
		}
	}

	/// Frees (unlinks) one slot. The window's LRU entry (candidate) competes with the main
	/// space's LRU entry (victim); the less frequently used of the two is evicted.
	fn evict(&mut self) -> usize {
		let main_victim = if self.heads[PROBATION] != NIL {
			self.heads[PROBATION]
		} else {
			self.heads[PROTECTED]
		};
		let victim = if self.lens[WINDOW] > 0 && self.lens[WINDOW] >= self.window_max {
			let candidate = self.heads[WINDOW];
			if main_victim == NIL {
				candidate
			} else if self.frequency_of(candidate) > self.frequency_of(main_victim) {
				self.unlink(candidate);
				self.push_back(candidate, PROBATION);
				self.window_ghost.tick(None);
				main_victim
			} else {
				candidate
			}
		} else if main_victim != NIL {
			main_victim
		} else {
			self.heads[WINDOW]
		};
		let hash = hash_key(&self.nodes[victim].key);
		if self.nodes[victim].queue == WINDOW {
			self.window_ghost.tick(Some(hash));
		} else {
			self.main_ghost.tick(Some(hash));
		}
		self.unlink(victim);
		victim
	}

	fn frequency_of(&self, idx: usize) -> u8 {
		self.sketch.frequency(hash_key(&self.nodes[idx].key))
	}

	fn trim_window(&mut self) {
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.unlink(idx);
			self.push_back(idx, PROBATION);
			self.window_ghost.tick(None);
		}
	}

	fn trim_protected(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			let idx = self.heads[PROTECTED];
			self.unlink(idx);
			self.push_back(idx, PROBATION);
		}
	}

	/// Ghost-driven window sizing, called once per miss. A miss on a key the window recently
	/// rejected means a larger window would have hit: grow it. A miss on a key the main space
	/// recently evicted means the main space deserved the room: shrink the window. Both ghosts
	/// share one horizon, so the split settles where the two marginal gains are equal.
	fn adapt(&mut self, hash: u64) {
		let window_hit = self.window_ghost.take(hash);
		let main_hit = self.main_ghost.take(hash);
		if self.capacity < 2 || window_hit == main_hit {
			return;
		}
		let step = (self.capacity / 1024).max(1);
		let target = if window_hit {
			(self.window_max + step).min(self.capacity - 1)
		} else {
			self.window_max.saturating_sub(step).max(1)
		};
		if target != self.window_max {
			self.resize_window(target);
		}
	}

	/// Shrinking hands window entries to probation at once; growing is lazy (new arrivals fill
	/// the window while the main space gives up its victims without an admission contest).
	fn resize_window(&mut self, target: usize) {
		self.window_max = target;
		self.protected_max = (self.capacity - target) * 4 / 5;
		self.trim_window();
		self.trim_protected();
	}

	/// Remove node from its current queue.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.heads[q] = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.tails[q] = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
		self.lens[q] -= 1;
	}

	/// Append node to the MRU end of queue `q`.
	fn push_back(&mut self, idx: usize, q: usize) {
		let tail = self.tails[q];
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.queue = q;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.heads[q] = idx;
		}
		self.tails[q] = idx;
		self.lens[q] += 1;
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
	fn eviction_keeps_newest_and_bounds_size() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // must evict exactly one older key

		assert_eq!(*cache.get(&3).unwrap(), 30);
		let resident = (1..=2).filter(|k| cache.get(k).is_some()).count();
		assert_eq!(resident, 1);
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
		}
		check(&cache);
		assert_eq!(*cache.get(&6).unwrap(), 60);
		let resident = (1..=6).filter(|k| cache.get(k).is_some()).count();
		assert_eq!(resident, 3);
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
	fn frequently_read_key_survives_churn() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		for k in 4..=20 {
			cache.put(k, k * 10);
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&20).unwrap(), 200);
		check(&cache);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40);
		// Key 2 was re-read (now protected) and key 4 is the newest: both stay.
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		let others = cache.get(&1).is_some() as usize + cache.get(&3).is_some() as usize;
		assert_eq!(others, 1);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&3);
		cache.put(4, 40);
		cache.put(5, 50);
		check(&cache);
		assert_eq!(cache.nodes.len(), 2); // slots are reused, the arena never grows
		assert_eq!(*cache.get(&5).unwrap(), 50);
		let resident = (1..=5).filter(|k| cache.get(k).is_some()).count();
		assert_eq!(resident, 2);
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
		cache.put(4, 40); // evicts exactly one older key
		assert_eq!(*cache.get(&4).unwrap(), 40);
		if let Some(v) = cache.get(&3) {
			assert_eq!(*v, 31);
		}
		let resident = (1..=4).filter(|k| cache.get(k).is_some()).count();
		assert_eq!(resident, 3);
	}

	/// Structural invariants: list links, queue tags, lengths, map/arena agreement, bounds.
	fn check<K: Hash + Eq + Clone + Debug, V: Debug>(cache: &LruCache<K, V>) {
		let mut total = 0;
		for q in 0..3 {
			let mut prev = NIL;
			let mut cur = cache.heads[q];
			let mut n = 0;
			while cur != NIL {
				assert_eq!(cache.nodes[cur].queue, q);
				assert_eq!(cache.nodes[cur].prev, prev);
				prev = cur;
				cur = cache.nodes[cur].next;
				n += 1;
			}
			assert_eq!(cache.tails[q], prev);
			assert_eq!(cache.lens[q], n);
			total += n;
		}
		assert_eq!(total, cache.map.len());
		assert_eq!(total, cache.nodes.len());
		assert!(total <= cache.capacity);
		assert!(cache.lens[WINDOW] <= cache.window_max);
		assert!(cache.lens[PROTECTED] <= cache.protected_max);
		for (k, &i) in &cache.map {
			assert_eq!(&cache.nodes[i].key, k);
		}
	}

	#[test]
	fn newest_key_is_always_resident_and_cache_fills() {
		for &cap in &[1usize, 2, 3, 10] {
			let mut cache = LruCache::new(cap);
			for k in 0..500u64 {
				cache.put(k, k * 2);
				assert_eq!(cache.get(&k).copied(), Some(k * 2));
			}
			let resident = (0..500u64).filter(|k| cache.get(k).is_some()).count();
			assert_eq!(resident, cap);
			check(&cache);
		}
	}

	#[test]
	fn randomized_workload_keeps_invariants() {
		for &cap in &[1usize, 2, 3, 5, 16, 100] {
			let mut cache = LruCache::new(cap);
			let mut model: std::collections::HashMap<u64, u64> = std::collections::HashMap::new();
			let mut s: u64 = 0x9e37_79b9 ^ cap as u64;
			for step in 0..20_000u64 {
				s = s
					.wrapping_mul(6_364_136_223_846_793_005)
					.wrapping_add(1_442_695_040_888_963_407);
				let r = s >> 33;
				let key = if r % 3 == 0 { r % 8 } else { r % (cap as u64 * 4 + 7) };
				if r % 11 == 0 {
					cache.put(key, step);
					model.insert(key, step);
				}
				let got = cache.get(&key).copied();
				match got {
					Some(v) => assert_eq!(model.get(&key), Some(&v)),
					None => {
						cache.put(key, step);
						model.insert(key, step);
						assert_eq!(cache.get(&key).copied(), Some(step));
					}
				}
				if step % 997 == 0 {
					check(&cache);
				}
			}
			check(&cache);
		}
	}

	#[test]
	fn frequent_keys_survive_a_scan() {
		fn access(cache: &mut LruCache<u64, u64>, k: u64) {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let mut cache = LruCache::new(100);
		for _ in 0..20 {
			for k in 0..50 {
				access(&mut cache, k);
			}
		}
		for k in 1_000..3_000 {
			access(&mut cache, k);
		}
		let survivors = (0..50u64).filter(|k| cache.get(k).is_some()).count();
		assert!(survivors >= 45, "only {survivors} hot keys survived the scan");
		check(&cache);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
