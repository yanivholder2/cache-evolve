// Adaptive W-TinyLFU cache (the policy behind Caffeine), O(1) per operation.
//
// * Window (LRU, starts at 1% of capacity): every new key enters here, so bursts
//   of recency are absorbed and a freshly put key is always resident.
// * Main = SLRU (probation ~20% / protected ~80%): a probation hit is promoted to
//   protected; protected overflow is demoted back to probation.
// * TinyLFU admission: when the window overflows, its LRU entry (candidate) must
//   beat the probation LRU entry (victim) on estimated frequency, else it is
//   dropped. Frequencies live in a 4-row count-min sketch of saturating 4-bit
//   counters that are halved every 10 * capacity additions (aging).
// * Hill climber: every 10 * capacity lookups the window/main split is nudged
//   in whichever direction improved the sampled hit rate (Caffeine's climber).
//
// All entries live in one arena (Vec) threaded by three intrusive lists; evicted
// slots are reused in place. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

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

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	heads: [usize; 3],      // LRU end of window / probation / protected
	tails: [usize; 3],      // MRU end of window / probation / protected
	lens: [usize; 3],
	capacity: usize,
	window_max: usize,
	protected_max: usize,
	sketch: Sketch,
	pending_miss: Option<u64>, // hash of the last get() miss (avoids double counting on put)
	hits: usize,
	misses: usize,
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
		log::info!("LruCache.try_new: capacity={capacity} (adaptive W-TinyLFU)");
		let window_max = capacity
			.saturating_sub((capacity as f64 * 0.99) as usize)
			.max(1);
		let protected_max = ((capacity - window_max) as f64 * 0.8) as usize;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			heads: [NIL; 3],
			tails: [NIL; 3],
			lens: [0; 3],
			capacity,
			window_max,
			protected_max,
			sketch: Sketch::new(capacity),
			pending_miss: None,
			hits: 0,
			misses: 0,
			prev_hit_rate: 0.0,
			step: -0.0625 * capacity as f64,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. Every lookup (hit or miss) feeds the
	/// frequency sketch and the hill climber's hit-rate sample.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_of(key);
		self.sketch.increment(hash);
		let found = self.map.get(key).copied();
		match found {
			Some(idx) => {
				self.pending_miss = None;
				self.hits += 1;
				self.on_access(idx);
				self.climb();
				Some(&self.nodes[idx].value)
			}
			None => {
				self.pending_miss = Some(hash);
				self.misses += 1;
				self.climb();
				None
			}
		}
	}

	/// Inserts or updates `key`. New keys always enter the window; when the cache is
	/// full the window's LRU entry competes with main's victim via TinyLFU.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_access(idx);
			return;
		}
		let hash = hash_of(&key);
		if self.pending_miss != Some(hash) {
			// Not preceded by a get() miss on this key: count this access too.
			self.sketch.increment(hash);
		}
		self.pending_miss = None;
		let idx = if self.nodes.len() >= self.capacity {
			let slot = self.evict();
			let node = &mut self.nodes[slot];
			node.key = key.clone();
			node.value = value;
			node.hash = hash;
			slot
		} else {
			self.nodes.push(Node {
				key: key.clone(),
				value,
				hash,
				prev: NIL,
				next: NIL,
				queue: WINDOW,
			});
			self.nodes.len() - 1
		};
		self.attach(idx, WINDOW);
		self.map.insert(key, idx);
		if self.lens[WINDOW] > self.window_max {
			// Still filling up: overflow the window into probation without eviction.
			let c = self.heads[WINDOW];
			self.detach(c);
			self.attach(c, PROBATION);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Frees one arena slot for an incoming key and returns its index.
	fn evict(&mut self) -> usize {
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
			} else if self.sketch.frequency(self.nodes[cand].hash)
				> self.sketch.frequency(self.nodes[rival].hash)
			{
				rival
			} else {
				cand
			}
		} else {
			[PROBATION, PROTECTED, WINDOW]
				.iter()
				.map(|&q| self.heads[q])
				.find(|&i| i != NIL)
				.expect("full cache has an entry")
		};
		self.detach(victim);
		self.map.remove(&self.nodes[victim].key);
		victim
	}

	/// Hit/update handling: window stays LRU, main entries go to protected.
	fn on_access(&mut self, idx: usize) {
		let q = self.nodes[idx].queue;
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

	/// Caffeine-style hill climbing of the window size.
	fn climb(&mut self) {
		let total = self.hits + self.misses;
		if total < self.sketch.sample_size {
			return;
		}
		let hit_rate = self.hits as f64 / total as f64;
		let change = hit_rate - self.prev_hit_rate;
		let amount = if change >= 0.0 { self.step } else { -self.step };
		let sign = if amount >= 0.0 { 1.0 } else { -1.0 };
		self.step = if change.abs() >= 0.05 {
			0.0625 * self.capacity as f64 * sign
		} else {
			0.98 * amount
		};
		self.prev_hit_rate = hit_rate;
		self.hits = 0;
		self.misses = 0;
		if amount >= 1.0 {
			self.grow_window(amount as usize);
		} else if amount <= -1.0 {
			self.shrink_window((-amount) as usize);
		}
	}

	fn grow_window(&mut self, n: usize) {
		let quota = n.min(self.protected_max);
		self.protected_max -= quota;
		self.window_max += quota;
		self.demote_overflow();
		for _ in 0..quota {
			let idx = if self.heads[PROBATION] != NIL {
				self.heads[PROBATION]
			} else {
				self.heads[PROTECTED]
			};
			if idx == NIL {
				break;
			}
			self.detach(idx);
			self.attach(idx, WINDOW);
		}
	}

	fn shrink_window(&mut self, n: usize) {
		let quota = n.min(self.window_max - 1);
		self.window_max -= quota;
		self.protected_max += quota;
		while self.lens[WINDOW] > self.window_max {
			let idx = self.heads[WINDOW];
			self.detach(idx);
			self.attach(idx, PROBATION);
		}
	}

	/// Remove node from the list it currently belongs to.
	fn detach(&mut self, idx: usize) {
		let (prev, next, q) = {
			let n = &self.nodes[idx];
			(n.prev, n.next, n.queue)
		};
		if prev == NIL {
			self.heads[q] = next;
		} else {
			self.nodes[prev].next = next;
		}
		if next == NIL {
			self.tails[q] = prev;
		} else {
			self.nodes[next].prev = prev;
		}
		self.lens[q] -= 1;
	}

	/// Append node at the MRU end of list `q`.
	fn attach(&mut self, idx: usize, q: usize) {
		let tail = self.tails[q];
		{
			let n = &mut self.nodes[idx];
			n.prev = tail;
			n.next = NIL;
			n.queue = q;
		}
		if tail == NIL {
			self.heads[q] = idx;
		} else {
			self.nodes[tail].next = idx;
		}
		self.tails[q] = idx;
		self.lens[q] += 1;
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

	/// Walks every list and checks links, sizes, segment bounds and map agreement.
	fn check_invariants<K: std::hash::Hash + Eq, V>(cache: &LruCache<K, V>) {
		assert!(cache.map.len() <= cache.capacity);
		assert_eq!(cache.lens.iter().sum::<usize>(), cache.map.len());
		assert!(cache.window_max >= 1);
		assert!(cache.window_max + cache.protected_max <= cache.capacity);
		assert!(cache.lens[WINDOW] <= cache.window_max);
		assert!(cache.lens[PROTECTED] <= cache.protected_max);
		for q in 0..3 {
			let mut count = 0;
			let mut prev = NIL;
			let mut cur = cache.heads[q];
			while cur != NIL {
				let node = &cache.nodes[cur];
				assert_eq!(node.queue, q);
				assert_eq!(node.prev, prev);
				assert_eq!(cache.map.get(&node.key), Some(&cur));
				prev = cur;
				cur = node.next;
				count += 1;
			}
			assert_eq!(prev, cache.tails[q]);
			assert_eq!(count, cache.lens[q]);
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
				// Alternate skewed and looping phases so the climber moves both ways.
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
}
