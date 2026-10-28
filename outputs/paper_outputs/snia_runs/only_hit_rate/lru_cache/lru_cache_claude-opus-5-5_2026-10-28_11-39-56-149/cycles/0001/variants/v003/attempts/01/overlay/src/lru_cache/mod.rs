// O(1) LRU cache using HashMap + arena-based doubly-linked list, gated by a
// TinyLFU admission filter. Nodes are stored in a Vec (arena) indexed by usize.
// On eviction the slot is reused in-place, so no free-list management is needed.
// When the arena is full, a new key replaces the LRU victim only if its
// estimated frequency (count-min sketch, see `sketch.rs`) is strictly higher;
// otherwise the new key is rejected and the victim stays resident.
// No unsafe, no external crates.

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod lru_cache_jni;
mod sketch;

use sketch::FrequencySketch;

/// Tie-break: a candidate this warm may still be admitted (rarely) when the
/// victim is at least as frequent, to avoid adversarial lock-out.
const TIE_BREAK_MIN_FREQ: u8 = 6;
/// Admit with probability 1/128 on the tie-break path.
const TIE_BREAK_MASK: u64 = 127;

#[inline]
fn hash_key<K: Hash>(key: &K) -> u64 {
	let mut s = DefaultHasher::new();
	key.hash(&mut s);
	s.finish()
}

#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // LRU (front)
	tail: Option<usize>,    // MRU (back)
	capacity: usize,
	sketch: FrequencySketch,
	rng: u64, // xorshift state for the tie-break
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
			sketch: FrequencySketch::new(capacity),
			rng: 0x2545_f491_4f6c_dd1d,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it to most-recently-used.
	/// Every call (hit or miss) records an access in the frequency sketch.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.sketch.increment(hash_key(key));
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Promote to MRU: detach from current position, re-attach at tail
		self.unlink(idx);
		self.push_back(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, the new key replaces the LRU
	/// entry only if the admission filter accepts it; otherwise it is dropped.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and promote to MRU
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.unlink(idx);
			self.push_back(idx);
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			debug_assert!(self.head.is_some(), "head must be Some when arena is full");
			let head_idx = self.head.expect("non-empty list has a head");
			// Admission: candidate must be strictly more frequent than the victim.
			let cand_freq = self.sketch.frequency(hash_key(&key));
			let victim_freq = self.sketch.frequency(hash_key(&self.nodes[head_idx].key));
			if !self.admit(cand_freq, victim_freq) {
				return; // rejected: victim stays resident, map/list unchanged
			}
			// Evict LRU (head) and reuse its slot for the new entry
			let evicted_key = self.nodes[head_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(head_idx);
			self.nodes[head_idx].key = key.clone();
			self.nodes[head_idx].value = value;
			head_idx
		} else {
			// Arena has room: allocate a new slot (no filtering during warm-up)
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
			});
			idx
		};
		// Link the new/reused slot at the tail (MRU) and register in the map
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	#[inline]
	fn admit(&mut self, cand_freq: u8, victim_freq: u8) -> bool {
		if cand_freq > victim_freq {
			return true;
		}
		if cand_freq >= TIE_BREAK_MIN_FREQ {
			// victim_freq >= cand_freq here
			return self.next_random() & TIE_BREAK_MASK == 0;
		}
		false
	}

	#[inline]
	fn next_random(&mut self) -> u64 {
		let mut x = self.rng;
		x ^= x << 13;
		x ^= x >> 7;
		x ^= x << 17;
		self.rng = x;
		x
	}

	/// Remove node from its current position in the list.
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

	/// Append node to the tail (MRU end) of the list.
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

#[cfg(test)]
mod tests {
	use super::*;

	/// Record `n` accesses (misses) for `key` so it out-ranks cold victims.
	fn warm(cache: &mut LruCache<i64, i64>, key: i64, n: usize) {
		for _ in 0..n {
			assert!(cache.get(&key).is_none());
		}
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
		// Should still have room for a second key without eviction
		cache.put(2, 40);
		assert_eq!(*cache.get(&1).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 40);
	}

	#[test]
	fn eviction_removes_lru() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		warm(&mut cache, 3, 1);
		cache.put(3, 30); // freq(3)=1 > freq(1)=0: evicts key 1

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn cold_candidate_is_rejected() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // freq(3)=0 is not > freq(1)=0: rejected

		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn frequent_victim_survives_less_frequent_candidate() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		for _ in 0..3 {
			cache.get(&1);
		}
		cache.get(&2); // order: 1 (LRU), 2 (MRU); freq(1)=3
		warm(&mut cache, 3, 2);
		cache.put(3, 30); // freq(3)=2 is not > 3: rejected
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // makes key 1 most recent
		warm(&mut cache, 3, 1);
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update key 1 makes it MRU
		warm(&mut cache, 3, 1);
		cache.put(3, 30); // evicts key 2 (now LRU)

		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn eviction_chain() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		warm(&mut cache, 4, 1);
		cache.put(4, 40); // evicts 1
		warm(&mut cache, 5, 1);
		cache.put(5, 50); // evicts 2
		warm(&mut cache, 6, 1);
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
		assert_eq!(*cache.get(&1).unwrap(), 10); // freq(1)=1
		warm(&mut cache, 2, 4);
		cache.put(2, 20); // freq(2)=4 > 1: evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn overwrite_at_full_capacity_does_not_evict() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20); // cache full
		cache.put(1, 11); // overwrite, must NOT evict key 2
		assert_eq!(*cache.get(&1).unwrap(), 11);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn repeated_get_preserves_eviction_order() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		for _ in 0..10 {
			cache.get(&1);
		}
		warm(&mut cache, 4, 1);
		cache.put(4, 40); // should evict key 2 (LRU), not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10); // LRU
		cache.put(2, 20); // middle
		cache.put(3, 30); // MRU
		cache.get(&2); // unlink middle, push to MRU
		// Order: 1 (LRU), 3, 2 (MRU)
		warm(&mut cache, 4, 1);
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn slot_reuse_after_promotion_and_re_eviction() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		warm(&mut cache, 3, 1);
		cache.put(3, 30); // evicts 1 -> slot 0 reused for key 3
		cache.get(&3); // promote slot 0 (key 3) to MRU; freq(3)=2
		warm(&mut cache, 4, 1);
		cache.put(4, 40); // evicts 2 (slot 1) -> slot 1 reused for key 4
		warm(&mut cache, 5, 3);
		cache.put(5, 50); // freq(5)=3 > freq(3)=2: evicts 3 (slot 0) again
		assert!(cache.get(&3).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify list is still intact
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full; 3 is MRU (tail)
		cache.put(3, 31); // overwrite MRU
		// Eviction should take 1 (LRU)
		warm(&mut cache, 4, 1);
		cache.put(4, 40); // evicts 1
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
