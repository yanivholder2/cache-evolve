// Adaptive W-TinyLFU cache (the policy behind Caffeine), kept under the historical
// `LruCache` name because the JNI boundary binds to it.
//
// A small LRU *window* admits every new key. When the window overflows, its LRU entry
// (the candidate) competes with a victim from the segmented-LRU *main* space
// (probation + protected): a count-min sketch of 4-bit counters, halved periodically so
// it ages, estimates popularity, and the candidate is admitted only if it is more
// popular. The victim is the least frequent of the few oldest probation entries.
//
// The window size is chosen by *shadow simulation* rather than hill climbing: key-only
// copies of the same policy run side by side at fixed window fractions (1% .. 100%, the
// last being plain LRU) and share the frequency sketch. Their exponentially decayed hit
// counts are compared periodically and the real cache adopts the window fraction of the
// leader, so it follows recency- vs frequency-friendly phases quickly and without the
// noise of a hill climber. Large caches feed the shadows a hash-sampled subset of keys
// (SHARDS-style) so the overhead stays bounded.
// All lists thread through one arena; slots are reused in place on eviction.
// No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

const WARM_CANDIDATE: u8 = 6;
/// How many of the oldest probation entries compete to be the eviction victim.
const VICTIM_SAMPLE: usize = 4;
/// Window fractions simulated by the shadow caches (1.0 = plain LRU).
const SHADOW_FRACTIONS: [f64; 9] = [0.01, 0.03, 0.07, 0.15, 0.25, 0.4, 0.6, 0.8, 1.0];
/// Largest shadow; bigger caches feed their shadows a hash-sampled subset of keys.
const SHADOW_MAX: usize = 8192;
const SAMPLE_SALT: u64 = 0x5851_f42d_4c95_7f2d;
/// Half-life of the shadow hit scores, in multiples of the shadow capacity.
const HALF_LIFE_CAPACITIES: usize = 8;
/// Minimum number of sampled accesses between window decisions.
const MIN_PERIOD: usize = 64;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

#[derive(Debug, Clone, Copy)]
struct Link {
	hash: u64,
	prev: usize,
	next: usize,
	queue: u8,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize, // LRU
	tail: usize, // MRU
	len: usize,
}

impl List {
	const fn new() -> Self {
		Self { head: NIL, tail: NIL, len: 0 }
	}
}

/// Identity hasher for shadow maps, whose keys are already well-mixed 64-bit hashes.
#[derive(Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = self.0.rotate_left(8) ^ u64::from(b);
		}
	}

	fn write_u64(&mut self, n: u64) {
		self.0 = n;
	}
}

type HashIndex = HashMap<u64, usize, BuildHasherDefault<IdHasher>>;

/// Count-min sketch (4 rows, saturating 4-bit counters, conservative update) with
/// periodic halving so that stale popularity decays.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample_size: usize,
}

fn mix(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
	z ^ (z >> 31)
}

fn hash_of<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	h.finish()
}

fn window_size(capacity: usize, fraction: f64) -> usize {
	((fraction * capacity as f64).round() as usize).clamp(1, capacity)
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.saturating_mul(4).clamp(64, 1 << 24).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(64),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			let h = mix(hash.wrapping_add((row as u64 + 1).wrapping_mul(0x9e37_79b9_7f4a_7c15)));
			*slot = row * self.width + (h as usize & (self.width - 1));
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&i| self.table[i]).min().unwrap_or(0)
	}

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

/// W-TinyLFU replacement policy over arena slots: window LRU + segmented-LRU main space.
/// It only tracks slot order; the owner maps keys to slots.
#[derive(Debug)]
struct Policy {
	links: Vec<Link>,
	lists: [List; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	rng: u64,
}

impl Policy {
	fn new(capacity: usize, window_max: usize, seed: u64) -> Self {
		let mut policy = Self {
			links: Vec::with_capacity(capacity),
			lists: [List::new(); 3],
			capacity,
			window_max: 1,
			protected_max: 0,
			rng: seed | 1,
		};
		policy.resize_window(window_max);
		policy
	}

