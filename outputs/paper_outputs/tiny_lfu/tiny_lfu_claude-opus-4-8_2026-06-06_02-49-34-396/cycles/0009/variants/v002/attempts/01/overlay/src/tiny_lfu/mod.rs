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
	/// Which SLRU segment this node currently sits in: `true` = protected,
	/// `false` = probation. `prev`/`next` are the single shared link pair
	/// threaded through whichever segment the flag names.
	protected: bool,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// Probation segment sentinels (LRU=head, MRU=tail). New admissions and
	/// demoted entries live here; victims are drawn from here.
	head: u32,
	tail: u32,
	/// Protected segment sentinels (LRU=prot_head, MRU=prot_tail). Proven-reused
	/// entries live here, shielded from one-hit-newcomer eviction.
	prot_head: u32,
	prot_tail: u32,
	/// Current count of nodes in the protected segment.
	protected_size: usize,
	/// Maximum protected-segment occupancy (~80% of capacity), with at least one
	/// probation slot reserved so a full cache always has a probation victim.
	protected_cap: usize,
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
		// ~80% of capacity is protected; the remainder is probation. Cap the
		// protected segment at `capacity - 1` so a full cache always retains at
		// least one probation entry to serve as an eviction victim.
		let protected_cap = (capacity - capacity / 5).min(capacity - 1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} protected_cap={protected_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			prot_head: NIL,
			prot_tail: NIL,
			protected_size: 0,
			protected_cap,
			capacity,
			sketch: CountMinSketch::new(width),
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

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.record_access(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.record_access(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
				protected: false,
			});
			self.push_back_probation(idx);
			self.map.insert(key, idx);
			return;
		}

		// Sampled-coldest victim selection restricted to the PROBATION segment:
		// walk forward from probation's LRU head along `next`, collecting up to
		// SAMPLE_VICTIMS candidates, and choose the one with the lowest sketch
		// estimate. Protected entries (proven reused) are never considered, so a
		// single-access newcomer cannot evict a twice-proven entry. The walk
		// stays within one segment because `next` links are per-segment.
		// (Fallback to protected only if probation is somehow empty, which the
		// `protected_cap <= capacity - 1` reservation prevents when full.)
		let start = if self.head != NIL { self.head } else { self.prot_head };
		debug_assert!(start != NIL, "TinyLfuCache: full cache has no victim");
		let mut victim_idx = start as usize;
		let mut victim_freq = self.sketch.estimate(&self.nodes[victim_idx].key);
		let mut cursor = self.nodes[victim_idx].next;
		let mut sampled = 1;
		while sampled < SAMPLE_VICTIMS {
			if cursor == NIL {
				break;
			}
			let candidate = cursor as usize;
			let candidate_freq = self.sketch.estimate(&self.nodes[candidate].key);
			if candidate_freq < victim_freq {
				victim_idx = candidate;
				victim_freq = candidate_freq;
			}
			cursor = self.nodes[candidate].next;
			sampled += 1;
		}

		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			let victim_was_protected = self.nodes[victim_idx].protected;
			let victim_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			if victim_was_protected {
				self.protected_size -= 1;
			}
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			// New admissions always enter probation.
			self.push_back_probation(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Record a hit/update on a resident node, applying SLRU promotion. A node in
	/// probation is promoted to the protected segment's MRU; if that overflows
	/// `protected_cap`, the protected LRU is demoted back to probation's MRU. A
	/// node already protected simply refreshes to the protected MRU.
	fn record_access(&mut self, idx: usize) {
		if self.nodes[idx].protected {
			self.unlink(idx);
			self.push_back_protected(idx);
		} else {
			self.unlink(idx);
			self.push_back_protected(idx);
			self.protected_size += 1;
			if self.protected_size > self.protected_cap {
				let demote = self.prot_head as usize;
				self.unlink(demote);
				self.push_back_probation(demote);
				self.protected_size -= 1;
			}
		}
	}

	/// Detach `idx` from whichever segment it currently belongs to (named by its
	/// `protected` flag), fixing that segment's sentinels.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let protected = self.nodes[idx].protected;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else if protected {
			self.prot_head = next;
		} else {
			self.head = next;
		}
		if next != NIL {
			self.nodes[next as usize].prev = prev;
		} else if protected {
			self.prot_tail = prev;
		} else {
			self.tail = prev;
		}
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	/// Append `idx` at the probation MRU (tail) and mark it as probation.
	fn push_back_probation(&mut self, idx: usize) {
		self.nodes[idx].prev = self.tail;
		self.nodes[idx].next = NIL;
		self.nodes[idx].protected = false;
		if self.tail != NIL {
			self.nodes[self.tail as usize].next = idx as u32;
		} else {
			self.head = idx as u32;
		}
		self.tail = idx as u32;
	}

	/// Append `idx` at the protected MRU (tail) and mark it as protected.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = NIL;
		self.nodes[idx].protected = true;
		if self.prot_tail != NIL {
			self.nodes[self.prot_tail as usize].next = idx as u32;
		} else {
			self.prot_head = idx as u32;
		}
		self.prot_tail = idx as u32;
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
