// Ensemble-adaptive W-TinyLFU behind the LruCache API (std only, no unsafe).
//
// * Policy core: window LRU -> TinyLFU admission (4-bit count-min sketch with conservative
//   update and periodic halving) -> segmented LRU main (probation + protected 80%).
// * Instead of noisy hill climbing, an ensemble of key-only shadow cores runs that policy on
//   the identical access stream with different window sizes (1% .. 100% = pure LRU),
//   admission biases (strict / TinyLFU / tie-admitting / always-admit = SLRU), protected
//   shares of main, and frequency histories (short / Caffeine-default / long sketch aging).
//   Their decayed hit counts are a paired, low-noise comparison; the best member leads.
// * The real cache stores values for exactly the leading core's resident keys (plus, right
//   after a leader switch, leftover "extras" that are dropped first when room is needed).
// Tiny caches (< MIN_ADAPTIVE_CAPACITY) use a single pure-LRU core (strict LRU semantics).

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
const MIN_ADAPTIVE_CAPACITY: usize = 16;
/// Frequency histories: (aging period as a multiple of capacity, counter ceiling).
/// Index 0 is Caffeine's default; 1 forgets quickly; 2 remembers ~3x longer.
const SKETCHES: [(usize, u8); 3] = [(10, 15), (3, 15), (32, 31)];
/// Caffeine's default protected share of the main region.
const PF: f64 = 0.8;
/// Ensemble member: (window fraction of capacity, admission bias, sketch index,
/// protected share of main).
/// A window candidate is admitted into main when freq(candidate) + bias > freq(victim):
///   0 = classic TinyLFU, 1 = ties admitted (recency-leaning), 2 = admitted unless clearly
///   colder, -1 = stricter (frequency-leaning), 64 = always admit (plain segmented-LRU main).
type Member = (f64, i32, usize, f64);
/// The first BASE_MEMBERS entries are the pure window-size sweep (classic TinyLFU).
const ENSEMBLE: [Member; 30] = [
	// Window-size sweep: classic TinyLFU, default history.
	(0.01, 0, 0, PF),
	(0.03, 0, 0, PF),
	(0.07, 0, 0, PF),
	(0.15, 0, 0, PF),
	(0.3, 0, 0, PF),
	(0.5, 0, 0, PF),
	(0.75, 0, 0, PF),
	(1.0, 0, 0, PF),
	// Admission-bias variants.
	(0.01, 1, 0, PF),
	(0.1, 1, 0, PF),
	(0.3, 1, 0, PF),
	(0.6, 1, 0, PF),
	(0.9, 1, 0, PF),
	(0.01, 2, 0, PF),
	(0.2, 2, 0, PF),
	(0.01, -1, 0, PF),
	(0.01, 64, 0, PF),
	(0.25, 64, 0, PF),
	(0.5, 64, 0, PF),
	// Larger probation segment (protected = half of main).
	(0.01, 0, 0, 0.5),
	(0.01, 1, 0, 0.5),
	// Short frequency history.
	(0.01, 0, 1, PF),
	(0.1, 0, 1, PF),
	(0.1, 1, 1, PF),
	(0.4, 1, 1, PF),
	// Long frequency history.
	(0.01, 0, 2, PF),
	(0.05, 0, 2, PF),
	(0.2, 0, 2, PF),
	(0.01, -1, 2, PF),
	(0.1, 1, 2, PF),
];
const BASE_MEMBERS: usize = 8;
/// Single pure-LRU core for tiny caches.
const PURE_LRU: [Member; 1] = [(1.0, 0, 0, PF)];
/// Reduced ensemble for very large caches (memory guard).
const LARGE_ENSEMBLE: [Member; 4] = [
	(0.01, 0, 0, PF),
	(0.1, 0, 0, PF),
	(0.4, 0, 0, PF),
	(1.0, 0, 0, PF),
];
/// Above this capacity only the base window sweep is simulated (memory guard).
const MEDIUM_CAPACITY: usize = 1 << 17;
const LARGE_CAPACITY: usize = 1 << 20;
/// Per-period decay of the ensemble scores.
const SCORE_DECAY: f64 = 0.9;
/// Minimum decayed hit-rate advantage (fraction of accesses) required to switch leader.
const SWITCH_MARGIN: f64 = 0.002;

