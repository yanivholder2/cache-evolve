// Ghost-guided adaptive W-TinyLFU cache.
//
// The resident policy is W-TinyLFU: a small LRU admission window in front of a segmented-LRU
// main space (probation + protected), guarded by a count-min frequency sketch. Rather than a
// hill climber that compares hit rates of *different* time periods (and so confuses workload
// drift with the effect of its own moves), a grid of key-less shadow ("ghost") W-TinyLFU
// caches with fixed window sizes replays the *same* access stream, sharing the sketch. Every
// period the resident cache adopts the window size of the ghost with the best recent hits.
// Large capacities use spatially sampled ghosts (SHARDS-style) to bound their cost.
// For tiny capacities the window spans the whole cache, i.e. the policy is strict LRU.
// Arena-based doubly-linked lists indexed by usize; no unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: usize = 0;
const PROBATION: usize = 1;
const PROTECTED: usize = 2;
/// Below this capacity the cache is plain LRU (frequency admission needs room to pay off).
const ADAPTIVE_MIN_CAPACITY: usize = 32;
/// Window sizes (fractions of capacity) simulated by the ghosts; 0.0 means a 1-entry window
/// and 1.0 means pure LRU.
const GHOST_WINDOWS: [f64; 9] = [0.0, 0.01, 0.03, 0.08, 0.16, 0.3, 0.5, 0.75, 1.0];
/// Ghosts for larger capacities are spatially sampled down to about this many entries.
const GHOST_LIMIT: usize = 1 << 15;
/// Per-period decay of ghost scores (an exponentially weighted hit count).
const SCORE_DECAY: f64 = 0.75;
/// A rival ghost must beat the current one by this factor to trigger a switch (hysteresis).
const SWITCH_MARGIN: f64 = 1.002;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

#[derive(Debug, Clone, Copy)]
struct Link {
	prev: usize,
	next: usize,
	queue: u8,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize, // LRU end
	tail: usize, // MRU end
	len: usize,
}

impl List {
	const EMPTY: List = List {
		head: NIL,
		tail: NIL,
		len: 0,
	};
}

const SEEDS: [u64; 4] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0xD6E8_FEB8_6659_FD93,
];

/// Count-min sketch (4 rows, counters saturating at 15, conservative update) with
/// periodic halving so that stale popularity ages out.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	mask: usize,
	additions: usize,
	sample_size: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let base = capacity.max(16);
		let width = base.saturating_mul(4).next_power_of_two();
		Self {
			table: vec![0; width * 4],
			width,
			mask: width - 1,
			additions: 0,
			sample_size: base.saturating_mul(10),
		}
	}

	#[inline]
	fn slots(&self, hash: u64) -> [usize; 4] {
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			let h = (hash ^ SEEDS[row].rotate_left(17)).wrapping_mul(SEEDS[row]);
			*slot = row * self.width + (((h >> 32) as usize) & self.mask);
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

/// Deterministic FNV-1a hasher (finalised with a SplitMix64 mix in `hash_key`).
struct FnvHasher(u64);

impl Hasher for FnvHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 ^= u64::from(b);
			self.0 = self.0.wrapping_mul(0x0000_0100_0000_01B3);
		}
	}
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut hasher = FnvHasher(0xCBF2_9CE4_8422_2325);
	key.hash(&mut hasher);
	let mut z = hasher.finish();
	z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
	z ^ (z >> 31)
}

/// W-TinyLFU replacement state over arena slots (no keys or values): an LRU window in front
/// of a segmented-LRU main space. Owners map their keys to the slots it hands out.
#[derive(Debug)]
struct Core {
	links: Vec<Link>,
	hashes: Vec<u64>,
	free: Vec<usize>, // recycled slots
	lists: [List; 3], // window, probation, protected
	capacity: usize,
	max_window: usize,
	max_protected: usize,
}

impl Core {
	fn new(capacity: usize, window: usize) -> Self {
		let mut core = Self {
			links: Vec::with_capacity(capacity.saturating_add(1)),
			hashes: Vec::with_capacity(capacity.saturating_add(1)),
			free: Vec::new(),
			lists: [List::EMPTY; 3],
			capacity,
			max_window: capacity,
			max_protected: 0,
		};
		core.set_window(window);
		core
	}

	fn len(&self) -> usize {
		self.lists[WINDOW].len + self.main_len()
	}

	fn main_len(&self) -> usize {
		self.lists[PROBATION].len + self.lists[PROTECTED].len
	}

	/// Allocate a slot for a new entry and place it at the window's MRU end.
	fn insert(&mut self, hash: u64) -> usize {
		let idx = match self.free.pop() {
			Some(slot) => {
				self.hashes[slot] = hash;
				slot
			}
			None => {
				self.links.push(Link {
					prev: NIL,
					next: NIL,
					queue: 0,
				});
				self.hashes.push(hash);
				self.links.len() - 1
			}
		};
		self.push_back(WINDOW, idx);
		idx
	}