	fn is_full(&self) -> bool {
		self.links.len() >= self.capacity
	}

	/// Sets the window size; the protected segment gets 80% of the main space.
	/// Only moves entries between lists, never evicts.
	fn resize_window(&mut self, window_max: usize) {
		self.window_max = window_max.clamp(1, self.capacity);
		self.protected_max = (self.capacity - self.window_max) * 4 / 5;
		self.drain_window();
		self.demote_protected();
	}

	/// Places `hash` at the window MRU, in `slot` (freed by `evict`) or in a new slot.
	fn insert(&mut self, slot: Option<usize>, hash: u64) -> usize {
		let idx = match slot {
			Some(s) => {
				self.links[s].hash = hash;
				s
			}
			None => {
				self.links.push(Link { hash, prev: NIL, next: NIL, queue: WINDOW });
				self.links.len() - 1
			}
		};
		self.push_back(idx, WINDOW);
		self.drain_window();
		idx
	}

	/// Frees one slot (unlinked; its link still carries the evicted hash) and returns it.
	fn evict(&mut self, sketch: &Sketch) -> usize {
		let victim = self.select_victim(sketch);
		let window = self.lists[WINDOW as usize];
		let idx = if window.len >= self.window_max && window.head != NIL {
			// The window is about to overflow: its LRU entry must win admission into main.
			let candidate = window.head;
			if victim != NIL && self.admit(sketch, candidate, victim) {
				self.move_to(candidate, PROBATION);
				victim
			} else {
				candidate
			}
		} else if victim != NIL {
			victim
		} else {
			window.head
		};
		self.unlink(idx);
		idx
	}

	/// The least frequently used of the few oldest probation entries (ties go to the
	/// oldest), or the protected LRU entry when probation is empty.
	fn select_victim(&self, sketch: &Sketch) -> usize {
		let mut cur = self.lists[PROBATION as usize].head;
		if cur == NIL {
			return self.lists[PROTECTED as usize].head;
		}
		let mut best = cur;
		let mut best_freq = sketch.frequency(self.links[cur].hash);
		for _ in 1..VICTIM_SAMPLE {
			cur = self.links[cur].next;
			if cur == NIL || best_freq == 0 {
				break;
			}
			let freq = sketch.frequency(self.links[cur].hash);
			if freq < best_freq {
				best = cur;
				best_freq = freq;
			}
		}
		best
	}

	fn admit(&mut self, sketch: &Sketch, candidate: usize, victim: usize) -> bool {
		let cf = sketch.frequency(self.links[candidate].hash);
		let vf = sketch.frequency(self.links[victim].hash);
		if cf > vf {
			return true;
		}
		if cf >= WARM_CANDIDATE {
			// Occasionally let a warm candidate in so a stale main space can turn over.
			self.rng ^= self.rng << 13;
			self.rng ^= self.rng >> 7;
			self.rng ^= self.rng << 17;
			return self.rng & 127 == 0;
		}
		false
	}

