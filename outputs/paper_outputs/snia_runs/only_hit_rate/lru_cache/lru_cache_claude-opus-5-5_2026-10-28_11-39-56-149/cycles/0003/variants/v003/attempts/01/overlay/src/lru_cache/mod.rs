// LIRS cache (Jiang & Zhang, SIGMETRICS 2002) behind the original `LruCache` API.
//
// Entries live in one arena `Vec<Node>` indexed by `u32` (NIL sentinel). Each node
// carries three independent intrusive link pairs:
//   - S: the LIRS recency stack (head = bottom / oldest, tail = top / newest),
//   - Q: the resident-HIR FIFO (head = next eviction victim),
//   - N: FIFO of non-resident HIR entries (head = oldest), used to bound metadata.
// `map` covers every tracked key, resident or non-resident. Freed slots go on a
// free list. No unsafe, no external crates.
//
// Sizing: the resident-HIR share is size-scaled (10% up to a 2048-entry HIR queue,
// then 5% with a 2048 floor), and non-resident history is bounded at 3 * capacity.
//
// `Node` stays `pub` (fields private): the metadata-size bin measures
// `size_of::<Node<u64, u64>>()` as the per-entry arena overhead.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

const NIL: u32 = u32::MAX;
const S: usize = 0;
const Q: usize = 1;
const N: usize = 2;

/// Multiplier of `capacity` bounding the non-resident HIR history.
const NONRES_FACTOR: usize = 3;

