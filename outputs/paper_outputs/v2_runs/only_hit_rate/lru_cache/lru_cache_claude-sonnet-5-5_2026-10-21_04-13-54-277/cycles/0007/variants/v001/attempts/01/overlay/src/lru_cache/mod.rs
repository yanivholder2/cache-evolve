// Segmented LRU cache: HashMap + arena shared by two doubly-linked lists
// (probation = 0, protected = 1). New keys enter probation; a hit promotes to protected.
// Eviction takes the coldest (by sketch) of the 4 oldest probation entries (fallback: protected)
// and reuses its arena slot.
// A TinyLFU-style count-min sketch, fronted by a one-bit doorkeeper, gates inserts into a
// full cache: a candidate colder than the victim is dropped. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const PROBATION: usize = 0;
const PROTECTED: usize = 1;
const ROWS: usize = 4;
const SCAN: usize = 4;
const SEEDS: [u64; ROWS] = [
	0x9E37_79B9_7F4A_7C15,
	0xC2B2_AE3D_27D4_EB4F,
	0x1656_67B1_9E37_79F9,
	0x2545_F491_4F6C_DD1D,
];

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	segment: bool, // false = probation, true = protected
}

#[derive(Debug, Clone, Copy, Default)]
struct List {
	head: Option<usize>, // LRU end
	tail: Option<usize>, // MRU end
	len: usize,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	lists: [List; 2],
	protected_cap: usize,
	capacity: usize,
	sketch: Vec<u8>, // ROWS * width counters
	width: usize,    // power of two
	additions: usize,
	sample: usize,
	door: Vec<u64>, // doorkeeper bitset
	door_mask: usize, // number of bits - 1 (power of two)
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
		// 95% protected share, at least 1, always leaving >= 1 probation slot.
		let protected_cap = if capacity > 1 {
			(capacity * 19 / 20).max(1).min(capacity - 1)
		} else {
			0
		};
		let width = capacity.next_power_of_two();
		let door_bits = (capacity.saturating_mul(8)).next_power_of_two().max(64);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			lists: [List::default(); 2],
			protected_cap,
			capacity,
			sketch: vec![0u8; ROWS * width],
			width,
			additions: 0,
			sample: capacity.saturating_mul(10),
			door: vec![0u64; door_bits / 64],
			door_mask: door_bits - 1,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	fn key_hash(key: &K) -> u64 {
		let mut h = DefaultHasher::new();
		key.hash(&mut h);
		h.finish()
	}

	fn slot(&self, row: usize, h: u64) -> usize {
		let x = (h ^ SEEDS[row]).wrapping_mul(0xFF51_AFD7_ED55_8CCD);
		let x = x ^ (x >> 32);
		row * self.width + (x as usize & (self.width - 1))
	}

	fn door_pos(&self, h: u64) -> (usize, u64) {
		let x = (h ^ 0xA076_1D64_78BD_642F).wrapping_mul(0xE703_7ED1_A0B4_28DB);
		let x = x ^ (x >> 29);
		let bit = x as usize & self.door_mask;
		(bit >> 6, 1u64 << (bit & 63))
	}

	fn door_has(&self, h: u64) -> bool {
		let (w, m) = self.door_pos(h);
		self.door[w] & m != 0
	}

	/// Record an access: first sighting only sets the doorkeeper bit.
	fn record(&mut self, h: u64) {
		let (w, m) = self.door_pos(h);
		if self.door[w] & m == 0 {
			self.door[w] |= m;
			return;
		}
		self.sketch_inc(h);
	}

	fn sketch_inc(&mut self, h: u64) {
		for r in 0..ROWS {
			let s = self.slot(r, h);
			if self.sketch[s] < u8::MAX {
				self.sketch[s] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.sample {
			for c in self.sketch.iter_mut() {
				*c >>= 1;
			}
			for w in self.door.iter_mut() {
				*w = 0;
			}
			self.additions = 0;
		}
	}

	fn sketch_est(&self, h: u64) -> u16 {
		let base = (0..ROWS).map(|r| self.sketch[self.slot(r, h)]).min().unwrap_or(0) as u16;
		base + self.door_has(h) as u16
	}

	/// Returns the value for `key` if present, recording the hit.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let h = Self::key_hash(key);
		self.record(h);
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the coldest of the few LRU-most
	/// probation entries (else protected), unless the candidate is estimated colder than
	/// that victim, in which case it is dropped.
	pub fn put(&mut self, key: K, value: V) {
		let h = Self::key_hash(&key);
		self.record(h);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		let idx = if self.nodes.len() >= self.capacity {
			let seg = if self.lists[PROBATION].head.is_some() { PROBATION } else { PROTECTED };
			let mut victim = self.lists[seg].head.expect("non-empty cache has a victim");
			let mut vh = Self::key_hash(&self.nodes[victim].key);
			let mut vest = self.sketch_est(vh);
			let mut cur = self.nodes[victim].next;
			let mut n = 1;
			while n < SCAN {
				let Some(c) = cur else { break };
				let ch = Self::key_hash(&self.nodes[c].key);
				let est = self.sketch_est(ch);
				if est < vest {
					victim = c;
					vh = ch;
					vest = est;
				}
				cur = self.nodes[c].next;
				n += 1;
			}
			let _ = vh;
			if self.sketch_est(h) < vest {
				return;
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
				segment: false,
			});
			idx
		};
		self.push_back(idx, PROBATION);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Handle a hit on `idx`: probation -> protected MRU (with demotion), protected -> protected MRU.
	fn touch(&mut self, idx: usize) {
		let was_protected = self.nodes[idx].segment;
		self.unlink(idx);
		self.push_back(idx, PROTECTED);
		if !was_protected {
			while self.lists[PROTECTED].len > self.protected_cap {
				let demote = self.lists[PROTECTED].head.expect("protected non-empty");
				self.unlink(demote);
				self.push_back(demote, PROBATION);
			}
		}
	}

	fn unlink(&mut self, idx: usize) {
		let seg = self.nodes[idx].segment as usize;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.lists[seg].head = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.lists[seg].tail = prev,
		}
		self.lists[seg].len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of segment `seg`.
	fn push_back(&mut self, idx: usize, seg: usize) {
		let tail = self.lists[seg].tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		self.nodes[idx].segment = seg == PROTECTED;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.lists[seg].head = Some(idx),
		}
		self.lists[seg].tail = Some(idx);
		self.lists[seg].len += 1;
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
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn scan_does_not_evict_hot_key() {
		let mut cache = LruCache::new(5);
		cache.put(1, 10);
		cache.get(&1); // promoted to protected
		for k in 2..20 {
			cache.put(k, k);
		}
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_zero_panics_guard() {
		assert!(LruCache::<i64, i64>::try_new(0).is_none());
	}
}
