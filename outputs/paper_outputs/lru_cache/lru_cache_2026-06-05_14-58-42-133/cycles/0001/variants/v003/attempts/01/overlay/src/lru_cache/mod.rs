// O(1) LRU cache using HashMap + arena-based doubly-linked list, with a
// TinyLFU-style frequency-sketch admission filter on the eviction path.
//
// On a put against a full arena, an incoming key is only admitted (displacing
// the LRU victim) when its estimated access frequency is at least that of the
// victim; otherwise the newcomer is rejected and cache state is left unchanged
// (apart from the sketch). This rejects cold scan traffic and raises hit-rate.
//
// Nodes are stored in a Vec (arena) indexed by usize. On eviction the slot is
// reused in-place, so no free-list management is needed. No unsafe, no external
// crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

// Frequency-sketch geometry: 4 count-min rows of `1 << 14` 4-bit-ish (u8,
// saturating at 15) counters each.
const SKETCH_ROWS: usize = 4;
const SKETCH_WIDTH: usize = 1 << 14;
const SKETCH_MAX: u8 = 15;
const SKETCH_SEEDS: [u64; SKETCH_ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0x27D4_EB2F_1656_67C5,
];

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	freq: Vec<u8>,  // count-min sketch: SKETCH_ROWS * SKETCH_WIDTH counters
	sample: usize,  // accesses since last aging reset
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
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			freq: vec![0u8; SKETCH_ROWS * SKETCH_WIDTH],
			sample: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Base hash of a key, mixed once. Distinct rows are derived by xor-ing in
	/// per-row seeds before a final avalanche mix.
	fn base_hash(key: &K) -> u64 {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Counter index for `row` derived from the key's base hash.
	fn row_index(base: u64, row: usize) -> usize {
		let mut x = base ^ SKETCH_SEEDS[row];
		// fmix64 finalizer for good dispersion.
		x ^= x >> 33;
		x = x.wrapping_mul(0xFF51_AFD7_ED55_8CCD);
		x ^= x >> 33;
		x = x.wrapping_mul(0xC4CE_B9FE_1A85_EC53);
		x ^= x >> 33;
		row * SKETCH_WIDTH + (x as usize % SKETCH_WIDTH)
	}

	/// Record an access to `key`: increment its row counters (saturating at
	/// `SKETCH_MAX`) and advance the sample counter, halving every counter once
	/// the sample window reaches `10 * capacity` (TinyLFU aging).
	fn bump(&mut self, key: &K) {
		let base = Self::base_hash(key);
		for row in 0..SKETCH_ROWS {
			let idx = Self::row_index(base, row);
			if self.freq[idx] < SKETCH_MAX {
				self.freq[idx] += 1;
			}
		}
		self.sample += 1;
		if self.sample >= 10 * self.capacity {
			for c in self.freq.iter_mut() {
				*c >>= 1;
			}
			self.sample = 0;
		}
	}

	/// Estimated access frequency of `key`: the minimum across its row counters.
	fn estimate(&self, key: &K) -> u8 {
		let base = Self::base_hash(key);
		let mut min = SKETCH_MAX;
		for row in 0..SKETCH_ROWS {
			let idx = Self::row_index(base, row);
			let c = self.freq[idx];
			if c < min {
				min = c;
			}
		}
		min
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Record the access in the sketch regardless of hit/miss.
		self.bump(key);
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, applies frequency-gated
	/// admission: the newcomer displaces the LRU victim only when its estimated
	/// frequency is at least the victim's, otherwise it is rejected.
	pub fn put(&mut self, key: K, value: V) {
		// Record the access in the sketch.
		self.bump(&key);

		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: consider evicting the LRU (head) victim.
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");
			let head_idx = self.head.expect("non-empty list has a head");
			let evicted_key = self.nodes[head_idx].key.clone();
			// TinyLFU admission: only admit when the newcomer is estimated to be
			// at least as frequent as the victim. Otherwise reject (no eviction,
			// no insert) — cache state stays unchanged apart from the sketch.
			if self.estimate(&key) < self.estimate(&evicted_key) {
				return;
			}
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			self.nodes[head_idx].key = key.clone();
			self.nodes[head_idx].value = value;
			head_idx
		} else {
			// Arena has room: allocate a new slot
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Remove node from its current position in the list.
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

	/// Append node to the tail (MRU end) of the list.
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
	fn capacity_zero_panics_via_try_new() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}
}
