// W-TinyLFU cache whose window size is picked by miniature (ghost) simulations.
//
// * Window (LRU): every new key enters here, so a freshly put key is always resident
//   and bursts of recency are absorbed.
// * Main = SLRU (probation / protected 80%): a probation hit moves to protected;
//   protected overflow is demoted back to probation.
// * TinyLFU admission: when the window overflows, its LRU entry (candidate) must beat
//   the probation LRU entry (victim) on estimated frequency, else it is dropped. The
//   count-min sketch uses 4-bit counters halved every 10 * capacity additions.
// * Window sizing: a hash-sampled slice of the key space (SHARDS-style spatial
//   sampling) drives several scaled-down ghost caches running the very same policy
//   with different fixed window fractions (1% .. 100%; 100% is exactly LRU). They
//   share the real frequency sketch (key frequency is independent of cache content).
//   Periodically the real cache adopts the window of the ghost with the best
//   exponentially-decayed hit count.
//
// The real cache and the ghosts share one `Policy` that manages slot indices threaded
// through three intrusive lists; evicted slots are reused in place.
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

/// Window fractions (of capacity) evaluated by the ghost caches.
const FRACTIONS: [f64; 9] = [0.01, 0.03, 0.07, 0.15, 0.25, 0.4, 0.6, 0.8, 1.0];
/// Fraction used until the ghosts have an opinion (Caffeine's default 1%).
const INITIAL_CHOICE: usize = 0;
/// Target number of entries per ghost cache (sets the key sampling rate).
const GHOST_TARGET: usize = 4096;
/// Minimum number of sampled accesses between two window decisions.
const MIN_PERIOD: usize = 512;
/// Weight kept by older ghost hit counts at every decision.
const DECAY: f64 = 0.75;
/// Share of the main region reserved for the protected segment.
const PROTECTED_SHARE: f64 = 0.8;

/// Count-min sketch with saturating (<= 15) counters, conservative update and aging.
#[derive(Debug)]
struct Sketch {
	table: Vec<u8>,
	width: usize, // per-row width, power of two
	additions: usize,
	sample_size: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.clamp(16, 1 << 20).next_power_of_two() * 4;
		Self {
			table: vec![0; width * 4],
			width,
			additions: 0,
			sample_size: capacity.saturating_mul(10).max(10),
		}
	}

	fn slots(&self, hash: u64) -> [usize; 4] {
		let mask = self.width - 1;
		let mut out = [0usize; 4];
		for (i, slot) in out.iter_mut().enumerate() {
			let h = mix(hash.wrapping_add((i as u64 + 1).wrapping_mul(0x9E37_79B9_7F4A_7C15)));
			*slot = i * self.width + ((h as usize) & mask);
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
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions /= 2;
		}
	}
}

fn mix(mut z: u64) -> u64 {
	z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
	z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
	z ^ (z >> 31)
}

fn hash_of<T: Hash + ?Sized>(value: &T) -> u64 {
	let mut hasher = DefaultHasher::new();
	value.hash(&mut hasher);
	mix(hasher.finish())
}

/// Window size (in entries) for a fraction of `capacity`; always in 1..=capacity.
fn window_for(capacity: usize, fraction: f64) -> usize {
	((capacity as f64 * fraction).round() as usize).clamp(1, capacity)
}

/// W-TinyLFU replacement decisions over slot indices (keys/values live elsewhere).
#[derive(Debug)]
struct Policy {
	hashes: Vec<u64>, // key hash per slot
	prev: Vec<usize>,
	next: Vec<usize>,
	queue: Vec<usize>,
	heads: [usize; 3], // LRU end of window / probation / protected
	tails: [usize; 3], // MRU end of window / probation / protected
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
}

impl Policy {
	fn new(capacity: usize, window_max: usize) -> Self {
		let mut policy = Self {
			hashes: Vec::with_capacity(capacity),
			prev: Vec::with_capacity(capacity),
			next: Vec::with_capacity(capacity),
			queue: Vec::with_capacity(capacity),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max: 1,
			protected_max: 0,
		};
		policy.resize(window_max);
		policy
	}

