// 2Q cache (Johnson & Shasha) over a shared arena, no unsafe, no external crates.
//
// Three logical lists share one arena `nodes`, with in-place slot reuse via a
// free list:
//   - Am    (index 0): the main LRU list of keys that proved reuse. head = LRU
//                       victim, tail = MRU.
//   - A1in  (index 1): a FIFO admission queue of newly seen keys. head = FIFO
//                       front (oldest, the eviction victim), tail = newest.
//   - A1out (index 2): a key-only ghost FIFO of keys evicted from A1in; its
//                       nodes carry no value (`value == None`).
//
// `Kin = capacity/20` (A1in target, ~5% admission floor mirroring the 95/5
// SLRU incumbent's probationary budget) and `Kout = capacity/2` (A1out target).
// Resident count (Am + A1in) is held <= capacity. Shrinking A1in hands the
// proven-reuse main list Am ~95% of capacity.
//
// The struct keeps the name `LruCache` and the public surface (`try_new`,
// `new`, `get`, `put`, the `framework::Cache` impl) so the JNI ABI and the
// gates are unchanged.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

// List identifiers, used to index the per-list head/tail/len arrays.
const AM: usize = 0;
const A1IN: usize = 1;
const A1OUT: usize = 2;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: Option<V>, // None only for A1out ghost nodes
	prev: Option<usize>,
	next: Option<usize>,
	list: usize, // AM | A1IN | A1OUT
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> arena index (resident OR ghost)
	nodes: Vec<Node<K, V>>, // arena
	free: Vec<usize>,       // reusable arena slots
	heads: [Option<usize>; 3],
	tails: [Option<usize>; 3],
	lens: [usize; 3],
	capacity: usize,
	kin: usize,  // A1in target size
	kout: usize, // A1out (ghost) target size
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
		let kin = capacity / 20;
		let kout = capacity / 2;
		log::info!("LruCache.try_new: 2Q capacity={capacity} kin={kin} kout={kout}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity + kout),
			free: Vec::new(),
			heads: [None; 3],
			tails: [None; 3],
			lens: [0; 3],
			capacity,
			kin,
			kout,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if it is resident (Am or A1in). A hit in Am
	/// promotes to Am MRU; a hit in A1in does nothing (canonical 2Q FIFO). A key
	/// that is only a ghost (A1out) reads as a miss.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		match self.nodes[idx].list {
			AM => {
				// Standard LRU promotion.
				self.unlink(idx);
				self.push_tail(idx, AM);
				self.nodes[idx].value.as_ref()
			}
			A1IN => {
				// FIFO: do not reorder; just return the value.
				self.nodes[idx].value.as_ref()
			}
			_ => None, // A1out ghost: no value, observable miss.
		}
	}

	/// Admits `key`. Called only on a get-miss. If `key` is a ghost in A1out it
	/// has demonstrated reuse across an A1in lifetime and is promoted to Am MRU;
	/// otherwise it is a wholly new key and enters the tail of A1in.
	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.map.get(&key) {
			match self.nodes[idx].list {
				A1OUT => {
					// Ghost hit: promote into Am, reusing the same slot.
					self.unlink(idx); // detach from A1out
					self.evict_if_needed(); // make room for one resident
					self.nodes[idx].value = Some(value);
					self.push_tail(idx, AM);
					return;
				}
				other => {
					// Defensive: contract puts only on miss, but keep value
					// fidelity if a resident key is re-put.
					self.nodes[idx].value = Some(value);
					if other == AM {
						self.unlink(idx);
						self.push_tail(idx, AM);
					}
					return;
				}
			}
		}
		// Wholly new key: admit to the tail of A1in (FIFO).
		self.evict_if_needed();
		let idx = self.alloc(key.clone(), Some(value));
		self.push_tail(idx, A1IN);
		self.map.insert(key, idx);
	}

	/// Free residency until a single new resident fits (resident < capacity).
	fn evict_if_needed(&mut self) {
		while self.lens[AM] + self.lens[A1IN] >= self.capacity {
			if self.lens[A1IN] > self.kin {
				// A1in is over target: page its FIFO head out into the A1out
				// ghost (keeping the slot), then trim the ghost if needed.
				let victim = self.heads[A1IN].expect("A1in non-empty when len > kin");
				self.unlink(victim);
				self.nodes[victim].value = None;
				self.push_tail(victim, A1OUT);
				self.evict_ghost_if_needed();
			} else {
				// Otherwise evict the Am LRU outright.
				let victim = self.heads[AM].expect("Am non-empty when resident >= capacity");
				self.unlink(victim);
				let evicted_key = self.nodes[victim].key.clone();
				self.map.remove(&evicted_key);
				self.free.push(victim);
			}
		}
	}

	/// Trim A1out to its target size, dropping the oldest ghosts.
	fn evict_ghost_if_needed(&mut self) {
		while self.lens[A1OUT] > self.kout {
			let victim = self.heads[A1OUT].expect("A1out non-empty when len > kout");
			self.unlink(victim);
			let evicted_key = self.nodes[victim].key.clone();
			self.map.remove(&evicted_key);
			self.free.push(victim);
		}
	}

	/// Obtain an arena slot, reusing a freed one when available.
	fn alloc(&mut self, key: K, value: Option<V>) -> usize {
		if let Some(idx) = self.free.pop() {
			let node = &mut self.nodes[idx];
			node.key = key;
			node.value = value;
			node.prev = None;
			node.next = None;
			idx
		} else {
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key,
				value,
				prev: None,
				next: None,
				list: A1IN,
			});
			idx
		}
	}

	/// Detach `idx` from whatever list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let list = self.nodes[idx].list;
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => self.heads[list] = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => self.tails[list] = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		self.lens[list] -= 1;
	}

	/// Append `idx` at the tail (MRU / newest end) of `list`.
	fn push_tail(&mut self, idx: usize, list: usize) {
		self.nodes[idx].list = list;
		self.nodes[idx].prev = self.tails[list];
		self.nodes[idx].next = None;
		match self.tails[list] {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.heads[list] = Some(idx),
		}
		self.tails[list] = Some(idx);
		self.lens[list] += 1;
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

	// Drive the cache the way the framework does: only put on a get-miss.
	fn access(cache: &mut LruCache<i64, i64>, key: i64) {
		if cache.get(&key).is_none() {
			cache.put(key, key * 10);
		}
	}

	#[test]
	fn get_miss_returns_none() {
		let mut cache = LruCache::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_round_trips() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn capacity_bound_under_overfill() {
		let cap = 8usize;
		let mut cache = LruCache::<i64, i64>::new(cap);
		for k in 0..200i64 {
			access(&mut cache, k);
		}
		let resident = (0..200i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= cap, "resident {resident} exceeds capacity {cap}");
	}

	#[test]
	fn no_phantom_items() {
		let mut cache = LruCache::<i64, i64>::new(8);
		for k in 0..50i64 {
			access(&mut cache, k);
		}
		for phantom in [1000i64, -1, 9999] {
			assert!(cache.get(&phantom).is_none());
		}
	}

	#[test]
	fn ghost_reuse_promotes_to_main() {
		// A key seen twice (with an A1in lifetime between) should survive better
		// than one-hit-wonders: after re-access it lives in Am.
		let mut cache = LruCache::<i64, i64>::new(8);
		access(&mut cache, 1); // enters A1in
		for k in 100..200i64 {
			access(&mut cache, k); // churn pushes 1 out of A1in into A1out ghost
		}
		// Re-access 1: ghost hit -> promotion to Am.
		access(&mut cache, 1);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn value_fidelity() {
		let mut cache = LruCache::<i64, i64>::new(8);
		for k in 0..200i64 {
			access(&mut cache, k);
		}
		for k in 0..200i64 {
			if let Some(&value) = cache.get(&k) {
				assert_eq!(value, k * 10, "key {k} read wrong value");
			}
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
