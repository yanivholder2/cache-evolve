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

/// TinyLFU cache with a protected (hot) segment and a probationary (cold) segment.
///
/// Builds on cycle/0007/v003's 8× sketch width, u32 counters, bijective remix hash,
/// and amortised strict gate.  Adds a segmented-LRU split:
///
/// - 20% of capacity (≥ 1) is the **protected** segment; nodes here have been
///   accessed at least once after initial insertion.
/// - The remaining 80% is the **probationary** segment; newly inserted nodes land
///   here.
/// - On a cache hit for a probationary node via `get`, it is promoted to the
///   protected segment (LRU tail).  If the protected segment is already full,
///   its LRU head is demoted back to the probationary segment.
/// - On a `put` for an existing probationary node, the node stays in the
///   probationary chain; only its MRU position is refreshed (no cross-chain
///   migration).  This reduces put-path latency at the cost of no additional
///   protected-slot churn from write-only events.
/// - Eviction victims for the admission gate always come from the probationary
///   segment's LRU head.  If the probationary chain is unexpectedly empty (all
///   slots occupied by protected nodes), the protected LRU head is force-demoted
///   first.
///
/// The `prot_flags` field is a packed `Vec<u64>` bitset (one bit per node slot),
/// so per-node memory is unchanged: `Node<u64,u64>` remains 48 bytes.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	/// LRU head of the **probationary** chain (least-recently-used).
	head: Option<usize>,
	/// LRU tail of the **probationary** chain (most-recently-used).
	tail: Option<usize>,
	capacity: usize,
	sketch: CountMinSketch,
	evict_counter: u32,
	/// LRU head of the **protected** chain.
	prot_head: Option<usize>,
	/// LRU tail of the **protected** chain.
	prot_tail: Option<usize>,
	/// Current number of nodes in the protected chain.
	prot_count: usize,
	/// Maximum number of nodes allowed in the protected chain (≈ capacity / 5).
	prot_capacity: usize,
	/// Packed bitset: bit `idx` is set when node slot `idx` is in the protected chain.
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
			prot_head: None,
			prot_tail: None,
			prot_count: 0,
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

	// ── bit-flag helpers ────────────────────────────────────────────────────

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

	// ── protected-chain linked-list helpers ─────────────────────────────────

	/// Remove node `idx` from the protected chain (structurally mirrors `unlink`).
	fn prot_unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.prot_head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.prot_tail = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node `idx` at the MRU tail of the protected chain (mirrors `push_back`).
	fn prot_push_back(&mut self, idx: usize) {
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
	}

	// ── promotion helper ────────────────────────────────────────────────────

	/// Promote node `idx` (already unlinked from the probationary chain) into the
	/// protected chain, demoting the protected LRU head first if the segment is full.
	fn promote_to_protected(&mut self, idx: usize) {
		// Count the incoming node tentatively so the overflow check is simple.
		self.prot_count += 1;
		if self.prot_count > self.prot_capacity {
			// Protected segment is full: demote its LRU head to probationary.
			let demote_idx = self.prot_head
				.expect("prot_head must exist when prot_count > 0");
			self.prot_unlink(demote_idx);
			self.clear_protected_flag(demote_idx);
			self.push_back(demote_idx);
			self.prot_count -= 1;
		}
		self.prot_push_back(idx);
		self.set_protected_flag(idx);
	}

	// ── public cache operations ─────────────────────────────────────────────

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;

		if self.is_protected(idx) {
			// Already in protected: refresh MRU position within the protected chain.
			self.prot_unlink(idx);
			self.prot_push_back(idx);
		} else {
			// In probationary: unlink and promote to protected.
			self.unlink(idx);
			self.promote_to_protected(idx);
		}

		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);

		// ── existing-key update ──────────────────────────────────────────────
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			if self.is_protected(idx) {
				// Already protected: just refresh MRU position within the protected chain.
				self.prot_unlink(idx);
				self.prot_push_back(idx);
			} else {
				// In probationary: stay in probationary, only refresh MRU position.
				// No cross-chain migration on write — promotion is deferred to `get`.
				self.unlink(idx);
				self.push_back(idx);
			}
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
			// New nodes always enter the probationary chain (flag bit = 0).
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		// ── full cache: amortised admission gate ─────────────────────────────
		self.evict_counter = self.evict_counter.wrapping_add(1);

		// Guard: probationary chain must be non-empty for the gate comparison.
		// If all slots are in the protected chain, force-demote the protected
		// LRU head so there is a victim available.
		if self.head.is_none() {
			let demote_idx = self.prot_head
				.expect("prot_head must exist when all slots are in protected chain");
			self.prot_unlink(demote_idx);
			self.clear_protected_flag(demote_idx);
			self.push_back(demote_idx);
			self.prot_count -= 1;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
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
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			// New node enters probationary; victim slot's flag bit is already 0.
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	// ── probationary linked-list helpers ────────────────────────────────────

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
	fn update_keeps_probationary_item_in_probationary_and_admits_hot_arrivals() {
		// With lazy promotion (put no longer promotes probationary→protected),
		// key 1's put-update keeps it in probationary at MRU position.
		// Key 2 remains at LRU of probationary and is the eviction victim.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1: stays in probationary, moves to MRU
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}
}
