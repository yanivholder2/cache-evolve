// Adaptive W-TinyLFU cache (Caffeine-style) behind the original `LruCache` API.
//
// Layout: a small LRU *window* admits every new key; keys leaving the window must win a
// frequency duel (TinyLFU count-min sketch) against the LRU victim of the *main* segmented
// LRU (probation + protected) to stay resident. The window size is picked by mini-simulation:
// key-less replicas pinned to a grid of window sizes replay the (hash-sampled) request stream
// against the shared sketch, and the real cache adopts the size whose replica scores best.
// The sketch is 4x wider than Caffeine's and uses conservative ("minimal increment") updates,
// so low counts — where admission duels are decided — carry far less collision noise.
// Nodes live in a Vec arena linked by indices; freed slots are recycled via a free list.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

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
		// 4x Caffeine's width: memory is free here, accuracy at low counts is not.
		let len = (capacity.max(16) * 4).next_power_of_two();
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

	/// Conservative update: only the counters equal to the current minimum are raised, which
	/// keeps the estimate an upper bound while sharply reducing collision-driven overestimates.
	fn increment(&mut self, hash: u64) {
		let start = ((hash & 3) << 2) as usize;
		let mut idx = [0usize; 4];
		let mut val = [0u64; 4];
		let mut min = 0xF;
		for i in 0..4 {
			idx[i] = self.index(hash, i);
			val[i] = (self.table[idx[i]] >> ((start + i) << 2)) & 0xF;
			min = min.min(val[i]);
		}
		if min == 0xF {
			return;
		}
		for i in 0..4 {
			if val[i] == min {
				self.table[idx[i]] += 1u64 << ((start + i) << 2);
			}
		}
		self.size += 1;
		if self.size >= self.sample_size {
			self.reset();
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

/// Window sizes (fraction of capacity) evaluated by the replicas; index 0 is Caffeine's default
/// and 1.0 is exact LRU.
const WINDOW_FRACTIONS: [f64; 8] = [0.01, 0.03, 0.07, 0.15, 0.3, 0.5, 0.75, 1.0];
/// Very large caches are hash-sampled so each replica holds roughly this many entries.
const SHADOW_TARGET: usize = 4096;
/// Per-period decay of replica scores (memory of ~10 periods).
const SCORE_DECAY: f64 = 0.9;

fn window_size(capacity: usize, frac: f64) -> usize {
	((capacity as f64 * frac).round() as usize).clamp(1, capacity)
}

fn protected_size(capacity: usize, window: usize) -> usize {
	((capacity - window) as f64 * 0.8) as usize
}

/// Cheap hasher for keys that are already well-mixed 64-bit hashes.
#[derive(Debug, Default, Clone, Copy)]
struct MixHasher(u64);

impl Hasher for MixHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0 ^ b as u64).wrapping_mul(0x100_0000_01b3);
		}
	}

	fn write_u64(&mut self, n: u64) {
		// Sampled hashes share zeroed top bits; the multiply re-spreads entropy upward.
		self.0 = n.wrapping_mul(0x9e37_79b9_7f4a_7c15);
	}
}

#[derive(Debug, Clone, Copy)]
struct Slot {
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

/// Key-less replica of the W-TinyLFU policy pinned to one window size. All replicas see the
/// same (sampled) requests and share the real cache's sketch, so their hit counts rank window
/// sizes on identical traffic, free of the period-to-period noise a hill climber suffers.
#[derive(Debug)]
struct Shadow {
	map: HashMap<u64, usize, BuildHasherDefault<MixHasher>>,
	slots: Vec<Slot>,
	free: Vec<usize>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	frac: f64,
	hits: u64,
	score: f64,
}

impl Shadow {
	fn new(capacity: usize, frac: f64) -> Self {
		let window_max = window_size(capacity, frac);
		Self {
			map: HashMap::with_capacity_and_hasher(capacity + 1, Default::default()),
			slots: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max: protected_size(capacity, window_max),
			frac,
			hits: 0,
			score: 0.0,
		}
	}

	/// Replays one request (get-or-insert); returns whether it hit.
	fn access(&mut self, hash: u64, sketch: &FrequencySketch) -> bool {
		if let Some(&idx) = self.map.get(&hash) {
			let q = self.slots[idx].queue;
			if q == PROBATION {
				self.move_to(PROTECTED, idx);
				while self.lists[PROTECTED].len > self.protected_max {
					let head = self.lists[PROTECTED].head;
					self.move_to(PROBATION, head);
				}
			} else {
				self.move_to(q, idx);
			}
			return true;
		}
		let slot = Slot { hash, prev: NIL, next: NIL, queue: WINDOW };
		let idx = match self.free.pop() {
			Some(i) => {
				self.slots[i] = slot;
				i
			}
			None => {
				self.slots.push(slot);
				self.slots.len() - 1
			}
		};
		self.push(WINDOW, idx);
		self.map.insert(hash, idx);
		// Window size is fixed, so once full the window overflows by exactly one per insert.
		while self.lists[WINDOW].len > self.window_max {
			let cand = self.lists[WINDOW].head;
			if self.map.len() > self.capacity {
				let victim = if self.lists[PROBATION].len > 0 {
					self.lists[PROBATION].head
				} else {
					self.lists[PROTECTED].head
				};
				if victim == NIL || sketch.frequency(self.slots[cand].hash) <= sketch.frequency(self.slots[victim].hash) {
					self.drop_slot(cand);
					continue;
				}
				self.drop_slot(victim);
			}
			self.move_to(PROBATION, cand);
		}
		false
	}

