// Adaptive W-TinyLFU cache whose window size is chosen by shadow simulation.
//
// The cache is a window LRU in front of a segmented (probation/protected) main LRU; a count-min
// frequency sketch decides whether a window evictee may displace the main victim (TinyLFU).
// Instead of a slow, noisy hill climber, a ladder of key-less shadow caches replays the live
// access stream through the same policy at several fixed window sizes (1% .. 100%, the latter
// being pure LRU); the real window is periodically resized to the shadow with the best decayed
// hit count. Large caches feed the shadows a hash-sampled, scaled-down stream (miniature
// simulation). Small capacities degenerate to strict LRU. HashMap + arena-based doubly-linked
// lists (one spare slot is recycled per eviction); no unsafe, std only.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
/// Below this capacity the cache behaves as a strict LRU.
const ADAPTIVE_MIN_CAPACITY: usize = 16;
/// Window sizes (per mille of capacity) simulated by the shadow caches; 1000 = pure LRU.
const WINDOW_PERMILLE: [usize; 9] = [10, 30, 80, 150, 250, 400, 600, 800, 1000];
/// Shadow caches never hold more than about this many entries; larger caches are sampled.
const SHADOW_MAX_CAPACITY: usize = 16_384;
/// Weight of the previous score when a new adaptation period is folded in.
const SCORE_DECAY: f64 = 0.7;
/// Minimum number of accesses between window adaptations.
const MIN_PERIOD: usize = 512;

/// Pass-through hasher for keys that are already well-mixed 64-bit hashes.
#[derive(Debug, Default)]
struct PassThroughHasher(u64);

impl Hasher for PassThroughHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0 ^ b as u64).wrapping_mul(0x0100_0000_01B3);
		}
	}

	fn write_u64(&mut self, n: u64) {
		self.0 = n;
	}
}

type HashIndex = HashMap<u64, usize, BuildHasherDefault<PassThroughHasher>>;

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
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

/// W-TinyLFU bookkeeping over arena slots: window LRU + probation/protected SLRU. It only knows
/// slot hashes, so the same code drives the real cache and the key-less shadow caches.
#[derive(Debug)]
struct Policy {
	prev: Vec<usize>,
	next: Vec<usize>,
	queue: Vec<usize>,
	hash: Vec<u64>,
	queues: [Queue; 3], // window, probation, protected
	free: Vec<usize>,
	size: usize,
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	rng: u64,
}

impl Policy {
	fn new(capacity: usize, window_max: usize) -> Self {
		let empty = Queue { head: NIL, tail: NIL, len: 0 };
		let mut policy = Self {
			prev: Vec::with_capacity(capacity + 1),
			next: Vec::with_capacity(capacity + 1),
			queue: Vec::with_capacity(capacity + 1),
			hash: Vec::with_capacity(capacity + 1),
			queues: [empty; 3],
			free: Vec::new(),
			size: 0,
			capacity,
			window_max: capacity,
			protected_max: 0,
			rng: 0x2545_F491_4F6C_DD1D,
		};
		policy.resize_window(window_max);
		policy
	}

	/// Sets the window target; the main space (80% protected) takes the rest. Shrinking moves
	/// window entries into probation; growing is lazy (new entries displace main victims).
	fn resize_window(&mut self, window_max: usize) {
		self.window_max = window_max.clamp(1, self.capacity);
		self.protected_max = (self.capacity - self.window_max) * 4 / 5;
		while self.queues[WINDOW].len > self.window_max {
			let head = self.queues[WINDOW].head;
			self.move_to(head, PROBATION);
		}
		self.demote_protected();
	}

	fn on_hit(&mut self, idx: usize) {
		match self.queue[idx] {
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
			q => self.move_to(idx, q),
		}
	}

	fn demote_protected(&mut self) {
		while self.queues[PROTECTED].len > self.protected_max {
			let head = self.queues[PROTECTED].head;
			self.move_to(head, PROBATION);
		}
	}

