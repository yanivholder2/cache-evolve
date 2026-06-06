use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
const SAMPLE: usize = 5;
// Doorkeeper uses k=2 hash positions; seeds distinct from the main sketch rows
// (0..SKETCH_DEPTH) so the Bloom bits are independent of the count-min cells.
const DOOR_SEED_A: u64 = 0x9E37_79B9_7F4A_7C15;
const DOOR_SEED_B: u64 = 0xC2B2_AE3D_27D4_EB4F;
// The doorkeeper is a short-term recency filter: clear its bits every
// DOOR_RESET_FACTOR * capacity observations so it never saturates over a long
// trace. The main count-min counters are never decayed.
const DOOR_RESET_FACTOR: u64 = 8;

#[derive(Debug)]
struct Node<K, V> {
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
		let door_reset = DOOR_RESET_FACTOR.saturating_mul(capacity as u64).max(1);
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} sketch_width={width} door_reset={door_reset}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
			sketch: CountMinSketch::new(width, door_reset),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.record(key);
		let &idx = self.map.get(key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.record(&key);
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
				prev: None,
				next: None,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		// H03: instead of fixing the victim as the strict LRU head, sample up to
		// SAMPLE resident nodes walking forward from the LRU head and choose the
		// one with the lowest estimated frequency. Ties favour the node closest
		// to the head (the oldest), since we only replace on a strictly-lower
		// estimate while iterating head -> tail.
		//
		// H02: frequency comes from the doorkeeper-gated estimator so one-hit
		// wonders score 1 from the Bloom bit alone and never inflate the main
		// count-min cells used to rank victims and admit arrivals.
		let mut victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let mut victim_freq = self.sketch.frequency(&self.nodes[victim_idx].key);
		let mut cursor = self.nodes[victim_idx].next;
		let mut sampled = 1;
		while sampled < SAMPLE {
			let candidate = match cursor {
				Some(c) => c,
				None => break,
			};
			let freq = self.sketch.frequency(&self.nodes[candidate].key);
			if freq < victim_freq {
				victim_idx = candidate;
				victim_freq = freq;
			}
			cursor = self.nodes[candidate].next;
			sampled += 1;
		}

		let victim_key = self.nodes[victim_idx].key.clone();
		let incoming_freq = self.sketch.frequency(&key);
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
	// TinyLFU doorkeeper: a small Bloom filter (k=2) sized to `width` that
	// absorbs first sightings so one-hit-wonder keys never reach the main
	// count-min counters below.
	doorkeeper: Vec<bool>,
	observations: u64,
	door_reset: u64,
}

impl CountMinSketch {
	fn new(width: usize, door_reset: u64) -> Self {
		let depth = SKETCH_DEPTH;
		let counts = vec![vec![0; width]; depth];
		let doorkeeper = vec![false; width];
		Self {
			width,
			depth,
			counts,
			doorkeeper,
			observations: 0,
			door_reset,
		}
	}

	/// Observe a key. First sightings only set the doorkeeper bits; a key whose
	/// doorkeeper bits are already all set is a genuine repeat and increments the
	/// main count-min sketch. The doorkeeper is periodically cleared so it stays
	/// a short-term recency filter.
	fn record<K: Hash>(&mut self, key: &K) {
		self.observations = self.observations.wrapping_add(1);
		if self.observations >= self.door_reset {
			for bit in &mut self.doorkeeper {
				*bit = false;
			}
			self.observations = 0;
		}

		let (a, b) = self.door_positions(key);
		if self.doorkeeper[a] && self.doorkeeper[b] {
			self.increment(key);
		} else {
			self.doorkeeper[a] = true;
			self.doorkeeper[b] = true;
		}
	}

	/// Frequency used for admission/eviction ranking: a key seen exactly once
	/// scores 1 from the doorkeeper bit alone, while recurring keys add their
	/// main count-min estimate on top.
	fn frequency<K: Hash>(&self, key: &K) -> u32 {
		(self.doorkeeper_contains(key) as u32).saturating_add(self.estimate(key))
	}

	fn doorkeeper_contains<K: Hash>(&self, key: &K) -> bool {
		let (a, b) = self.door_positions(key);
		self.doorkeeper[a] && self.doorkeeper[b]
	}

	fn door_positions<K: Hash>(&self, key: &K) -> (usize, usize) {
		(
			self.position(key, DOOR_SEED_A),
			self.position(key, DOOR_SEED_B),
		)
	}

	fn increment<K: Hash>(&mut self, key: &K) {
		for row in 0..self.depth {
			let pos = self.position(key, row as u64);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	fn estimate<K: Hash>(&self, key: &K) -> u32 {
		self
			.counts
			.iter()
			.enumerate()
			.map(|(row, _)| self.counts[row][self.position(key, row as u64)])
			.min()
			.unwrap_or(0)
	}

	fn position<K: Hash>(&self, key: &K, seed: u64) -> usize {
		let mut hasher = DefaultHasher::new();
		hasher.write_u64(seed);
		key.hash(&mut hasher);
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
}
