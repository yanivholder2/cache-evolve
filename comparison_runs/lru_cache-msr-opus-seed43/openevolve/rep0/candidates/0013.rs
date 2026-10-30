// W-TinyLFU cache: a small LRU "window" admits every new key; keys leaving the window
// must beat the main space's eviction victim on estimated frequency (Count-Min sketch
// with periodic aging) to enter the segmented-LRU main space (probation + protected).
// The configuration is tuned by miniature simulations: key-less shadow caches replay a
// hash-sampled sub-stream, each with a fixed window fraction (pure TinyLFU .. pure LRU)
// and frequency-history length (Caffeine's 10x capacity, or a longer 32x memory); the
// real cache adopts the configuration of the shadow with the best decayed hit count.
// Tiny capacities (< LRU_ONLY_BELOW) degenerate to strict LRU (window == capacity).
// Arena of nodes indexed by usize with intrusive links; no unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
const LRU_ONLY_BELOW: usize = 16;
/// Window sizes (fractions of capacity) evaluated by the shadow caches: pure TinyLFU .. pure LRU.
const FRACTIONS: [f64; 9] = [0.0, 0.01, 0.03, 0.08, 0.16, 0.3, 0.5, 0.75, 1.0];
/// Index into FRACTIONS used until the shadows have an opinion (Caffeine's 1% default).
const START_FRACTION: usize = 1;
/// Frequency sketches as (aging period, counter ceiling, row width), period and width in
/// multiples of capacity. Sketch 0 is Caffeine's; sketch 1 remembers about 3x longer.
const SKETCHES: [(usize, u8, usize); 2] = [(10, 15, 4), (32, 31, 8)];
/// Upper bound on a sketch row width (a power of two), keeping memory bounded for huge caches.
const MAX_SKETCH_WIDTH: usize = 1 << 23;
/// Large caches are simulated on a hash-sampled sub-stream scaled to about this many entries.
const MINI_TARGET: usize = 1024;
/// Per-period decay of the shadows' hit scores (memory of roughly ten periods).
const SCORE_DECAY: f64 = 0.9;
/// Relative score advantage required before the real cache switches configuration.
const SWITCH_MARGIN: f64 = 1.001;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

/// Intrusive queue links of one arena slot, plus the key hash used for frequency lookups.
#[derive(Debug, Clone, Copy)]
struct Link {
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

fn window_for(fraction: f64, capacity: usize) -> usize {
	((fraction * capacity as f64).round() as usize).clamp(1, capacity)
}

/// Cheap hasher for maps keyed by already well-mixed 64-bit key hashes (shadow caches).
#[derive(Default)]
struct MixHasher(u64);

impl Hasher for MixHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0 ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01B3);
		}
	}

	fn write_u64(&mut self, x: u64) {
		let x = (self.0 ^ x).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		self.0 = x ^ (x >> 32);
	}
}

type FastMap = HashMap<u64, usize, BuildHasherDefault<MixHasher>>;

/// W-TinyLFU queue structure (window LRU + probation/protected SLRU) over arena slots.
/// It only knows key hashes, so it serves both the real cache and the shadow caches.
#[derive(Debug)]
struct Segments {
	links: Vec<Link>,
	free: Vec<usize>, // reusable arena slots
	head: [usize; 3], // LRU end of window / probation / protected
	tail: [usize; 3], // MRU end of window / probation / protected
	len: [usize; 3],
	capacity: usize,
	window_cap: usize,
	protected_cap: usize,
}

impl Segments {
	fn new(capacity: usize, window_cap: usize) -> Self {
		let mut seg = Self {
			links: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			head: [NIL; 3],
			tail: [NIL; 3],
			len: [0; 3],
			capacity,
			window_cap: 1,
			protected_cap: 0,
		};
		seg.set_window(window_cap);
		seg
	}

	fn size(&self) -> usize {
		self.len[WINDOW] + self.len[PROBATION] + self.len[PROTECTED]
	}

	fn on_hit(&mut self, idx: usize) {
		let q = self.links[idx].queue;
		self.unlink(idx);
		if q == PROBATION {
			self.push_mru(PROTECTED, idx);
			self.shrink_protected();
		} else {
			self.push_mru(q, idx);
		}
	}

