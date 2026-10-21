// S3-FIFO cache (small probation FIFO + main FIFO with lazy reinsertion + ghost queues)
// with an ARC-style adaptive small-queue target driven by two ghosts:
//   - small ghost hit  -> probation was too short: grow small_target
//   - main ghost hit   -> main was too small:      shrink small_target
// Nodes live in a Vec arena indexed by u32; evicted slots are reused in place.
// Public surface (LruCache, Node, try_new, new, get, put, framework::Cache impl) is
// unchanged so the JNI layer and bins keep working. No unsafe, no external crates.

use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	freq: u8,
	#[allow(dead_code)] // read by tests / Debug; queue membership is authoritative
	in_main: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,   // key -> arena index
	nodes: Vec<Node<K, V>>, // arena
	small: VecDeque<u32>,   // probation FIFO (front = oldest)
	main: VecDeque<u32>,    // main FIFO (front = oldest)
	ghost_q: VecDeque<K>,   // recently evicted-from-small keys, FIFO
	ghost_set: HashMap<K, ()>,
	main_ghost_q: VecDeque<K>, // recently evicted-from-main keys, FIFO
	main_ghost_set: HashMap<K, ()>,
	small_target: usize, // adaptive, within [min_small, max_small]
	min_small: usize,
	max_small: usize,
	ghost_cap: usize,
	main_ghost_cap: usize,
	capacity: usize,
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
		let initial_small = std::cmp::max(1, capacity / 10);
		let min_small = std::cmp::max(1, capacity / 100);
		let max_small = std::cmp::max(min_small, capacity / 2);
		let small_target = initial_small.clamp(min_small, max_small);
		// Small ghost sizing is unchanged from v001 (based on the initial target).
		let ghost_cap = std::cmp::max(1, capacity - initial_small);
		let main_ghost_cap = std::cmp::max(1, capacity / 2);
		log::info!(
			"LruCache.try_new: capacity={capacity} small_target={small_target} \
			 bounds=[{min_small},{max_small}] ghost_cap={ghost_cap} main_ghost_cap={main_ghost_cap}"
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			small: VecDeque::with_capacity(capacity),
			main: VecDeque::with_capacity(capacity),
			ghost_q: VecDeque::with_capacity(ghost_cap + 1),
			ghost_set: HashMap::with_capacity(ghost_cap + 1),
			main_ghost_q: VecDeque::with_capacity(main_ghost_cap + 1),
			main_ghost_set: HashMap::with_capacity(main_ghost_cap + 1),
			small_target,
			min_small,
			max_small,
			ghost_cap,
			main_ghost_cap,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, bumping its (saturating) access frequency.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.map.get(key)? as usize;
		let node = &mut self.nodes[idx];
		if node.freq < MAX_FREQ {
			node.freq += 1;
		}
		Some(&node.value)
	}

	/// Inserts or updates `key`. New keys enter `small`, or `main` on a ghost hit.
	/// Ghost hits adapt `small_target` ARC-style.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			let node = &mut self.nodes[idx as usize];
			node.value = value;
			if node.freq < MAX_FREQ {
				node.freq += 1;
			}
			return;
		}

		let small_ghost_len = self.ghost_q.len();
		let main_ghost_len = self.main_ghost_q.len();

		let to_main = if self.ghost_set.remove(&key).is_some() {
			// Evicted from probation too early: grow the probation queue.
			let delta = std::cmp::max(1, main_ghost_len / std::cmp::max(1, small_ghost_len));
			self.small_target = std::cmp::min(self.max_small, self.small_target.saturating_add(delta));
			true
		} else if self.main_ghost_set.remove(&key).is_some() {
			// Main was too small: shrink the probation queue.
			let delta = std::cmp::max(1, small_ghost_len / std::cmp::max(1, main_ghost_len));
			self.small_target = std::cmp::max(self.min_small, self.small_target.saturating_sub(delta));
			true
		} else {
			false
		};
		// Stale entries in the ghost FIFO deques are tolerated: on pop, the set
		// removal is a no-op unless the key was re-ghosted, which is benign.

		let idx = if self.map.len() >= self.capacity {
			let slot = self.evict_one();
			let node = &mut self.nodes[slot as usize];
			node.key = key.clone();
			node.value = value;
			node.freq = 0;
			node.in_main = to_main;
			slot
		} else {
			let slot = self.nodes.len() as u32;
			self.nodes.push(Node {
				key: key.clone(),
				value,
				freq: 0,
				in_main: to_main,
			});
			slot
		};

		if to_main {
			self.main.push_back(idx);
		} else {
			self.small.push_back(idx);
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// Evicts exactly one resident entry and returns its (now free) arena slot.
	/// Terminates: every iteration either lowers some node's freq or frees a slot.
	fn evict_one(&mut self) -> u32 {
		loop {
			if self.small.len() >= self.small_target || self.main.is_empty() {
				let idx = self.small.pop_front().expect("small must be non-empty when main is empty and cache is full");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq = 0;
					node.in_main = true;
					self.main.push_back(idx);
					continue;
				}
				let evicted = node.key.clone();
				self.map.remove(&evicted);
				self.push_ghost(evicted);
				return idx;
			} else {
				let idx = self.main.pop_front().expect("main checked non-empty");
				let node = &mut self.nodes[idx as usize];
				if node.freq > 0 {
					node.freq -= 1;
					self.main.push_back(idx);
					continue;
				}
				let evicted = node.key.clone();
				self.map.remove(&evicted);
				self.push_main_ghost(evicted);
				return idx;
			}
		}
	}

	fn push_ghost(&mut self, key: K) {
		self.ghost_set.insert(key.clone(), ());
		self.ghost_q.push_back(key);
		while self.ghost_q.len() > self.ghost_cap {
			if let Some(old) = self.ghost_q.pop_front() {
				self.ghost_set.remove(&old);
			}
		}
	}

	fn push_main_ghost(&mut self, key: K) {
		self.main_ghost_set.insert(key.clone(), ());
		self.main_ghost_q.push_back(key);
		while self.main_ghost_q.len() > self.main_ghost_cap {
			if let Some(old) = self.main_ghost_q.pop_front() {
				self.main_ghost_set.remove(&old);
			}
		}
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

	fn in_main(cache: &LruCache<i64, i64>, key: i64) -> bool {
		let idx = cache.map[&key] as usize;
		cache.nodes[idx].in_main
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_overwrites_existing_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn overwrite_does_not_grow_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(1, 20);
		cache.put(1, 30);
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // evicts 1 (one-hit wonder from small)
		cache.put(5, 50); // evicts 2
		cache.put(6, 60); // evicts 3

		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
		assert_eq!(*cache.get(&6).unwrap(), 60);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(cache.map.len(), 1);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11);
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn one_hit_wonders_evicted_from_small_first() {
		let mut cache = LruCache::new(10); // small_target = 1
		for k in 1..=10 {
			cache.put(k, k * 10);
		}
		cache.get(&1); // key 1 is reused; keys 2..=10 are one-hit wonders
		cache.put(11, 110); // 1 promoted to main, 2 evicted
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn accessed_small_entry_promoted_to_main() {
		let mut cache = LruCache::new(10);
		for k in 1..=10 {
			cache.put(k, k);
		}
		cache.get(&2);
		assert!(!in_main(&cache, 2));
		cache.put(11, 11); // evicts 1 (freq 0)
		assert!(!in_main(&cache, 2));
		cache.put(12, 12); // 2 has freq > 0 -> promoted to main; 3 evicted
		assert!(in_main(&cache, 2));
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 2);
	}

	#[test]
	fn ghost_hit_admitted_into_main() {
		let mut cache = LruCache::new(10);
		for k in 1..=10 {
			cache.put(k, k);
		}
		cache.put(11, 11); // evicts 1 into ghost
		assert!(cache.get(&1).is_none());
		assert!(cache.ghost_set.contains_key(&1));
		let before = cache.small_target;
		cache.put(1, 100); // ghost hit -> straight into main, small_target grows
		assert!(in_main(&cache, 1));
		assert!(!cache.ghost_set.contains_key(&1));
		assert!(cache.small_target > before);
		assert_eq!(*cache.get(&1).unwrap(), 100);
		assert!(cache.map.len() <= 10);
	}

	#[test]
	fn main_evictions_not_ghosted() {
		let mut cache = LruCache::new(1);
		cache.put(1, 1);
		cache.put(2, 2); // 1 -> small ghost
		cache.put(1, 1); // ghost hit -> main; 2 evicted from small -> small ghost
		assert!(in_main(&cache, 1));
		cache.put(3, 3); // small empty, main non-empty: evict 1 from main
		assert!(cache.get(&1).is_none());
		assert!(!cache.ghost_set.contains_key(&1));
		assert!(cache.main_ghost_set.contains_key(&1));
		assert_eq!(*cache.get(&3).unwrap(), 3);
	}

	#[test]
	fn main_ghost_hit_shrinks_small_target() {
		let mut cache = LruCache::new(100);
		cache.small_target = 20;
		cache.push_main_ghost(500);
		cache.put(500, 5);
		assert!(in_main(&cache, 500));
		assert!(!cache.main_ghost_set.contains_key(&500));
		assert_eq!(cache.small_target, 19);
	}

	#[test]
	fn adaptive_target_stays_in_bounds_under_churn() {
		for &cap in &[1usize, 2, 8, 100, 300] {
			let mut cache = LruCache::new(cap);
			for i in 0..20_000i64 {
				// Mix of a loop larger than the cache and a hot set.
				let k = if i % 3 == 0 { i % 7 } else { (i * 7919) % (cap as i64 * 3 + 5) };
				if cache.get(&k).is_none() {
					cache.put(k, k * 10);
				}
				assert!(cache.small_target >= cache.min_small);
				assert!(cache.small_target <= cache.max_small);
				assert!(cache.map.len() <= cap);
				assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
				assert!(cache.ghost_q.len() <= cache.ghost_cap);
				assert!(cache.main_ghost_q.len() <= cache.main_ghost_cap);
			}
		}
	}

	#[test]
	fn capacity_bound_under_churn() {
		let mut cache = LruCache::new(8);
		for i in 0..10_000i64 {
			let k = (i * 7919) % 37;
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
			assert!(cache.map.len() <= 8);
			assert_eq!(cache.small.len() + cache.main.len(), cache.map.len());
		}
		for k in 0..37 {
			if let Some(&v) = cache.get(&k) {
				assert_eq!(v, k * 10);
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
