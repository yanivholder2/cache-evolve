// CLOCK / second-chance cache using HashMap + arena-based doubly-linked list.
//
// This is a refinement of the O(1) LRU baseline (H03): instead of promoting a
// node on every `get` hit by unlinking and re-pushing it (strict recency), each
// node carries a `referenced` bit. A hit merely sets that bit. Eviction runs a
// CLOCK-style second-chance scan from the head: a node whose bit is set is
// given a second chance (bit cleared, node moved to the tail) and skipped; the
// first node whose bit is already clear is evicted. Recency is thus approximated
// by the reference bit plus list order rather than by moving on every access.
//
// Slots are still reused in place on eviction, so there is no free-list. No
// unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// Second-chance bit: set on access, cleared by the CLOCK hand.
	referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // arena
	head: Option<usize>,    // CLOCK hand start (front)
	tail: Option<usize>,    // insertion end (back)
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
		log::info!("LruCache.try_new: capacity={capacity}");
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			head: None,
			tail: None,
			capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, setting its reference bit. Unlike
	/// the LRU baseline this does *not* move the node — recency is approximated
	/// by the bit plus list order.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss
		let &idx = self.map.get(key)?;
		// Second chance: mark referenced, but leave list position untouched.
		self.nodes[idx].referenced = true;
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts a victim chosen by the
	/// CLOCK second-chance scan.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and give it a second chance.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.nodes[idx].referenced = true;
			return;
		}
		// New key — need an arena slot
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: select a victim via the CLOCK hand and reuse its slot.
			let victim_idx = self.clock_victim();
			let evicted_key = self.nodes[victim_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(victim_idx);
			self.nodes[victim_idx].key = key.clone();
			self.nodes[victim_idx].value = value;
			self.nodes[victim_idx].referenced = false;
			victim_idx
		} else {
			// Arena has room: allocate a new slot
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
		// Link the new/reused slot at the tail and register in the map.
		self.push_back(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
	}

	/// CLOCK second-chance scan. Advances the hand from the head: a referenced
	/// node has its bit cleared and is moved to the tail (second chance); the
	/// first node with a clear bit is returned as the victim. Each move clears
	/// one bit, so after at most `capacity` moves a victim is guaranteed —
	/// the scan terminates having visited at most `2 * capacity` nodes.
	fn clock_victim(&mut self) -> usize {
		loop {
			let head_idx = self.head.expect("non-empty list has a head");
			if self.nodes[head_idx].referenced {
				// Grant a second chance: clear the bit and recycle to the tail.
				self.nodes[head_idx].referenced = false;
				self.unlink(head_idx);
				self.push_back(head_idx);
			} else {
				return head_idx;
			}
		}
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

	/// Append node to the tail (insertion end) of the list.
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
	fn capacity_bound_holds_under_inserts() {
		// Insert-on-miss many distinct keys; never exceed capacity.
		let mut cache = LruCache::new(3);
		for key in 0i64..50 {
			if cache.get(&key).is_none() {
				cache.put(key, key * 10);
			}
		}
		let resident = (0i64..50).filter(|k| cache.get(k).is_some()).count();
		assert!(resident <= 3, "resident={resident}");
	}

	#[test]
	fn unreferenced_key_is_evicted_first() {
		// Fill to capacity; key 1 is never touched again, so with all bits
		// initially clear the CLOCK hand evicts from the head (key 1).
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40); // head (key 1) bit clear -> evicted
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn referenced_key_survives_one_pass() {
		// Touch key 1 so its bit is set; the first eviction should grant it a
		// second chance and evict an unreferenced node instead.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1); // set reference bit on key 1
		cache.put(4, 40); // hand: key1 referenced -> spare+move; key2 clear -> evict
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn all_referenced_still_evicts_exactly_one() {
		// Every resident key referenced: a full pass clears all bits and the
		// first-recycled node becomes the victim — termination guaranteed.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&1);
		cache.get(&2);
		cache.get(&3);
		cache.put(4, 40); // must terminate, freeing exactly one slot
		let resident = (1i64..=4).filter(|k| cache.get(k).is_some()).count();
		assert_eq!(resident, 3);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn capacity_one() {
		let mut cache = LruCache::new(1);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20); // evicts 1 (its bit is clear)
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
	fn slot_reuse_across_evictions() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0
		cache.put(2, 20); // slot 1
		cache.put(3, 30); // evicts 1 -> slot 0 reused
		cache.put(4, 40); // evicts 2 -> slot 1 reused
		assert!(cache.get(&1).is_none());
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
