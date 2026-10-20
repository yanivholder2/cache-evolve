// W-TinyLFU-style admission window in front of a 90/10 SLRU.
//
// The cache is partitioned into three arena-backed LRU lists that share one
// `nodes` arena and one `map`:
//
//   * a small *admission window* (`window_cap == capacity / 64`) where every
//     brand-new key lands first,
//   * a *probationary* main segment (keys that earned a second touch), and
//   * a *protected* main segment (keys that earned a hit while in main).
//
// New keys must earn a hit inside the window before they are admitted into the
// main SLRU; a burst of never-reused keys dies in the small window without ever
// displacing a protected key. This is scan resistance: one-hit-wonders are
// filtered before the SLRU machinery ever sees them. The 90% protected split is
// applied to `main_cap` (= capacity - window_cap), not to the whole capacity.
//
// For tiny capacities `window_cap` floors to 0, so the window is inert and the
// policy degrades gracefully to the parent's plain 90/10 SLRU (this is the case
// exercised by the invariant suite at capacity 8).
//
// Capacities still sum to exactly `capacity` (window_cap + main_cap), so the P1
// capacity bound is identical to the baseline. Eviction reuses slots in place;
// because window->main promotion decouples an eviction from its replacement
// insert, a small recycle list hands those freed slots back to the next insert
// instead of growing the arena. No unsafe, no external crates.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

/// Which of the three LRU lists a node currently occupies.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Region {
	Window,
	Probationary,
	Protected,
}

#[derive(Debug)]
struct Node<K, V> {
	key: K,
	value: V,
	prev: Option<usize>,
	next: Option<usize>,
	region: Region,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, usize>, // key -> node index
	nodes: Vec<Node<K, V>>, // shared arena

	// Admission window: brand-new keys land here and must earn a hit to leave.
	win_head: Option<usize>, // LRU (front)
	win_tail: Option<usize>, // MRU (back)
	win_len: usize,

	// Probationary main segment: keys promoted out of the window (second touch).
	prob_head: Option<usize>, // LRU (front)
	prob_tail: Option<usize>, // MRU (back)
	prob_len: usize,

	// Protected main segment: keys that earned a hit while resident in main.
	prot_head: Option<usize>, // LRU (front)
	prot_tail: Option<usize>, // MRU (back)
	prot_len: usize,

	// Slots freed by a window->main promotion that evicted a main victim. The
	// next insert reuses one of these before extending the arena, so the arena
	// never exceeds `capacity` slots.
	free_slots: Vec<usize>,