	/// Hit: window entries move to the window MRU; main entries move to the protected MRU.
	fn on_access(&mut self, idx: usize) {
		let queue = self.links[idx].queue as usize;
		self.unlink(idx);
		if queue == WINDOW {
			self.push_back(WINDOW, idx);
		} else {
			self.push_back(PROTECTED, idx);
			self.demote_protected_overflow();
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.lists[PROTECTED].len > self.max_protected {
			let idx = self.lists[PROTECTED].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn main_victim(&self) -> usize {
		let probation = self.lists[PROBATION].head;
		if probation != NIL {
			probation
		} else {
			self.lists[PROTECTED].head
		}
	}

	/// Restore the size bounds after an insertion: window overflow is resolved by a TinyLFU
	/// frequency contest against the main-space victim. Released slots go to `evicted`.
	fn evict(&mut self, sketch: &Sketch, evicted: &mut Vec<usize>) {
		let main_cap = self.capacity - self.max_window;
		while self.lists[WINDOW].len > self.max_window {
			let candidate = self.lists[WINDOW].head;
			self.unlink(candidate);
			if self.main_len() < main_cap {
				self.push_back(PROBATION, candidate);
				continue;
			}
			let victim = self.main_victim();
			if victim != NIL
				&& sketch.frequency(self.hashes[candidate]) > sketch.frequency(self.hashes[victim])
			{
				self.unlink(victim);
				self.release(victim, evicted);
				self.push_back(PROBATION, candidate);
			} else {
				self.release(candidate, evicted);
			}
		}
		while self.len() > self.capacity {
			let mut victim = self.main_victim();
			if victim == NIL {
				victim = self.lists[WINDOW].head;
			}
			self.unlink(victim);
			self.release(victim, evicted);
		}
	}

	fn release(&mut self, idx: usize, evicted: &mut Vec<usize>) {
		self.free.push(idx);
		evicted.push(idx);
	}

	/// Resize the window, moving entries across the window/main boundary to keep the bounds.
	fn set_window(&mut self, window: usize) {
		self.max_window = window.clamp(1, self.capacity);
		let main_cap = self.capacity - self.max_window;
		self.max_protected = main_cap * 4 / 5;
		// Window grew: hand main-space LRU entries to the window so totals stay bounded.
		while self.lists[WINDOW].len < self.max_window && self.main_len() > main_cap {
			let idx = self.main_victim();
			self.unlink(idx);
			self.push_back(WINDOW, idx);
		}
		// Window shrank: demote its LRU entries into probation.
		while self.lists[WINDOW].len > self.max_window {
			let idx = self.lists[WINDOW].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
		self.demote_protected_overflow();
	}

	/// Remove a slot from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let Link { prev, next, queue } = self.links[idx];
		let q = queue as usize;
		if prev == NIL {
			self.lists[q].head = next;
		} else {
			self.links[prev].next = next;
		}
		if next == NIL {
			self.lists[q].tail = prev;
		} else {
			self.links[next].prev = prev;
		}
		self.lists[q].len -= 1;
	}

	/// Append a slot to the MRU end of the given list.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.lists[queue].tail;
		self.links[idx] = Link {
			prev: tail,
			next: NIL,
			queue: queue as u8,
		};
		if tail == NIL {
			self.lists[queue].head = idx;
		} else {
			self.links[tail].next = idx;
		}
		self.lists[queue].tail = idx;
		self.lists[queue].len += 1;
	}
}

/// Pass-through hasher for maps keyed by already well-mixed 64-bit hashes.
#[derive(Debug, Default, Clone, Copy)]
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

/// Key-less shadow W-TinyLFU with a fixed window size, fed the same (possibly sampled)
/// access stream as the resident cache; it only counts the hits it would have had.
#[derive(Debug)]
struct Ghost {
	core: Core,
	map: HashIndex,
	evicted: Vec<usize>,
	window: usize, // window size this ghost represents, at the resident cache's scale
	hits: u64,
	score: f64,
}

impl Ghost {
	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.core.on_access(idx);
			self.hits += 1;
			return;
		}
		let idx = self.core.insert(hash);
		self.map.insert(hash, idx);
		self.core.evict(sketch, &mut self.evicted);
		for &slot in &self.evicted {
			self.map.remove(&self.core.hashes[slot]);
		}
		self.evicted.clear();
	}
}