	/// Adds a new entry at the window MRU; returns its slot and the evicted slot, if any.
	/// The evicted slot keeps its old contents until a later insert reuses it, and the
	/// new entry itself is never the one evicted (window_cap >= 1).
	fn insert(&mut self, hash: u64, sketch: &Sketch) -> (usize, Option<usize>) {
		let link = Link {
			hash,
			prev: NIL,
			next: NIL,
			queue: WINDOW,
		};
		let idx = match self.free.pop() {
			Some(i) => {
				self.links[i] = link;
				i
			}
			None => {
				self.links.push(link);
				self.links.len() - 1
			}
		};
		self.push_mru(WINDOW, idx);
		// Window overflow: its LRU entry becomes the admission candidate (in probation MRU).
		let mut candidate = NIL;
		while self.len[WINDOW] > self.window_cap {
			candidate = self.head[WINDOW];
			self.unlink(candidate);
			self.push_mru(PROBATION, candidate);
		}
		if self.size() <= self.capacity {
			return (idx, None);
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
			if victim != NIL
				&& sketch.frequency(self.links[candidate].hash)
					> sketch.frequency(self.links[victim].hash)
			{
				victim
			} else {
				candidate
			}
		};
		self.unlink(loser);
		self.free.push(loser);
		(idx, Some(loser))
	}

	/// Resizes the window, moving entries across the window/main boundary (never evicts).
	fn set_window(&mut self, target: usize) {
		self.window_cap = target.clamp(1, self.capacity);
		let main_cap = self.capacity - self.window_cap;
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

	fn shrink_protected(&mut self) {
		while self.len[PROTECTED] > self.protected_cap {
			let idx = self.head[PROTECTED];
			self.unlink(idx);
			self.push_mru(PROBATION, idx);
		}
	}

	/// Remove slot from its queue.
	fn unlink(&mut self, idx: usize) {
		let Link {
			prev, next, queue, ..
		} = self.links[idx];
		if prev == NIL {
			self.head[queue] = next;
		} else {
			self.links[prev].next = next;
		}
		if next == NIL {
			self.tail[queue] = prev;
		} else {
			self.links[next].prev = prev;
		}
		self.len[queue] -= 1;
	}

	/// Append slot to the MRU end of queue `q`.
	fn push_mru(&mut self, q: usize, idx: usize) {
		let t = self.tail[q];
		{
			let link = &mut self.links[idx];
			link.prev = t;
			link.next = NIL;
			link.queue = q;
		}
		if t == NIL {
			self.head[q] = idx;
		} else {
			self.links[t].next = idx;
		}
		self.tail[q] = idx;
		self.len[q] += 1;
	}
}

/// Key-less W-TinyLFU replica with a fixed (window fraction, sketch) configuration,
/// fed a hash-sampled sub-stream.
#[derive(Debug)]
struct Shadow {
	seg: Segments,
	map: FastMap,
	fraction: f64,
	sketch: usize,
	hits: usize,
	score: f64,
}

impl Shadow {
	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.hits += 1;
			self.seg.on_hit(idx);
			return;
		}
		let (idx, evicted) = self.seg.insert(hash, sketch);
		if let Some(e) = evicted {
			let old = self.seg.links[e].hash;
			self.map.remove(&old);
		}
		self.map.insert(hash, idx);
	}
}

/// Miniature-simulation tuner: picks the configuration whose shadow scores the most hits.
#[derive(Debug)]
struct Tuner {
	shadows: Vec<Shadow>,
	modulus: u64,  // 1-in-`modulus` key sampling
	period: usize, // sampled events between decisions
	events: usize,
	best: usize,
}

impl Tuner {
	fn new(capacity: usize) -> Self {
		let modulus = (capacity / MINI_TARGET).max(1);
		let mini = ((capacity + modulus / 2) / modulus).max(1);
		let mut shadows = Vec::new();
		for sketch in 0..SKETCHES.len() {
			for &fraction in FRACTIONS.iter() {
				// Pure LRU never consults frequencies, so one instance of it suffices.
				if sketch > 0 && fraction >= 1.0 {
					continue;
				}
				shadows.push(Shadow {
					seg: Segments::new(mini, window_for(fraction, mini)),
					map: FastMap::with_capacity_and_hasher(mini + 1, BuildHasherDefault::default()),
					fraction,
					sketch,
					hits: 0,
					score: 0.0,
				});
			}
		}
		Self {
			shadows,
			modulus: modulus as u64,
			period: mini.max(256),
			events: 0,
			best: START_FRACTION, // shadow (FRACTIONS[START_FRACTION], sketch 0)
		}
	}

	/// Replays one access in every shadow; at the end of each period returns the best
	/// configuration as (window fraction, sketch index).
	fn record(&mut self, hash: u64, sketches: &[Sketch]) -> Option<(f64, usize)> {
		if (hash.wrapping_mul(0x9E37_79B9_7F4A_7C15) >> 32) % self.modulus != 0 {
			return None;
		}
		for shadow in self.shadows.iter_mut() {
			let s = shadow.sketch;
			shadow.access(hash, &sketches[s]);
		}
		self.events += 1;
		if self.events < self.period {
			return None;
		}
		self.events = 0;
		for shadow in self.shadows.iter_mut() {
			shadow.score = shadow.score * SCORE_DECAY + shadow.hits as f64;
			shadow.hits = 0;
		}
		let mut best = 0;
		for (i, shadow) in self.shadows.iter().enumerate() {
			if shadow.score > self.shadows[best].score {
				best = i;
			}
		}
		if self.shadows[best].score > self.shadows[self.best].score * SWITCH_MARGIN {
			self.best = best;
		}
		let chosen = &self.shadows[self.best];
		Some((chosen.fraction, chosen.sketch))
	}
}