	fn on_access(&mut self, idx: usize) {
		match self.links[idx].queue {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.demote_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	fn drain_window(&mut self) {
		while self.lists[WINDOW as usize].len > self.window_max {
			let head = self.lists[WINDOW as usize].head;
			self.move_to(head, PROBATION);
		}
	}

	fn demote_protected(&mut self) {
		while self.lists[PROTECTED as usize].len > self.protected_max {
			let head = self.lists[PROTECTED as usize].head;
			self.move_to(head, PROBATION);
		}
	}

	fn move_to(&mut self, idx: usize, queue: u8) {
		self.unlink(idx);
		self.push_back(idx, queue);
	}

	/// Remove a slot from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let Link { prev, next, queue, .. } = self.links[idx];
		if prev != NIL {
			self.links[prev].next = next;
		} else {
			self.lists[queue as usize].head = next;
		}
		if next != NIL {
			self.links[next].prev = prev;
		} else {
			self.lists[queue as usize].tail = prev;
		}
		self.lists[queue as usize].len -= 1;
		self.links[idx].prev = NIL;
		self.links[idx].next = NIL;
	}

	/// Append a slot to the MRU end (tail) of `queue`.
	fn push_back(&mut self, idx: usize, queue: u8) {
		let tail = self.lists[queue as usize].tail;
		let link = &mut self.links[idx];
		link.prev = tail;
		link.next = NIL;
		link.queue = queue;
		if tail != NIL {
			self.links[tail].next = idx;
		} else {
			self.lists[queue as usize].head = idx;
		}
		let list = &mut self.lists[queue as usize];
		list.tail = idx;
		list.len += 1;
	}
}

/// Key-only copy of the policy at a fixed window fraction, scored by its recent hits.
#[derive(Debug)]
struct Shadow {
	policy: Policy,
	map: HashIndex,
	fraction: f64,
	hits: u32,
	score: f64,
}

impl Shadow {
	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.hits += 1;
			self.policy.on_access(idx);
			return;
		}
		let slot = if self.policy.is_full() {
			let s = self.policy.evict(sketch);
			self.map.remove(&self.policy.links[s].hash);
			Some(s)
		} else {
			None
		};
		let idx = self.policy.insert(slot, hash);
		self.map.insert(hash, idx);
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> slot index
	nodes: Vec<Node<K, V>>, // arena, parallel to `policy.links`
	policy: Policy,
	sketch: Sketch,
	capacity: usize,
	last_miss: Option<u64>,
	shadows: Vec<Shadow>,
	sample_threshold: u64,
	ticks: usize,
	period: usize,
	decay: f64,
	current: usize, // shadow whose window fraction the real cache uses
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
		let shadow_cap = capacity.min(SHADOW_MAX);
		let sample_threshold = if capacity <= SHADOW_MAX {
			u64::MAX
		} else {
			(shadow_cap as f64 / capacity as f64 * u64::MAX as f64) as u64
		};
		let shadows = SHADOW_FRACTIONS
			.iter()
			.enumerate()
			.map(|(i, &fraction)| Shadow {
				policy: Policy::new(
					shadow_cap,
					window_size(shadow_cap, fraction),
					0x9e37_79b9_7f4a_7c15 ^ i as u64,
				),
				map: HashIndex::with_capacity_and_hasher(shadow_cap, Default::default()),
				fraction,
				hits: 0,
				score: 0.0,
			})
			.collect();
		let period = (shadow_cap / 8).max(MIN_PERIOD);
		let half_life = (HALF_LIFE_CAPACITIES * shadow_cap).max(1024) as f64;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			policy: Policy::new(
				capacity,
				window_size(capacity, SHADOW_FRACTIONS[0]),
				0x2545_f491_4f6c_dd1d,
			),
			sketch: Sketch::new(capacity),
			capacity,
			last_miss: None,
			shadows,
			sample_threshold,
			ticks: 0,
			period,
			decay: 0.5f64.powf(period as f64 / half_life),
			current: 0,
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
				self.last_miss = None;
				self.policy.on_access(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				// The driver normally follows a miss with a put of the same key; remember
				// it so that this access is not recorded twice.
				self.last_miss = Some(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; at capacity exactly one entry
	/// is evicted (window candidate or main victim, decided by TinyLFU admission) and its
	/// arena slot is reused.
	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_of(&key);
		if self.last_miss.take() != Some(hash) {
			self.record(hash);
		}
		// Key already exists: update value in place and treat it as an access
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.policy.on_access(idx);
			return;
		}
		let slot = if self.policy.is_full() {
			let s = self.policy.evict(&self.sketch);
			self.map.remove(&self.nodes[s].key);
			let node = &mut self.nodes[s];
			node.key = key.clone();
			node.value = value;
			Some(s)
		} else {
			self.nodes.push(Node { key: key.clone(), value });
			None
		};
		let idx = self.policy.insert(slot, hash);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Records one logical access: feeds the frequency sketch and the (sampled) shadows,
	/// and periodically re-chooses the window size.
	fn record(&mut self, hash: u64) {
		self.sketch.increment(hash);
		if mix(hash ^ SAMPLE_SALT) > self.sample_threshold {
			return;
		}
		let sketch = &self.sketch;
		for shadow in self.shadows.iter_mut() {
			shadow.access(hash, sketch);
		}
		self.ticks += 1;
		if self.ticks % self.period == 0 {
			self.adapt();
		}
	}

	/// Decays the shadow scores and adopts the window fraction of the best shadow
	/// (switching only when another shadow is strictly better than the current one).
	fn adapt(&mut self) {
		let decay = self.decay;
		for shadow in self.shadows.iter_mut() {
			shadow.score = shadow.score * decay + f64::from(shadow.hits);
			shadow.hits = 0;
		}
		let mut best = self.current;
		for (i, shadow) in self.shadows.iter().enumerate() {
			if shadow.score > self.shadows[best].score {
				best = i;
			}
		}
		if best != self.current {
			self.current = best;
			let window = window_size(self.capacity, self.shadows[best].fraction);
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

	/// Structural invariants: lists are consistent, cover every resident key exactly
	/// once, and the cache (and every shadow) never exceeds its capacity.
	fn check<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		assert!(c.map.len() <= c.capacity);
		assert_eq!(c.map.len(), c.nodes.len());
		assert_eq!(c.nodes.len(), c.policy.links.len());
		let p = &c.policy;
		let mut seen = 0;
		for queue in [WINDOW, PROBATION, PROTECTED] {
			let list = p.lists[queue as usize];
			let mut prev = NIL;
			let mut cur = list.head;
			let mut n = 0;
			while cur != NIL {
				let link = &p.links[cur];
				assert_eq!(link.queue, queue);
				assert_eq!(link.prev, prev);
				assert_eq!(link.hash, hash_of(&c.nodes[cur].key));
				assert_eq!(c.map.get(&c.nodes[cur].key), Some(&cur));
				prev = cur;
				cur = link.next;
				n += 1;
			}
			assert_eq!(prev, list.tail);
			assert_eq!(n, list.len);
			seen += n;
		}
		assert_eq!(seen, c.map.len());
		assert!(p.window_max >= 1 && p.window_max <= c.capacity);
		assert!(p.lists[WINDOW as usize].len <= p.window_max);
		assert!(p.lists[PROTECTED as usize].len <= p.protected_max);
		for s in &c.shadows {
			assert!(s.map.len() <= s.policy.capacity);
			assert_eq!(s.map.len(), s.policy.links.len());
		}
	}

	#[test]
	fn fill_to_capacity_keeps_everything() {
		let mut cache = LruCache::new(50);
		for k in 0..50 {
			cache.put(k, k * 10);
		}
		for k in 0..50 {
			assert_eq!(*cache.get(&k).unwrap(), k * 10);
		}
		check(&cache);
	}

	#[test]
	fn new_key_is_resident_after_put() {
		for cap in [1, 2, 3, 7, 100] {
			let mut cache = LruCache::new(cap);
			for k in 0..1_000i64 {
				let key = k % 37 * 13 + k / 37;
				cache.put(key, k);
				assert_eq!(*cache.get(&key).unwrap(), k);
			}
			check(&cache);
		}
	}

	#[test]
	fn eviction_bounds_size() {
		let mut cache = LruCache::new(3);
		for k in 0..10 {
			cache.put(k, k);
			check(&cache);
		}
		let resident = (0..10).filter(|k| cache.map.contains_key(k)).count();
		assert_eq!(resident, 3);
	}

	#[test]
	fn frequent_key_survives_scan() {
		let mut cache = LruCache::new(10);
		cache.put(0, 0);
		for _ in 0..5 {
			assert!(cache.get(&0).is_some());
		}
		for k in 1..400 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			if k % 12 == 0 {
				assert_eq!(*cache.get(&0).unwrap(), 0);
			}
		}
		check(&cache);
	}

	#[test]
	fn hot_set_beats_lru_under_scan_pollution() {
		// Hot keys 0..8 interleaved with one-hit wonders: strict LRU with capacity 10
		// never hits (reuse distance 23); W-TinyLFU must keep the hot set resident.
		let mut cache = LruCache::new(10);
		let mut hits = 0;
		let mut next_cold = 1_000;
		for round in 0..500 {
			for k in 0..8 {
				if cache.get(&k).is_some() {
					hits += 1;
				} else {
					cache.put(k, k);
				}
				for _ in 0..2 {
					if cache.get(&next_cold).is_none() {
						cache.put(next_cold, next_cold);
					}
					next_cold += 1;
				}
			}
			if round % 50 == 0 {
				check(&cache);
			}
		}
		assert!(hits > 2_000, "hits = {hits}");
	}

	#[test]
	fn victim_is_least_frequent_of_oldest_probation_entries() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		for _ in 0..3 {
			cache.get(&1); // window hits: key 1 becomes popular without leaving the window
		}
		cache.put(2, 20); // 1 drains to probation
		cache.put(3, 30); // 2 drains to probation behind 1
		cache.get(&3);
		cache.put(4, 40); // candidate 3 beats the cold victim 2, not the popular head 1
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
		check(&cache);
	}