	capacity: usize,
	// Window budget: capacity / 64, floored. 0 for capacities < 64, where the
	// window is inert and the policy is plain SLRU.
	window_cap: usize,
	// Main budget: capacity - window_cap. The two sum to exactly `capacity`.
	main_cap: usize,
	// Protected budget within main (~90% of main_cap, floored). The probationary
	// budget is the remainder of main_cap.
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
		// A small admission window in front of the SLRU. Floors to 0 below 64,
		// degrading to plain SLRU. main_cap takes the remainder so the two sum
		// to exactly `capacity`.
		let window_cap = capacity / 64;
		let main_cap = capacity - window_cap;
		// floor(0.9 * main_cap); probationary budget within main is the rest.
		let protected_cap = (main_cap * 9) / 10;
		log::info!(
			"LruCache.try_new: capacity={capacity} window_cap={window_cap} main_cap={main_cap} protected_cap={protected_cap} probationary_cap={}",
			main_cap - protected_cap
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
			free_slots: Vec::new(),
			capacity,
			window_cap,
			main_cap,
			protected_cap,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if present, recording the access (window keys
	/// are promoted into main; main keys are promoted into the protected
	/// segment).
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		self.record_access(idx);
		Some(&self.nodes[idx].value)
	}

	/// Inserts or updates `key`. A brand-new key enters the admission window
	/// (or, when the window is inert, the main probationary segment). An
	/// existing key is updated in place and treated as an access.
	pub fn put(&mut self, key: K, value: V) {
		// Key already exists: update value in-place and treat as an access. The
		// JNI contract only ever puts non-resident keys, so this is the defensive
		// path that keeps the in-crate unit tests honest.
		if let Some(&idx) = self.map.get(&key) {
			self.nodes[idx].value = value;
			self.record_access(idx);
			return;
		}
		// Brand-new key.
		if self.window_cap == 0 {
			// Window inert: plain SLRU — land directly in main probationary.
			self.insert_new_main(key, value);
		} else {
			// Land in the admission window; it must earn a hit to enter main.
			self.insert_new_window(key, value);
		}
		debug_assert!(self.map.len() <= self.capacity);
		debug_assert_eq!(self.win_len + self.prob_len + self.prot_len, self.map.len());
		debug_assert!(self.nodes.len() <= self.capacity);
	}

	/// Record an access on a resident node, dispatching on its region.
	fn record_access(&mut self, idx: usize) {
		match self.nodes[idx].region {
			Region::Window => self.promote_window_to_main(idx),
			Region::Probationary | Region::Protected => self.record_main_hit(idx),
		}
	}

	/// A hit on a main-resident key: promote it to the protected MRU end and
	/// demote the protected LRU if the protected budget is now exceeded.
	fn record_main_hit(&mut self, idx: usize) {
		self.unlink(idx);
		self.push_back_protected(idx);
		self.enforce_protected_budget();
	}

	/// A hit on a window-resident key: its second touch. Promote it out of the
	/// window into the main probationary MRU end. If main is at budget, evict
	/// main's probationary LRU (else its protected LRU) first.
	fn promote_window_to_main(&mut self, idx: usize) {
		self.unlink(idx); // leave the window list
		if self.prob_len + self.prot_len >= self.main_cap {
			// Main is full: evict a main victim to make room. The freed slot is
			// recycled for a future insert.
			let victim = match self.prob_head {
				Some(head) => head,
				None => self
					.prot_head
					.expect("a full main segment always has a victim"),
			};
			let victim_key = self.nodes[victim].key.clone();
			self.map.remove(&victim_key);
			self.unlink(victim);
			self.free_slots.push(victim);
		}
		self.push_back_probationary(idx);
	}

	/// Insert a brand-new key at the admission window's MRU end. If the window
	/// is over budget, evict the window LRU entirely and reuse its slot in
	/// place — one-hit-wonders die here without ever touching main.
	fn insert_new_window(&mut self, key: K, value: V) {
		if self.win_len >= self.window_cap {
			// Window full: evict its LRU and reuse that exact slot for the newcomer.
			let evict_idx = self
				.win_head
				.expect("a full window always has a victim");
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.push_back_window(evict_idx);
			self.map.insert(key, evict_idx);
			return;
		}
		// Window has room; the total is below capacity, so a slot is available.
		let idx = self.acquire_slot(key.clone(), value);
		self.push_back_window(idx);
		self.map.insert(key, idx);
	}

	/// Insert a brand-new key directly into the main probationary segment. Used
	/// only when the window is inert (`window_cap == 0`), i.e. plain SLRU.
	fn insert_new_main(&mut self, key: K, value: V) {
		if self.prob_len + self.prot_len >= self.main_cap {
			// Main full: evict probationary LRU first, else protected LRU, and
			// reuse that slot in place.
			let evict_idx = match self.prob_head {
				Some(head) => head,
				None => self
					.prot_head
					.expect("a full main segment always has a victim"),
			};
			let evicted_key = self.nodes[evict_idx].key.clone();
			self.map.remove(&evicted_key);
			self.unlink(evict_idx);
			self.nodes[evict_idx].key = key.clone();
			self.nodes[evict_idx].value = value;
			self.push_back_probationary(evict_idx);
			self.map.insert(key, evict_idx);
			return;
		}
		let idx = self.acquire_slot(key.clone(), value);
		self.push_back_probationary(idx);
		self.map.insert(key, idx);
	}

	/// Obtain an arena slot for a net-new resident: recycle a freed slot if one
	/// exists, otherwise extend the arena. Callers only invoke this when the
	/// resident count is below capacity, so the arena never overflows.
	fn acquire_slot(&mut self, key: K, value: V) -> usize {
		if let Some(idx) = self.free_slots.pop() {
			self.nodes[idx].key = key;
			self.nodes[idx].value = value;
			self.nodes[idx].prev = None;
			self.nodes[idx].next = None;
			return idx;
		}
		debug_assert!(self.nodes.len() < self.capacity);
		let idx = self.nodes.len();
		self.nodes.push(Node {
			key,
			value,
			prev: None,
			next: None,
			region: Region::Window,
		});
		idx
	}

	/// Remove a node from whichever list it currently occupies.
	fn unlink(&mut self, idx: usize) {
		let prev = self.nodes[idx].prev;
		let next = self.nodes[idx].next;
		let region = self.nodes[idx].region;
		match prev {
			Some(p) => self.nodes[p].next = next,
			None => *self.head_mut(region) = next,
		}
		match next {
			Some(n) => self.nodes[n].prev = prev,
			None => *self.tail_mut(region) = prev,
		}
		self.nodes[idx].prev = None;
		self.nodes[idx].next = None;
		match region {
			Region::Window => self.win_len -= 1,
			Region::Probationary => self.prob_len -= 1,
			Region::Protected => self.prot_len -= 1,
		}
	}

	fn head_mut(&mut self, region: Region) -> &mut Option<usize> {
		match region {
			Region::Window => &mut self.win_head,
			Region::Probationary => &mut self.prob_head,
			Region::Protected => &mut self.prot_head,
		}
	}

	fn tail_mut(&mut self, region: Region) -> &mut Option<usize> {
		match region {
			Region::Window => &mut self.win_tail,
			Region::Probationary => &mut self.prob_tail,
			Region::Protected => &mut self.prot_tail,
		}
	}

	/// Append a node at the MRU (tail) end of the admission window.
	fn push_back_window(&mut self, idx: usize) {
		self.nodes[idx].region = Region::Window;
		self.nodes[idx].prev = self.win_tail;
		self.nodes[idx].next = None;
		match self.win_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.win_head = Some(idx),
		}
		self.win_tail = Some(idx);
		self.win_len += 1;
	}