/// Resident-HIR share: 10% while that is <= 2048 entries, otherwise
/// max(2048, 5%). Always at least 1.
fn hir_share(capacity: usize) -> usize {
	if capacity / 10 <= 2048 {
		std::cmp::max(1, capacity / 10)
	} else {
		std::cmp::max(2048, capacity / 20)
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Status {
	Lir,
	HirResident,
	HirNonResident,
}

#[derive(Debug, Clone, Copy)]
struct Link {
	prev: u32,
	next: u32,
	linked: bool,
}

const UNLINKED: Link = Link {
	prev: NIL,
	next: NIL,
	linked: false,
};

/// Arena node: one tracked key (resident or non-resident HIR).
#[derive(Debug)]
pub struct Node<K, V> {
	key: K,
	value: Option<V>, // None => non-resident HIR (or free slot)
	status: Status,
	links: [Link; 3], // indexed by S / Q / N
}

#[derive(Debug, Clone, Copy)]
struct ListEnds {
	head: u32,
	tail: u32,
}

const EMPTY_LIST: ListEnds = ListEnds {
	head: NIL,
	tail: NIL,
};

#[derive(Debug)]
pub struct LruCache<K, V> {
	map: HashMap<K, u32>,
	entries: Vec<Node<K, V>>,
	free: Vec<u32>,
	lists: [ListEnds; 3],
	capacity: usize,
	lir_cap: usize,
	lir_count: usize,
	resident_count: usize,
	nonres_count: usize,
	nonres_limit: usize,
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
		// Arena may hold up to capacity + NONRES_FACTOR * capacity + 1 slots.
		if capacity >= NIL as usize / (NONRES_FACTOR + 2) {
			log::error!("LruCache.try_new: capacity {capacity} too large");
			return None;
		}
		let hir_cap = hir_share(capacity);
		let lir_cap = capacity - hir_cap; // capacity 1 => lir_cap 0 (all HIR)
		debug_assert!(capacity < 2 || lir_cap >= 1);
		log::info!("LruCache.try_new (LIRS): capacity={capacity} lir_cap={lir_cap} hir_cap={hir_cap}");
		Some(Self {
			map: HashMap::with_capacity(capacity * 2),
			entries: Vec::with_capacity(capacity),
			free: Vec::new(),
			lists: [EMPTY_LIST; 3],
			capacity,
			lir_cap,
			lir_count: 0,
			resident_count: 0,
			nonres_count: 0,
			nonres_limit: NONRES_FACTOR * capacity,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("LruCache::new: initialization failed")
	}

	/// Returns the value for `key` if resident, applying the LIRS hit logic.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let &idx = self.map.get(key)?;
		if self.entries[idx as usize].value.is_none() {
			// Non-resident HIR: a miss; the follow-up `put` handles promotion.
			return None;
		}
		self.access(idx);
		self.entries[idx as usize].value.as_ref()
	}

	/// Inserts or updates `key`. If at capacity, evicts the resident-HIR queue head.
	pub fn put(&mut self, key: K, value: V) {
		let existing = self.map.get(&key).copied();
		if let Some(idx) = existing {
			if self.entries[idx as usize].value.is_some() {
				self.entries[idx as usize].value = Some(value);
				self.access(idx);
				return;
			}
		}

		// Miss: make room first.
		if self.resident_count >= self.capacity {
			self.evict();
		}

		// Eviction may have released `existing`'s slot only if it was resident,
		// which it is not here, so `existing` stays valid.
		match existing {
			Some(idx) => {
				// Non-resident HIR still tracked (it is necessarily in S).
				self.list_remove(N, idx);
				self.nonres_count -= 1;
				let i = idx as usize;
				self.entries[i].value = Some(value);
				self.resident_count += 1;
				if self.lir_count < self.lir_cap {
					self.entries[i].status = Status::Lir;
					self.lir_count += 1;
					self.move_to_s_top(idx);
				} else if self.entries[i].links[S].linked {
					// Reuse distance smaller than the oldest LIR's recency: promote.
					self.entries[i].status = Status::Lir;
					self.lir_count += 1;
					self.move_to_s_top(idx);
					self.demote_bottom_lir();
				} else {
					// Defensive: should not happen (non-resident entries live only in S).
					self.entries[i].status = Status::HirResident;
					if self.lir_cap > 0 {
						self.move_to_s_top(idx);
					}
					self.list_push_tail(Q, idx);
				}
			}
			None => {
				let idx = self.alloc(key.clone(), value);
				self.map.insert(key, idx);
				self.resident_count += 1;
				if self.lir_count < self.lir_cap {
					self.entries[idx as usize].status = Status::Lir;
					self.lir_count += 1;
					self.list_push_tail(S, idx);
				} else {
					self.entries[idx as usize].status = Status::HirResident;
					if self.lir_cap > 0 {
						self.list_push_tail(S, idx);
					}
					self.list_push_tail(Q, idx);
				}
			}
		}

		self.bound_nonresident();
		debug_assert!(self.resident_count <= self.capacity);
	}

	/// LIRS hit logic for a resident entry.
	fn access(&mut self, idx: u32) {
		let i = idx as usize;
		match self.entries[i].status {
			Status::Lir => {
				let was_bottom = self.lists[S].head == idx;
				self.move_to_s_top(idx);
				if was_bottom {
					self.prune();
				}
			}
			Status::HirResident => {
				if self.entries[i].links[S].linked {
					// Promote to LIR; demote the bottom LIR.
					self.list_remove(Q, idx);
					self.entries[i].status = Status::Lir;
					self.lir_count += 1;
					self.move_to_s_top(idx);
					self.demote_bottom_lir();
				} else {
					if self.lir_cap > 0 {
						self.list_push_tail(S, idx);
					}
					self.list_remove(Q, idx);
					self.list_push_tail(Q, idx);
				}
			}
			Status::HirNonResident => {}
		}
	}

	/// Evict the head of Q (resident HIR).
	fn evict(&mut self) {
		let victim = self.lists[Q].head;
		if victim == NIL {
			return; // unreachable: lir_cap < capacity guarantees a resident HIR
		}
		self.list_remove(Q, victim);
		let v = victim as usize;
		self.entries[v].value = None;
		self.resident_count -= 1;
		if self.entries[v].links[S].linked {
			self.entries[v].status = Status::HirNonResident;
			self.list_push_tail(N, victim);
			self.nonres_count += 1;
		} else {
			self.release(victim);
		}
	}

	/// Turn the S-bottom LIR into a resident HIR at the Q tail, then prune.
	fn demote_bottom_lir(&mut self) {
		let b = self.lists[S].head;
		if b == NIL {
			return;
		}
		debug_assert_eq!(self.entries[b as usize].status, Status::Lir);
		self.list_remove(S, b);
		self.entries[b as usize].status = Status::HirResident;
		self.lir_count -= 1;
		self.list_push_tail(Q, b);
		self.prune();
	}

	/// Pop HIR entries from the S bottom until the bottom is LIR (or S is empty).
	fn prune(&mut self) {
		loop {
			let b = self.lists[S].head;
			if b == NIL || self.entries[b as usize].status == Status::Lir {
				break;
			}
			self.list_remove(S, b);
			if self.entries[b as usize].status == Status::HirNonResident {
				self.list_remove(N, b);
				self.nonres_count -= 1;
				self.release(b);
			}
		}
	}

	/// Keep non-resident metadata bounded at `NONRES_FACTOR * capacity` by
	/// dropping the oldest non-resident entry (head of N) from N, S and `map`.
	fn bound_nonresident(&mut self) {
		while self.nonres_count > self.nonres_limit {
			let n = self.lists[N].head;
			if n == NIL {
				break;
			}
			self.list_remove(N, n);
			self.list_remove(S, n);
			self.nonres_count -= 1;
			self.release(n);
		}
	}

	fn move_to_s_top(&mut self, idx: u32) {
		self.list_remove(S, idx);
		self.list_push_tail(S, idx);
	}

	fn alloc(&mut self, key: K, value: V) -> u32 {
		if let Some(idx) = self.free.pop() {
			let e = &mut self.entries[idx as usize];
			e.key = key;
			e.value = Some(value);
			e.links = [UNLINKED; 3];
			idx
		} else {
			let idx = self.entries.len() as u32;
			self.entries.push(Node {
				key,
				value: Some(value),
				status: Status::HirResident,
				links: [UNLINKED; 3],
			});
			idx
		}
	}

	/// Remove a (fully unlinked) entry from the map and recycle its slot.
	fn release(&mut self, idx: u32) {
		let i = idx as usize;
		debug_assert!(!self.entries[i].links.iter().any(|l| l.linked));
		let key = self.entries[i].key.clone();
		self.map.remove(&key);
		self.entries[i].value = None;
		self.entries[i].status = Status::HirNonResident;
		self.free.push(idx);
	}

	fn list_remove(&mut self, l: usize, idx: u32) {
		let link = self.entries[idx as usize].links[l];
		if !link.linked {
			return;
		}
		if link.prev != NIL {
			self.entries[link.prev as usize].links[l].next = link.next;
		} else {
			self.lists[l].head = link.next;
		}
		if link.next != NIL {
			self.entries[link.next as usize].links[l].prev = link.prev;
		} else {
			self.lists[l].tail = link.prev;
		}
		self.entries[idx as usize].links[l] = UNLINKED;
	}

	fn list_push_tail(&mut self, l: usize, idx: u32) {
		if self.entries[idx as usize].links[l].linked {
			self.list_remove(l, idx);
		}
		let tail = self.lists[l].tail;
		self.entries[idx as usize].links[l] = Link {
			prev: tail,
			next: NIL,
			linked: true,
		};
		if tail != NIL {
			self.entries[tail as usize].links[l].next = idx;
		} else {
			self.lists[l].head = idx;
		}
		self.lists[l].tail = idx;
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

	/// get-then-put-on-miss, mirroring the JNI call contract.
	fn touch(cache: &mut LruCache<i64, i64>, key: i64) -> bool {
		if cache.get(&key).is_some() {
			true
		} else {
			cache.put(key, key * 10);
			false
		}
	}

	fn residents(cache: &mut LruCache<i64, i64>, range: std::ops::Range<i64>) -> usize {
		range.filter(|k| cache.get(k).is_some()).count()
	}

	fn check_structure(cache: &LruCache<i64, i64>) {
		let lir = cache.entries.iter().enumerate().filter(|(i, e)| {
			!cache.free.contains(&(*i as u32)) && e.status == Status::Lir
		});
		assert_eq!(lir.count(), cache.lir_count);
		assert!(cache.lir_count <= cache.lir_cap);
		assert!(cache.resident_count <= cache.capacity);
		assert!(cache.nonres_count <= cache.nonres_limit);
		assert_eq!(cache.map.len(), cache.resident_count + cache.nonres_count);
		let b = cache.lists[S].head;
		if b != NIL {
			assert_eq!(cache.entries[b as usize].status, Status::Lir, "S bottom must be LIR");
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
	fn put_overwrites_resident_key() {
		let mut cache = LruCache::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
		cache.put(2, 30);
		assert_eq!(*cache.get(&2).unwrap(), 30);
	}

	#[test]
	fn node_type_is_public_and_sized() {
		assert!(std::mem::size_of::<Node<u64, u64>>() > 0);
	}

	#[test]
	fn hir_share_is_size_scaled() {
		assert_eq!(hir_share(1), 1);
		assert_eq!(hir_share(2), 1);
		assert_eq!(hir_share(100), 10);
		assert_eq!(hir_share(16384), 1638);
		assert_eq!(hir_share(131072), 6553);
		let small = LruCache::<i64, i64>::new(16384);
		assert_eq!(small.capacity - small.lir_cap, 1638);
		assert_eq!(small.nonres_limit, 3 * 16384);
		let large = LruCache::<i64, i64>::new(131072);
		assert_eq!(large.capacity - large.lir_cap, 6553);
		assert_eq!(large.nonres_limit, 3 * 131072);
		for capacity in 2..5000usize {
			assert!(capacity - hir_share(capacity) >= 1, "lir_cap must be >= 1 at {capacity}");
		}
	}

	#[test]
	fn capacity_one_behaves_as_single_slot() {
		let mut cache = LruCache::new(1);
		assert_eq!(cache.lir_cap, 0);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.put(2, 20);
		assert!(cache.get(&1).is_none());
		assert_eq!(*cache.get(&2).unwrap(), 20);
		for k in 0..50 {
			touch(&mut cache, k);
			check_structure(&cache);
		}
		assert_eq!(cache.map.len(), 1);
	}

	#[test]
	fn capacity_two_promotes_non_resident_hir() {
		let mut cache = LruCache::new(2);
		assert_eq!(cache.lir_cap, 1);
		cache.put(1, 10); // LIR
		cache.put(2, 20); // resident HIR
		cache.put(3, 30); // evicts 2 -> non-resident in S; 3 resident HIR
		assert!(cache.get(&2).is_none());
		cache.put(2, 21); // evicts 3; 2 promoted to LIR; 1 demoted to HIR
		check_structure(&cache);
		assert_eq!(cache.entries[cache.map[&2] as usize].status, Status::Lir);
		assert_eq!(cache.entries[cache.map[&1] as usize].status, Status::HirResident);
		assert_eq!(*cache.get(&1).unwrap(), 10);
		assert_eq!(*cache.get(&2).unwrap(), 21);
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn evicted_key_out_of_s_reenters_as_hir() {
		// capacity 10 => hir_cap 1, lir_cap 9.
		let mut cache = LruCache::new(10);
		assert_eq!(cache.lir_cap, 9);
		for k in 0..9 {
			touch(&mut cache, k); // LIR
		}
		touch(&mut cache, 100); // resident HIR, in S
		touch(&mut cache, 101); // evicts 100 -> non-resident in S
		check_structure(&cache);
		// Re-reference 100 while still in S: becomes LIR, bottom LIR (0) demoted.
		touch(&mut cache, 100);
		check_structure(&cache);
		assert_eq!(cache.entries[cache.map[&100] as usize].status, Status::Lir);
		assert_eq!(cache.entries[cache.map[&0] as usize].status, Status::HirResident);
	}

	#[test]
	fn hot_lir_set_survives_scan() {
		let capacity = 100usize;
		let mut cache = LruCache::new(capacity);
		assert_eq!(cache.lir_cap, 90);
		let hot = 0i64..80;
		for _ in 0..3 {
			for k in hot.clone() {
				touch(&mut cache, k);
			}
		}
		for k in 10_000..10_000 + 10 * capacity as i64 {
			touch(&mut cache, k);
			check_structure(&cache);
		}
		assert_eq!(residents(&mut cache, hot.clone()), 80);
		// LRU would have flushed the hot set entirely.
		for k in hot {
			assert!(touch(&mut cache, k), "hot key {k} lost to scan");
		}
	}

	#[test]
	fn metadata_stays_bounded() {
		let capacity = 16usize;
		let mut cache = LruCache::new(capacity);
		let mut x: u64 = 0x1234_5678;
		for _ in 0..20_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			touch(&mut cache, (x % 500) as i64);
			check_structure(&cache);
		}
		assert!(cache.map.len() <= (NONRES_FACTOR + 1) * capacity);
		assert!(cache.entries.len() <= (NONRES_FACTOR + 1) * capacity + 1);
	}

	#[test]
	fn resident_count_bounded_large_random_workload() {
		let capacity = 1000usize;
		let mut cache = LruCache::new(capacity);
		let mut x: u64 = 0x9e37_79b9_7f4a_7c15;
		for _ in 0..100_000 {
			x ^= x << 13;
			x ^= x >> 7;
			x ^= x << 17;
			touch(&mut cache, (x % 5000) as i64);
			assert!(cache.resident_count <= capacity);
			assert!(cache.lir_count <= cache.lir_cap);
			assert!(cache.nonres_count <= cache.nonres_limit);
			assert_eq!(cache.map.len(), cache.resident_count + cache.nonres_count);
		}
		check_structure(&cache);
		assert!(residents(&mut cache, 0..5000) <= capacity);
	}

	#[test]
	fn randomized_value_fidelity_and_capacity() {
		for capacity in [1usize, 2, 3, 8, 50] {
			let mut cache = LruCache::new(capacity);
			let mut x: u64 = 0xdead_beef ^ capacity as u64;
			for _ in 0..5_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				touch(&mut cache, (x % 200) as i64);
				check_structure(&cache);
			}
			assert!(residents(&mut cache, 0..200) <= capacity);
			for k in 0..200 {
				if let Some(&v) = cache.get(&k) {
					assert_eq!(v, k * 10);
				}
			}
			assert!(cache.get(&-1).is_none());
		}
	}

	#[test]
	#[should_panic(expected = "initialization failed")]
	fn capacity_zero_panics() {
		LruCache::<i64, i64>::new(0);
	}
}