/// Count-Min sketch (4 rows, saturating counters) with periodic halving (aging).
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample_size: usize,
	ceiling: u8,
}

impl Sketch {
	/// `config` is (aging period, counter ceiling, row width); period and width are in
	/// multiples of capacity.
	fn new(capacity: usize, config: (usize, u8, usize)) -> Self {
		let (period, ceiling, width_mult) = config;
		let base = capacity.max(16);
		let width = (base * width_mult).next_power_of_two().min(MAX_SKETCH_WIDTH);
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample_size: base * period,
			ceiling,
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
		if min >= self.ceiling {
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
	map: HashMap<K, usize>, // key -> arena slot
	entries: Vec<Node<K, V>>, // arena: key/value per slot (queue links live in `seg`)
	seg: Segments,
	capacity: usize,
	sketches: Vec<Sketch>, // every sketch configuration, kept current on each access
	active: usize, // index of the sketch consulted by the real cache's admission filter
	tuner: Option<Tuner>, // None for tiny capacities (strict LRU)
	last_miss: Option<u64>, // hash of the key that just missed in get (avoid double counting)
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
		let window_cap = if adaptive {
			window_for(FRACTIONS[START_FRACTION], capacity)
		} else {
			capacity
		};
		// Strict-LRU mode never consults frequencies, so it keeps just one (small) sketch.
		let sketch_count = if adaptive { SKETCHES.len() } else { 1 };
		let sketches: Vec<Sketch> = SKETCHES[..sketch_count]
			.iter()
			.map(|&config| Sketch::new(capacity, config))
			.collect();
		Some(Self {
			map: HashMap::with_capacity(capacity + 1),
			entries: Vec::with_capacity(capacity + 1),
			seg: Segments::new(capacity, window_cap),
			capacity,
			sketches,
			active: 0,
			tuner: if adaptive { Some(Tuner::new(capacity)) } else { None },
			last_miss: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.observe(hash); // may resize the window, but never evicts
		match self.map.get(key).copied() {
			Some(idx) => {
				self.last_miss = None;
				self.seg.on_hit(idx);
				Some(&self.entries[idx].value)
			}
			None => {
				self.last_miss = Some(hash);
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys enter the window; at capacity the
	/// TinyLFU filter decides between the window's outgoing key and the main victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(idx) = self.map.get(&key).copied() {
			self.last_miss = None;
			self.entries[idx].value = value;
			self.seg.on_hit(idx);
			return;
		}
		let hash = hash_key(&key);
		// If this put follows a get-miss for the same key, the access is already counted.
		if self.last_miss.take() != Some(hash) {
			self.observe(hash);
		}
		let (idx, evicted) = self.seg.insert(hash, &self.sketches[self.active]);
		if let Some(e) = evicted {
			debug_assert_ne!(e, idx);
			self.map.remove(&self.entries[e].key);
		}
		let node = Node {
			key: key.clone(),
			value,
		};
		if idx == self.entries.len() {
			self.entries.push(node);
		} else {
			self.entries[idx] = node;
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Counts one access in every frequency sketch and replays it in the shadow caches,
	/// adopting the best-scoring (window size, sketch) at the end of each tuning period.
	fn observe(&mut self, hash: u64) {
		for sketch in self.sketches.iter_mut() {
			sketch.increment(hash);
		}
		if let Some(tuner) = self.tuner.as_mut() {
			if let Some((fraction, sketch)) = tuner.record(hash, &self.sketches) {
				self.active = sketch;
				self.seg.set_window(window_for(fraction, self.capacity));
			}
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

	#[test]
	fn sampled_tuner_respects_capacity_and_keeps_new_keys() {
		let mut cache = LruCache::new(3000);
		let mut x: u64 = 12345;
		for i in 0..60_000i64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = if i % 3 == 0 { (x % 20_000) as i64 } else { (x % 2_000) as i64 };
			if cache.get(&k).is_none() {
				cache.put(k, i);
				assert!(cache.get(&k).is_some());
			}
			assert!(cache.map.len() <= 3000);
		}
		assert!(cache.seg.window_cap >= 1 && cache.seg.window_cap <= 3000);
		assert!(cache.active < SKETCHES.len());
	}

	#[test]
	fn tiny_capacity_stays_strict_lru() {
		let cache = LruCache::<i64, i64>::new(8);
		assert!(cache.tuner.is_none());
		assert_eq!(cache.sketches.len(), 1);
		assert_eq!(cache.seg.window_cap, 8);
	}
}