	/// Append a node at the MRU (tail) end of the probationary segment.
	fn push_back_probationary(&mut self, idx: usize) {
		self.nodes[idx].region = Region::Probationary;
		self.nodes[idx].prev = self.prob_tail;
		self.nodes[idx].next = None;
		match self.prob_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prob_head = Some(idx),
		}
		self.prob_tail = Some(idx);
		self.prob_len += 1;
	}

	/// Append a node at the MRU (tail) end of the protected segment.
	fn push_back_protected(&mut self, idx: usize) {
		self.nodes[idx].region = Region::Protected;
		self.nodes[idx].prev = self.prot_tail;
		self.nodes[idx].next = None;
		match self.prot_tail {
			Some(t) => self.nodes[t].next = Some(idx),
			None => self.prot_head = Some(idx),
		}
		self.prot_tail = Some(idx);
		self.prot_len += 1;
	}

	/// If the protected segment exceeds its budget, demote its LRU node to the
	/// MRU end of the probationary segment. Only ever one node over budget per
	/// promotion, so a single demotion restores the bound.
	fn enforce_protected_budget(&mut self) {
		if self.prot_len > self.protected_cap {
			if let Some(lru) = self.prot_head {
				self.unlink(lru);
				self.push_back_probationary(lru);
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
		cache.get(&1); // promotes key 1 into the protected segment
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
		cache.put(1, 11); // update promotes key 1 into the protected segment
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
		cache.get(&2); // promote key 2 into the protected segment
		cache.put(4, 40); // evicts probationary LRU (key 1)
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(*cache.get(&2).unwrap(), 20);
	}

	#[test]
	fn promoted_key_survives_while_probationary_churns() {
		// SLRU: a key promoted to the protected segment is shielded from
		// eviction as long as the probationary segment can supply a victim.
		let mut cache = LruCache::new(2);
		cache.put(1, 10); // slot 0, probationary
		cache.put(2, 20); // slot 1, probationary
		cache.put(3, 30); // evicts probationary LRU (key 1), reuses slot 0
		cache.get(&3); // promote key 3 into the protected segment
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
		assert_eq!(*cache.get(&1).unwrap(), 10);
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
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
