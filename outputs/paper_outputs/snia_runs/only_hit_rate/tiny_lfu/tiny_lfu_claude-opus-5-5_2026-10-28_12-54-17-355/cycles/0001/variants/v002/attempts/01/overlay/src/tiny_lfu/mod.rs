use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Which segment of the segmented LRU a node currently lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Probation,
	Protected,
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: Region,
}

/// TinyLFU admission over a segmented LRU (probation 20% / protected 80%).
///
/// New entries land at probation MRU; a re-reference promotes to protected MRU,
/// demoting the protected LRU back to probation MRU when protected overflows.
/// The admission-contest victim is the probation LRU (falling back to the
/// protected LRU only when probation is empty).
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	prob_head: Option<usize>,
	prob_tail: Option<usize>,
	prob_len: usize,
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	prot_len: usize,
	protected_cap: usize,
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
		let protected_cap = capacity * 8 / 10;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} protected_cap={protected_cap} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			protected_cap,
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
		self.on_hit(idx);
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		self.sketch.increment(&key);
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.on_hit(idx);
			return;
		}

		if self.nodes.len() < self.capacity {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				region: Region::Probation,
			});
			self.push_back(idx);
			self.map.insert(key, idx);
			return;
		}

		let victim_idx = self
			.prob_head
			.or(self.prot_head)
			.expect("TinyLfuCache: full cache has no victim");
		let victim_key = self.nodes[victim_idx].key.clone();
		let victim_freq = self.sketch.estimate(&victim_key);
		let incoming_freq = self.sketch.estimate(&key);
		if incoming_freq > victim_freq {
			self.map.remove(&victim_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].region = Region::Probation;
			self.push_back(victim_idx);
			self.map.insert(key, victim_idx);
		}
	}

	/// Segmented-LRU reaction to a re-reference of a resident node.
	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].region {
			Region::Protected => {
				self.unlink(idx);
				self.push_back(idx);
			}
			Region::Probation => {
				self.unlink(idx);
				if self.protected_cap == 0 {
					self.push_back(idx);
					return;
				}
				self.nodes[idx].region = Region::Protected;
				self.push_back(idx);
				if self.prot_len > self.protected_cap {
					let demote = self
						.prot_head
						.expect("TinyLfuCache: overfull protected segment has no head");
					self.unlink(demote);
					self.nodes[demote].region = Region::Probation;
					self.push_back(demote);
				}
			}
		}
	}

	/// Unlink `idx` from the list identified by its region tag.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let region = self.nodes[idx].region;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => match region {
				Region::Probation => self.prob_head = next,
				Region::Protected => self.prot_head = next,
			},
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => match region {
				Region::Probation => self.prob_tail = prev,
				Region::Protected => self.prot_tail = prev,
			},
		}
		match region {
			Region::Probation => self.prob_len -= 1,
			Region::Protected => self.prot_len -= 1,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Push `idx` at the MRU end of the list identified by its region tag.
	fn push_back(&mut self, idx: usize) {
		let region = self.nodes[idx].region;
		let tail = match region {
			Region::Probation => self.prob_tail,
			Region::Protected => self.prot_tail,
		};
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => match region {
				Region::Probation => self.prob_head = Some(idx),
				Region::Protected => self.prot_head = Some(idx),
			},
		}
		match region {
			Region::Probation => {
				self.prob_tail = Some(idx);
				self.prob_len += 1;
			}
			Region::Protected => {
				self.prot_tail = Some(idx);
				self.prot_len += 1;
			}
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

	fn check_structure(cache: &TinyLfuCache<i64, i64>) {
		assert_eq!(cache.map.len(), cache.prob_len + cache.prot_len);
		assert!(cache.map.len() <= cache.capacity);
		assert!(cache.prot_len <= cache.protected_cap);
		for (head, region, len) in [
			(cache.prob_head, Region::Probation, cache.prob_len),
			(cache.prot_head, Region::Protected, cache.prot_len),
		] {
			let mut n = 0;
			let mut cur = head;
			while let Some(i) = cur {
				assert_eq!(cache.nodes[i].region, region);
				assert_eq!(cache.map.get(&cache.nodes[i].key), Some(&i));
				n += 1;
				cur = cache.nodes[i].next;
			}
			assert_eq!(n, len);
		}
	}

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

	#[test]
	fn slru_structure_holds_under_mixed_workload() {
		for cap in [1usize, 2, 3, 5, 8, 16] {
			let mut cache = TinyLfuCache::<i64, i64>::new(cap);
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
			for _ in 0..5000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let key = (x % 40) as i64;
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
				check_structure(&cache);
			}
		}
	}

	#[test]
	fn protected_keys_survive_probation_churn() {
		let mut cache = TinyLfuCache::new(10); // protected_cap = 8
		for k in 0..10 {
			cache.put(k, k);
		}
		cache.get(&0); // promote 0 to protected
		// Hot newcomers displace probation residents, never key 0.
		for k in 100..110 {
			for _ in 0..5 {
				cache.get(&k);
			}
			cache.put(k, k);
		}
		assert!(cache.get(&0).is_some());
		check_structure(&cache);
	}
}
