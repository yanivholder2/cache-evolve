// W-TinyLFU cache: a small LRU "window" admits every new key; keys leaving the window
// must beat the main space's eviction victim on estimated frequency (Count-Min sketch
// with periodic aging) to enter the segmented-LRU main space (probation + protected).
// A Caffeine-style hill climber resizes the window to track recency vs. frequency bias.
// Tiny capacities (< LRU_ONLY_BELOW) degenerate to strict LRU (window == capacity).
// Arena of nodes indexed by usize with intrusive links; no unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
const LRU_ONLY_BELOW: usize = 16;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

/// Count-Min sketch (4 rows, counters saturating at 15) with periodic halving (aging).
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample_size: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = (capacity.max(16) * 4).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample_size: capacity.max(16) * 10,
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (i, slot) in out.iter_mut().enumerate() {
			let mut x = hash.wrapping_add((i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
			x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
			x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
			x ^= x >> 31;
			*slot = i * self.width + ((x as usize) & (self.width - 1));
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&s| self.table[s]).min().unwrap_or(0)
	}

	/// Conservative update: only the minimal counters are incremented.
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
		if self.additions >= self.sample_size {
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
	head: [usize; 3],       // LRU end of window / probation / protected
	tail: [usize; 3],       // MRU end of window / probation / protected
	len: [usize; 3],
	capacity: usize,
	window_cap: usize,
	protected_cap: usize,
	sketch: Sketch,
	adaptive: bool,
	last_miss: Option<u64>, // hash of the key that just missed in get (avoid double counting)
	hits: usize,
	requests: usize,
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
		let adaptive = capacity >= LRU_ONLY_BELOW;
		let window_cap = if adaptive { (capacity / 100).max(1) } else { capacity };
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			head: [NIL; 3],
			tail: [NIL; 3],
			len: [0; 3],
			capacity,
			window_cap,
			protected_cap: (capacity - window_cap) * 4 / 5,
			sketch: Sketch::new(capacity),
			adaptive,
			last_miss: None,
			hits: 0,
			requests: 0,
			prev_hit_rate: 0.0,
			step: -0.0625 * capacity as f64,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		self.requests += 1;
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.hits += 1;
				self.last_miss = None;
				self.on_hit(idx);
				self.climb(); // only moves nodes between queues, never evicts
				Some(&self.nodes[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; at capacity the
	/// TinyLFU filter decides between the window's outgoing key and the main victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(idx) = self.map.get(&key).copied() {
			self.last_miss = None;
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}
		let hash = hash_key(&key);
		// If this put follows a get-miss for the same key, the access is already counted.
		if self.last_miss.take() != Some(hash) {
			self.sketch.increment(hash);
			self.requests += 1;
			self.climb();
		}
		let idx = self.alloc(key.clone(), value, hash);
		self.map.insert(key, idx);
		self.push_mru(WINDOW, idx);
		self.evict_if_needed();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn on_hit(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
		self.unlink(idx);
		if q == PROBATION {
			self.push_mru(PROTECTED, idx);
			self.shrink_protected();
		} else {
			self.push_mru(q, idx);
		}
	}

	fn alloc(&mut self, key: K, value: V, hash: u64) -> usize {
		let node = Node {
			key,
			value,
			hash,
			prev: NIL,
			next: NIL,
			queue: WINDOW,
		};
		if let Some(idx) = self.free.pop() {
			self.nodes[idx] = node;
			idx
		} else {
			self.nodes.push(node);
			self.nodes.len() - 1
		}
	}

	fn evict_if_needed(&mut self) {
		// Window overflow: its LRU entry becomes the admission candidate (in probation MRU).
		let mut candidate = NIL;
		while self.len[WINDOW] > self.window_cap {
			candidate = self.head[WINDOW];
			self.unlink(candidate);
			self.push_mru(PROBATION, candidate);
		}
		if self.map.len() <= self.capacity {
			return;
		}
		let loser = if candidate == NIL {
			[PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.head[q])
				.find(|&i| i != NIL)
				.expect("over-capacity cache is non-empty")
		} else {
			let victim = if self.head[PROBATION] != candidate {
				self.head[PROBATION]
			} else {
				self.head[PROTECTED]
			};
			if victim == NIL {
				candidate
			} else if self.sketch.frequency(self.nodes[candidate].hash)
				> self.sketch.frequency(self.nodes[victim].hash)
			{
				victim
			} else {
				candidate
			}
		};
		self.evict(loser);
	}

	fn evict(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	fn shrink_protected(&mut self) {
		while self.len[PROTECTED] > self.protected_cap {
			let idx = self.head[PROTECTED];
			self.unlink(idx);
			self.push_mru(PROBATION, idx);
		}
	}

	/// Caffeine-style hill climber: every sample period, step the window size in the
	/// direction that improved the hit rate; restart large steps on big shifts, else decay.
	fn climb(&mut self) {
		if !self.adaptive || self.requests < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / self.requests as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * amount.signum()
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.requests = 0;
		let target = (self.window_cap as f64 + amount).round();
		let target = target.max(1.0).min((self.capacity - 1) as f64) as usize;
		self.resize_window(target);
	}

	fn resize_window(&mut self, target: usize) {
		self.window_cap = target;
		let main_cap = self.capacity - target;
		self.protected_cap = main_cap * 4 / 5;
		while self.len[WINDOW] > self.window_cap {
			let idx = self.head[WINDOW];
			self.unlink(idx);
			self.push_mru(PROBATION, idx);
		}
		while self.len[WINDOW] < self.window_cap
			&& self.len[PROBATION] + self.len[PROTECTED] > main_cap
		{
			let idx = if self.head[PROBATION] != NIL {
				self.head[PROBATION]
			} else {
				self.head[PROTECTED]
			};
			self.unlink(idx);
			self.push_mru(WINDOW, idx);
		}
		self.shrink_protected();
	}

	/// Remove node from its queue.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		if prev == NIL {
			self.head[q] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tail[q] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.len[q] -= 1;
	}

	/// Append node to the MRU end of queue `q`.
	fn push_mru(&mut self, q: usize, idx: usize) {
		let t = self.tail[q];
		{
			let n = &mut self.nodes[idx];
			n.prev = t;
			n.next = NIL;
			n.queue = q;
		}
		if t == NIL {
			self.head[q] = idx;
		} else {
			self.nodes[t].next = idx;
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

	#[test]
	fn tinylfu_mode_respects_capacity_and_keeps_new_keys() {
		let mut cache = LruCache::new(100);
		for i in 0..10_000i64 {
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
		for k in 1000..5000i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let survivors = (0..50i64).filter(|k| cache.get(k).is_some()).count();
		assert!(survivors >= 40);
	}
}