#[inline]
fn mix64(mut x: u64) -> u64 {
	x ^= x >> 33;
	x = x.wrapping_mul(0xff51_afd7_ed55_8ccd);
	x ^= x >> 33;
	x = x.wrapping_mul(0xc4ce_b9fe_1a85_ec53);
	x ^ (x >> 33)
}

/// Fast multiplicative hasher with a strong finalizer (used for map and sketch).
#[derive(Debug, Default, Clone, Copy)]
struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, w: u64) {
		self.hash = (self.hash.rotate_left(5) ^ w).wrapping_mul(0x517c_c1b7_2722_0a95);
	}
}

impl Hasher for FxHasher {
	fn write(&mut self, bytes: &[u8]) {
		for chunk in bytes.chunks(8) {
			let mut buf = [0u8; 8];
			buf[..chunk.len()].copy_from_slice(chunk);
			self.add(u64::from_le_bytes(buf));
		}
	}
	fn write_u8(&mut self, n: u8) {
		self.add(n as u64);
	}
	fn write_u16(&mut self, n: u16) {
		self.add(n as u64);
	}
	fn write_u32(&mut self, n: u32) {
		self.add(n as u64);
	}
	fn write_u64(&mut self, n: u64) {
		self.add(n);
	}
	fn write_usize(&mut self, n: usize) {
		self.add(n as u64);
	}
	fn finish(&self) -> u64 {
		mix64(self.hash)
	}
}

#[inline]
fn hash_key<K: Hash + ?Sized>(key: &K) -> u64 {
	let mut h = FxHasher::default();
	key.hash(&mut h);
	h.finish()
}

/// 4-row count-min sketch of saturating counters with periodic halving (aging).
#[derive(Debug)]
struct FrequencySketch {
	table: Vec<u8>,
	width: usize,
	mask: usize,
	additions: usize,
	sample_size: usize,
	max: u8,
}

impl FrequencySketch {
	fn new(capacity: usize, period: usize, max: u8) -> Self {
		let width = capacity
			.max(16)
			.next_power_of_two()
			.saturating_mul(4)
			.min(1 << 22);
		Self {
			table: vec![0; width * 4],
			width,
			mask: width - 1,
			additions: 0,
			sample_size: capacity.max(16).saturating_mul(period),
			max,
		}
	}

	#[inline]
	fn slots(&self, hash: u64) -> [usize; 4] {
		let step = mix64(hash ^ 0x9e37_79b9_7f4a_7c15) | 1;
		let mut out = [0usize; 4];
		for (i, slot) in out.iter_mut().enumerate() {
			let h = hash.wrapping_add(step.wrapping_mul(i as u64));
			*slot = i * self.width + ((h as usize) & self.mask);
		}
		out
	}

	fn frequency(&self, hash: u64) -> u8 {
		self.slots(hash).iter().map(|&s| self.table[s]).min().unwrap_or(0)
	}

