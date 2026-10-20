use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u8 = u8::MAX;
const SAMPLE_VICTIMS: usize = 5;

const HASH_SEED: u64 = 0x9e37_79b9_7f4a_7c15;
const HASH_MUL: u64 = 0x517c_c1b7_2722_0a95;

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

/// Which intrusive LRU chain a resident node currently lives on. W-TinyLFU
/// partitions capacity into a small recency *window* that buffers newcomers and
/// a larger frequency-governed *main* region. A node's `prev`/`next` links are
/// interpreted against whichever chain its `region` names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Window,
	Main,
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: Region,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	// Recency window LRU chain (head = LRU, tail = MRU).
	window_head: Option<usize>,
	window_tail: Option<usize>,
	window_len: usize,
	window_cap: usize,
	// Frequency-governed main LRU chain (head = LRU, tail = MRU).
	main_head: Option<usize>,
	main_tail: Option<usize>,
	main_len: usize,
	main_cap: usize,
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
		// Carve a tiny recency window (~1% of capacity, but never fewer than one
		// slot) off the front of the cache; the remainder is the main region.
		let window_cap = (capacity / 100).max(1);
		let main_cap = capacity - window_cap;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} window_cap={window_cap} main_cap={main_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			window_head: None,
			window_tail: None,
			window_len: 0,
			window_cap,
			main_head: None,
			main_tail: None,
			main_len: 0,
			main_cap,
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
		let region = self.nodes[idx].region;
		// Promote to MRU within the node's own region.
		self.unlink(idx);
		self.push_back(idx, region);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			let region = self.nodes[idx].region;
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx, region);
			return;
		}

		if self.window_len + self.main_len < self.capacity {
			self.admit_not_full(key, value);
		} else {
			self.admit_full(key, value);
		}
	}

	/// New key while the cache still has spare capacity: park it in the window's
	/// MRU. If that overflows the window, the displaced window LRU spills into the
	/// main region (which is guaranteed to have room while the cache isn't full).
	fn admit_not_full(&mut self, key: K, value: V) {
		let idx = self.nodes.len();
		self.nodes.push(Node {
			key: key.clone(),
			value,
			prev: None,
			next: None,
			region: Region::Window,
		});
		self.push_back(idx, Region::Window);
		self.window_len += 1;
		self.map.insert(key, idx);

		if self.window_len > self.window_cap {
			let candidate = self
				.window_head
				.expect("TinyLfuCache: overflowing window has an LRU entry");
			self.unlink(candidate);
			self.window_len -= 1;
			self.push_back(candidate, Region::Main);
			self.main_len += 1;
		}
	}

	/// New key on a full cache (W-TinyLFU admission). The newcomer enters the
	/// window MRU; the window's LRU becomes the *admission candidate* and faces
	/// the main region's sampled-coldest victim. The candidate is admitted to the
	/// main region (evicting the victim) only if its frequency estimate strictly
	/// exceeds the victim's; otherwise the candidate is dropped. Either way the
	/// freed slot is reused for the newcomer so residency stays at capacity.
	fn admit_full(&mut self, key: K, value: V) {
		let candidate = self
			.window_head
			.expect("TinyLfuCache: full cache has a window LRU candidate");
		let cand_freq = self.sketch.estimate(&self.nodes[candidate].key);

		// Degenerate split (main_cap == 0, i.e. capacity == 1): no main region to
		// arbitrate against, so the window behaves as a plain LRU.
		let victim = match self.sample_victim() {
			Some(v) => v,
			None => {
				self.reuse_into_window(candidate, key, value);
				return;
			}
		};
		let victim_freq = self.sketch.estimate(&self.nodes[victim].key);

		if cand_freq > victim_freq {
			// Admit candidate to main, evict victim, newcomer takes victim's slot.
			let victim_key = self.nodes[victim].key.clone();
			self.map.remove(&victim_key);
			self.unlink(victim);
			self.main_len -= 1;

			self.unlink(candidate);
			self.window_len -= 1;
			self.push_back(candidate, Region::Main);
			self.main_len += 1;

			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.push_back(victim, Region::Window);
			self.window_len += 1;
			self.map.insert(key, victim);
		} else {
			// Reject candidate; newcomer reuses the dropped candidate's slot.
			self.reuse_into_window(candidate, key, value);
		}
	}

	/// Drop the entry currently occupying `slot` (already unlinked-able) and place
	/// `key`/`value` into that same slot at the window's MRU position.
	fn reuse_into_window(&mut self, slot: usize, key: K, value: V) {
		let old_key = self.nodes[slot].key.clone();
		self.map.remove(&old_key);
		self.unlink(slot);
		self.window_len -= 1;
		self.nodes[slot].key = key.clone();
		self.nodes[slot].value = value;
		self.push_back(slot, Region::Window);
		self.window_len += 1;
		self.map.insert(key, slot);
	}

	/// Sampled-coldest victim within the main region: walk forward from the main
	/// LRU head along `next`, collecting up to `SAMPLE_VICTIMS` candidates, and
	/// choose the one with the lowest sketch estimate. Returns `None` when the
	/// main region is empty.
	fn sample_victim(&self) -> Option<usize> {
		let mut victim = self.main_head?;
		let mut victim_freq = self.sketch.estimate(&self.nodes[victim].key);
		let mut cursor = self.nodes[victim].next;
		let mut sampled = 1;
		while sampled < SAMPLE_VICTIMS {
			let candidate = match cursor {
				Some(c) => c,
				None => break,
			};
			let candidate_freq = self.sketch.estimate(&self.nodes[candidate].key);
			if candidate_freq < victim_freq {
				victim = candidate;
				victim_freq = candidate_freq;
			}
			cursor = self.nodes[candidate].next;
			sampled += 1;
		}
		Some(victim)
	}

	fn set_head(&mut self, region: Region, value: Option<usize>) {
		match region {
			Region::Window => self.window_head = value,
			Region::Main => self.main_head = value,
		}
	}

	fn set_tail(&mut self, region: Region, value: Option<usize>) {
		match region {
			Region::Window => self.window_tail = value,
			Region::Main => self.main_tail = value,
		}
	}

	fn tail_of(&self, region: Region) -> Option<usize> {
		match region {
			Region::Window => self.window_tail,
			Region::Main => self.main_tail,
		}
	}

	/// Detach `idx` from whichever region chain its `region` flag names.
	fn unlink(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.set_head(region, next),
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.set_tail(region, prev),
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append `idx` to the MRU (tail) of `region`, updating its `region` flag.
	fn push_back(&mut self, idx: usize, region: Region) {
		self.nodes[idx].region = region;
		let tail = self.tail_of(region);
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.set_head(region, Some(idx)),
		}
		self.set_tail(region, Some(idx));
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
	fn new_arrival_enters_admission_window() {
		// Under W-TinyLFU a brand-new key is buffered in the recency window rather
		// than being rejected outright (the parent's strict-admission behaviour).
		// The newcomer (key 3) lands in the window and displaces the prior window
		// occupant (key 2); the frequency-favoured main resident (key 1) survives.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30);

		assert!(cache.get(&1).is_some()); // hot main resident kept
		assert!(cache.get(&3).is_some()); // newcomer buffered in the window
		assert!(cache.get(&2).is_none()); // displaced window occupant evicted
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