	/// Sets a new window size; overflowing entries move into probation.
	fn resize(&mut self, window_max: usize) {
		self.window_max = window_max.clamp(1, self.capacity);
		self.protected_max =
			((self.capacity - self.window_max) as f64 * PROTECTED_SHARE) as usize;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.detach(idx);
			self.attach(idx, PROBATION);
		}
		self.demote_overflow();
	}

	/// Places a new key. Returns its slot and, if the slot was recycled, the hash of
	/// the evicted occupant.
	fn insert(&mut self, hash: u64, sketch: &Sketch) -> (usize, Option<u64>) {
		let (idx, evicted) = if self.hashes.len() >= self.capacity {
			let slot = self.evict(sketch);
			(slot, Some(std::mem::replace(&mut self.hashes[slot], hash)))
		} else {
			self.hashes.push(hash);
			self.prev.push(NIL);
			self.next.push(NIL);
			self.queue.push(WINDOW);
			(self.hashes.len() - 1, None)
		};
		self.attach(idx, WINDOW);
		if self.lens[WINDOW] > self.window_max {
			// Still filling up: overflow the window into probation without eviction.
			let head = self.heads[WINDOW];
			self.detach(head);
			self.attach(head, PROBATION);
		}
		(idx, evicted)
	}

	/// Detaches and returns the slot to recycle for an incoming key.
	fn evict(&mut self, sketch: &Sketch) -> usize {
		let victim = if self.lens[WINDOW] >= self.window_max && self.heads[WINDOW] != NIL {
			// The window's LRU entry becomes the admission candidate for main.
			let cand = self.heads[WINDOW];
			self.detach(cand);
			self.attach(cand, PROBATION);
			let mut rival = self.heads[PROBATION];
			if rival == cand {
				rival = self.heads[PROTECTED];
			}
			if rival == NIL {
				cand
			} else if sketch.frequency(self.hashes[cand]) > sketch.frequency(self.hashes[rival])
			{
				rival
			} else {
				cand
			}
		} else {
			// Window below its target size (it just grew): let it grow by evicting
			// main's least recently used entry.
			[PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.heads[q])
				.find(|&i| i != NIL)
				.expect("full cache has an entry")
		};
		self.detach(victim);
		victim
	}

	/// Hit/update handling: window stays LRU, main entries go to protected.
	fn on_access(&mut self, idx: usize) {
		let q = self.queue[idx];
		self.detach(idx);
		if q == WINDOW {
			self.attach(idx, WINDOW);
		} else {
			self.attach(idx, PROTECTED);
			self.demote_overflow();
		}
	}

	fn demote_overflow(&mut self) {
		while self.lens[PROTECTED] > self.protected_max {
			let h = self.heads[PROTECTED];
			self.detach(h);
			self.attach(h, PROBATION);
		}
	}

	/// Remove slot from the list it currently belongs to.
	fn detach(&mut self, idx: usize) {
		let (p, n, q) = (self.prev[idx], self.next[idx], self.queue[idx]);
		if p == NIL {
			self.heads[q] = n;
		} else {
			self.next[p] = n;
		}
		if n == NIL {
			self.tails[q] = p;
		} else {
			self.prev[n] = p;
		}
		self.lens[q] -= 1;
	}

	/// Append slot at the MRU end of list `q`.
	fn attach(&mut self, idx: usize, q: usize) {
		let t = self.tails[q];
		self.prev[idx] = t;
		self.next[idx] = NIL;
		self.queue[idx] = q;
		if t == NIL {
			self.heads[q] = idx;
		} else {
			self.next[t] = idx;
		}
		self.tails[q] = idx;
		self.lens[q] += 1;
	}
}

/// Scaled-down key-only cache with a fixed window fraction, fed by sampled keys.
#[derive(Debug)]
struct Ghost {
	map: HashMap<u64, usize>, // key hash -> slot
	policy: Policy,
	score: f64, // decayed hit count
}

impl Ghost {
	fn new(capacity: usize, fraction: f64) -> Self {
		Self {
			map: HashMap::with_capacity(capacity),
			policy: Policy::new(capacity, window_for(capacity, fraction)),
			score: 0.0,
		}
	}

