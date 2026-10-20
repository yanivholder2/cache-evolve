use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 8;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

/// TinyLFU cache with a CLOCK second-chance eviction policy.
///
/// Builds on cycle/0009/v002's 8× sketch width, u32 counters, bijective remix hash,
/// and amortised strict gate.  Replaces the dual-chain SLRU with a **single LRU list**
/// plus a CLOCK "referenced" bit stored in `prot_flags`:
///
/// - All nodes live in one LRU list (single head/tail).
/// - On a cache hit (`get`) or existing-key `put`, the node is moved to the MRU
///   tail and its CLOCK reference bit is set.
/// - On eviction (full cache, new key): scan the LRU head; if the head node's
///   reference bit is set and fewer than `prot_capacity` nodes have been given a
///   second chance, clear the bit and move the node to the MRU tail.  The first
///   node without a reference bit — or the head after the scan limit is reached —
///   is the eviction victim.
/// - `prot_capacity` (≈ 20% of capacity) caps the CLOCK scan per eviction to the
///   same fraction as the old SLRU protected segment.
///
/// The `prot_flags` field is a packed `Vec<u64>` bitset (one bit per node slot),
/// so per-node memory is unchanged: `Node<u64,u64>` remains 48 bytes.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// LRU head (least-recently-used end of the single list).
	head: Option<usize>,
	/// LRU tail (most-recently-used end of the single list).
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
	evict_counter: u32,
	/// Maximum nodes that may receive a CLOCK second chance per eviction (≈ capacity / 5).
	prot_capacity: usize,
	/// Packed bitset: bit `idx` is set when node slot `idx` has been referenced (CLOCK bit).
	prot_flags: Vec<u64>,
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
			evict_counter: 0,
			prot_capacity: (capacity / 5).max(1),
			prot_flags: vec![0u64; (capacity + 63) / 64],
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

	// ── CLOCK bit-flag helpers ───────────────────────────────────────────────

	#[inline(always)]
	fn is_protected(&self, idx: usize) -> bool {
		(self.prot_flags[idx / 64] >> (idx % 64)) & 1 != 0
	}

	#[inline(always)]
	fn set_protected_flag(&mut self, idx: usize) {
		self.prot_flags[idx / 64] |= 1u64 << (idx % 64);
	}

	#[inline(always)]
	fn clear_protected_flag(&mut self, idx: usize) {
		self.prot_flags[idx / 64] &= !(1u64 << (idx % 64));
	}

	// ── public cache operations ─────────────────────────────────────────────

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);           // move to MRU end of the single list
		self.set_protected_flag(idx);  // mark as "referenced" for CLOCK
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);

		// ── existing-key update ──────────────────────────────────────────────
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			self.set_protected_flag(idx);
			return;
		}

		// ── fill an empty slot ───────────────────────────────────────────────
		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			// New nodes enter with CLOCK reference bit = 0 (unreferenced).
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		// ── full cache: amortised admission gate with bounded CLOCK scan ──────
		self.evict_counter = self.evict_counter.wrapping_add(1);

		// Bounded CLOCK scan: give referenced nodes a second chance.
		// Referenced (hot) nodes have their bit cleared and are moved to the MRU
		// tail.  The first unreferenced node — or the LRU head after the scan
		// limit is reached — becomes the eviction candidate.
		let mut scanned = 0usize;
		loop {
			let candidate = self.head.expect("full cache has no LRU head");
			if self.is_protected(candidate) && scanned < self.prot_capacity {
				// Second chance: clear the reference bit and move to MRU tail.
				self.clear_protected_flag(candidate);
				self.unlink(candidate);
				self.push_back(candidate);
				scanned += 1;
			} else {
				// Either unreferenced or scan limit reached — this is the victim.
				break;
			}
		}
		let victim_idx = self.head.expect("full cache has no victim after clock scan");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		let admit = if self.evict_counter & 7 == 0 {
			// Every 8th eviction: require the incoming key to beat the victim by at least 1.
			incoming_freq > victim_freq.saturating_add(1)
		} else {
			// Remaining 7 out of 8: standard "> victim" comparison.
			incoming_freq > victim_freq
		};
		if admit {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			// Clear the victim slot's CLOCK bit before reuse.
			self.clear_protected_flag(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			// New node enters with CLOCK reference bit = 0 (unreferenced).
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	// ── linked-list helpers ─────────────────────────────────────────────────

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

// ── CountMinSketch (8× width, u32 counters, bijective remix hash) ────────────

#[derive(Debug)]
struct CountMinSketch {
	width: usize,
	depth: usize,
	counts: Vec<Vec<u32>>,
}

/// Compute a seeded 64-bit hash by injecting `seed` before hashing `key`.
/// Using a seed of 0 produces the base hash value; rows 2–3 are positioned
/// using a cheap bijective multiply-xorshift re-mix of that base value.
#[inline(always)]
fn seeded_hash<K: Hash>(key: &K, seed: u64) -> u64 {
	let mut hasher = DefaultHasher::new();
	hasher.write_u64(seed);
	key.hash(&mut hasher);
	hasher.finish()
}

/// Bijective multiply-xorshift re-mix of a 64-bit hash value.
///
/// Uses the Knuth/Fibonacci multiplicative constant `0x9e3779b97f4a7c15` to
/// spread bits, then `rotate_right(32)` mixes the high and low 32-bit halves
/// before the XOR. This ensures positions derived from the upper bits of `h1`
/// (rows 2–3) are not simple affine functions of the upper bits of `h0`
/// (rows 0–1). The operation is three 64-bit ALU instructions with no serial
/// data dependency on a second SipHash invocation.
#[inline(always)]
fn remix(h: u64) -> u64 {
	h.wrapping_mul(0x9e3779b97f4a7c15_u64)
		.rotate_right(32)
		^ h
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		// h0 covers rows 0–1; h1 is a cheap bijective re-mix of h0 for rows 2–3.
		// Two non-overlapping 16-bit windows of each hash give the row positions.
		// The re-mix avoids a second SipHash call while providing sufficient
		// independence between the two pairs of rows.
		let h0 = seeded_hash(key, 0);
		let h1 = remix(h0);
		let mask = self.width - 1;

		let pos0 = (h0 as usize) & mask;
		let pos1 = ((h0 >> 16) as usize) & mask;
		let pos2 = (h1 as usize) & mask;
		let pos3 = ((h1 >> 16) as usize) & mask;

		let c = &mut self.counts[0][pos0];
		if *c < MAX_COUNT { *c += 1; }

		let c = &mut self.counts[1][pos1];
		if *c < MAX_COUNT { *c += 1; }

		let c = &mut self.counts[2][pos2];
		if *c < MAX_COUNT { *c += 1; }

		let c = &mut self.counts[3][pos3];
		if *c < MAX_COUNT { *c += 1; }
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		let h0 = seeded_hash(key, 0);
		let h1 = remix(h0);
		let mask = self.width - 1;

		let c0 = self.counts[0][(h0 as usize) & mask];
		let c1 = self.counts[1][((h0 >> 16) as usize) & mask];
		let c2 = self.counts[2][(h1 as usize) & mask];
		let c3 = self.counts[3][((h1 >> 16) as usize) & mask];

		c0.min(c1).min(c2).min(c3)
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