	#[test]
	fn randomized_operations_keep_invariants_and_values() {
		let mut cache = LruCache::new(17);
		let mut shadow = HashMap::new();
		let mut x: u64 = 88_172_645_463_325_252;
		for step in 0..50_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = (x % 64) as i64;
			if x & 0x300 == 0 {
				cache.put(key, step);
				shadow.insert(key, step);
				assert_eq!(*cache.get(&key).unwrap(), step);
			} else if let Some(v) = cache.get(&key) {
				assert_eq!(Some(v), shadow.get(&key));
			} else {
				cache.put(key, step);
				shadow.insert(key, step);
			}
			if step % 997 == 0 {
				check(&cache);
			}
		}
		check(&cache);
	}

	#[test]
	fn adaptation_keeps_window_in_bounds() {
		let mut cache = LruCache::new(20);
		for k in 0..20_000i64 {
			// Phase change: frequency-friendly loop, then pure recency stream.
			let key = if k < 10_000 { (k * 7919) % 30 } else { k / 3 };
			if cache.get(&key).is_none() {
				cache.put(key, k);
			}
			let p = &cache.policy;
			assert!(p.window_max >= 1 && p.window_max + p.protected_max <= 20);
		}
		check(&cache);
	}

	#[test]
	fn large_capacity_samples_shadows() {
		let cap = SHADOW_MAX * 2;
		let mut cache = LruCache::new(cap);
		assert!(cache.shadows.iter().all(|s| s.policy.capacity == SHADOW_MAX));
		let mut x: u64 = 0x1234_5678_9abc_def1;
		for step in 0..60_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = x % 40_000;
			if cache.get(&key).is_none() {
				cache.put(key, step);
			}
		}
		check(&cache);
		assert!(cache.ticks > 0 && cache.ticks < 60_000);
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
		// W-TinyLFU: window candidate 2 replaces main victim 1 only when more popular.
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&2);
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
		cache.put(4, 40); // cold window candidate 3 is rejected
		cache.get(&4);
		cache.get(&4);
		cache.put(5, 50); // popular candidate 4 evicts victim 1
		cache.get(&5);
		cache.get(&5);
		cache.put(6, 60); // popular candidate 5 evicts victim 2

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
		cache.get(&3); // window candidate 3 now outranks probation victim 2
		cache.put(4, 40); // should evict key 2, not the hot key 1
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
		cache.get(&3); // window candidate 3 now outranks probation victim 1
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
		cache.put(3, 30); // cold candidate 2 rejected -> slot 1 reused for key 3
		cache.get(&3); // key 3 now outranks key 1
		cache.put(4, 40); // 3 admitted, victim 1 evicted -> slot 0 reused for key 4
		cache.get(&4);
		cache.get(&4); // key 4 now outranks key 3
		cache.put(5, 50); // 4 admitted, victim 3 evicted -> slot 1 reused AGAIN for key 5
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
