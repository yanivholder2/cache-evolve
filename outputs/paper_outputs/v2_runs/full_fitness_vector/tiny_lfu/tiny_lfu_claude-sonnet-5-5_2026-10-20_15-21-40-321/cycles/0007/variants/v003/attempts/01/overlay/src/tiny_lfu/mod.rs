use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

const NIL: u32 = 0x7FFF_FFFF;
const PROT_BIT: u32 = 0x8000_0000;
const LINK_MASK: u32 = 0x7FFF_FFFF;

/// Links are u32; the top bit of `prev` marks protected-segment membership.
#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: u32,
	next: u32,
}

#[derive(Debug, Clone, Copy)]
struct List {
	head: u32,
	tail: u32,
	len: usize,
}

impl List {
	const fn new() -> Self {
		List { head: NIL, tail: NIL, len: 0 }
	}
}

#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	probation: List,
	protected: List,
	protected_max: usize,
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
		if capacity >= NIL as usize {
			log::error!("TinyLfuCache.try_new: capacity too large");
			return None;
		}
		let width = (capacity * SKETCH_WIDTH_FACTOR)
			.max(MIN_SKETCH_WIDTH)
			.next_power_of_two();
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			probation: List::new(),
			protected: List::new(),
			protected_max: (capacity * 4 / 5).max(1),
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

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(key);
		let &idx = self.map.get(key)?;
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: NIL,
				next: NIL,
			});
			self.push_tail(idx, false);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = if self.probation.head != NIL {
			self.probation.head as usize
		} else {
			self.protected.head as usize
		};
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_tail(victim_idx, false);
			self.map.insert(key, victim_idx);
		}
	}

	/// Hit handling: probation -> protected (promote), protected -> protected MRU.
	fn touch(&mut self, idx: usize) {
		let was_protected = self.is_protected(idx);
		self.unlink(idx);
		self.push_tail(idx, true);
		if !was_protected {
			while self.protected.len > self.protected_max {
				let d = self.protected.head as usize;
				self.unlink(d);
				self.push_head_probation(d);
			}
		}
	}

	fn is_protected(&self, idx: usize) -> bool {
		self.nodes[idx].prev & PROT_BIT != 0
	}

	fn list_mut(&mut self, protected: bool) -> &mut List {
		if protected {
			&mut self.protected
		} else {
			&mut self.probation
		}
	}

	fn unlink(&mut self, idx: usize) {
		let protected = self.is_protected(idx);
		let prev = self.nodes[idx].prev & LINK_MASK;
		let next = self.nodes[idx].next;
		if prev != NIL {
			self.nodes[prev as usize].next = next;
		} else {
			self.list_mut(protected).head = next;
		}
		if next != NIL {
			let p = &mut self.nodes[next as usize].prev;
			*p = (*p & PROT_BIT) | prev;
		} else {
			self.list_mut(protected).tail = prev;
		}
		self.list_mut(protected).len -= 1;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = NIL;
	}

	fn push_tail(&mut self, idx: usize, protected: bool) {
		let tail = self.list_mut(protected).tail;
		let flag = if protected { PROT_BIT } else { 0 };
		self.nodes[idx].prev = tail | flag;
		self.nodes[idx].next = NIL;
		if tail != NIL {
			self.nodes[tail as usize].next = idx as u32;
		} else {
			self.list_mut(protected).head = idx as u32;
		}
		let l = self.list_mut(protected);
		l.tail = idx as u32;
		l.len += 1;
	}

	fn push_head_probation(&mut self, idx: usize) {
		let head = self.probation.head;
		self.nodes[idx].prev = NIL;
		self.nodes[idx].next = head;
		if head != NIL {
			let p = &mut self.nodes[head as usize].prev;
			*p = (*p & PROT_BIT) | idx as u32;
		} else {
			self.probation.tail = idx as u32;
		}
		self.probation.head = idx as u32;
		self.probation.len += 1;
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
