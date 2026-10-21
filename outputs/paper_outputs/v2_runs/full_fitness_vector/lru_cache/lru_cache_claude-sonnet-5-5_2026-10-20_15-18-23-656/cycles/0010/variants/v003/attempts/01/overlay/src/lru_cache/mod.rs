// LRU cache with a second-chance eviction walk whose step cap adapts to ghost hits.
// HashMap + arena-based doubly-linked list. No unsafe, no external crates.

use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const WINDOW: u32 = 256;
const GHOST_THRESHOLD: u32 = 8;
const CAP_MIN: u8 = 1;
const CAP_MAX: u8 = 8;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>,
	nodes: Vec<Node<K, V>>,
	head: Option<usize>, // eviction end
	tail: Option<usize>, // insertion end
	capacity: usize,
	walk_cap: u8,
	puts_in_window: u32,
	ghost_hits_in_window: u32,
	ghost_set: HashSet<K>,
	ghost_fifo: VecDeque<K>,
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
			head: None,
			tail: None,
			capacity,
			walk_cap: 2,
			puts_in_window: 0,
			ghost_hits_in_window: 0,
			ghost_set: HashSet::new(),
			ghost_fifo: VecDeque::new(),
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present; marks it referenced (second chance).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.nodes[idx].referenced = true;
		Some(&self.nodes[idx].value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].referenced = true;
			return;
		}
		self.tick(&key);
		let idx = if self.nodes.len() >= self.capacity {
			let victim = self.pick_victim();
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim);
			self.record_ghost(evicted_key);
			self.nodes[victim].key = key.clone();
			self.nodes[victim].value = value;
			self.nodes[victim].referenced = false;
			victim
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				referenced: false,
			});
			idx
		};
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Window bookkeeping and ghost-hit accounting; adapts `walk_cap` every WINDOW puts.
	fn tick(&mut self, key: &K) {
		if self.ghost_set.remove(key) {
			self.ghost_hits_in_window += 1;
		}
		self.puts_in_window += 1;
		if self.puts_in_window >= WINDOW {
			if self.ghost_hits_in_window > GHOST_THRESHOLD {
				self.walk_cap = (self.walk_cap + 1).min(CAP_MAX);
			} else {
				self.walk_cap = self.walk_cap.saturating_sub(1).max(CAP_MIN);
			}
			self.puts_in_window = 0;
			self.ghost_hits_in_window = 0;
		}
	}

	fn record_ghost(&mut self, key: K) {
		if self.ghost_set.insert(key.clone()) {
			self.ghost_fifo.push_back(key);
		}
		while self.ghost_fifo.len() > self.capacity {
			if let Some(old) = self.ghost_fifo.pop_front() {
				self.ghost_set.remove(&old);
			}
		}
	}

	/// Capped second-chance walk from the eviction end.
	fn pick_victim(&mut self) -> usize {
		let mut steps = 0u8;
		loop {
			let h = self.head.expect("non-empty list has a head");
			if !self.nodes[h].referenced || steps >= self.walk_cap {
				return h;
			}
			self.nodes[h].referenced = false;
			self.unlink(h);
			self.push_back(h);
			steps += 1;
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
