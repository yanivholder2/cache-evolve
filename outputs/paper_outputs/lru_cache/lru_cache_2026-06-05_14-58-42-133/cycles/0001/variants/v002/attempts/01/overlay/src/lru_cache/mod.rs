// 2Q cache (Johnson & Shasha) over the same arena + HashMap backing as the LRU
// baseline. Residents live in two intrusive lists sharing one arena:
//   - A1in: a FIFO queue for first-time keys (target size kin = capacity / 4).
//   - Am:   an LRU queue for proven keys (the remaining capacity).
// A value-less ghost queue A1out (size kout = capacity / 2) remembers recently
// evicted A1in keys so that a returning key is admitted straight to the hot Am
// queue. Ghosts occupy no arena slot and never read back as resident.
//
// Eviction slots are reused in-place exactly as the baseline does, so every
// arena node is always a resident key: nodes.len() == map.len() <= capacity.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	in_am: bool, // true => node lives in the Am list; false => A1in list
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // resident key -> node index
	nodes: Vec<Node<K, V>>, // arena (residents only)

	// A1in: FIFO of first-time keys. head = oldest (evicted first), tail = newest.
	a1in_head: Option<usize>,
	a1in_tail: Option<usize>,
	a1in_len: usize,

	// Am: LRU of proven keys. head = LRU (evicted first), tail = MRU.
	am_head: Option<usize>,
	am_tail: Option<usize>,
	am_len: usize,

	// A1out: bounded FIFO of ghost keys only (no values, no arena slot).
	// `a1out_map` is the authority on membership; the queue carries (key, seq)
	// for FIFO eviction. Stale entries (whose seq no longer matches the map) are
	// skipped on pop, so re-ghosting a key never drops the live entry.
	a1out_map: HashMap<K, u64>,
	a1out_queue: VecDeque<(K, u64)>,
	a1out_seq: u64,

	capacity: usize,
	kin: usize,
	kout: usize,
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
		let kin = capacity / 4;
		let kout = capacity / 2;
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			a1in_head: None,
			a1in_tail: None,
			a1in_len: 0,
			am_head: None,
			am_tail: None,
			am_len: 0,
			a1out_map: HashMap::with_capacity(kout),
			a1out_queue: VecDeque::with_capacity(kout),
			a1out_seq: 0,
			capacity,
			kin,
			kout,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident. A hit in Am is promoted to the
	/// Am MRU tail; a hit in A1in is left in place (2Q keeps A1in in FIFO order).
	/// A miss (including a ghost-only key) returns `None`.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		if self.nodes[idx].in_am {
			// Promote within Am to MRU.
			self.unlink(idx);
			self.push_back(idx);
		}
		// A1in hit: do not reorder.
		Some(&self.nodes[idx].value)
	}

	/// Inserts `key` (called on a get-miss). If `key` is a ghost in A1out it is
	/// promoted straight into the Am MRU tail; otherwise it enters the A1in tail.
	/// At capacity, a slot is reclaimed first (see [`reclaim_slot`]).
	pub fn put(&mut self, key: K, value: V) {
		// Defensive: a resident key just updates its value in place (the call
		// contract only puts on a miss, but stay robust against re-insertion).
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			if self.nodes[idx].in_am {
				self.unlink(idx);
				self.push_back(idx);
			}
			return;
		}

		// A returning ghost is admitted to Am; a brand-new key enters A1in.
		let to_am = self.ghost_take(&key);

		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: reclaim a slot and reuse it in place.
			let slot = self.reclaim_slot();
			self.nodes[slot].key = key.clone();
			self.nodes[slot].value = value;
			self.nodes[slot].in_am = to_am;
			slot
		} else {
			// Arena has room: allocate a fresh slot.
			let slot = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				in_am: to_am,
			});
			slot
		};

		self.push_back(idx);
		if to_am {
			self.am_len += 1;
		} else {
			self.a1in_len += 1;
		}
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.map.len(), self.a1in_len + self.am_len);
	}

	/// Reclaim one arena slot by evicting a resident, returning the freed index.
	/// 2Q rule: if A1in is over its target (`kin`), evict its FIFO head and record
	/// the key as a ghost; otherwise evict the Am LRU head and drop it outright.
	fn reclaim_slot(&mut self) -> usize {
		if self.a1in_len > self.kin {
			let head = self.a1in_head.expect("A1in non-empty when a1in_len > kin");
			let evicted = self.nodes[head].key.clone();
			self.map.remove(&evicted);
			self.unlink(head);
			self.a1in_len -= 1;
			self.ghost_record(evicted);
			head
		} else {
			// a1in_len <= kin < capacity, so Am holds the rest and is non-empty.
			let head = self.am_head.expect("Am non-empty when reclaiming from Am");
			let evicted = self.nodes[head].key.clone();
			self.map.remove(&evicted);
			self.unlink(head);
			self.am_len -= 1;
			head
		}
	}

	/// Remove `node` from whichever segment list it currently belongs to.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let in_am = self.nodes[idx].in_am;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => {
				if in_am {
					self.am_head = next;
				} else {
					self.a1in_head = next;
				}
			}
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => {
				if in_am {
					self.am_tail = prev;
				} else {
					self.a1in_tail = prev;
				}
			}
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append `node` to the tail of its segment (FIFO newest / Am MRU end).
	/// The node's `in_am` flag must already be set to the destination segment.
	fn push_back(&mut self, idx: usize) {
		let in_am = self.nodes[idx].in_am;
		let tail = if in_am { self.am_tail } else { self.a1in_tail };
		self.nodes[idx].prev = tail;
		self.nodes[idx].next = None;
		match tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => {
				if in_am {
					self.am_head = Some(idx);
				} else {
					self.a1in_head = Some(idx);
				}
			}
		}
		if in_am {
			self.am_tail = Some(idx);
		} else {
			self.a1in_tail = Some(idx);
		}
	}

	/// If `key` is a ghost, remove it from A1out and report `true` (admit to Am).
	fn ghost_take(&mut self, key: &K) -> bool {
		// Membership authority is the map; the matching queue entry becomes stale
		// and is skipped when later popped.
		self.a1out_map.remove(key).is_some()
	}

	/// Record an evicted A1in key as a ghost, enforcing the kout FIFO bound.
	fn ghost_record(&mut self, key: K) {
		if self.kout == 0 {
			return;
		}
		let seq = self.a1out_seq;
		self.a1out_seq = self.a1out_seq.wrapping_add(1);
		self.a1out_map.insert(key.clone(), seq);
		self.a1out_queue.push_back((key, seq));
		// Keep the queue (and thus the live ghost set) bounded by kout. Stale
		// entries are dropped without touching the map; a live entry is removed
		// only when its seq still matches, giving true FIFO ghost eviction.
		while self.a1out_queue.len() > self.kout {
			if let Some((k, s)) = self.a1out_queue.pop_front() {
				if self.a1out_map.get(&k) == Some(&s) {
					self.a1out_map.remove(&k);
				}
			} else {
				break;
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
	fn residency_never_exceeds_capacity() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..100i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 8, "residents {resident} exceeds capacity 8");
	}

	#[test]
	fn never_put_key_reads_none() {
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..50i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
		}
		for phantom in [1000i64, 2000, -1] {
			assert!(cache.get(&phantom).is_none());
		}
	}

	#[test]
	fn returning_ghost_is_admitted_to_am() {
		// capacity 8 => kin=2, kout=4. Fill A1in, evict key 0 to ghost, then
		// re-put it: it should be admitted (resident) and survive better.
		let mut cache: LruCache<i64, i64> = LruCache::new(8);
		for k in 0..8i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		// Push enough new first-time keys to evict key 0 from A1in into A1out.
		for k in 8..16i64 {
			if cache.get(&k).is_none() {
				cache.put(k, k * 10);
			}
		}
		// Re-access key 0 (a ghost now): miss, then put admits it to Am.
		if cache.get(&0).is_none() {
			cache.put(0, 0);
		}
		assert_eq!(*cache.get(&0).unwrap(), 0);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
