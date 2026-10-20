use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Top bit of `Node::prev`: set while the node lives in the protected segment.
const PROT_BIT: usize = 1 << (usize::BITS - 1);
/// Link sentinel meaning "no neighbour". Real indices are < capacity, so this never collides.
const NIL: usize = !PROT_BIT;

fn link(raw: usize) -> Option<usize> {
	if raw == NIL {
		None
	} else {
		Some(raw)
	}
}

/// Resident entry. The protected flag is packed into the top bit of `prev`, so the node
/// stays at four machine words (no separate bool, no padding).
#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	/// Top bit: protected flag. Low bits: previous link, or `NIL`.
	prev: usize,
	/// Next link, or `NIL`.
	next: usize,
}

impl<K, V> Node<K, V> {
	fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			prev: NIL,
			next: NIL,
		}
	}

	fn prev_idx(&self) -> Option<usize> {
		link(self.prev & !PROT_BIT)
	}

	fn next_idx(&self) -> Option<usize> {
		link(self.next)
	}

	fn set_prev_idx(&mut self, idx: Option<usize>) {
		self.prev = (self.prev & PROT_BIT) | idx.unwrap_or(NIL);
	}

	fn set_next_idx(&mut self, idx: Option<usize>) {
		self.next = idx.unwrap_or(NIL);
	}

	#[inline(always)]
	fn is_protected(&self) -> bool {
		(self.prev & PROT_BIT) != 0
	}

	#[inline(always)]
	fn set_protected(&mut self) {
		self.prev |= PROT_BIT;
	}

	#[inline(always)]
	fn clear_protected(&mut self) {
		self.prev &= !PROT_BIT;
	}
}

/// One doubly-linked list holds both segments, probation first and protected second:
///
/// `head -> [probation LRU .. probation MRU] [protected LRU .. protected MRU] -> tail`
///
/// `prot_head` marks the first protected node (the boundary); the protected MRU is `tail`.
/// The victim is always `head`: the probation LRU if probation is non-empty, else the
/// protected LRU.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>,
	tail: Option<usize>,
	prot_head: Option<usize>,
	prot_len: usize,
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
		log::info!("TinyLfuCache.try_new: capacity={capacity} sketch_width={width}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			prot_head: None,
			prot_len: 0,
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
			self.nodes.push(Node::new(key.clone(), value));
			self.push_probation_mru(idx);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self.head.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.push_probation_mru(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	fn protected_limit(&self) -> usize {
		(self.capacity * 4 / 5).max(1)
	}

	/// Record a hit on resident node `idx`.
	/// A protected hit moves it to the protected MRU. A probation hit promotes it to the
	/// protected MRU, first demoting the protected LRU to probation MRU when the protected
	/// segment is at its limit.
	///
	/// The protected flag is read straight from the packed `prev` word (one load, one mask
	/// test) rather than through the accessor on this hot path.
	fn touch(&mut self, idx: usize) {
		if (self.nodes[idx].prev & PROT_BIT) != 0 {
			self.unlink(idx);
			self.push_protected_mru(idx);
			return;
		}
		if self.prot_len >= self.protected_limit() {
			let demoted = self
				.prot_head
				.expect("TinyLfuCache: protected segment empty at limit");
			self.unlink(demoted);
			self.push_probation_mru(demoted);
		}
		self.unlink(idx);
		self.push_protected_mru(idx);
	}

	/// Detach `idx` from the list, keeping the segment anchors consistent.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev_idx();
		let next = self.nodes[idx].next_idx();
		let was_protected = (self.nodes[idx].prev & PROT_BIT) != 0;
		match prev {
			Some(p) => self.nodes[p].set_next_idx(next),
			None => self.head = next,
		}
		match next {
			Some(n) => self.nodes[n].set_prev_idx(prev),
			None => self.tail = prev,
		}
		if self.prot_head == Some(idx) {
			self.prot_head = next;
		}
		if was_protected {
			self.prot_len -= 1;
		}
		let node = &mut self.nodes[idx];
		node.prev &= !PROT_BIT;
		node.set_prev_idx(None);
		node.set_next_idx(None);
	}

	/// Append an unlinked node at the protected MRU end (the global tail).
	fn push_protected_mru(&mut self, idx: usize) {
		let prev = self.tail;
		{
			let node = &mut self.nodes[idx];
			node.set_prev_idx(prev);
			node.set_next_idx(None);
			node.set_protected();
		}
		match prev {
			Some(t) => self.nodes[t].set_next_idx(Some(idx)),
			None => self.head = Some(idx),
		}
		self.tail = Some(idx);
		if self.prot_head.is_none() {
			self.prot_head = Some(idx);
		}
		self.prot_len += 1;
	}

	/// Insert an unlinked node at the probation MRU end: immediately before the protected
	/// boundary, or at the global tail when the protected segment is empty.
	fn push_probation_mru(&mut self, idx: usize) {
		let next = self.prot_head;
		let prev = match next {
			Some(b) => self.nodes[b].prev_idx(),
			None => self.tail,
		};
		{
			let node = &mut self.nodes[idx];
			node.clear_protected();
			node.set_prev_idx(prev);
			node.set_next_idx(next);
		}
		match prev {
			Some(p) => self.nodes[p].set_next_idx(Some(idx)),
			None => self.head = Some(idx),
		}
		match next {
			Some(b) => self.nodes[b].set_prev_idx(Some(idx)),
			None => self.tail = Some(idx),
		}
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
