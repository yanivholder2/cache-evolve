// O(1) W-TinyLFU cache: a Caffeine-style three-region layout sitting on top of a
// count-min frequency sketch.
//
// The arena (`nodes: Vec<Node>`) and `map: HashMap<K, usize>` are unchanged from
// the baseline LRU. The single intrusive list is split into THREE regions, each a
// doubly-linked list threaded through the shared arena:
//   * a *window* segment (`window_head`/`window_tail`), a small LRU admission buffer
//     (~1% of capacity) where every freshly inserted key lands;
//   * a *probationary* segment (`head`/`tail`), the main region's entry tier;
//   * a *protected* segment (`prot_head`/`prot_tail`), holding main keys hit twice.
//
// New keys always enter the window MRU tail. A window hit is pure LRU (move to the
// window MRU tail, no promotion). When the window exceeds `window_cap`, its LRU
// node becomes a *candidate* for the main region: its estimated frequency is
// compared against the main probationary-LRU victim. If the candidate out-ranks
// (>=) the victim it is admitted to the main probationary MRU tail and the victim
// is evicted; otherwise the candidate itself is evicted. Inside the main region a
// probationary hit promotes to the protected MRU tail (second-touch rule); the
// protected segment is capped at `prot_cap = (main_cap * 4) / 5`, overflow demoting
// the protected-LRU back to the probationary MRU tail. Slots are reused in place
// exactly as the baseline does. No unsafe, no external crates.
//
// Layered underneath is a lightweight count-min frequency sketch (4 rows x 16384
// saturating 8-bit counters). Each access (get-hit and put) bumps the incoming key
// in the sketch; periodically all counters are halved to age out stale frequency.
// The sketch is reused unchanged from the parent SLRU variant; only the admission
// gate now judges the window→main *candidate*, the W-TinyLFU recency buffer that a
// plain TinyLFU gate lacks.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// Count-min sketch geometry: 4 rows of 16384 saturating 8-bit counters.
const SKETCH_ROWS: usize = 4;
const SKETCH_COLS: usize = 16384;
// Distinct per-row seeds mixed into each row's hash so the rows are independent.
const SKETCH_SEEDS: [u64; SKETCH_ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0xD6E8_FEB8_6659_FD93,
];

