use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod tiny_lfu_jni;

const SKETCH_DEPTH: usize = 4;
const SKETCH_WIDTH_FACTOR: usize = 4;
const MIN_SKETCH_WIDTH: usize = 16;
const MAX_COUNT: u32 = u32::MAX;

/// Segment tags (also used as list selectors for the intrusive lists).
const WINDOW: u8 = 0;
const PROBATION: u8 = 1;
const PROTECTED: u8 = 2;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	seg: u8,
}

/// W-TinyLFU: a small admission-window LRU feeding a segmented (probation /
/// protected) main region. The TinyLFU admission contest runs only when a
/// window evictee meets the main region's victim.
#[derive(Debug)]
pub struct TinyLfuCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	win_head: Option<usize>,
	win_tail: Option<usize>,
	win_len: usize,
	prob_head: Option<usize>,
	prob_tail: Option<usize>,
	prob_len: usize,
	prot_head: Option<usize>,
	prot_tail: Option<usize>,
	prot_len: usize,
	capacity: usize,
	window_cap: usize,
	main_cap: usize,
	protected_cap: usize,
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
		let (window_cap, main_cap) = if capacity == 1 {
			(1, 0)
		} else {
			let w = (capacity / 100).max(1);
			(w, capacity - w)
		};
		let protected_cap = main_cap * 80 / 100;
		log::info!(
			"TinyLfuCache.try_new: capacity={capacity} window={window_cap} main={main_cap} \
			 protected={protected_cap} sketch_width={width}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			win_head: None,
			win_tail: None,
			win_len: 0,
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			prot_head: None,
			prot_tail: None,
			prot_len: 0,
			capacity,
			window_cap,
			main_cap,
			protected_cap,
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
				seg: WINDOW,
			});
			self.push_back(WINDOW, idx);
			self.map.insert(key, idx);
			if self.win_len > self.window_cap {
				// Not full: the main region necessarily has room.
				let cand = self.win_head.expect("TinyLfuCache: window overflow with empty window");
				self.unlink(WINDOW, cand);
				if self.prob_len + self.prot_len < self.main_cap {
					self.push_back(PROBATION, cand);
				} else {
					// Defensive: should be unreachable while not full.
					let freed = self.contest(cand);
					self.drop_slot_unreachable(freed);
				}
			}
			return;
		}

		// Full: decide which slot is freed before writing the new key.
		let cand = self.win_head.expect("TinyLfuCache: full cache has empty window");
		self.unlink(WINDOW, cand);
		let freed = if self.prob_len + self.prot_len < self.main_cap {
			// Unreachable when full (window would be over cap); fall back to
			// evicting the main victim so the arena never grows.
			self.push_back(PROBATION, cand);
			let victim = self
				.prob_head
				.or(self.prot_head)
				.expect("TinyLfuCache: no victim");
			let seg = self.nodes[victim].seg;
			self.unlink(seg, victim);
			let vk = self.nodes[victim].key.clone();
			self.map.remove(&vk);
			victim
		} else {
			self.contest(cand)
		};
		self.nodes[freed].key = key.clone();
		self.nodes[freed].value = value;
		self.push_back(WINDOW, freed);
		self.map.insert(key, freed);
	}

	/// Admission contest between the (already unlinked) window candidate and the
	/// main region's victim (probation LRU, else protected LRU). Returns the index
	/// of the freed slot, already unlinked and removed from `map`.
	fn contest(&mut self, cand: usize) -> usize {
		let victim = self.prob_head.or(self.prot_head);
		match victim {
			Some(v) => {
				let cand_freq = self.sketch.estimate(&self.nodes[cand].key);
				let victim_freq = self.sketch.estimate(&self.nodes[v].key);
				if cand_freq > victim_freq {
					let seg = self.nodes[v].seg;
					self.unlink(seg, v);
					let vk = self.nodes[v].key.clone();
					self.map.remove(&vk);
					self.push_back(PROBATION, cand);
					v
				} else {
					let ck = self.nodes[cand].key.clone();
					self.map.remove(&ck);
					cand
				}
			}
			None => {
				let ck = self.nodes[cand].key.clone();
				self.map.remove(&ck);
				cand
			}
		}
	}

	/// Remove an unlinked, unmapped slot from the arena by swapping with the last
	/// slot. Only used on a defensive path that should never run.
	fn drop_slot_unreachable(&mut self, freed: usize) {
		let last = self.nodes.len() - 1;
		if freed != last {
			let seg = self.nodes[last].seg;
			self.unlink(seg, last);
			self.nodes.swap(freed, last);
			self.nodes.pop();
			let k = self.nodes[freed].key.clone();
			self.map.insert(k, freed);
			self.push_back(seg, freed);
		} else {
			self.nodes.pop();
		}
	}

	fn on_hit(&mut self, idx: usize) {
		match self.nodes[idx].seg {
			WINDOW => {
				self.unlink(WINDOW, idx);
				self.push_back(WINDOW, idx);
			}
			PROBATION => {
				self.unlink(PROBATION, idx);
				self.push_back(PROTECTED, idx);
				if self.prot_len > self.protected_cap {
					let demote = self.prot_head.expect("TinyLfuCache: protected empty");
					self.unlink(PROTECTED, demote);
					self.push_back(PROBATION, demote);
				}
			}
			_ => {
				self.unlink(PROTECTED, idx);
				self.push_back(PROTECTED, idx);
			}
		}
	}

	fn list_mut(&mut self, list: u8) -> (&mut Option<usize>, &mut Option<usize>, &mut usize) {
		match list {
			WINDOW => (&mut self.win_head, &mut self.win_tail, &mut self.win_len),
			PROBATION => (&mut self.prob_head, &mut self.prob_tail, &mut self.prob_len),
			_ => (&mut self.prot_head, &mut self.prot_tail, &mut self.prot_len),
		}
	}

	fn unlink(&mut self, list: u8, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		if let Some(p) = prev {
			self.nodes[p].next = next;
		}
		if let Some(n) = next {
			self.nodes[n].prev = prev;
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		let (head, tail, len) = self.list_mut(list);
		if prev.is_none() {
			*head = next;
		}
		if next.is_none() {
			*tail = prev;
		}
		*len -= 1;
	}

	fn push_back(&mut self, list: u8, idx: usize) {
		let old_tail = {
			let (_, tail, _) = self.list_mut(list);
			*tail
		};
		self.nodes[idx].prev = old_tail;
		self.nodes[idx].next = None;
		self.nodes[idx].seg = list;
		if let Some(t) = old_tail {
			self.nodes[t].next = Some(idx);
		}
		let (head, tail, len) = self.list_mut(list);
		if old_tail.is_none() {
			*head = Some(idx);
		}
		*tail = Some(idx);
		*len += 1;
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
		assert_eq!(cache.map.len(), cache.nodes.len());
		assert!(cache.nodes.len() <= cache.capacity);
		assert_eq!(cache.win_len + cache.prob_len + cache.prot_len, cache.nodes.len());
		assert!(cache.win_len <= cache.window_cap);
		assert!(cache.prot_len <= cache.protected_cap);
		for (list, head, len) in [
			(WINDOW, cache.win_head, cache.win_len),
			(PROBATION, cache.prob_head, cache.prob_len),
			(PROTECTED, cache.prot_head, cache.prot_len),
		] {
			let mut n = 0;
			let mut cur = head;
			while let Some(i) = cur {
				assert_eq!(cache.nodes[i].seg, list);
				assert_eq!(cache.map[&cache.nodes[i].key], i);
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
		// capacity 2: window=1, main=1.
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // window overflow: 1 -> probation (main has room)
		cache.get(&1); // make key 1 hotter than key 2
		cache.get(&3);
		cache.get(&3);
		cache.put(3, 30); // contest: window evictee 2 vs probation victim 1 -> 2 loses
		check_structure(&cache);

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn new_item_is_resident_and_contest_runs_at_window_overflow() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1);
		cache.put(3, 30); // low-frequency newcomer is still admitted to the window
		check_structure(&cache);

		assert!(cache.get(&3).is_some());
		assert!(cache.get(&1).is_some());
		assert!(cache.get(&2).is_none()); // window evictee lost the contest
	}

	#[test]
	fn window_evictee_beats_cold_main_victim() {
		let mut cache = TinyLfuCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // 1 -> probation
		cache.get(&2);
		cache.get(&2);
		cache.put(3, 30); // evictee 2 (hot) beats victim 1 (cold)
		check_structure(&cache);

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn capacity_one_drops_window_evictee() {
		let mut cache = TinyLfuCache::new(1);
		cache.put(1, 10);
		cache.put(2, 20);
		check_structure(&cache);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
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
		check_structure(&cache);

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn structure_holds_under_mixed_workload() {
		for cap in [1usize, 2, 3, 8, 150, 300] {
			let mut cache = TinyLfuCache::new(cap);
			let mut x: u64 = 0x9e3779b97f4a7c15;
			for _ in 0..5000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				let k = (x % (cap as u64 * 3 + 5)) as i64;
				if cache.get(&k).is_none() {
					cache.put(k, k * 10);
					assert_eq!(*cache.get(&k).unwrap(), k * 10);
				}
				check_structure(&cache);
			}
		}
	}
}