	fn drop_slot(&mut self, idx: usize) {
		self.unlink(idx);
		self.map.remove(&self.slots[idx].hash);
		self.free.push(idx);
	}

	fn move_to(&mut self, q: usize, idx: usize) {
		self.unlink(idx);
		self.push(q, idx);
	}

	fn unlink(&mut self, idx: usize) {
		let Slot { prev, next, queue, .. } = self.slots[idx];
		if prev == NIL {
			self.lists[queue].head = next;
		} else {
			self.slots[prev].next = next;
		}
		if next == NIL {
			self.lists[queue].tail = prev;
		} else {
			self.slots[next].prev = prev;
		}
		self.lists[queue].len -= 1;
	}

	fn push(&mut self, q: usize, idx: usize) {
		let tail = self.lists[q].tail;
		self.slots[idx].prev = tail;
		self.slots[idx].next = NIL;
		self.slots[idx].queue = q;
		if tail == NIL {
			self.lists[q].head = idx;
		} else {
			self.slots[tail].next = idx;
		}
		self.lists[q].tail = idx;
		self.lists[q].len += 1;
	}
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
	// mini-simulation state
	shadows: Vec<Shadow>,
	active: usize,    // replica whose window size is currently in force
	sample_mask: u64, // a request is replayed iff `hash & sample_mask == 0`
	sampled: usize,
	period: usize,
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
		// Tiny caches degrade to exact LRU (window = everything, no replicas): sketch noise
		// dominates at that size and strict recency is the expected contract there.
		let small = capacity <= 16;
		let mut bits = 0usize;
		while capacity >> (bits + 1) >= SHADOW_TARGET {
			bits += 1;
		}
		let shadow_cap = capacity >> bits;
		let shadows: Vec<Shadow> = if small {
			Vec::new()
		} else {
			WINDOW_FRACTIONS.iter().map(|&f| Shadow::new(shadow_cap, f)).collect()
		};
		let window_max = if small { capacity } else { window_size(capacity, WINDOW_FRACTIONS[0]) };
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			nodes: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max: protected_size(capacity, window_max),
			sketch: FrequencySketch::new(capacity),
			last_miss: None,
			shadows,
			active: 0,
			sample_mask: if bits == 0 { 0 } else { u64::MAX << (64 - bits) },
			sampled: 0,
			period: shadow_cap.max(64),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.record(hash);
		match self.map.get(key).copied() {
			Some(idx) => {
				self.on_hit(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
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
			self.record(hash);
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

	/// Counts the access in the sketch and replays it (if sampled) through every replica.
	fn record(&mut self, hash: u64) {
		self.sketch.increment(hash);
		if self.shadows.is_empty() || (hash & self.sample_mask) != 0 {
			return;
		}
		let sketch = &self.sketch;
		for s in self.shadows.iter_mut() {
			if s.access(hash, sketch) {
				s.hits += 1;
			}
		}
		self.sampled += 1;
		if self.sampled >= self.period {
			self.sampled = 0;
			self.adapt();
		}
	}

	/// Adopt the window size of the replica with the best decayed hit count; a small
	/// hysteresis keeps near-ties from causing churn.
	fn adapt(&mut self) {
		for s in self.shadows.iter_mut() {
			s.score = s.score * SCORE_DECAY + s.hits as f64;
			s.hits = 0;
		}
		let mut best = self.active;
		for (i, s) in self.shadows.iter().enumerate() {
			if s.score > self.shadows[best].score {
				best = i;
			}
		}
		let current = self.shadows[self.active].score;
		if best != self.active && self.shadows[best].score > current * 1.002 + 0.5 {
			self.active = best;
			self.window_max = window_size(self.capacity, self.shadows[best].frac);
			self.protected_max = protected_size(self.capacity, self.window_max);
			self.demote_protected();
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
	fn replicas_stay_consistent_and_window_valid() {
		let mut cache = LruCache::new(300);
		let mut x: u64 = 7;
		for i in 0..60_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			// alternate a recency-friendly phase with a skewed one
			let key = if (i / 10_000) % 2 == 0 { (i / 4 + x % 50) as i64 } else { (x % 3000).min(x % 400) as i64 };
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
			assert!(cache.window_max >= 1 && cache.window_max <= cache.capacity);
		}
		check_links(&cache);
		for s in &cache.shadows {
			assert!(s.map.len() <= s.capacity);
			assert_eq!(s.lists.iter().map(|l| l.len).sum::<usize>(), s.map.len());
		}
	}

	#[test]
	fn sketch_conservative_update_is_monotone_and_saturates() {
		let mut s = FrequencySketch::new(64);
		let h = hash_of(&12345i64);
		let mut last = 0;
		for _ in 0..20 {
			s.increment(h);
			let f = s.frequency(h);
			assert!(f >= last && f <= 15);
			last = f;
		}
		assert_eq!(last, 15);
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
