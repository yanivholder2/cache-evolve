// Adaptive W-TinyLFU cache (Caffeine-style), kept under the `LruCache` name for the JNI boundary.
// - Window LRU (~1% initially) admits every new key, so put-then-get always hits.
// - Main space is a segmented LRU: probation + protected (80% of main).
// - A count-min sketch (conservative update, periodic halving) estimates frequency; the window's
//   LRU candidate only displaces the probation LRU victim when it is more popular.
// - The window share is picked by racing miniature shadow caches (one per candidate share, fed a
//   hash-sampled slice of the stream) and adopting the share with the most recent hits.
// - Frequency ties are broken LRU-2 style: a key returning from a bounded ghost history of
//   evicted keys wins if its previous access is newer than the victim's last access.
// O(1) per operation, arena-based intrusive lists, no unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
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
	stamp: u64, // tick of the last access
	prior: u64, // last access before the most recent (re)insertion; 0 = unknown
	prev: usize,
	next: usize,
	queue: usize,
}

/// Count-min sketch: 4 rows, counters saturate at 15, conservative update, periodic halving.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize,
	additions: usize,
	sample_size: usize,
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
		let mut out = [0usize; 4];
		for (row, slot) in out.iter_mut().enumerate() {
			let mut x = hash.wrapping_add((row as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15));
			x = (x ^ (x >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
			x = (x ^ (x >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
			*slot = row * self.width + ((x ^ (x >> 31)) as usize & (self.width - 1));
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
		if self.additions >= self.sample_size {
			// Aging: halve every counter so stale popularity fades.
			self.table.iter_mut().for_each(|c| *c >>= 1);
			self.additions /= 2;
		}
	}
}

/// Window shares raced by the shadow caches (fraction of capacity given to the window LRU).
/// 0.82 fills the wide gap below pure LRU (1.0), where recency-biased phases tend to peak.
const SHARES: [f64; 10] = [0.01, 0.03, 0.07, 0.12, 0.2, 0.3, 0.45, 0.65, 0.82, 1.0];
/// Target size of a shadow cache; larger caches are modelled on a hash-sampled key subset.
const MINI_CAPACITY: usize = 1000;

/// Hasher for maps keyed by an already uniform 64-bit hash (re-mixed so sampled keys spread).
#[derive(Debug, Default)]
struct IdHasher(u64);

impl Hasher for IdHasher {
	fn finish(&self) -> u64 {
		self.0
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			let v = self.0 ^ u64::from(b);
			self.write_u64(v);
		}
	}

	fn write_u64(&mut self, n: u64) {
		let x = (n ^ self.0.rotate_left(29)).wrapping_mul(0x9E37_79B9_7F4A_7C15);
		self.0 = x ^ (x >> 32);
	}
}

#[derive(Debug, Clone, Copy)]
struct SNode {
	hash: u64,
	prev: usize,
	next: usize,
	queue: usize,
}

/// Miniature W-TinyLFU with a fixed window share, replaying a hash-sampled slice of the stream
/// against the shared sketch. Shadows race different shares; the main cache adopts the winner.
#[derive(Debug)]
struct Shadow {
	map: HashMap<u64, usize, BuildHasherDefault<IdHasher>>,
	nodes: Vec<SNode>,
	heads: [usize; 3],
	tails: [usize; 3],
	sizes: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	share: f64,
	score: f64, // exponentially decayed hit count
}

impl Shadow {
	fn new(capacity: usize, share: f64) -> Self {
		let window_max = ((capacity as f64 * share).round() as usize).clamp(1, capacity);
		Self {
			map: HashMap::with_capacity_and_hasher(capacity, Default::default()),
			nodes: Vec::with_capacity(capacity),
			heads: [NIL; 3],
			tails: [NIL; 3],
			sizes: [0; 3],
			capacity,
			window_max,
			protected_max: ((capacity - window_max) as f64 * 0.8) as usize,
			share,
			score: 0.0,
		}
	}

	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.score += 1.0;
			let queue = self.nodes[idx].queue;
			self.relink(idx, if queue == WINDOW { WINDOW } else { PROTECTED });
			while self.sizes[PROTECTED] > self.protected_max {
				let head = self.heads[PROTECTED];
				self.relink(head, PROBATION);
			}
			return;
		}
		let idx = if self.nodes.len() < self.capacity {
			self.nodes.push(SNode { hash, prev: NIL, next: NIL, queue: WINDOW });
			self.nodes.len() - 1
		} else {
			let evict = self.choose_evict(sketch);
			self.unlink(evict);
			self.map.remove(&self.nodes[evict].hash);
			self.nodes[evict].hash = hash;
			evict
		};
		self.push(idx, WINDOW);
		self.map.insert(hash, idx);
		while self.sizes[WINDOW] > self.window_max {
			let head = self.heads[WINDOW];
			self.relink(head, PROBATION);
		}
	}

	/// The window's LRU candidate enters probation only if the sketch rates it above the victim.
	fn choose_evict(&mut self, sketch: &Sketch) -> usize {
		let candidate = self.heads[WINDOW];
		let victim = if self.heads[PROBATION] != NIL { self.heads[PROBATION] } else { self.heads[PROTECTED] };
		if victim == NIL {
			return candidate;
		}
		if candidate == NIL || self.sizes[WINDOW] < self.window_max {
			return victim;
		}
		if sketch.frequency(self.nodes[candidate].hash) > sketch.frequency(self.nodes[victim].hash) {
			self.relink(candidate, PROBATION);
			victim
		} else {
			candidate
		}
	}

	fn relink(&mut self, idx: usize, queue: usize) {
		self.unlink(idx);
		self.push(idx, queue);
	}

	fn unlink(&mut self, idx: usize) {
		let SNode { prev, next, queue, .. } = self.nodes[idx];
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.heads[queue] = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.tails[queue] = prev;
		}
		self.sizes[queue] -= 1;
	}

	fn push(&mut self, idx: usize, queue: usize) {
		let tail = self.tails[queue];
		let node = &mut self.nodes[idx];
		node.prev = tail;
		node.next = NIL;
		node.queue = queue;
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.heads[queue] = idx;
		}
		self.tails[queue] = idx;
		self.sizes[queue] += 1;
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // arena slots released by eviction
	heads: [usize; 3],      // LRU end of window / probation / protected
	tails: [usize; 3],      // MRU end of each queue
	sizes: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	last_hash: Option<u64>, // hash already recorded by the immediately preceding get
	tick: u64,              // logical access clock
	ghost: HashMap<u64, u64>, // hash of a recently evicted key -> its last access tick
	ghost_fifo: VecDeque<(u64, u64)>, // eviction order bounding the ghost history
	shadows: Vec<Shadow>, // miniature caches racing the candidate window shares
	chosen: usize,        // index of the shadow whose share the window currently uses
	sample_rate: u64,     // shadows replay keys whose hash (high half) is divisible by this
	adapt_period: usize,  // accesses between window-share decisions
	until_adapt: usize,
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
		let window_max = capacity.saturating_sub((capacity as f64 * 0.99) as usize).max(1);
		let sample_rate = (capacity / MINI_CAPACITY).max(1);
		let mini = ((capacity as f64 / sample_rate as f64).round() as usize).max(1);
		let adapt_period = capacity.max(MINI_CAPACITY).saturating_mul(2);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			heads: [NIL; 3],
			tails: [NIL; 3],
			sizes: [0; 3],
			capacity,
			window_max,
			protected_max: ((capacity - window_max) as f64 * 0.8) as usize,
			sketch: Sketch::new(capacity),
			last_hash: None,
			tick: 0,
			ghost: HashMap::new(),
			ghost_fifo: VecDeque::new(),
			shadows: SHARES.iter().map(|&s| Shadow::new(mini, s)).collect(),
			chosen: 0,
			sample_rate: sample_rate as u64,
			adapt_period,
			until_adapt: adapt_period,
			rng: 0x2545_F491_4F6C_DD1D,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	fn hash_of(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Returns the value for `key` if present, recording the access for the policy.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = Self::hash_of(key);
		self.access(hash);
		self.last_hash = Some(hash);
		let idx = self.map.get(key).copied()?;
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys always enter the window; when full, the window's LRU
	/// entry competes with the main victim by estimated frequency.
	pub fn put(&mut self, key: K, value: V) {
		let hash = Self::hash_of(&key);
		// A put right after a get of the same key (miss-then-fill) is the same access.
		let counted = self.last_hash.take() == Some(hash);
		if !counted {
			self.access(hash);
		}
		if let Some(idx) = self.map.get(&key).copied() {
			self.nodes[idx].value = value;
			if !counted {
				self.on_hit(idx);
			}
			return;
		}
		// Last access of this key before it was evicted (if still in the ghost history).
		let prior = self.ghost.get(&hash).copied().unwrap_or(0);
		if self.map.len() >= self.capacity {
			self.evict_one();
		}
		let stamp = self.tick;
		let idx = match self.free.pop() {
			Some(i) => {
				let node = &mut self.nodes[i];
				node.key = key.clone();
				node.value = value;
				node.hash = hash;
				node.stamp = stamp;
				node.prior = prior;
				i
			}
			None => {
				self.nodes.push(Node {
					key: key.clone(),
					value,
					hash,
					stamp,
					prior,
					prev: NIL,
					next: NIL,
					queue: WINDOW,
				});
				self.nodes.len() - 1
			}
		};
		self.push_back(WINDOW, idx);
		self.map.insert(key, idx);
		self.rebalance_window();
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn on_hit(&mut self, idx: usize) {
		let tick = self.tick;
		self.nodes[idx].stamp = tick;
		let queue = self.nodes[idx].queue;
		if queue == PROBATION {
			self.move_to(PROTECTED, idx);
			self.trim_protected();
		} else {
			self.move_to(queue, idx);
		}
	}

	/// Frees exactly one slot: either the window candidate or the main victim leaves.
	fn evict_one(&mut self) {
		let candidate = self.heads[WINDOW];
		let victim = if self.heads[PROBATION] != NIL { self.heads[PROBATION] } else { self.heads[PROTECTED] };
		let evict = if candidate != NIL && self.sizes[WINDOW] >= self.window_max {
			// The incoming key overflows the window: its LRU entry must earn a place in main.
			if victim != NIL && self.admit(candidate, victim) {
				self.move_to(PROBATION, candidate);
				victim
			} else {
				candidate
			}
		} else if victim != NIL {
			victim // main is over its share (window was just grown)
		} else {
			candidate
		};
		self.unlink(evict);
		let (hash, stamp) = (self.nodes[evict].hash, self.nodes[evict].stamp);
		self.map.remove(&self.nodes[evict].key);
		self.free.push(evict);
		self.remember(hash, stamp);
	}

	/// Records an evicted key's last access in the bounded ghost history (FIFO by eviction).
	fn remember(&mut self, hash: u64, stamp: u64) {
		self.ghost.insert(hash, stamp);
		self.ghost_fifo.push_back((hash, stamp));
		while self.ghost_fifo.len() > self.capacity.saturating_mul(2) {
			if let Some((h, s)) = self.ghost_fifo.pop_front() {
				// Only drop the entry if it was not refreshed by a later eviction of the same key.
				if self.ghost.get(&h) == Some(&s) {
					self.ghost.remove(&h);
				}
			}
		}
	}

	fn admit(&mut self, candidate: usize, victim: usize) -> bool {
		let cf = self.sketch.frequency(self.nodes[candidate].hash);
		let vf = self.sketch.frequency(self.nodes[victim].hash);
		if cf > vf {
			return true;
		}
		// Tie-break by recency (LRU-2 style): the candidate returned from the ghost history and
		// its access before that is newer than the victim's last access, so both of its last two
		// accesses are more recent than anything the victim has. Bursts inside the window and
		// first-time keys (prior == 0) never qualify, keeping scan and loop resistance intact.
		if cf == vf && self.nodes[candidate].prior > self.nodes[victim].stamp {
			return true;
		}
		if cf < 6 {
			return false;
		}
		// Rare random admission of warm candidates avoids getting stuck behind a hot victim.
		self.rng ^= self.rng << 13;
		self.rng ^= self.rng >> 7;
		self.rng ^= self.rng << 17;
		self.rng & 127 == 0
	}

	/// Records one logical access: clock, frequency sketch, shadow caches, and periodically the
	/// window-share decision.
	fn access(&mut self, hash: u64) {
		self.tick += 1;
		self.sketch.increment(hash);
		if (hash >> 32) % self.sample_rate == 0 {
			for shadow in &mut self.shadows {
				shadow.access(hash, &self.sketch);
			}
		}
		self.until_adapt -= 1;
		if self.until_adapt == 0 {
			self.until_adapt = self.adapt_period;
			self.adapt();
		}
	}

	/// Moves the window to the share of the shadow with the most (decayed) hits. Ties keep the
	/// current share, so warm-up (no evictions yet, equal scores) changes nothing.
	fn adapt(&mut self) {
		let mut best = self.chosen;
		for (i, shadow) in self.shadows.iter().enumerate() {
			if shadow.score > self.shadows[best].score {
				best = i;
			}
		}
		for shadow in &mut self.shadows {
			shadow.score *= 0.8;
		}
		if best != self.chosen {
			self.chosen = best;
			let window = (self.shadows[best].share * self.capacity as f64).round() as usize;
			self.resize_window(window);
		}
	}

	/// Sets the window share; main keeps an 80% protected segment. Shrinking demotes window
	/// entries to probation; growing lets main evict its own victims until the window fills.
	fn resize_window(&mut self, window: usize) {
		self.window_max = window.clamp(1, self.capacity);
		self.protected_max = ((self.capacity - self.window_max) as f64 * 0.8) as usize;
		self.trim_protected();
		self.rebalance_window();
	}

	fn rebalance_window(&mut self) {
		while self.sizes[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.move_to(PROBATION, idx);
		}
	}

	fn trim_protected(&mut self) {
		while self.sizes[PROTECTED] > self.protected_max {
			let idx = self.heads[PROTECTED];
			self.move_to(PROBATION, idx);
		}
	}

	fn move_to(&mut self, queue: usize, idx: usize) {
		self.unlink(idx);
		self.push_back(queue, idx);
	}

	/// Remove node from its current queue.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, queue) = (self.nodes[idx].prev, self.nodes[idx].next, self.nodes[idx].queue);
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.heads[queue] = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.tails[queue] = prev;
		}
		self.sizes[queue] -= 1;
	}

	/// Append node to the MRU end of `queue`.
	fn push_back(&mut self, queue: usize, idx: usize) {
		let tail = self.tails[queue];
		let node = &mut self.nodes[idx];
		node.queue = queue;
		node.prev = tail;
		node.next = NIL;
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.heads[queue] = idx;
		}
		self.tails[queue] = idx;
		self.sizes[queue] += 1;
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
		cache.put(3, 30); // evicts exactly one older key; the new key is resident

		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(cache.map.len(), 2);
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
		cache.put(6, 60);

		assert_eq!(cache.map.len(), 3);
		assert_eq!(cache.nodes.len(), 3);
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
		cache.put(4, 40); // must evict an unpopular key, never the hot key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
								 // Order: 1 (LRU), 3, 2 (MRU)
		cache.put(4, 40); // one unpromoted entry is evicted
		assert_eq!(*cache.get(&2).unwrap(), 20); // promoted entry survives
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to MRU
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		cache.put(5, 50); // evicts one entry and reuses its slot for key 5
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(cache.map.len(), 2);
		assert_eq!(cache.nodes.len(), 2); // arena never grows past capacity
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

	fn check<K: Hash + Eq + Clone + Debug, V: Debug>(c: &LruCache<K, V>) {
		assert!(c.map.len() <= c.capacity);
		assert!(c.nodes.len() <= c.capacity);
		assert!(c.window_max >= 1 && c.window_max + c.protected_max <= c.capacity);
		assert!(c.sizes[WINDOW] <= c.window_max);
		assert!(c.sizes[PROTECTED] <= c.protected_max);
		assert_eq!(c.sizes.iter().sum::<usize>(), c.map.len());
		for q in 0..3 {
			let (mut cur, mut prev, mut n) = (c.heads[q], NIL, 0);
			while cur != NIL {
				assert_eq!(c.nodes[cur].queue, q);
				assert_eq!(c.nodes[cur].prev, prev);
				prev = cur;
				cur = c.nodes[cur].next;
				n += 1;
			}
			assert_eq!(c.tails[q], prev);
			assert_eq!(n, c.sizes[q]);
		}
		for (k, &i) in &c.map {
			assert_eq!(&c.nodes[i].key, k);
		}
		assert!(c.ghost.len() <= c.ghost_fifo.len());
		assert!(c.ghost_fifo.len() <= c.capacity * 2);
	}

	#[test]
	fn hot_key_survives_scan() {
		let mut cache = LruCache::new(10);
		for i in 0..4 {
			cache.put(i, i);
		}
		assert_eq!(*cache.get(&0).unwrap(), 0); // promoted into the protected segment
		for i in 4..3000 {
			if cache.get(&i).is_none() {
				cache.put(i, i);
			}
			if i % 15 == 0 {
				// Strict LRU would lose key 0 here: 14 distinct keys pass between accesses.
				assert_eq!(*cache.get(&0).unwrap(), 0);
			}
		}
		check(&cache);
	}

	#[test]
	fn randomized_operations_keep_invariants() {
		for &cap in &[1usize, 2, 3, 8, 64] {
			let mut cache = LruCache::new(cap);
			let mut model: HashMap<i64, u64> = HashMap::new();
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
			for step in 0..20_000u64 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let key = (x % (cap as u64 * 3 + 5)) as i64;
				let got = cache.get(&key).copied();
				match got {
					Some(v) => assert_eq!(model.get(&key), Some(&v)),
					None => {
						cache.put(key, step);
						model.insert(key, step);
						assert_eq!(cache.get(&key), Some(&step));
					}
				}
				if x % 7 == 0 {
					cache.put(key, step + 1);
					model.insert(key, step + 1);
				}
				if step % 97 == 0 {
					check(&cache);
				}
			}
			check(&cache);
		}
	}
}
