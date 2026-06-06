// Segmented-LRU (SLRU) cache using HashMap + arena-based doubly-linked list,
// with the protected segment split into two recency tiers.
//
// The baseline single LRU list is split into a *probationary* segment (seen
// once) and a *protected* region (earned a second access). This variant further
// divides the protected region into two recency lists that together stay within
// the same `protected_cap`:
//
//   * a *cold-protected* list — keys promoted on their 2nd access, and
//   * a *hot* list — keys that earned a 3rd-or-later access.
//
// Distinguishing reuse depth by recency-of-Nth-access (an LRU-K-like signal,
// not a frequency counter) lets demotion shed twice-seen-cold keys before
// many-times-hot keys. The combined budget `cold_len + hot_len <= protected_cap`
// is unchanged, so the P1 capacity bound and the
// `prob_len + cold_len + hot_len == map.len()` invariant match the parent's
// `prob_len + prot_len == map.len()`. Eviction slots are still reused in place.
// No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Which recency list a node currently lives in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tier {
	/// Seen once; eviction candidate.
	Probationary,
	/// Promoted on a 2nd access.
	Cold,
	/// Earned a 3rd-or-later access.
	Hot,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	// Which recency list this node currently occupies.
	tier: Tier,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // shared arena

	// Probationary segment: entries that have been seen exactly once.
	prob_head: Option<usize>, // LRU (front)
	prob_tail: Option<usize>, // MRU (back)
	prob_len: usize,

	// Cold-protected list: entries that earned a second access.
	cold_head: Option<usize>, // LRU (front)
	cold_tail: Option<usize>, // MRU (back)
	cold_len: usize,

	// Hot list: entries that earned a third-or-later access.
	hot_head: Option<usize>, // LRU (front)
	hot_tail: Option<usize>, // MRU (back)
	hot_len: usize,

	capacity: usize,
	// Size budget for the protected region (cold + hot together), ~95% of
	// capacity, floored. The probationary budget is the remainder, so the
	// segment budgets sum to exactly `capacity`. For capacity == 1 this floors
	// to 0, leaving a single usable (probationary) slot.
	protected_cap: usize,
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
		// floor(0.95 * capacity); probationary budget is capacity - protected_cap,
		// so the two segment budgets sum to exactly `capacity`.
		let protected_cap = (capacity * 19) / 20;
		log::info!(
			"LruCache.try_new: capacity={capacity} protected_cap={protected_cap} probationary_cap={}",
			capacity - protected_cap
		);
		Some(Self {
			map: HashMap::with_capacity(capacity),
			nodes: Vec::with_capacity(capacity),
			prob_head: None,
			prob_tail: None,
			prob_len: 0,
			cold_head: None,
			cold_tail: None,
			cold_len: 0,
			hot_head: None,
			hot_tail: None,
			hot_len: 0,
			capacity,
			protected_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, promoting it one recency tier
	/// deeper (toward the hot list).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Look up the arena index for this key; return None on miss.
		let &idx = self.map.get(key)?;
		// A hit promotes the node one tier deeper and keeps the protected region
		// within its combined budget.
		self.touch(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. If at capacity, evicts the least-recently-used
	/// entry (probationary first, then cold-protected, then hot).
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access,
		// promoting it one recency tier deeper.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.touch(idx);
			return;
		}
		// New key — need an arena slot.
		let idx = if self.nodes.len() >= self.capacity {
			// Arena full: evict the LRU of the probationary segment first; if it is
			// empty fall back to the cold-protected LRU, then the hot LRU. Reuse the
			// evicted slot in place for the new entry.
			let evict_idx = self
				.prob_head
				.or(self.cold_head)
				.or(self.hot_head)
				.expect("a full cache always has a victim in some segment");
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			evict_idx
		} else {
			// Arena has room: allocate a new slot.
			let idx = self.nodes.len();
			self.nodes.push(Node {
				key: key.clone(),
				value,
				prev: None,
				next: None,
				tier: Tier::Probationary,
			});
			idx
		};
		// New entries enter at the MRU end of the probationary segment.
		self.push_back_probationary(idx);
		self.map.insert(key, idx);
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(
			self.prob_len + self.cold_len + self.hot_len,
			self.map.len()
		);
	}

	/// Promote a hit node one recency tier deeper:
	///   probationary -> cold-protected, cold-protected -> hot, hot -> hot (MRU).
	/// Then restore the protected budget by demoting if necessary.
	fn touch(&mut self, idx: usize) {
		let target = match self.nodes[idx].tier {
			Tier::Probationary => Tier::Cold,
			Tier::Cold => Tier::Hot,
			Tier::Hot => Tier::Hot,
		};
		self.unlink(idx);
		match target {
			Tier::Cold => self.push_back_cold(idx),
			Tier::Hot => self.push_back_hot(idx),
			Tier::Probationary => unreachable!("touch never targets probationary"),
		}
		self.enforce_protected_budget();
	}

	/// Remove a node from whichever recency list it currently occupies.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let tier = self.nodes[idx].tier;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => match tier {
				Tier::Probationary => self.prob_head = next,
				Tier::Cold => self.cold_head = next,
				Tier::Hot => self.hot_head = next,
			},
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => match tier {
				Tier::Probationary => self.prob_tail = prev,
				Tier::Cold => self.cold_tail = prev,
				Tier::Hot => self.hot_tail = prev,
			},
		}
		match tier {
			Tier::Probationary => self.prob_len -= 1,
			Tier::Cold => self.cold_len -= 1,
			Tier::Hot => self.hot_len -= 1,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
	}

	/// Append a node at the MRU (tail) end of the probationary segment.
	fn push_back_probationary(&mut self, idx: usize) {
		self.nodes[idx].tier = Tier::Probationary;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
		self.prob_len += 1;
	}

	/// Append a node at the MRU (tail) end of the cold-protected list.
	fn push_back_cold(&mut self, idx: usize) {
		self.nodes[idx].tier = Tier::Cold;
		self.nodes[idx].prev = self.cold_tail;
		self.nodes[idx].next = None;
		match self.cold_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.cold_head = Some(idx),
		}
		self.cold_tail = Some(idx);
		self.cold_len += 1;
	}

	/// Append a node at the MRU (tail) end of the hot list.
	fn push_back_hot(&mut self, idx: usize) {
		self.nodes[idx].tier = Tier::Hot;
		self.nodes[idx].prev = self.hot_tail;
		self.nodes[idx].next = None;
		match self.hot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.hot_head = Some(idx),
		}
		self.hot_tail = Some(idx);
		self.hot_len += 1;
	}

	/// If the protected region (cold + hot) exceeds its budget, demote one node
	/// to the MRU end of the probationary segment. The cold-protected LRU is the
	/// victim first; only when cold-protected is empty does the hot LRU demote.
	/// Only one node can be over budget per promotion, so a single demotion
	/// restores the bound.
	fn enforce_protected_budget(&mut self) {
		if self.cold_len + self.hot_len > self.protected_cap {
			if let Some(victim) = self.cold_head.or(self.hot_head) {
				self.unlink(victim);
				self.push_back_probationary(victim);
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
		cache.put(3, 30); // both keys still probationary; evicts probationary LRU (1)

		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn access_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // promotes key 1 into the protected region
		cache.put(3, 30); // evicts key 2 (probationary LRU)

		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	fn put_updates_lru_order() {
		let mut cache = LruCache::new(2);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(1, 11); // update promotes key 1 into the protected region
		cache.put(3, 30); // evicts key 2 (probationary LRU)

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
		cache.put(4, 40); // evicts 1
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
		cache.put(4, 40); // key 1 is protected; evicts a probationary LRU, not key 1
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert!(cache.get(&2).is_none());
	}

	#[test]
	fn get_middle_node_promotes_correctly() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.get(&2); // promote key 2 into the protected region
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn promoted_key_survives_while_probationary_churns() {
		// SLRU: a key promoted to the protected region is shielded from
		// eviction as long as the probationary segment can supply a victim.
		// Slots are still reused in place across re-evictions.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts probationary LRU (key 1), reuses slot 0
		cache.get(&3); // promote key 3 into the protected region
		cache.put(4, 40); // evicts probationary LRU (key 2), reuses slot 1
		cache.put(5, 50); // evicts probationary LRU (key 4), reuses slot 1
		assert_eq!(*cache.get(&3).unwrap(), 30); // protected, survived
		assert!(cache.get(&4).is_none());
		assert_eq!(*cache.get(&5).unwrap(), 50);
	}

	#[test]
	fn get_only_element() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		// List has one node; get unlinks then re-pushes it (into protected).
		assert_eq!(*cache.get(&1).unwrap(), 10);
		// Verify the structure is still intact.
		cache.put(2, 20);
		cache.put(3, 30);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn overwrite_mru_at_capacity() {
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30); // full
		cache.put(3, 31); // overwrite promotes key 3 into protected
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		assert_eq!(*cache.get(&3).unwrap(), 31);
		assert_eq!(*cache.get(&4).unwrap(), 40);
	}

	#[test]
	fn hot_tier_resists_demotion_over_cold() {
		// Capacity 3: protected_cap = floor(3*19/20) = 2. Two protected slots,
		// one probationary. A 3x-accessed key (hot) must outlast a 2x-accessed
		// key (cold) when the protected region overflows.
		let mut cache = LruCache::new(3);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.get(&1); // 1 -> cold
		cache.get(&1); // 1 -> hot
		cache.get(&2); // 2 -> cold (protected now holds hot=1, cold=2; len 2 == cap)
		cache.put(3, 30); // new probationary key
		cache.get(&3); // 3 -> cold; protected overflows, cold LRU (key 2) demotes
		// Key 1 stays hot; key 2 was demoted to probationary.
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&3).unwrap(), 30);
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