	fn increment(&mut self, hash: u64) {
		let slots = self.slots(hash);
		let min = slots.iter().map(|&s| self.table[s]).min().unwrap_or(0);
		if min >= self.max {
			return;
		}
		// Conservative update: only bump the counters that hold the minimum.
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

#[derive(Debug, Clone, Copy)]
struct List {
	head: usize, // LRU
	tail: usize, // MRU
	len: usize,
}

const EMPTY_LIST: List = List {
	head: NIL,
	tail: NIL,
	len: 0,
};

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: usize,
	next: usize,
	region: u8,
}

type FxBuild = BuildHasherDefault<FxHasher>;

/// Key-only W-TinyLFU policy state with a fixed window size (one ensemble member).
/// Node values hold the key's sketch hash so admission never re-hashes.
#[derive(Debug)]
struct Core<K> {
	map: HashMap<K, usize, FxBuild>, // key -> node index
	nodes: Vec<Node<K, u64>>,        // arena (value = key hash)
	free: Vec<usize>,                // freed arena slots
	lists: [List; 3],                // window, probation, protected
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	bias: i32,       // admission bias: admit iff freq(cand) + bias > freq(victim)
	sketch: usize,   // index of the frequency history this member consults
	track: bool,     // report evictions (set only on the leader)
	evicted: Vec<K>, // leader evictions not yet mirrored into the value store
	hits: u64,       // hits in the current decision period
	score: f64,      // exponentially decayed hits
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, V, FxBuild>, // resident values: leader's keys (+ leftover extras)
	cores: Vec<Core<K>>,         // shadow ensemble (one is the leader)
	leader: usize,
	sketches: Vec<FrequencySketch>, // shared frequency histories (see SKETCHES)
	adaptive: bool,
	capacity: usize,
	extras: Vec<K>, // keys kept from a previous leader; dropped first when room is needed
	ticks: usize,
	period: usize,
	weight: f64, // decayed access count (same decay as the scores)
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
		let adaptive = capacity >= MIN_ADAPTIVE_CAPACITY;
		let members: &[Member] = if !adaptive {
			&PURE_LRU[..]
		} else if capacity > LARGE_CAPACITY {
			&LARGE_ENSEMBLE[..]
		} else if capacity > MEDIUM_CAPACITY {
			&ENSEMBLE[..BASE_MEMBERS]
		} else {
			&ENSEMBLE[..]
		};
		// Only build the frequency histories some member actually consults.
		let used = members.iter().map(|m| m.2 + 1).max().unwrap_or(1);
		let sketch_cap = if adaptive { capacity } else { 1 };
		let sketches: Vec<FrequencySketch> = SKETCHES[..used]
			.iter()
			.map(|&(period, max)| FrequencySketch::new(sketch_cap, period, max))
			.collect();
		let mut cores: Vec<Core<K>> = members
			.iter()
			.map(|&(f, bias, sketch, pf)| Core::new(capacity, f, bias, sketch, pf))
			.collect();
		cores[0].track = true; // start with the smallest window (Caffeine default 1%)
		Some(Self {
			map: HashMap::with_capacity_and_hasher(capacity.saturating_add(1), FxBuild::default()),
			cores,
			leader: 0,
			sketches,
			adaptive,
			capacity,
			extras: Vec::new(),
			ticks: 0,
			period: (capacity / 2).max(512),
			weight: 0.0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present (recording the access in the ensemble).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		if self.adaptive {
			self.record(key);
		} else if self.map.contains_key(key) {
			self.cores[0].touch(key);
		}
		self.map.get(key)
	}

	/// Inserts or updates `key`. Only keys resident in the leader core hold values.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(slot) = self.map.get_mut(&key) {
			*slot = value;
			if !self.adaptive {
				self.cores[0].touch(&key);
			}
			return;
		}
		if !self.adaptive {
			self.cores[0].insert(key.clone(), 0, &self.sketches[0]);
		} else if !self.cores[self.leader].contains(&key) {
			// Put without a preceding get: treat it as an access.
			self.record(&key);
		}
		self.drain();
		self.map.insert(key, value);
		while self.map.len() > self.capacity {
			self.drop_extra();
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Feed one access to the sketches and every shadow core (a miss inserts, like get+put).
	fn record(&mut self, key: &K) {
		let hash = hash_key(key);
		for sketch in self.sketches.iter_mut() {
			sketch.increment(hash);
		}
		let sketches = &self.sketches;
		for core in self.cores.iter_mut() {
			if core.touch(key) {
				core.hits += 1;
			} else {
				let s = core.sketch;
				core.insert(key.clone(), hash, &sketches[s]);
			}
		}
		self.drain();
		self.ticks += 1;
		if self.ticks >= self.period {
			self.decide();
		}
	}

	/// Mirror the leader's evictions into the value store.
	fn drain(&mut self) {
		let leader = self.leader;
		for k in self.cores[leader].evicted.drain(..) {
			self.map.remove(&k);
		}
	}

	/// Free one slot by dropping a leftover key that the leader does not hold.
	fn drop_extra(&mut self) {
		let leader = &self.cores[self.leader];
		while let Some(k) = self.extras.pop() {
			if !leader.contains(&k) && self.map.remove(&k).is_some() {
				return;
			}
		}
		// Unreachable in practice; keeps the capacity bound unconditionally.
		let victim = self
			.map
			.keys()
			.find(|k| !leader.contains(k))
			.or_else(|| self.map.keys().next())
			.cloned();
		if let Some(k) = victim {
			self.map.remove(&k);
		}
	}

	/// End of a decision period: update decayed scores and follow the best core.
	fn decide(&mut self) {
		self.ticks = 0;
		self.weight = self.weight * SCORE_DECAY + self.period as f64;
		for core in self.cores.iter_mut() {
			core.score = core.score * SCORE_DECAY + core.hits as f64;
			core.hits = 0;
		}
		let mut best = self.leader;
		for (i, core) in self.cores.iter().enumerate() {
			if core.score > self.cores[best].score {
				best = i;
			}
		}
		let gain = self.cores[best].score - self.cores[self.leader].score;
		if best != self.leader && gain > SWITCH_MARGIN * self.weight {
			self.switch_leader(best);
		}
	}

	/// Make `next` the leader. Resident values it does not hold become extras, ordered so
	/// that the least valuable (old extras, then old probation LRU end) are dropped first.
	fn switch_leader(&mut self, next: usize) {
		let prev = self.leader;
		let mut extras = Vec::new();
		{
			let (from, to) = (&self.cores[prev], &self.cores[next]);
			for &r in &[PROTECTED, WINDOW, PROBATION] {
				let mut i = from.lists[r as usize].tail;
				while i != NIL {
					let k = &from.nodes[i].key;
					if !to.contains(k) && self.map.contains_key(k) {
						extras.push(k.clone());
					}
					i = from.nodes[i].prev;
				}
			}
			for k in self.extras.drain(..) {
				if !from.contains(&k) && !to.contains(&k) && self.map.contains_key(&k) {
					extras.push(k);
				}
			}
		}
		self.extras = extras;
		self.cores[prev].track = false;
		self.cores[prev].evicted.clear();
		self.cores[next].track = true;
		self.leader = next;
	}
}

impl<K: Hash + Eq + Clone> Core<K> {
	fn new(capacity: usize, window_fraction: f64, bias: i32, sketch: usize, protected: f64) -> Self {
		let window_max = ((capacity as f64 * window_fraction).round() as usize).clamp(1, capacity);
		let protected_max = ((capacity - window_max) as f64 * protected) as usize;
		let slots = capacity.saturating_add(1);
		Self {
			map: HashMap::with_capacity_and_hasher(slots, FxBuild::default()),
			nodes: Vec::with_capacity(slots),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			window_max,
			protected_max,
			bias,
			sketch,
			track: false,
			evicted: Vec::new(),
			hits: 0,
			score: 0.0,
		}
	}

