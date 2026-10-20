use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 16;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;
const NIL: u32 = u32::MAX;

/// Cheap multiplicative hasher with a final avalanche mix.
struct FastHasher(u64);

impl Hasher for FastHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517c_c1b7_2722_0a95);
		}
	}
	#[inline]
	fn write_u64(&mut self, x: u64) {
		self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(0x517c_c1b7_2722_0a95);
	}
	#[inline]
	fn write_i64(&mut self, x: i64) {
		self.write_u64(x as u64);
	}
	#[inline]
	fn write_usize(&mut self, x: usize) {
		self.write_u64(x as u64);
	}
	#[inline]
	fn finish(&self) -> u64 {
		let mut z = self.0;
		z ^= z >> 32;
		z = z.wrapping_mul(0x9e37_79b9_7f4a_7c15);
		z ^= z >> 29;
		z = z.wrapping_mul(0xbf58_476d_1ce4_e5b9);
		z ^ (z >> 32)
	}
}

#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = FastHasher(0x243f_6a88_85a3_08d3);
	key.hash(&mut h);
	h.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	hash: u64,
	prev: u32,
	next: u32,
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	table: Vec<u32>,
	mask: usize,
	nodes: Vec<Node<K, V>>,
	head: u32,
	tail: u32,
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
		let tsize = (capacity * 2).next_power_of_two().max(4);
		Some(Self {
			table: vec![NIL; tsize],
			mask: tsize - 1,
			nodes: Vec::with_capacity(capacity),
			head: NIL,
			tail: NIL,
			capacity,
			sketch: CountMinSketch::new(width),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("TinyLfuCache::new: initialization failed")
	}

	/// Heap bytes held by the frequency sketch: `depth * width * size_of::<u32>()`.
	pub fn sketch_heap_bytes(&self) -> usize {
		self.sketch.heap_bytes()
	}

	/// Returns (table slot, node idx) if present, else (empty slot, NIL).
	#[inline]
	fn find(&self, key: &K, hash: u64) -> (usize, u32) {
		let mut pos = (hash as usize) & self.mask;
		loop {
			let idx = self.table[pos];
			if idx == NIL {
				return (pos, NIL);
			}
			let n = &self.nodes[idx as usize];
			if n.hash == hash && n.key == *key {
				return (pos, idx);
			}
			pos = (pos + 1) & self.mask;
		}
	}

	/// Backward-shift deletion of the entry at table slot `pos`.
	fn remove_slot(&mut self, mut pos: usize) {
		let mut next = (pos + 1) & self.mask;
		loop {
			let idx = self.table[next];
			if idx == NIL {
				break;
			}
			let home = (self.nodes[idx as usize].hash as usize) & self.mask;
			// can the entry at `next` move into `pos`?
			let dist_next = next.wrapping_sub(home) & self.mask;
			let dist_pos = next.wrapping_sub(pos) & self.mask;
			if dist_next >= dist_pos {
				self.table[pos] = idx;
				pos = next;
			}
			next = (next + 1) & self.mask;
		}
		self.table[pos] = NIL;
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let hash = hash_key(key);
		self.sketch.increment(hash);
		let (_, idx) = self.find(key, hash);
		if idx == NIL {
			return None;
		}
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx as usize].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let hash = hash_key(&key);
		self.sketch.increment(hash);
		let (pos, found) = self.find(&key, hash);
		if found != NIL {
			self.nodes[found as usize].value = value;
			self.unlink(found);
			self.push_back(found);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len() as u32;
			self.nodes.push(Node {
				key,
				value,
				hash,
				prev: NIL,
				next: NIL,
			});
			self.push_back(idx);
			self.table[pos] = idx;
			return;
		}

		let victim_idx = self.head;
		debug_assert!(victim_idx != NIL, "TinyLfuCache: full cache has no victim");
		let victim_hash = self.nodes[victim_idx as usize].hash;
		let victim_freq = self.sketch.estimate(victim_hash);
		let incoming_freq = self.sketch.estimate(hash);
		if incoming_freq > victim_freq {
			// locate victim's table slot
			let mut vpos = (victim_hash as usize) & self.mask;
			while self.table[vpos] != victim_idx {
				vpos = (vpos + 1) & self.mask;
			}
			self.remove_slot(vpos);
			self.unlink(victim_idx);
			{
				let n = &mut self.nodes[victim_idx as usize];
				n.key = key;
				n.value = value;
				n.hash = hash;
			}
			self.push_back(victim_idx);
			// re-probe for empty slot (table changed)
			let mut ipos = (hash as usize) & self.mask;
			while self.table[ipos] != NIL {
				ipos = (ipos + 1) & self.mask;
			}
			self.table[ipos] = victim_idx;
		}
	}

	#[inline]
	fn unlink(&mut self, idx: u32) {
		let prev = self.nodes[idx as usize].prev;
		let next = self.nodes[idx as usize].next;
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
		let n = &mut self.nodes[idx as usize];
		n.prev = NIL;
		n.next = NIL;
	}

	#[inline]
	fn push_back(&mut self, idx: u32) {
		let tail = self.tail;
		{
			let n = &mut self.nodes[idx as usize];
			n.prev = tail;
			n.next = NIL;
		}
		if tail != NIL {
			self.nodes[tail as usize].next = idx;
		} else {
			self.head = idx;
		}
		self.tail = idx;
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
		Self { width, depth, counts }
	}

	/// Heap bytes backing the counter table: `depth * width * size_of::<u32>()`.
	fn heap_bytes(&self) -> usize {
		self.depth * self.width * std::mem::size_of::<u32>()
	}

	#[inline]
	fn position(&self, hash: u64, row: usize) -> usize {
		let h1 = hash;
		let h2 = (hash >> 32) | 1;
		(h1.wrapping_add((row as u64).wrapping_mul(h2)) as usize) & (self.width - 1)
	}

	#[inline]
	fn increment(&mut self, hash: u64) {
		for row in 0..self.depth {
			let pos = self.position(hash, row);
			let counter = &mut self.counts[row][pos];
			if *counter < MAX_COUNT {
				*counter += 1;
			}
		}
	}

	#[inline]
	fn estimate(&self, hash: u64) -> u32 {
		let mut m = u32::MAX;
		for row in 0..self.depth {
			let c = self.counts[row][self.position(hash, row)];
			if c < m {
				m = c;
			}
		}
		m
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
		cache.get(&1);
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}
}