fn window_size(fraction: f64, capacity: usize) -> usize {
	((fraction * capacity as f64).round() as usize).clamp(1, capacity)
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,           // key -> arena slot
	entries: Vec<Option<Node<K, V>>>, // payloads, indexed by core slot
	core: Core,                       // resident replacement policy
	evicted: Vec<usize>,              // scratch: slots released by the core
	sketch: Sketch,
	adaptive: bool,
	last_miss: Option<u64>, // hash of the key whose get just missed (avoid double counting)
	// ghost-guided window selection
	ghosts: Vec<Ghost>,
	choice: usize,    // ghost whose window size the resident cache currently uses
	sample_mask: u64, // ghosts see keys with (hash >> 32) & sample_mask == 0
	period: usize,
	ticks: usize,
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
		let mut ghosts: Vec<Ghost> = Vec::new();
		let mut sample_mask = 0u64;
		if adaptive {
			let rate = (capacity / GHOST_LIMIT).next_power_of_two();
			sample_mask = rate as u64 - 1;
			let ghost_cap = (capacity / rate).max(ADAPTIVE_MIN_CAPACITY);
			for &fraction in GHOST_WINDOWS.iter() {
				let window = window_size(fraction, capacity);
				if ghosts.last().map_or(false, |g| g.window == window) {
					continue;
				}
				ghosts.push(Ghost {
					core: Core::new(ghost_cap, window_size(fraction, ghost_cap)),
					map: HashIndex::with_capacity_and_hasher(ghost_cap + 1, Default::default()),
					evicted: Vec::new(),
					window,
					hits: 0,
					score: 0.0,
				});
			}
		}
		let window = ghosts.first().map_or(capacity, |g| g.window);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			entries: Vec::with_capacity(capacity.saturating_add(1)),
			core: Core::new(capacity, window),
			evicted: Vec::new(),
			sketch: Sketch::new(if adaptive { capacity } else { 1 }),
			adaptive,
			last_miss: None,
			ghosts,
			choice: 0,
			sample_mask,
			period: capacity.max(256),
			ticks: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access (frequency + recency).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let found = self.map.get(key).copied();
		if self.adaptive {
			let hash = match found {
				Some(idx) => self.core.hashes[idx],
				None => hash_key(key),
			};
			self.sketch.increment(hash);
			self.last_miss = if found.is_some() { None } else { Some(hash) };
			self.observe(hash);
		}
		let idx = found?;
		self.core.on_access(idx);
		self.entries[idx].as_ref().map(|node| &node.value)
	}

	/// Inserts or updates `key`. New keys enter the admission window; overflow is
	/// resolved by a TinyLFU frequency contest against the main-space victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(idx) = self.map.get(&key).copied() {
			if let Some(node) = self.entries[idx].as_mut() {
				node.value = value;
			}
			self.core.on_access(idx);
			return;
		}
		let hash = hash_key(&key);
		if self.adaptive && self.last_miss != Some(hash) {
			self.sketch.increment(hash);
		}
		self.last_miss = None;
		let idx = self.core.insert(hash);
		let node = Some(Node {
			key: key.clone(),
			value,
		});
		if idx == self.entries.len() {
			self.entries.push(node);
		} else {
			self.entries[idx] = node;
		}
		self.map.insert(key, idx);
		self.core.evict(&self.sketch, &mut self.evicted);
		for &slot in &self.evicted {
			if let Some(node) = self.entries[slot].take() {
				self.map.remove(&node.key);
			}
		}
		self.evicted.clear();
		debug_assert!(self.map.len() <= self.core.capacity);
	}

	/// Feed the ghosts (sampled stream) and, once per period, adopt the window size of the
	/// ghost with the best exponentially weighted hit count.
	fn observe(&mut self, hash: u64) {
		if (hash >> 32) & self.sample_mask == 0 {
			for ghost in self.ghosts.iter_mut() {
				ghost.access(hash, &self.sketch);
			}
		}
		self.ticks += 1;
		if self.ticks < self.period {
			return;
		}
		self.ticks = 0;
		for ghost in self.ghosts.iter_mut() {
			ghost.score = ghost.score * SCORE_DECAY + ghost.hits as f64;
			ghost.hits = 0;
		}
		let current = self.ghosts[self.choice].score;
		let mut best = self.choice;
		for (i, ghost) in self.ghosts.iter().enumerate() {
			if ghost.score > self.ghosts[best].score {
				best = i;
			}
		}
		if best != self.choice && self.ghosts[best].score > current * SWITCH_MARGIN {
			self.choice = best;
			self.core.set_window(self.ghosts[best].window);
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
	fn frequent_keys_survive_scan() {
		// Adaptive mode: hot keys promoted to the protected segment must not be
		// flushed by a long scan of one-hit-wonders (LRU would lose all of them).
		let mut cache = LruCache::new(100);
		for k in 0..50i64 {
			cache.put(k, k);
		}
		for _ in 0..5 {
			for k in 0..50i64 {
				assert_eq!(*cache.get(&k).unwrap(), k);
			}
		}
		for k in 1_000..2_000i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for k in 0..49i64 {
			assert_eq!(*cache.get(&k).unwrap(), k);
		}
	}

	#[test]
	fn adaptive_cache_stays_consistent() {
		// Mixed skewed/recency workload exercising ghost-driven window switches.
		let mut cache = LruCache::new(64);
		let mut x: u64 = 12345;
		for step in 0..50_000u64 {
			x = x
				.wrapping_mul(6_364_136_223_846_793_005)
				.wrapping_add(1_442_695_040_888_963_407);
			let key = if step % 3 == 0 {
				(x >> 33) % 40
			} else {
				(step / 2) % 500 + (x >> 60)
			};
			let hit = cache.get(&key).copied();
			match hit {
				Some(v) => assert_eq!(v, key * 7),
				None => cache.put(key, key * 7),
			}
			assert!(cache.map.len() <= 64);
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
