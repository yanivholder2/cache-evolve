use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// SipHash-1-3 initial state for keys (0, 0), i.e. the state `DefaultHasher::new()` starts from.
const SIP_INIT: [u64; 4] = [
	0x736f6d6570736575,
	0x646f72616e646f6d,
	0x6c7967656e657261,
	0x7465646279746573,
];

/// One SipHash round over the four-word state, usable in const context.
const fn sip_round(s: [u64; 4]) -> [u64; 4] {
	let [mut v0, mut v1, mut v2, mut v3] = s;
	v0 = v0.wrapping_add(v1);
	v1 = v1.rotate_left(13);
	v1 ^= v0;
	v0 = v0.rotate_left(32);
	v2 = v2.wrapping_add(v3);
	v3 = v3.rotate_left(16);
	v3 ^= v2;
	v0 = v0.wrapping_add(v3);
	v3 = v3.rotate_left(21);
	v3 ^= v0;
	v2 = v2.wrapping_add(v1);
	v1 = v1.rotate_left(17);
	v1 ^= v2;
	v2 = v2.rotate_left(32);
	[v0, v1, v2, v3]
}

/// Absorbs one full 8-byte little-endian block `m` into a SipHash-1-3 state.
const fn sip_absorb(s: [u64; 4], m: u64) -> [u64; 4] {
	let s = sip_round([s[0], s[1], s[2], s[3] ^ m]);
	[s[0] ^ m, s[1], s[2], s[3]]
}

/// Finishes a SipHash-1-3 state whose absorbed input is `length` bytes, all in full blocks.
const fn sip_finish(s: [u64; 4], length: u64) -> u64 {
	let b = (length & 0xff) << 56;
	let s = sip_round([s[0], s[1], s[2], s[3] ^ b]);
	let s = sip_round(sip_round(sip_round([s[0] ^ b, s[1], s[2] ^ 0xff, s[3]])));
	s[0] ^ s[1] ^ s[2] ^ s[3]
}

/// Per-row seed states: the SipHash state after absorbing `row` (as a u64), built at compile time.
const fn build_row_seed_states() -> [[u64; 4]; SKETCH_DEPTH] {
	let mut states = [[0u64; 4]; SKETCH_DEPTH];
	let mut row = 0;
	while row < SKETCH_DEPTH {
		states[row] = sip_absorb(SIP_INIT, (row as u64).to_le());
		row += 1;
	}
	states
}

const ROW_SEED_STATES: [[u64; 4]; SKETCH_DEPTH] = build_row_seed_states();

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>,
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
}

impl<K, V> TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u32>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		if Some(idx) == self.tail {
			// Already the MRU tail: unlink + push_back would be a no-op.
			return Some(&self.nodes[idx].value);
		}
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		// Hash the key once; the same hash drives the sketch increment and the admission estimate.
		let key_hash = CountMinSketch::hash_key(&key);
		self.sketch.increment_hashed(key_hash);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			if Some(idx) == self.tail {
				// Already the MRU tail: only the value changed.
				return;
			}
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate_hashed(key_hash);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = None;
		match self.tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self {
			width,
			depth,
			counts,
		}
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	/// Single SipHash of the key; reused across all sketch rows via `row_position`.
	fn hash_key<K: Hash>(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		self.increment_hashed(Self::hash_key(key));
	}

	fn increment_hashed(&mut self, hash: u64) {
		for row in 0..self.depth {
			let pos = self.row_position(hash, row);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self.estimate_hashed(Self::hash_key(key))
	}

	fn estimate_hashed(&self, hash: u64) -> u32 {
		(0..self.depth)
			.map(|row| self.counts[row][self.row_position(hash, row)])
			.min()
			.unwrap_or(0)
	}

	/// Derives the column for `row` from the compile-time seed state in `ROW_SEED_STATES` and the
	/// base hash, by absorbing the base hash and finishing SipHash-1-3. Produces exactly the index
	/// `row_position_reference` produces; the debug assertion checks that on every access in
	/// debug/test builds.
	fn row_position(&self, hash: u64, row: usize) -> usize {
		let state = sip_absorb(ROW_SEED_STATES[row], hash.to_le());
		let pos = (sip_finish(state, 16) as usize) & (self.width - 1);
		debug_assert_eq!(
			pos,
			self.row_position_reference(hash, row),
			"row_position: const seed table derivation diverged from reference"
		);
		pos
	}

	/// Reference derivation: the per-row seed and the base hash fed through a freshly seeded
	/// SipHasher on every call. Kept as the ground truth for `row_position`.
	fn row_position_reference(&self, hash: u64, row: usize) -> usize {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(row as u64);
		hasher.write_u64(hash);
		(hasher.finish() as usize) & (self.width - 1)
	}
}

impl<K, V> framework::Cache<K, V> for TinyLfuCache<K, V>
where
	K: Hash + Eq + Clone + Debug,
	V: Debug,
{
	fn put(&mut self, key: K, value: V) {
		TinyLfuCache::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		TinyLfuCache::get(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = TinyLfuCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = TinyLfuCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn eviction_prefers_hot_items() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn low_frequency_new_item_is_rejected() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&2).is_some());
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn update_promotes_existing_item_and_admits_hot_arrivals() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn const_seed_table_row_positions_match_reference() {
		let sketch = CountMinSketch::new(64);
		for k in 0..1000u64 {
			let hash = CountMinSketch::hash_key(&k);
			for row in 0..SKETCH_DEPTH {
				assert_eq!(
					sketch.row_position(hash, row),
					sketch.row_position_reference(hash, row)
				);
			}
		}
	}

	#[test]
	fn const_seed_table_matches_default_hasher_state() {
		for row in 0..SKETCH_DEPTH {
			for hash in [0u64, 1, 0xdead_beef, u64::MAX] {
				let mut hasher = DefaultHasher::new();
				hasher.write_u64(row as u64);
				hasher.write_u64(hash);
				let state = sip_absorb(ROW_SEED_STATES[row], hash.to_le());
				assert_eq!(sip_finish(state, 16), hasher.finish());
			}
		}
	}
}