	/// Adds a new entry at the window MRU end. Returns its slot and, if the policy went over
	/// capacity, the (detached, now free) slot of the evicted entry. Never evicts the new entry.
	fn insert(&mut self, hash: u64, sketch: &Sketch) -> (usize, Option<usize>) {
		let slot = match self.free.pop() {
			Some(s) => s,
			None => {
				self.prev.push(NIL);
				self.next.push(NIL);
				self.queue.push(WINDOW);
				self.hash.push(0);
				self.hash.len() - 1
			}
		};
		self.hash[slot] = hash;
		self.attach(slot, WINDOW);
		self.size += 1;
		let mut candidate = NIL;
		while self.queues[WINDOW].len > self.window_max {
			candidate = self.queues[WINDOW].head;
			self.move_to(candidate, PROBATION);
		}
		if self.size <= self.capacity {
			return (slot, None);
		}
		let victim = [PROBATION, PROTECTED]
			.iter()
			.map(|&q| self.queues[q].head)
			.find(|&i| i != NIL && i != candidate)
			.unwrap_or(if candidate != NIL { candidate } else { self.queues[WINDOW].head });
		let evict = if candidate != NIL && victim != candidate && !self.admit(candidate, victim, sketch) {
			candidate
		} else {
			victim
		};
		self.detach(evict);
		self.free.push(evict);
		self.size -= 1;
		(slot, Some(evict))
	}

	/// TinyLFU admission: the candidate replaces the victim only if it is more frequent
	/// (with a small random chance for warm candidates, to escape hash-collision lockout).
	fn admit(&mut self, candidate: usize, victim: usize, sketch: &Sketch) -> bool {
		let cf = sketch.frequency(self.hash[candidate]);
		let vf = sketch.frequency(self.hash[victim]);
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

	fn move_to(&mut self, idx: usize, queue: usize) {
		self.detach(idx);
		self.attach(idx, queue);
	}

	/// Remove a slot from its current queue.
	fn detach(&mut self, idx: usize) {
		let (prev, next, queue) = (self.prev[idx], self.next[idx], self.queue[idx]);
		match prev {
			NIL => self.queues[queue].head = next,
			p => self.next[p] = next,
		}
		match next {
			NIL => self.queues[queue].tail = prev,
			n => self.prev[n] = prev,
		}
		self.queues[queue].len -= 1;
	}

	/// Append a slot to the MRU end of `queue`.
	fn attach(&mut self, idx: usize, queue: usize) {
		let tail = self.queues[queue].tail;
		self.prev[idx] = tail;
		self.next[idx] = NIL;
		self.queue[idx] = queue;
		match tail {
			NIL => self.queues[queue].head = idx,
			t => self.next[t] = idx,
		}
		self.queues[queue].tail = idx;
		self.queues[queue].len += 1;
	}
}

/// Metadata-only replica of the policy at one fixed window size, scored on the live stream.
#[derive(Debug)]
struct Shadow {
	policy: Policy,
	index: HashIndex,
	hits: u64,
	score: f64,
}

impl Shadow {
	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.index.get(&hash) {
			self.hits += 1;
			self.policy.on_hit(idx);
			return;
		}
		let (slot, evicted) = self.policy.insert(hash, sketch);
		if let Some(e) = evicted {
			self.index.remove(&self.policy.hash[e]);
		}
		self.index.insert(hash, slot);
	}
}

