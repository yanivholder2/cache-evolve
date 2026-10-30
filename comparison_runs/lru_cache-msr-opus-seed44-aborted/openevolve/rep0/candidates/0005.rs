// Adaptive W-TinyLFU cache: a small LRU admission window in front of a segmented-LRU
// main space (probation + protected), guarded by a count-min frequency sketch, with a
// hill climber that resizes the window based on the observed hit rate.
// Arena-based doubly-linked lists indexed by usize; no unsafe, no external crates.
// For tiny capacities the window spans the whole cache, i.e. the policy is strict LRU.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const NIL: usize = usize::MAX;
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;
/// Below this capacity the cache is plain LRU (frequency admission needs room to pay off).
const ADAPTIVE_MIN_CAPACITY: usize = 32;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
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

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // recycled arena slots
	lists: [List; 3],       // window, probation, protected
	capacity: usize,
	max_window: usize,
	max_protected: usize,
	sketch: Sketch,
	adaptive: bool,
	last_miss: Option<u64>, // hash of the key whose get just missed (avoid double counting)
	// hill climber state
	window_target: f64,
	step: f64,
	prev_hit_rate: f64,
	sample_hits: usize,
	sample_count: usize,
	sample_period: usize,
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
		let max_window = if adaptive { (capacity / 100).max(1) } else { capacity };
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			free: Vec::new(),
			lists: [List::EMPTY; 3],
			capacity,
			max_window,
			max_protected: (capacity - max_window) * 4 / 5,
			sketch: Sketch::new(if adaptive { capacity } else { 1 }),
			adaptive,
			last_miss: None,
			window_target: max_window as f64,
			step: -0.0625 * capacity as f64,
			prev_hit_rate: 0.0,
			sample_hits: 0,
			sample_count: 0,
			sample_period: capacity.saturating_mul(10),
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
				Some(idx) => self.nodes[idx].hash,
				None => hash_key(key),
			};
			self.sketch.increment(hash);
			self.last_miss = if found.is_some() { None } else { Some(hash) };
			self.record_sample(found.is_some());
		}
		let idx = found?;
		self.on_access(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys enter the admission window; overflow is
	/// resolved by a TinyLFU frequency contest against the main-space victim.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(idx) = self.map.get(&key).copied() {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let hash = hash_key(&key);
		if self.adaptive && self.last_miss != Some(hash) {
			self.sketch.increment(hash);
		}
		self.last_miss = None;
		let node = Node {
			key: key.clone(),
			value,
			hash,
			prev: NIL,
			next: NIL,
			queue: WINDOW,
		};
		let idx = match self.free.pop() {
			Some(slot) => {
				self.nodes[slot] = node;
				slot
			}
			None => {
				self.nodes.push(node);
				self.nodes.len() - 1
			}
		};
		self.map.insert(key, idx);
		self.push_back(WINDOW, idx);
		self.maintain();
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Hit handling: window entries move to window MRU; main entries move to protected MRU.
	fn on_access(&mut self, idx: usize) {
		let queue = self.nodes[idx].queue;
		self.unlink(idx);
		if queue == WINDOW {
			self.push_back(WINDOW, idx);
		} else {
			self.push_back(PROTECTED, idx);
			self.demote_protected_overflow();
		}
	}

	fn demote_protected_overflow(&mut self) {
		while self.lists[PROTECTED as usize].len > self.max_protected {
			let idx = self.lists[PROTECTED as usize].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
	}

	fn main_len(&self) -> usize {
		self.lists[PROBATION as usize].len + self.lists[PROTECTED as usize].len
	}

	fn main_victim(&self) -> usize {
		let probation = self.lists[PROBATION as usize].head;
		if probation != NIL {
			probation
		} else {
			self.lists[PROTECTED as usize].head
		}
	}

	fn admit(&self, candidate: usize, victim: usize) -> bool {
		self.sketch.frequency(self.nodes[candidate].hash) > self.sketch.frequency(self.nodes[victim].hash)
	}

	/// Drop a detached node from the map and recycle its arena slot.
	fn release(&mut self, idx: usize) {
		self.map.remove(&self.nodes[idx].key);
		self.free.push(idx);
	}

	/// Restore size bounds after an insertion.
	fn maintain(&mut self) {
		let main_cap = self.capacity - self.max_window;
		while self.lists[WINDOW as usize].len > self.max_window {
			let candidate = self.lists[WINDOW as usize].head;
			self.unlink(candidate);
			if self.main_len() < main_cap {
				self.push_back(PROBATION, candidate);
				continue;
			}
			let victim = self.main_victim();
			if victim != NIL && self.admit(candidate, victim) {
				self.unlink(victim);
				self.release(victim);
				self.push_back(PROBATION, candidate);
			} else {
				self.release(candidate);
			}
		}
		while self.map.len() > self.capacity {
			let mut victim = self.main_victim();
			if victim == NIL {
				victim = self.lists[WINDOW as usize].head;
			}
			self.unlink(victim);
			self.release(victim);
		}
	}

	/// Hill climber: every sample period, step the window size in the direction that
	/// improved the hit rate (Caffeine's simple climber with step decay and restarts).
	fn record_sample(&mut self, hit: bool) {
		self.sample_count += 1;
		if hit {
			self.sample_hits += 1;
		}
		if self.sample_count < self.sample_period {
			return;
		}
		let hit_rate = self.sample_hits as f64 / self.sample_count as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * amount.signum()
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.sample_hits = 0;
		self.sample_count = 0;
		self.resize_window(amount);
	}

	fn resize_window(&mut self, amount: f64) {
		let upper = (self.capacity - 1) as f64;
		self.window_target = (self.window_target + amount).clamp(1.0, upper);
		self.max_window = self.window_target.round() as usize;
		let main_cap = self.capacity - self.max_window;
		self.max_protected = main_cap * 4 / 5;
		// Window grew: hand main-space LRU entries to the window so totals stay bounded.
		while self.lists[WINDOW as usize].len < self.max_window && self.main_len() > main_cap {
			let idx = self.main_victim();
			self.unlink(idx);
			self.push_back(WINDOW, idx);
		}
		// Window shrank: demote its LRU entries into probation.
		while self.lists[WINDOW as usize].len > self.max_window {
			let idx = self.lists[WINDOW as usize].head;
			self.unlink(idx);
			self.push_back(PROBATION, idx);
		}
		self.demote_protected_overflow();
	}

	/// Remove node from the list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let q = self.nodes[idx].queue as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
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

	/// Append node to the MRU end of the given list.
	fn push_back(&mut self, queue: u8, idx: usize) {
		let q = queue as usize;
		let tail = self.lists[q].tail;
		let node = &mut self.nodes[idx];
		node.queue = queue;
		node.prev = tail;
		node.next = NIL;
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
