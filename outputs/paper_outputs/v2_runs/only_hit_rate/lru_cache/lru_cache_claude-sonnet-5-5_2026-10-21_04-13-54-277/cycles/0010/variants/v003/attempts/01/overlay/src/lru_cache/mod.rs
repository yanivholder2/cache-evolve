// Segmented LRU (probation + protected) over an arena-based doubly-linked list,
// with a 4-row count-min sketch gating probation->protected promotion and
// admission when the cache is full. No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;

const PROBATION: u8 = 0;
const PROTECTED: u8 = 1;
const ROWS: usize = 4;
const MAX_COUNT: u8 = 15;
const PROMOTE_THRESHOLD: u8 = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: u8,
}

#[derive(Debug, Clone, Copy, Default)]
struct List {
	head: Option<usize>, // LRU end
	tail: Option<usize>, // MRU end
	len: usize,
}

#[derive(Debug)]
struct Sketch {
	table: Vec<u8>, // ROWS * width
	mask: usize,
	additions: usize,
	reset_at: usize,
}

impl Sketch {
	fn new(capacity: usize) -> Self {
		let width = capacity.max(16).next_power_of_two();
		Self {
			table: vec![0; ROWS * width],
			mask: width - 1,
			additions: 0,
			reset_at: capacity.saturating_mul(10).max(1),
		}
	}

	#[inline]
	fn index(&self, row: usize, h: u64) -> usize {
		let x = (h ^ (h >> 29)).wrapping_mul(0x9E37_79B9_7F4A_7C15u64.wrapping_add((row as u64) * 0x2545_F491_4F6C_DD1D));
		let x = x ^ (x >> 32);
		row * (self.mask + 1) + ((x as usize) & self.mask)
	}

	fn estimate(&self, h: u64) -> u8 {
		let mut m = u8::MAX;
		for r in 0..ROWS {
			m = m.min(self.table[self.index(r, h)]);
		}
		m
	}

	fn record(&mut self, h: u64) {
		for r in 0..ROWS {
			let i = self.index(r, h);
			if self.table[i] < MAX_COUNT {
				self.table[i] += 1;
			}
		}
		self.additions += 1;
		if self.additions >= self.reset_at {
			for c in self.table.iter_mut() {
				*c >>= 1;
			}
			self.additions = 0;
		}
	}
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	probation: List,
	protected: List,
	protected_cap: usize,
	sketch: Sketch,
	capacity: usize,
}

fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut h = DefaultHasher::new();
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
			probation: List::default(),
			protected: List::default(),
			protected_cap: capacity / 100 * 95 + (capacity % 100) * 95 / 100,
			sketch: Sketch::new(capacity),
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, updating recency/segment.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = match self.map.get(key) {
			Some(&i) => i,
			None => {
				let h = hash_key(key);
				self.sketch.record(h);
				return None;
			}
		};
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts `key` (only called for non-resident keys, but resident is handled).
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		let h = hash_key(&key);
		self.sketch.record(h);

		let idx = if self.nodes.len() >= self.capacity {
			let victim = match self.probation.head.or(self.protected.head) {
				Some(v) => v,
				None => return,
			};
			let ce = self.sketch.estimate(h);
			let ve = self.sketch.estimate(hash_key(&self.nodes[victim].key));
			if ce <= ve {
				return; // candidate rejected
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
		if self.nodes[idx].seg == PROTECTED {
			self.unlink(idx);
			self.push_back(idx, PROTECTED);
			return;
		}
		let est = self.sketch.estimate(hash_key(&self.nodes[idx].key));
		if est >= PROMOTE_THRESHOLD && self.protected_cap > 0 {
			self.unlink(idx);
			if self.protected.len >= self.protected_cap {
				if let Some(d) = self.protected.head {
					self.unlink(d);
					self.push_back(d, PROBATION);
				}
			}
			self.push_back(idx, PROTECTED);
		} else {
			self.unlink(idx);
			self.push_back(idx, PROBATION);
		}
	}

	fn list_mut(&mut self, seg: u8) -> &mut List {
		if seg == PROTECTED {
			&mut self.protected
		} else {
			&mut self.probation
		}
	}

	/// Remove node from its current list.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let seg = self.nodes[idx].seg;
		if let Some(p) = prev {
			self.nodes[p].next = next;
		}
		if let Some(n) = next {
			self.nodes[n].prev = prev;
		}
		let l = self.list_mut(seg);
		if prev.is_none() {
			l.head = next;
		}
		if next.is_none() {
			l.tail = prev;
		}
		l.len -= 1;
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append node to the MRU end of the given segment.
	fn push_back(&mut self, idx: usize, seg: u8) {
		let tail = self.list_mut(seg).tail;
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		self.nodes[idx].seg = seg;
		if let Some(t) = tail {
			self.nodes[t].next = Some(idx);
		}
		let l = self.list_mut(seg);
		if l.head.is_none() {
			l.head = Some(idx);
		}
		l.tail = Some(idx);
		l.len += 1;
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
