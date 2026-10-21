// Segmented LRU (probation + protected) with a TinyLFU-style count-min sketch admission gate.
// Arena-based doubly-linked lists (two lists share one arena). No unsafe, no external crates.
// Sketch uses 4-bit saturating counters (two per byte) with conservative updates.
// Each miss is recorded once: get-miss records, and the following put of the same key skips.
// Admission: candidate must exceed victim estimate (margin 0).

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const PROBATION: u8 = 0;
const PROTECTED: u8 = 1;
const ROWS: usize = 4;
const MAX_COUNT: u8 = 15;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: u8,
}

/// Cheap multiplicative hasher (fold of written words).
#[derive(Default)]
struct FastHasher(u64);

impl Hasher for FastHasher {
	fn finish(&self) -> u64 {
		let mut h = self.0;
		h ^= h >> 32;
		h = h.wrapping_mul(0x9E37_79B9_7F4A_7C15);
		h ^ (h >> 29)
	}
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(5) ^ b as u64).wrapping_mul(0x517C_C1B7_2722_0A95);
		}
	}
	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0.rotate_left(5) ^ i).wrapping_mul(0x517C_C1B7_2722_0A95);
	}
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}
}

#[derive(Debug)]
struct Sketch {
	table: Vec<u8>, // ROWS * width nibbles, two per byte
	mask: usize,
	additions: usize,
	sample: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.next_power_of_two().max(4);
		Self {
			table: vec![0; ROWS * width / 2],
			mask: width - 1,
			additions: 0,
			sample: 10 * capacity,
		}
	}

	#[inline]
	fn idx(&self, h: u64, row: usize) -> usize {
		let x = h.wrapping_mul(0xFF51_AFD7_ED55_8CCD ^ ((row as u64) << 1 | 1));
		let x = (x >> (8 + 8 * row as u32)) ^ (x >> 7);
		row * (self.mask + 1) + (x as usize & self.mask)
	}

	#[inline]
	fn get(&self, i: usize) -> u8 {
		(self.table[i >> 1] >> ((i & 1) * 4)) & 0xF
	}

	fn estimate(&self, h: u64) -> u8 {
		let mut m = MAX_COUNT;
		for r in 0..ROWS {
			m = m.min(self.get(self.idx(h, r)));
		}
		m
	}

	fn record(&mut self, h: u64) {
		let mut ids = [0usize; ROWS];
		let mut m = MAX_COUNT;
		for r in 0..ROWS {
			ids[r] = self.idx(h, r);
			m = m.min(self.get(ids[r]));
		}
		if m < MAX_COUNT {
			for &i in ids.iter() {
				if self.get(i) == m {
					self.table[i >> 1] += 1 << ((i & 1) * 4);
				}
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for b in self.table.iter_mut() {
				*b = (*b >> 1) & 0x77;
			}
			self.additions = 0;
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: [Option<usize>; 2], // LRU end per segment
	tail: [Option<usize>; 2], // MRU end per segment
	len: [usize; 2],
	protected_cap: usize,
	sketch: Sketch,
	capacity: usize,
	last_miss: Option<u64>,
}

#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = FastHasher::default();
	key.hash(&mut h);
	h.finish()
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
			head: [None; 2],
			tail: [None; 2],
			len: [0; 2],
			protected_cap: capacity * 95 / 100,
			sketch: Sketch::new(capacity),
			capacity,
			last_miss: None,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present. A hit promotes; a miss records frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = match self.map.get(key) {
			Some(&i) => i,
			None => {
				let h = hash_key(key);
				self.sketch.record(h);
				self.last_miss = Some(h);
				return None;
			}
		};
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts `key` (non-resident). If full, the candidate competes with the victim.
	pub fn put(&mut self, key: K, value: V) {
		// Defensive: resident key -> update in place.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		let h = hash_key(&key);
		if self.last_miss == Some(h) {
			self.last_miss = None;
		} else {
			self.sketch.record(h);
		}

		let idx = if self.nodes.len() >= self.capacity {
			let victim = match self.head[PROBATION as usize] {
				Some(v) => v,
				None => self.head[PROTECTED as usize].expect("non-empty list has a head"),
			};
			let vh = hash_key(&self.nodes[victim].key);
			let ce = self.sketch.estimate(h) as u32;
			let ve = self.sketch.estimate(vh) as u32;
			let admit = if ve == 0 { ce >= 1 } else { ce > ve };
			if !admit {
				return; // reject candidate
			}
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				seg: PROBATION,
			});
			idx
		};
		self.push_back(idx, PROBATION);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	fn touch(&mut self, idx: usize) {
		let seg = self.nodes[idx].seg;
		if seg == PROTECTED {
			self.unlink(idx);
			self.push_back(idx, PROTECTED);
		} else if self.protected_cap == 0 {
			self.unlink(idx);
			self.push_back(idx, PROBATION);
		} else {
			self.unlink(idx);
			self.push_back(idx, PROTECTED);
			while self.len[PROTECTED as usize] > self.protected_cap {
				let d = self.head[PROTECTED as usize].expect("protected non-empty");
				self.unlink(d);
				self.push_back(d, PROBATION);
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].seg as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.head[seg] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tail[seg] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.len[seg] -= 1;
	}

	fn push_back(&mut self, idx: usize, seg: u8) {
		let s = seg as usize;
		self.nodes[idx].seg = seg;
		self.nodes[idx].prev = self.tail[s];
		self.nodes[idx].next = None;
		match self.tail[s] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.head[s] = Some(idx),
		}
		self.tail[s] = Some(idx);
		self.len[s] += 1;
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
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