	/// Simulator semantics: a hit is promoted, a miss is inserted.
	fn access(&mut self, hash: u64, sketch: &Sketch) {
		if let Some(&idx) = self.map.get(&hash) {
			self.policy.on_access(idx);
			self.score += 1.0;
			return;
		}
		let (idx, evicted) = self.policy.insert(hash, sketch);
		if let Some(old) = evicted {
			self.map.remove(&old);
		}
		self.map.insert(hash, idx);
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> slot index
	nodes: Vec<Node<K, V>>, // arena, parallel to the policy's slots
	policy: Policy,
	capacity: usize,
	sketch: Sketch,
	pending_miss: Option<u64>, // hash of the last get() miss (avoids double counting on put)
	ghosts: Vec<Ghost>,
	sample_rate: u64,
	samples: usize,
	period: usize,
	choice: usize, // index into FRACTIONS currently used by the real cache
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
		let sample_rate = (capacity / GHOST_TARGET).max(1);
		let ghost_capacity = (capacity / sample_rate).max(1);
		let ghost_count = FRACTIONS.len();
		log::info!(
			"LruCache.try_new: capacity={capacity} (W-TinyLFU, {ghost_count} ghosts of {ghost_capacity}, sampling 1/{sample_rate})"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			policy: Policy::new(capacity, window_for(capacity, FRACTIONS[INITIAL_CHOICE])),
			capacity,
			sketch: Sketch::new(capacity),
			pending_miss: None,
			ghosts: FRACTIONS
				.iter()
				.map(|&f| Ghost::new(ghost_capacity, f))
				.collect(),
			sample_rate: sample_rate as u64,
			samples: 0,
			period: ghost_capacity.max(MIN_PERIOD),
			choice: INITIAL_CHOICE,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. Every lookup (hit or miss) feeds the
	/// frequency sketch and, if its key is sampled, the ghost caches.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		self.observe(hash);
		match self.map.get(key).copied() {
			Some(idx) => {
				self.pending_miss = None;
				self.policy.on_access(idx);
				Some(&self.nodes[idx].value)
			}
			None => {
				self.pending_miss = Some(hash);
				None
			}
		}
	}

	/// Replays a sampled access on every ghost and periodically re-picks the window.
	fn observe(&mut self, hash: u64) {
		if (hash >> 24) % self.sample_rate != 0 {
			return;
		}
		let sketch = &self.sketch;
		for ghost in self.ghosts.iter_mut() {
			ghost.access(hash, sketch);
		}
		self.samples += 1;
		if self.samples >= self.period {
			self.samples = 0;
			self.adapt();
		}
	}

	/// Adopts the window fraction of the ghost with the best decayed hit count
	/// (ties keep the current choice).
	fn adapt(&mut self) {
		let mut best = self.choice;
		for i in 0..self.ghosts.len() {
			if self.ghosts[i].score > self.ghosts[best].score {
				best = i;
			}
		}
		for ghost in self.ghosts.iter_mut() {
			ghost.score *= DECAY;
		}
		if best != self.choice {
			self.choice = best;
			self.policy
				.resize(window_for(self.capacity, FRACTIONS[best]));
		}
	}

	/// Inserts or updates `key`. New keys always enter the window; when the cache is
	/// full the window's LRU entry competes with main's victim via TinyLFU.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.policy.on_access(idx);
			return;
		}
		let hash = hash_of(&key);
		if self.pending_miss != Some(hash) {
			// Not preceded by a get() miss on this key: count this access too.
			self.sketch.increment(hash);
			self.observe(hash);
		}
		self.pending_miss = None;
		let (idx, evicted) = self.policy.insert(hash, &self.sketch);
		if evicted.is_some() {
			let old = std::mem::replace(&mut self.nodes[idx].key, key.clone());
			self.nodes[idx].value = value;
			self.map.remove(&old);
		} else {
			debug_assert_eq!(idx, self.nodes.len());
			self.nodes.push(Node {
				key: key.clone(),
				value,
			});
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
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
	fn eviction_removes_least_frequent() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&2); // 2 is now more popular than 1
		cache.put(3, 30); // window candidate 2 beats probation victim 1 -> evicts key 1

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
		// Keys 3..=5 were requested (and missed) earlier, so each outranks the
		// probation victim it is compared against when it leaves the window.
		for k in 3..=5 {
			for _ in 0..k {
				cache.get(&k);
			}
		}
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // candidate 3 beats victim 1 -> evicts 1
		cache.put(5, 50); // candidate 4 beats victim 2 -> evicts 2
		cache.put(6, 60); // candidate 5 beats victim 3 -> evicts 3

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
			cache.get(&1); // 1 moves to the protected segment
		}
		cache.get(&3);
		cache.put(4, 40); // candidate 3 outranks victim 2 -> evicts 2, never hot key 1
		for k in 5..20 {
			cache.put(k, k * 10); // one-hit wonders cannot displace key 1
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // probation (LRU end)
		cache.put(2, 20); // probation
		cache.put(3, 30); // window
		cache.get(&2); // probation hit -> protected segment
		cache.get(&3); // window hit; 3 now outranks 1
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
		cache.put(3, 30); // candidate 2 loses the tie -> slot 1 reused for key 3
		cache.get(&3); // 3 gains frequency
		cache.put(4, 40); // candidate 3 beats victim 1 -> slot 0 reused for key 4
		assert_eq!(cache.nodes.len(), 2);
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
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
		cache.put(3, 30); // full; 3 is in the window (MRU)
		cache.get(&2); // probation hit -> protected
		cache.get(&3); // 3 now outranks 1
		cache.put(3, 31); // overwrite in place, no eviction
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

	/// Walks every list of a policy and checks links, sizes and segment bounds.
	fn check_policy(p: &Policy) {
		assert!(p.window_max >= 1);
		assert!(p.window_max + p.protected_max <= p.capacity);
		assert!(p.lens[WINDOW] <= p.window_max);
		assert!(p.lens[PROTECTED] <= p.protected_max);
		assert!(p.hashes.len() <= p.capacity);
		assert_eq!(p.lens.iter().sum::<usize>(), p.hashes.len());
		for q in 0..3 {
			let mut count = 0;
			let mut prev = NIL;
			let mut cur = p.heads[q];
			while cur != NIL {
				assert_eq!(p.queue[cur], q);
				assert_eq!(p.prev[cur], prev);
				prev = cur;
				cur = p.next[cur];
				count += 1;
			}
			assert_eq!(prev, p.tails[q]);
			assert_eq!(count, p.lens[q]);
		}
	}

	/// Checks the real cache, its agreement with the policy, and every ghost.
	fn check_invariants<K: std::hash::Hash + Eq, V>(cache: &LruCache<K, V>) {
		check_policy(&cache.policy);
		assert!(cache.map.len() <= cache.capacity);
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert_eq!(cache.policy.hashes.len(), cache.nodes.len());
		for (i, node) in cache.nodes.iter().enumerate() {
			assert_eq!(cache.map.get(&node.key), Some(&i));
			assert_eq!(cache.policy.hashes[i], hash_of(&node.key));
		}
		assert_eq!(
			cache.policy.window_max,
			window_for(cache.capacity, FRACTIONS[cache.choice])
		);
		assert_eq!(cache.ghosts.len(), FRACTIONS.len());
		for ghost in &cache.ghosts {
			check_policy(&ghost.policy);
			assert_eq!(ghost.map.len(), ghost.policy.hashes.len());
			for (&h, &i) in ghost.map.iter() {
				assert_eq!(ghost.policy.hashes[i], h);
			}
		}
	}

	/// Simulator-style access: get, and put on miss. Returns true on hit.
	fn access(cache: &mut LruCache<u64, u64>, key: u64) -> bool {
		if cache.get(&key).is_some() {
			return true;
		}
		cache.put(key, key);
		false
	}

	#[test]
	fn newest_entry_is_always_resident() {
		let mut cache = LruCache::new(4);
		for k in 0..100u64 {
			cache.put(k, k);
			assert_eq!(*cache.get(&k).unwrap(), k);
		}
		check_invariants(&cache);
	}

	#[test]
	fn hot_set_survives_scan() {
		let mut cache = LruCache::new(100);
		for _ in 0..20 {
			for k in 0..10 {
				access(&mut cache, k);
			}
		}
		// A scan 5x larger than the cache would flush every hot key under LRU.
		for k in 1_000..1_500 {
			access(&mut cache, k);
		}
		for k in 0..10 {
			assert!(cache.map.contains_key(&k), "hot key {k} was flushed by the scan");
		}
		check_invariants(&cache);
	}

	#[test]
	fn invariants_hold_under_mixed_workload() {
		for &cap in &[1usize, 2, 3, 10, 64] {
			let mut cache = LruCache::new(cap);
			let mut x: u64 = 0x2545_F491_4F6C_DD1D;
			for step in 0..20_000u64 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				// Alternate skewed and looping phases so the window moves both ways.
				let key = if (step / 2_000) % 2 == 0 {
					(x % 1000) % (1 + x % 50)
				} else {
					step % (cap as u64 * 2 + 1)
				};
				if !access(&mut cache, key) {
					assert!(cache.map.contains_key(&key));
				}
				if step % 11 == 0 {
					cache.put(key, step); // overwrite path
				}
				if step % 97 == 0 {
					check_invariants(&cache);
				}
			}
			check_invariants(&cache);
		}
	}

	#[test]
	fn large_capacity_uses_sampled_ghosts() {
		let mut cache = LruCache::new(20_000);
		assert!(cache.sample_rate > 1);
		for ghost in &cache.ghosts {
			assert!(ghost.policy.capacity <= 2 * GHOST_TARGET);
		}
		let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..60_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let key = (x % 50_000) % (1 + x % 30_000);
			access(&mut cache, key);
		}
		check_invariants(&cache);
	}
}