fn window_size(capacity: usize, choice: usize) -> usize {
	(capacity * WINDOW_PERMILLE[choice] / 1000).max(1)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> arena slot
	nodes: Vec<Node<K, V>>, // arena, indexed by policy slot
	policy: Policy,
	sketch: Sketch,
	shadows: Vec<Shadow>, // empty for small (strict LRU) caches
	choice: usize,        // index into WINDOW_PERMILLE currently applied to the real cache
	sample_threshold: u64,
	accesses: usize,
	period: usize,
	last_miss: Option<u64>, // hash of the last get-miss (already recorded)
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
		// Fraction (out of 65536) of the key space fed to the shadows, and their scaled capacity.
		let threshold = ((SHADOW_MAX_CAPACITY as u128 * 65_536) / capacity as u128).clamp(1, 65_536) as u64;
		let shadow_capacity = ((capacity as u128 * threshold as u128 / 65_536) as usize).max(ADAPTIVE_MIN_CAPACITY);
		let shadows: Vec<Shadow> = if adaptive {
			(0..WINDOW_PERMILLE.len())
				.map(|choice| Shadow {
					policy: Policy::new(shadow_capacity, window_size(shadow_capacity, choice)),
					index: HashIndex::with_capacity_and_hasher(shadow_capacity + 1, Default::default()),
					hits: 0,
					score: 0.0,
				})
				.collect()
		} else {
			Vec::new()
		};
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			policy: Policy::new(capacity, if adaptive { window_size(capacity, 0) } else { capacity }),
			sketch: Sketch::new(if adaptive { capacity } else { 1 }),
			shadows,
			choice: 0,
			sample_threshold: threshold,
			accesses: 0,
			period: capacity.max(MIN_PERIOD),
			last_miss: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		if !self.shadows.is_empty() {
			let hash = hash_key(key);
			self.record(hash);
			self.last_miss = if found.is_none() { Some(hash) } else { None };
		}
		let idx = found?;
		self.policy.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. A new key enters the window; if the cache is then over capacity
	/// the W-TinyLFU policy evicts either the window evictee or the main LRU victim.
	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		if !self.shadows.is_empty() && self.last_miss.take() != Some(hash) {
			self.record(hash);
		}
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.policy.on_hit(idx);
			return;
		}
		let (slot, evicted) = self.policy.insert(hash, &self.sketch);
		if let Some(e) = evicted {
			self.map.remove(&self.nodes[e].key);
		}
		let node = Node { key: key.clone(), value };
		if slot == self.nodes.len() {
			self.nodes.push(node);
		} else {
			self.nodes[slot] = node;
		}
		self.map.insert(key, slot);
		debug_assert!(self.map.len() <= self.policy.capacity);
	}

	/// Feeds one access to the frequency sketch and the (sampled) shadows; periodically
	/// re-tunes the real window size.
	fn record(&mut self, hash: u64) {
		self.sketch.increment(hash);
		if ((hash >> 40) & 0xFFFF) < self.sample_threshold {
			for shadow in self.shadows.iter_mut() {
				shadow.access(hash, &self.sketch);
			}
		}
		self.accesses += 1;
		if self.accesses % self.period == 0 {
			self.adapt();
		}
	}

	/// Applies the window size of the shadow with the best decayed hit count (ties keep the current).
	fn adapt(&mut self) {
		for shadow in self.shadows.iter_mut() {
			shadow.score = shadow.score * SCORE_DECAY + shadow.hits as f64;
			shadow.hits = 0;
		}
		let mut best = self.choice;
		for (i, shadow) in self.shadows.iter().enumerate() {
			if shadow.score > self.shadows[best].score {
				best = i;
			}
		}
		if best != self.choice {
			self.choice = best;
			let window = window_size(self.policy.capacity, best);
			self.policy.resize_window(window);
		}
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
		let total: usize = cache.policy.queues.iter().map(|q| q.len).sum();
		assert_eq!(total, cache.map.len());
		for shadow in &cache.shadows {
			assert!(shadow.index.len() <= shadow.policy.capacity);
		}
	}

	#[test]
	fn sampled_shadows_for_large_capacity() {
		let cap = 40_000;
		let mut cache = LruCache::new(cap);
		for i in 0..120_000i64 {
			let k = (i * 7919) % 60_000;
			if cache.get(&k).is_none() {
				cache.put(k, i);
			}
			assert!(cache.map.len() <= cap);
		}
		for shadow in &cache.shadows {
			assert!(shadow.policy.capacity <= SHADOW_MAX_CAPACITY);
			assert!(shadow.index.len() <= shadow.policy.capacity);
		}
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