	#[inline]
	fn contains(&self, key: &K) -> bool {
		self.map.contains_key(key)
	}

	/// Promote `key` if resident; returns whether it was.
	fn touch(&mut self, key: &K) -> bool {
		match self.map.get(key) {
			Some(&idx) => {
				self.on_hit(idx);
				true
			}
			None => false,
		}
	}

	/// Insert a non-resident key into the window; overflow goes through admission.
	fn insert(&mut self, key: K, hash: u64, sketch: &FrequencySketch) {
		let idx = self.alloc(key.clone(), hash);
		self.push_back(idx, WINDOW);
		self.map.insert(key, idx);
		self.maintain(sketch);
	}

	fn alloc(&mut self, key: K, value: u64) -> usize {
		if let Some(idx) = self.free.pop() {
			let n = &mut self.nodes[idx];
			n.key = key;
			n.value = value;
			idx
		} else {
			self.nodes.push(Node {
				key,
				value,
				prev: NIL,
				next: NIL,
				region: WINDOW,
			});
			self.nodes.len() - 1
		}
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region {
			WINDOW => self.move_to(idx, WINDOW),
			PROBATION => {
				self.move_to(idx, PROTECTED);
				self.rebalance_protected();
			}
			_ => self.move_to(idx, PROTECTED),
		}
	}

	fn rebalance_protected(&mut self) {
		while self.lists[PROTECTED as usize].len > self.protected_max {
			let h = self.lists[PROTECTED as usize].head;
			self.move_to(h, PROBATION);
		}
	}

