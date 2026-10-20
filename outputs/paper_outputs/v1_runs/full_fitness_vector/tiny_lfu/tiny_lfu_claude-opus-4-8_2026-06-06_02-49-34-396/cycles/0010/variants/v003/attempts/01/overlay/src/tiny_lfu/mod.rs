use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u8 = u8::MAX;
const SAMPLE_VICTIMS: usize = 5;

const HASH_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const HASH_MUL: u64 = 0x517c_c1b7_2722_0a95;

/// Sentinel for "no link" in the intrusive doubly-linked list. Imported from
/// v002's compact node layout: encoding `prev`/`next` as `u32` with `NIL` for
/// the absent case halves the per-node footprint versus `Option<usize>`.
const NIL: u32 = u32::MAX;

/// Cheap multiply-xorshift fold (imported from v002's hashing path). One pass
/// over the key produces a 64-bit hash that the sketch then splits into two
/// independent indices via double hashing — replacing the per-row SipHash
/// (`DefaultHasher`) the parent ran for every counter row on every op.
#[derive(Debug)]
struct FastHasher {
	state: u64,
}

impl FastHasher {
	fn new() -> Self {
		Self { state: HASH_SEED }
	}
}

impl Hasher for FastHasher {
	fn finish(&self) -> u64 {
		// xorshift finalizer to spread the accumulated state.
		let mut x = self.state;
		x ^= x >> 33;
		x = x.wrapping_mul(HASH_MUL);
		x ^= x >> 29;
		x
	}

	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.state = (self.state ^ b as u64).wrapping_mul(HASH_MUL);
		}
	}

	fn write_u64(&mut self, i: u64) {
		self.state = (self.state ^ i).wrapping_mul(HASH_MUL);
	}

	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: u32,
	tail: u32,
	capacity: usize,
	sketch: CountMinSketch,
	/// Xorshift PRNG state for random sampled-LFU eviction. Seeded at
	/// construction; lives on the cache struct (not on `Node`) so the scored
	/// node metadata-size stays at 24 bytes.
	rng_state: u64,
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
			head: NIL,
			tail: NIL,
			capacity,
			sketch: CountMinSketch::new(width),
			// Seed from HASH_SEED mixed with capacity so different cache sizes
			// draw different victim streams; always non-zero, as xorshift needs.
			rng_state: HASH_SEED ^ (capacity as u64).wrapping_mul(HASH_MUL) | 1,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch for this cache: `depth * width * size_of::<u8>()`.
	///
	/// This is the FIXED metadata overhead TinyLfu carries beyond the per-entry nodes — it is
	/// sized at construction from capacity (see `try_new`) and does not grow with population.
	/// The metadata-size fitness bin reports it so variant edits to the sketch geometry are visible.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	/// Advance the xorshift64 PRNG and return the new state. Inline, branchless,
	/// no memory traffic — the point of the random-sampling eviction is to avoid
	/// the link-chasing cache misses of the old LRU-window walk.
	#[inline]
	fn next_rng(&mut self) -> u64 {
		let mut x = self.rng_state;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.rng_state = x;
		x
	}

	/// Draw a uniform index in `0..n` from the PRNG via Lemire multiply-shift
	/// (no modulo, no power-of-two requirement on `n`).
	#[inline]
	fn sample_index(&mut self, n: usize) -> usize {
		let r = self.next_rng();
		(((r as u128) * (n as u128)) >> 64) as usize
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		// Random sampled-LFU victim selection (Redis-style). The cache is full,
		// so every slot in `0..self.nodes.len()` holds a live entry and can be a
		// candidate by direct index — no link traversal, no chain of random
		// `Vec` cache misses. Draw up to SAMPLE_VICTIMS indices, keep the one
		// with the lowest sketch estimate (replacing only on a strictly-lower
		// estimate), and stop early once the best candidate hits the freq floor.
		debug_assert!(self.nodes.len() == self.capacity, "TinyLfuCache: sampling a non-full cache");
		let n = self.nodes.len();
		let mut victim_idx = self.sample_index(n);
		let mut victim_freq = self.sketch.estimate(&self.nodes[victim_idx].key);
		let mut sampled = 1;
		while sampled < SAMPLE_VICTIMS {
			if victim_freq == 0 {
				break;
			}
			let candidate = self.sample_index(n);
			let candidate_freq = self.sketch.estimate(&self.nodes[candidate].key);
			if candidate_freq < victim_freq {
				victim_idx = candidate;
				victim_freq = candidate_freq;
			}
			sampled += 1;
		}

		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			let victim_key = self.nodes[victim_idx].key.clone();
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
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else {
			self.tail = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	fn push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = NIL;
		if self.tail != NIL {
			self.nodes[self.tail as usize].next = idx as u32;
		} else {
			self.head = idx as u32;
		}
		self.tail = idx as u32;
	}
}

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u8>>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0u8; width]; depth];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u8>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u8>()
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		let h = self.base_hash(key);
		for row in 0..self.depth {
			let pos = self.position_for(h, row);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		let h = self.base_hash(key);
		(0..self.depth)
			.map(|row| self.counts[row][self.position_for(h, row)])
			.min()
			.map(|c| c as u32)
			.unwrap_or(0)
	}

	/// Single cheap fold over the key (one `FastHasher` pass), replacing the
	/// per-row `DefaultHasher` the parent ran for every counter row.
	fn base_hash<K: Hash>(&self, key: &K) -> u64 {
		let mut hasher = FastHasher::new();
		key.hash(&mut hasher);
		hasher.finish()
	}

	/// Derive a per-row column from a single base hash via double hashing:
	/// `(h1 + row * (h2 | 1)) & (width - 1)`. `h2` is forced odd so successive
	/// rows step through distinct columns of the power-of-two-sized table.
	fn position_for(&self, h: u64, row: usize) -> usize {
		let h1 = h;
		let h2 = (h >> 32) | 1;
		(h1.wrapping_add((row as u64).wrapping_mul(h2)) as usize) & (self.width - 1)
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
}