/// Which of the three W-TinyLFU regions a node currently occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Window,
	Probationary,
	Protected,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: Region,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	// Window segment list (LRU admission buffer).
	window_head: Option<usize>, // window LRU (front)
	window_tail: Option<usize>, // window MRU (back)
	window_len: usize,          // number of resident window nodes
	window_cap: usize,          // window-segment capacity (~1% of capacity)
	// Probationary segment list (main region entry tier).
	head: Option<usize>, // probationary LRU (front)
	tail: Option<usize>, // probationary MRU (back)
	// Protected segment list (main region, hit twice).
	prot_head: Option<usize>, // protected LRU (front)
	prot_tail: Option<usize>, // protected MRU (back)
	prot_len: usize,          // number of resident protected nodes
	prot_cap: usize,          // protected-segment capacity (~80% of main)
	main_cap: usize,          // main-region capacity (capacity - window_cap)
	capacity: usize,
	// TinyLFU-style count-min frequency sketch (SKETCH_ROWS x SKETCH_COLS u8s).
	sketch: Vec<u8>,
	sketch_ops: usize,   // accesses recorded since the last halving
	sketch_reset: usize, // halve every counter once this many accesses accrue
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
		// ~1% of total capacity (at least one slot) is the LRU admission window.
		let window_cap = (capacity / 100).max(1);
		// The remainder is the main region, ~80% of which is protected.
		let main_cap = capacity - window_cap;
		let prot_cap = (main_cap * 4) / 5;
		log::info!(
			"LruCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} prot_cap={prot_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			window_head: None,
			window_tail: None,
			window_len: 0,
			window_cap,
			head: None,
			tail: None,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			prot_cap,
			main_cap,
			capacity,
			// Fixed-size sketch, allocated once at construction (never reallocated).
			sketch: vec![0u8; SKETCH_ROWS * SKETCH_COLS],
			sketch_ops: 0,
			sketch_reset: 10 * capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, updating its position per region.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss.
		let &idx = self.map.get(key)?;
		// Record the access frequency before returning.
		self.bump(key);
		// A hit updates the node's position within / across the regions.
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. New keys land in the window; at capacity the
	/// W-TinyLFU admission gate decides what is retained.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and re-position (mirror a get-hit).
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.bump(&key);
			self.touch(idx);
			return;
		}
		// New key — always destined for the window MRU tail.
		if self.nodes.len() >= self.capacity {
			self.insert_full(key, value);
		} else {
			self.insert_room(key, value);
		}
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Re-position a node on a hit. Window/protected hits are pure LRU (move to the
	/// region's MRU tail); a probationary hit promotes the node into the protected
	/// MRU tail (possibly demoting the protected-LRU to keep `prot_len <= prot_cap`).
	fn touch(&mut self, idx: usize) {
		match self.nodes[idx].region {
			Region::Window | Region::Protected => {
				// Pure LRU within the same region.
				self.unlink(idx);
				self.push_back(idx);
			}
			Region::Probationary => {
				// Probationary -> protected (second touch).
				self.unlink(idx);
				self.nodes[idx].region = Region::Protected;
				self.prot_len += 1;
				self.push_back(idx);
				if self.prot_len > self.prot_cap {
					self.demote_one();
				}
			}
		}
	}

	/// Insert a new key when the arena still has free slots (no eviction needed).
	/// The key enters the window MRU tail; any window overflow spills the window-LRU
	/// candidate into the main probationary segment (room exists, so no eviction).
	fn insert_room(&mut self, key: K, value: V) {
		let idx = self.nodes.len();
		self.nodes.push(Node {
			key: key.clone(),
			value,
			prev: None,
			next: None,
			region: Region::Window,
		});
		self.window_len += 1;
		self.push_back(idx);
		self.bump(&key);
		self.map.insert(key, idx);
		// Window overflow: spill the window-LRU into the main probationary segment.
		while self.window_len > self.window_cap {
			self.window_to_probationary();
		}
	}

	/// Insert a new key when the arena is full. The key always enters the window;
	/// the W-TinyLFU admission gate decides which resident is evicted to keep the
	/// total resident count at `capacity`.
	fn insert_full(&mut self, key: K, value: V) {
		if self.window_len < self.window_cap {
			// Window has room but the arena does not: free a main slot (probationary
			// LRU, falling back to protected LRU) and place the newcomer in the window.
			let victim = self
				.head
				.or(self.prot_head)
				.expect("full arena with window under cap implies a non-empty main region");
			self.evict_node(victim);
			self.bump(&key);
			self.place_new(victim, key, value, Region::Window);
			self.window_len += 1;
			return;
		}

		// Window at capacity: its LRU node is the candidate for the main region.
		let candidate = self
			.window_head
			.expect("window_len >= window_cap >= 1 implies a non-empty window");

		if self.main_cap == 0 {
			// No main region (capacity == 1): plain LRU within the window. Evict the
			// candidate and reuse its slot for the newcomer.
			let candidate_key = self.nodes[candidate].key.clone();
			self.map.remove(&candidate_key);
			self.unlink(candidate);
			self.window_len -= 1;
			self.bump(&key);
			self.place_new(candidate, key, value, Region::Window);
			self.window_len += 1;
			return;
		}

		// Compare the candidate against the main probationary victim (falling back to
		// the protected LRU only when probationary is empty).
		let main_victim = self
			.head
			.or(self.prot_head)
			.expect("full arena with a non-empty main region has a main victim");
		let candidate_key = self.nodes[candidate].key.clone();
		let main_victim_key = self.nodes[main_victim].key.clone();

		if self.estimate(&candidate_key) >= self.estimate(&main_victim_key) {
			// Admit the candidate to the main probationary MRU tail and evict the main
			// victim; the freed main slot hosts the newcomer in the window.
			self.unlink(candidate);
			self.window_len -= 1;
			self.nodes[candidate].region = Region::Probationary;
			self.push_back(candidate);
			self.evict_node(main_victim);
			self.bump(&key);
			self.place_new(main_victim, key, value, Region::Window);
			self.window_len += 1;
		} else {
			// Reject the candidate: evict it and reuse its slot for the newcomer.
			self.map.remove(&candidate_key);
			self.unlink(candidate);
			self.window_len -= 1;
			self.bump(&key);
			self.place_new(candidate, key, value, Region::Window);
			self.window_len += 1;
		}
	}

	/// Move the window-LRU node into the main probationary MRU tail (no eviction).
	fn window_to_probationary(&mut self) {
		if let Some(wh) = self.window_head {
			self.unlink(wh);
			self.window_len -= 1;
			self.nodes[wh].region = Region::Probationary;
			self.push_back(wh);
		}
	}

	/// Remove `idx` from the cache entirely: drop it from the map, unlink it from
	/// its region's list, and decrement the relevant region length.
	fn evict_node(&mut self, idx: usize) {
		let key = self.nodes[idx].key.clone();
		self.map.remove(&key);
		match self.nodes[idx].region {
			Region::Window => self.window_len -= 1,
			Region::Protected => self.prot_len -= 1,
			Region::Probationary => {}
		}
		self.unlink(idx);
	}

	/// Initialise a (freed) arena slot for a new key in `region` and link it at that
	/// region's MRU tail. Region length bookkeeping is the caller's responsibility.
	fn place_new(&mut self, slot: usize, key: K, value: V, region: Region) {
		self.nodes[slot].region = region;
		self.nodes[slot].key = key.clone();
		self.nodes[slot].value = value;
		self.nodes[slot].prev = None;
		self.nodes[slot].next = None;
		self.push_back(slot);
		self.map.insert(key, slot);
	}

	/// Map `key` to its column in sketch `row`, mixing a distinct per-row seed so
	/// the rows behave as independent hash functions.
	fn slot(key: &K, row: usize) -> usize {
		use std::hash::Hasher;
		let mut hasher = std::collections::hash_map::DefaultHasher::new();
		hasher.write_u64(SKETCH_SEEDS[row]);
		key.hash(&mut hasher);
		row * SKETCH_COLS + (hasher.finish() as usize % SKETCH_COLS)
	}

	/// Record one access of `key`: increment each row's counter (saturating at 255)
	/// and, once `sketch_reset` accesses accrue, halve every counter to age the
	/// sketch.
	fn bump(&mut self, key: &K) {
		for row in 0..SKETCH_ROWS {
			let i = Self::slot(key, row);
			if self.sketch[i] < u8::MAX {
				self.sketch[i] += 1;
			}
		}
		self.sketch_ops += 1;
		if self.sketch_ops >= self.sketch_reset {
			for c in self.sketch.iter_mut() {
				*c >>= 1;
			}
			self.sketch_ops = 0;
		}
	}

	/// Estimate the access frequency of `key` as the minimum counter across rows.
	fn estimate(&self, key: &K) -> u8 {
		let mut est = u8::MAX;
		for row in 0..SKETCH_ROWS {
			let c = self.sketch[Self::slot(key, row)];
			if c < est {
				est = c;
			}
		}
		est
	}

	/// Demote the protected-LRU node back to the probationary MRU tail.
	fn demote_one(&mut self) {
		if let Some(ph) = self.prot_head {
			// `ph` is currently protected, so unlink updates the protected list.
			self.unlink(ph);
			self.nodes[ph].region = Region::Probationary;
			self.prot_len -= 1;
			self.push_back(ph);
		}
	}

	/// Mutable handle to a region's LRU (front) endpoint.
	fn head_mut(&mut self, region: Region) -> &mut Option<usize> {
		match region {
			Region::Window => &mut self.window_head,
			Region::Probationary => &mut self.head,
			Region::Protected => &mut self.prot_head,
		}
	}

	/// Mutable handle to a region's MRU (back) endpoint.
	fn tail_mut(&mut self, region: Region) -> &mut Option<usize> {
		match region {
			Region::Window => &mut self.window_tail,
			Region::Probationary => &mut self.tail,
			Region::Protected => &mut self.prot_tail,
		}
	}

	/// Remove node from its current region's list. The node's `region` tag selects
	/// which region's head/tail endpoints to fix up.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let region = self.nodes[idx].region;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => *self.head_mut(region) = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => *self.tail_mut(region) = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU tail of its current region (selected by the node's
	/// `region` tag).
	fn push_back(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		let old_tail = *self.tail_mut(region);
		self.nodes[idx].prev = old_tail;
		self.nodes[idx].next = None;
		match old_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => *self.head_mut(region) = Some(idx),
		}
		*self.tail_mut(region) = Some(idx);
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
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1 (window-only, plain LRU)
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
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it within the window.
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify lists are still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