	/// Restore size invariants after an insertion.
	fn maintain(&mut self, sketch: &FrequencySketch) {
		while self.lists[WINDOW as usize].len > self.window_max {
			let cand = self.lists[WINDOW as usize].head;
			self.move_to(cand, PROBATION);
			if self.map.len() > self.capacity {
				self.admit(cand, sketch);
			}
		}
		while self.map.len() > self.capacity {
			let victim = [PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&r| self.lists[r as usize].head)
				.find(|&h| h != NIL);
			match victim {
				Some(v) => self.evict(v),
				None => break,
			}
		}
	}

	/// TinyLFU admission: candidate (probation MRU) vs main's LRU victim.
	fn admit(&mut self, cand: usize, sketch: &FrequencySketch) {
		let mut victim = self.lists[PROBATION as usize].head;
		if victim == cand {
			victim = self.lists[PROTECTED as usize].head;
		}
		if victim == NIL {
			self.evict(cand);
			return;
		}
		let cf = sketch.frequency(self.nodes[cand].value) as i32;
		let vf = sketch.frequency(self.nodes[victim].value) as i32;
		if cf + self.bias > vf {
			self.evict(victim);
		} else {
			self.evict(cand);
		}
	}

	fn evict(&mut self, idx: usize) {
		self.map.remove(&self.nodes[idx].key);
		self.unlink(idx);
		self.free.push(idx);
		if self.track {
			self.evicted.push(self.nodes[idx].key.clone());
		}
	}

	fn move_to(&mut self, idx: usize, region: u8) {
		self.unlink(idx);
		self.push_back(idx, region);
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let (prev, next, r) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.region as usize)
		};
		if prev != NIL {
			self.nodes[prev].next = next;
		} else {
			self.lists[r].head = next;
		}
		if next != NIL {
			self.nodes[next].prev = prev;
		} else {
			self.lists[r].tail = prev;
		}
		self.lists[r].len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append node to the MRU end of `region`'s list.
	fn push_back(&mut self, idx: usize, region: u8) {
		let r = region as usize;
		let tail = self.lists[r].tail;
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.region = region;
		}
		if tail != NIL {
			self.nodes[tail].next = idx;
		} else {
			self.lists[r].head = idx;
		}
		self.lists[r].tail = idx;
		self.lists[r].len += 1;
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
	fn adaptive_mode_keeps_contract() {
		let mut cache = LruCache::new(64);
		let mut x: u64 = 12345;
		for _ in 0..100_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			let k = (x % 300) as i64;
			match cache.get(&k) {
				Some(v) => assert_eq!(*v, k * 2),
				None => cache.put(k, k * 2),
			}
			assert!(cache.map.len() <= 64);
		}
		cache.put(7, 1);
		assert_eq!(*cache.get(&7).unwrap(), 1);
	}

	#[test]
	fn biased_members_keep_contract() {
		// Exercise every (window, bias, history, protected share) member on a key-only core.
		for &(f, bias, s, pf) in ENSEMBLE.iter() {
			let (period, max) = SKETCHES[s];
			let mut sketch = FrequencySketch::new(50, period, max);
			let mut core: Core<i64> = Core::new(50, f, bias, s, pf);
			let mut x: u64 = 42;
			for _ in 0..20_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let k = (x % 200) as i64;
				let h = hash_key(&k);
				sketch.increment(h);
				if !core.touch(&k) {
					core.insert(k, h, &sketch);
				}
				assert!(core.map.len() <= 50);
				let total: usize = core.lists.iter().map(|l| l.len).sum();
				assert_eq!(total, core.map.len());
			}
		}
	}

	#[test]
	fn leader_switches_keep_contract() {
		let cap = 100;
		let mut cache = LruCache::new(cap);
		let mut x: u64 = 987_654_321;
		for step in 0..200_000u64 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			// Alternate a frequency-skewed phase with a loop slightly larger than capacity.
			let k = if (step / 20_000) % 2 == 0 {
				((x % 1000) * (x % 1000) / 1000) as i64
			} else {
				(step % 150) as i64 + 10_000
			};
			match cache.get(&k) {
				Some(v) => assert_eq!(*v, k + 1),
				None => cache.put(k, k + 1),
			}
			assert!(cache.map.len() <= cap);
		}
	}
}
