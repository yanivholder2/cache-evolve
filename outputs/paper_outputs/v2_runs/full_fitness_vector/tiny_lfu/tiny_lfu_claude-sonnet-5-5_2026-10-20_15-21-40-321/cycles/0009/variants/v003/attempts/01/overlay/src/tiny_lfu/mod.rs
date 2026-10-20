use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

const ROW_SEEDS: [u64; SKETCH_DEPTH] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0xD6E8_FEB8_6659_FD93,
];

/// Cheap multiplicative hasher with a final avalanche; used once per operation.
#[derive(Default)]
struct FastHasher {
	state: u64,
}

impl Hasher for FastHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.state = (self.state.rotate_left(5) ^ b as u64).wrapping_mul(0x517C_C1B7_2722_0A95);
		}
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.state = (self.state.rotate_left(5) ^ i).wrapping_mul(0x517C_C1B7_2722_0A95);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}
	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.write_u64(i as u64);
	}
	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.write_u64(i as u64);
	}
	#[inline]
	fn write_i32(&mut self, i: i32) {
		self.write_u64(i as u64);
	}
	#[inline]
	fn finish(&self) -> u64 {
		let mut z = self.state;
		z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
		z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
		z ^ (z >> 31)
	}
}

#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = FastHasher::default();
	key.hash(&mut h);
	h.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug, Clone, Copy)]
struct Slot {
	hash: u64,
	/// slab index + 1; 0 means empty.
	idx: u32,
}

const EMPTY: Slot = Slot { hash: 0, idx: 0 };

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	table: Vec<Slot>,
	mask: usize,
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
		if capacity == 0 || capacity >= (u32::MAX as usize) / 4 {
			log::error!("TinyLfuCache.try_new: capacity must be > 0");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		let table_size = (capacity * 2).next_power_of_two().max(4);
		Some(Self {
			table: vec![EMPTY; table_size],
			mask: table_size - 1,
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

	/// Returns (table slot, slab index) for the key.
	#[inline]
	fn find(&self, hash: u64, key: &K) -> Option<(usize, usize)> {
		let mut i = (hash as usize) & self.mask;
		loop {
			let s = self.table[i];
			if s.idx == 0 {
				return None;
			}
			if s.hash == hash {
				let idx = (s.idx - 1) as usize;
				if self.nodes[idx].key == *key {
					return Some((i, idx));
				}
			}
			i = (i + 1) & self.mask;
		}
	}

	#[inline]
	fn table_insert(&mut self, hash: u64, idx: usize) {
		let mut i = (hash as usize) & self.mask;
		while self.table[i].idx != 0 {
			i = (i + 1) & self.mask;
		}
		self.table[i] = Slot { hash, idx: (idx + 1) as u32 };
	}

	/// Tombstone-free backward-shift deletion.
	fn table_remove_slot(&mut self, mut i: usize) {
		let mask = self.mask;
		loop {
			let mut j = i;
			loop {
				j = (j + 1) & mask;
				let s = self.table[j];
				if s.idx == 0 {
					self.table[i] = EMPTY;
					return;
				}
				let k = (s.hash as usize) & mask;
				let in_range = if i <= j { i < k && k <= j } else { i < k || k <= j };
				if !in_range {
					self.table[i] = s;
					i = j;
					break;
				}
			}
		}
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let (_, idx) = self.find(hash, key)?;
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		self.sketch.increment(hash);
		if let Some((_, idx)) = self.find(hash, &key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				prev: None,
				next: None,
			});
			self.push_back(idx);
			self.table_insert(hash, idx);
			return;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_hash = hash_key(&self.nodes[victim_idx].key);
		let victim_freq = self.sketch.estimate(victim_hash);
		let incoming_freq = self.sketch.estimate(hash);
		if incoming_freq > victim_freq {
			// Locate the victim's table slot and delete it.
			let mut i = (victim_hash as usize) & self.mask;
			while (self.table[i].idx as usize) != victim_idx + 1 {
				i = (i + 1) & self.mask;
			}
			self.table_remove_slot(i);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key;
			self.nodes[victim_idx].value = value;
			self.push_back(victim_idx);
			self.table_insert(hash, victim_idx);
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
	shift: u32,
	counts: Vec<u32>,
}

impl CountMinSketch {
	fn new(width: usize) -> Self {
		let depth = SKETCH_DEPTH;
		let shift = 64 - width.trailing_zeros();
		Self { width, depth, shift, counts: vec![0; width * depth] }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	#[inline]
	fn position(&self, hash: u64, row: usize) -> usize {
		row * self.width + (hash.wrapping_mul(ROW_SEEDS[row]) >> self.shift) as usize
	}

	#[inline]
	fn increment(&mut self, hash: u64) {
		for row in 0..self.depth {
			let pos = self.position(hash, row);
			let counter = &mut self.counts[pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	#[inline]
	fn estimate(&self, hash: u64) -> u32 {
		(0..self.depth)
			.map(|row| self.counts[self.position(hash, row)])
			.min()
			.unwrap_or(0)
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
