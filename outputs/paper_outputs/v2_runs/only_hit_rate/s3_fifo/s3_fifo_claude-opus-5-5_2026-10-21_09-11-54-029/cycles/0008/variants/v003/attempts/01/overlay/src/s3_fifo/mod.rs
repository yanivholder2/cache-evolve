use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Policy (unchanged from cycle/0003/v003): 5% small queue, MAX_FREQ = 7,
// graduation from small at freq > 1, ghost re-admission at the head of main
// with freq 0.
//
// H01 (cycle 4): structure only. Entries live in a slab addressed by slot id;
// a HashMap index gives O(1) residency lookup; the small FIFO holds slot ids.
//
// H03 (cycle 5): main-queue eviction victims are also recorded in the ghost
// (same main_size cap), so a returning main victim re-enters main directly.
//
// H02 (cycle 6): the ghost is a fixed-size ring of key fingerprints
// (`ghost_ring`, length main_size.max(1)) plus a `ghost_map` from fingerprint
// to ring slot. No `K` is cloned into the ghost and there is no compaction.
//   * A ghost hit removes the entry and zeroes its slot, leaving a hole.
//   * Fingerprint collisions are accepted (policy-only).
//
// H02 (cycle 7): main eviction is SIEVE-style instead of FIFO reinsertion.
// Main is an intrusive doubly linked list over slab slots (head = newest,
// tail = oldest; `prev` points toward the head). A `hand` sweeps from the
// tail toward the head: an entry with freq > 0 has freq decremented by 1 and
// stays in place; the first freq == 0 entry is the victim. The hand then
// rests on the victim's predecessor (NIL = restart at the tail).
//
// H03 (cycle 8): representation only. Fingerprints are 32-bit (64-bit
// DefaultHasher output folded as (h ^ (h >> 32)) as u32, 0 mapped to 1), the
// ghost ring holds u32 slots, and `ghost_map` uses an identity hasher on the
// already-hashed fingerprint (spread by an odd 64-bit multiply so hashbrown's
// top-7-bit control bytes are well distributed). False ghost hits occur with
// odds about main_size / 2^32 per lookup.
const MAX_FREQ: u8 = 7;

/// Null sentinel for intrusive list links and the hand.
const NIL: u32 = u32::MAX;

/// Fixed-seed 32-bit fingerprint of a key. 0 is reserved for "empty slot".
#[inline]
fn fingerprint<K: Hash>(key: &K) -> u32 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	let h = h.finish();
	let fp = (h ^ (h >> 32)) as u32;
	if fp == 0 {
		1
	} else {
		fp
	}
}

/// Hasher for keys that are already well-mixed fingerprints. `u32` keys are
/// taken as-is and spread across all 64 bits in `finish()` so that hashbrown's
/// control bits (top 7 bits) and bucket bits (low bits) both vary.
#[derive(Default, Clone, Copy, Debug)]
pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0.wrapping_mul(0x9E37_79B9_7F4A_7C15)
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		// Fallback path (not used for u32 keys).
		for &b in bytes {
			self.0 = self.0.rotate_left(8) ^ (b as u64);
		}
	}

	#[inline]
	fn write_u32(&mut self, x: u32) {
		self.0 = x as u64;
	}

	#[inline]
	fn write_u64(&mut self, x: u64) {
		self.0 = x;
	}
}

type IdentityBuild = BuildHasherDefault<IdentityHasher>;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
	/// Toward the main head (newer). Only meaningful while linked in main.
	prev: u32,
	/// Toward the main tail (older). Only meaningful while linked in main.
	next: u32,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, freq: 0, prev: NIL, next: NIL }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Option<Entry<K, V>>>,
	free: Vec<usize>,
	index: HashMap<K, usize>,
	small: VecDeque<usize>,
	main_head: u32,
	main_tail: u32,
	main_len: usize,
	hand: u32,
	ghost_ring: Vec<u32>,
	ghost_head: usize,
	ghost_map: HashMap<u32, u32, IdentityBuild>,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		// Slot ids (up to capacity + 1 slots) must fit below the NIL sentinel.
		if capacity >= (u32::MAX as usize) - 2 {
			log::error!("S3Fifo.try_new: capacity too large for u32 slot links");
			return None;
		}
		// Small queue is 5% of capacity (inherited from cycle/0002/v003).
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		// Ghost ring slot indices are stored as u32 in `ghost_map`.
		if u32::try_from(main_size.max(1)).is_err() {
			log::error!("S3Fifo.try_new: main size too large for u32 ghost slots");
			return None;
		}
		Some(Self {
			slab: Vec::with_capacity(capacity + 1),
			free: Vec::with_capacity(capacity + 1),
			index: HashMap::with_capacity(capacity + 1),
			small: VecDeque::with_capacity(small_size),
			main_head: NIL,
			main_tail: NIL,
			main_len: 0,
			hand: NIL,
			ghost_ring: vec![0u32; main_size.max(1)],
			ghost_head: 0,
			ghost_map: HashMap::with_capacity_and_hasher(main_size + 1, IdentityBuild::default()),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn entry_mut(&mut self, idx: usize) -> &mut Entry<K, V> {
		self.slab[idx].as_mut().expect("S3Fifo: queued slot must be occupied")
	}

	#[inline]
	#[allow(dead_code)]
	fn entry(&self, idx: usize) -> &Entry<K, V> {
		self.slab[idx].as_ref().expect("S3Fifo: queued slot must be occupied")
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		let entry = Entry::new(key.clone(), value);
		let idx = match self.free.pop() {
			Some(idx) => {
				self.slab[idx] = Some(entry);
				idx
			}
			None => {
				self.slab.push(Some(entry));
				self.slab.len() - 1
			}
		};
		self.index.insert(key, idx);
		idx
	}

	fn release(&mut self, idx: usize) -> Entry<K, V> {
		let entry = self.slab[idx].take().expect("S3Fifo: released slot must be occupied");
		self.free.push(idx);
		self.index.remove(&entry.key);
		entry
	}

	/// Link slot `idx` at the main head (newest end).
	fn main_push_head(&mut self, idx: usize) {
		let old_head = self.main_head;
		{
			let e = self.entry_mut(idx);
			e.prev = NIL;
			e.next = old_head;
		}
		if old_head != NIL {
			self.entry_mut(old_head as usize).prev = idx as u32;
		} else {
			self.main_tail = idx as u32;
		}
		self.main_head = idx as u32;
		self.main_len += 1;
	}

	/// Unlink slot `idx` from main. Caller is responsible for the hand.
	fn main_unlink(&mut self, idx: usize) {
		let (p, n) = {
			let e = self.entry_mut(idx);
			let pn = (e.prev, e.next);
			e.prev = NIL;
			e.next = NIL;
			pn
		};
		if p != NIL {
			self.entry_mut(p as usize).next = n;
		} else {
			self.main_head = n;
		}
		if n != NIL {
			self.entry_mut(n as usize).prev = p;
		} else {
			self.main_tail = p;
		}
		self.main_len -= 1;
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.index.get(key)?;
		let entry = self.slab[idx].as_mut()?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.index.get(&key) {
			self.entry_mut(idx).value = value;
			return;
		}

		let fp: u32 = fingerprint(&key);
		if let Some(slot) = self.ghost_map.remove(&fp) {
			self.ghost_ring[slot as usize] = 0;
			if self.main_len >= self.main_size {
				self.evict_main();
			}
			let idx = self.alloc(key, value);
			self.main_push_head(idx);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let idx = self.alloc(key, value);
		self.small.push_front(idx);
	}

	/// SIEVE sweep: survivors keep their position; the hand moves toward the
	/// head, wrapping to the tail.
	fn evict_main(&mut self) {
		if self.main_len == 0 {
			return;
		}
		let mut cur = if self.hand == NIL { self.main_tail } else { self.hand };
		loop {
			let e = self.entry_mut(cur as usize);
			if e.freq > 0 {
				e.freq -= 1;
				let p = e.prev;
				cur = if p == NIL { self.main_tail } else { p };
			} else {
				let p = e.prev;
				self.hand = p;
				let idx = cur as usize;
				self.main_unlink(idx);
				// H03: remember main victims in the ghost so a return
				// re-enters main directly.
				let entry = self.release(idx);
				self.ghost_insert(&entry.key);
				return;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(idx) = self.small.pop_back() {
			if self.entry_mut(idx).freq > 1 {
				if self.main_len >= self.main_size {
					self.evict_main();
				}
				self.main_push_head(idx);
			} else {
				let entry = self.release(idx);
				self.ghost_insert(&entry.key);
			}
		}
	}

	fn ghost_insert(&mut self, key: &K) {
		let fp: u32 = fingerprint(key);
		let slot = self.ghost_head;
		let old: u32 = self.ghost_ring[slot];
		// Evict the oldest ring entry only if the map still points at this
		// slot (a stale slot — hole or superseded duplicate — is just overwritten).
		if old != 0 && self.ghost_map.get(&old) == Some(&(slot as u32)) {
			self.ghost_map.remove(&old);
		}
		self.ghost_ring[slot] = fp;
		self.ghost_map.insert(fp, slot as u32);
		self.ghost_head += 1;
		if self.ghost_head == self.ghost_ring.len() {
			self.ghost_head = 0;
		}
	}
}

impl<K: PartialEq + Eq + std::hash::Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
	fn put(&mut self, key: K, value: V) {
		S3Fifo::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		S3Fifo::get(self, key)
	}
}

#[cfg(test)]
impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	/// Link at the main tail (oldest end). Test-only.
	fn test_push_main_back(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		let old_tail = self.main_tail;
		{
			let e = self.entry_mut(idx);
			e.freq = freq;
			e.prev = old_tail;
			e.next = NIL;
		}
		if old_tail != NIL {
			self.entry_mut(old_tail as usize).next = idx as u32;
		} else {
			self.main_head = idx as u32;
		}
		self.main_tail = idx as u32;
		self.main_len += 1;
	}

	fn test_push_main_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.main_push_head(idx);
	}

	fn test_push_small_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.small.push_front(idx);
	}

	fn queue_keys(&self, q: &VecDeque<usize>) -> Vec<(K, u8)> {
		q.iter()
			.map(|&idx| {
				let e = self.entry(idx);
				(e.key.clone(), e.freq)
			})
			.collect()
	}

	/// Main list walked head (newest) -> tail (oldest), with link checks.
	fn main_queue_keys(&self) -> Vec<(K, u8)> {
		let mut out = Vec::with_capacity(self.main_len);
		let mut cur = self.main_head;
		let mut prev = NIL;
		while cur != NIL {
			let e = self.entry(cur as usize);
			assert_eq!(e.prev, prev, "main list prev link broken");
			out.push((e.key.clone(), e.freq));
			prev = cur;
			cur = e.next;
		}
		assert_eq!(prev, self.main_tail, "main tail mismatch");
		assert_eq!(out.len(), self.main_len, "main_len mismatch");
		out
	}

	fn main_keys(&self) -> Vec<K> {
		self.main_queue_keys().into_iter().map(|(k, _)| k).collect()
	}

	fn small_keys(&self) -> Vec<K> {
		self.queue_keys(&self.small).into_iter().map(|(k, _)| k).collect()
	}

	fn freq_of(&self, key: &K) -> Option<u8> {
		self.index.get(key).map(|&idx| self.entry(idx).freq)
	}

	fn hand_key(&self) -> Option<K> {
		if self.hand == NIL {
			None
		} else {
			Some(self.entry(self.hand as usize).key.clone())
		}
	}

	/// Fingerprints are irreversible; ghost membership is checked by key.
	fn ghost_contains(&self, key: &K) -> bool {
		self.ghost_map.contains_key(&fingerprint(key))
	}

	fn resident_len(&self) -> usize {
		self.small.len() + self.main_len
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn get_miss_returns_none() {
		let mut cache = S3Fifo::<i64, i64>::new(4);
		assert!(cache.get(&42).is_none());
	}

	#[test]
	fn put_then_get_finds_value() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		assert_eq!(*cache.get(&1).unwrap(), 10);
	}

	#[test]
	fn put_existing_updates_value() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(1, 20);
		assert_eq!(*cache.get(&1).unwrap(), 20);
	}

	#[test]
	fn small_eviction_promotes_to_main_after_multiple_hits() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1);
		cache.put(2, 20);
		assert!(cache.main_keys().contains(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main_keys().contains(&1));
		assert!(!cache.ghost_contains(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		// SIEVE (cycle 7): the hand starts at the tail (4, freq 1): 4 is
		// decremented in place, the hand moves to 3 (freq 0) which is the
		// victim. Same expected victim as under FIFO reinsertion; we also
		// check 4 kept its tail position and the hand rests on 2.
		let mut cache = S3Fifo::new(5);
		cache.test_push_main_back(1, 10, 0);
		cache.test_push_main_back(2, 20, 2);
		cache.test_push_main_back(3, 30, 0);
		cache.test_push_main_back(4, 40, 1);
		cache.test_push_small_front(5, 50, 2);
		cache.evict_small();
		assert!(cache.main_keys().contains(&4));
		assert!(!cache.main_keys().contains(&3));
		assert!(cache.get(&3).is_none());
		assert_eq!(cache.main_keys(), vec![5, 1, 2, 4]);
		assert_eq!(cache.freq_of(&4), Some(0));
		assert_eq!(cache.hand_key(), Some(2));
	}

	#[test]
	fn sieve_survivors_keep_position() {
		// Survivors are not moved to the head; a freq-0 newcomer at the head
		// is examined after older survivors once their credit drains.
		let mut cache = S3Fifo::<i64, i64>::new(8); // main_size 7
		cache.test_push_main_back(1, 1, 1);
		cache.test_push_main_back(2, 2, 0);
		cache.test_push_main_back(3, 3, 1);
		// head..tail = 1, 2, 3. Hand: 3 (1->0), 2 (victim).
		cache.evict_main();
		assert_eq!(cache.main_keys(), vec![1, 3]);
		assert_eq!(cache.hand_key(), Some(1));
		assert!(cache.ghost_contains(&2));
		// Next sweep continues at 1 (1->0), wraps to tail 3 (freq 0) victim.
		cache.evict_main();
		assert_eq!(cache.main_keys(), vec![1]);
		assert!(cache.ghost_contains(&3));
	}

	#[test]
	fn main_victim_is_ghosted_and_returns_to_main() {
		// capacity 20 -> small_size 1, main_size 19.
		let mut cache = S3Fifo::<i64, i64>::new(20);
		assert_eq!(cache.main_size, 19);
		cache.test_push_main_back(7, 70, 0);
		for k in 100..118i64 {
			cache.test_push_main_front(k, k, 1);
		}
		assert_eq!(cache.main_len, cache.main_size);
		assert_eq!(*cache.main_keys().last().unwrap(), 7);

		cache.test_push_small_front(50, 50, 2);
		cache.evict_small();
		assert!(cache.get(&7).is_none());
		assert!(cache.ghost_contains(&7));
		assert!(cache.main_keys().contains(&50));

		cache.put(7, 71);
		assert!(cache.main_keys().contains(&7));
		assert!(!cache.small_keys().contains(&7));
		assert!(!cache.ghost_contains(&7));
		assert_eq!(cache.freq_of(&7), Some(0));
		assert!(cache.resident_len() <= 20);
		assert_eq!(*cache.get(&7).unwrap(), 71);
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
			assert!(cache.ghost_map.len() <= cache.main_size.max(1));
		}
		assert!(cache.resident_len() <= 4);
		assert_eq!(cache.index.len(), cache.resident_len());
		assert!(cache.ghost_map.len() <= cache.main_size);
	}

	#[test]
	fn small_queue_is_five_percent_of_capacity() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_size, 102);
		assert_eq!(cache.main_size, 1946);
		assert_eq!(cache.ghost_ring.len(), 1946);
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
	}

	#[test]
	fn main_resident_frequency_saturates_at_seven() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.test_push_main_front(1, 10, 0);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert_eq!(cache.freq_of(&1), Some(7));
		assert_eq!(MAX_FREQ, 7);
	}

	#[test]
	fn max_freq_key_survives_seven_main_passes_then_evicted_on_eighth() {
		// SIEVE (cycle 7): each pass the hand restarts at the tail (key 1),
		// decrements it in place and evicts the freq-0 cold key at the head;
		// the hand then becomes NIL (victim had no prev). Expected victims are
		// unchanged from the FIFO-reinsertion version.
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.test_push_main_back(1, 10, 0);
		for _ in 0..10 {
			cache.get(&1);
		}
		assert_eq!(cache.freq_of(&1), Some(MAX_FREQ));

		for pass in 0..7i64 {
			let cold = 100 + pass;
			cache.test_push_main_front(cold, cold, 0);
			cache.evict_main();
			assert!(cache.main_keys().contains(&1), "hot key evicted on pass {}", pass + 1);
			assert!(!cache.main_keys().contains(&cold));
			assert!(cache.get(&cold).is_none());
		}
		assert_eq!(cache.freq_of(&1), Some(0));

		cache.test_push_main_front(200, 200, 0);
		assert_eq!(*cache.main_keys().last().unwrap(), 1);
		cache.evict_main();
		assert!(!cache.main_keys().contains(&1));
		assert!(cache.main_keys().contains(&200));
	}

	#[test]
	fn ghost_ring_is_fixed_size_and_bounded() {
		let mut cache = S3Fifo::<i64, i64>::new(40);
		let ring_len = cache.ghost_ring.len();
		for key in 0..10_000i64 {
			let k = key % 500;
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert!(cache.ghost_map.len() <= cache.main_size.max(1));
			assert_eq!(cache.ghost_ring.len(), ring_len);
			assert!(cache.ghost_head < ring_len);
		}
	}

	#[test]
	fn fingerprint_never_zero_and_deterministic() {
		for k in -1000i64..1000 {
			let fp: u32 = fingerprint(&k);
			assert_ne!(fp, 0);
			assert_eq!(fp, fingerprint(&k));
		}
	}

	#[test]
	fn identity_hashed_ghost_map_round_trips_u32_fingerprints() {
		// Exercise the identity-hasher path for u32 keys with ~10k distinct
		// fingerprints, checked against a SipHash-backed reference map
		// (fingerprint collisions among the inputs are handled by the reference).
		let mut cache = S3Fifo::<i64, i64>::new(16);
		cache.ghost_map.clear();
		let mut reference: HashMap<u32, u32> = HashMap::new();
		for k in 0..10_000i64 {
			let fp = fingerprint(&k);
			cache.ghost_map.insert(fp, k as u32);
			reference.insert(fp, k as u32);
		}
		// Also dense sequential u32 keys, the worst case for a naive identity hash.
		for x in 1..=2_000u32 {
			cache.ghost_map.insert(x, x.wrapping_mul(3));
			reference.insert(x, x.wrapping_mul(3));
		}
		assert!(reference.len() > 9_900);
		assert_eq!(cache.ghost_map.len(), reference.len());
		for (fp, slot) in &reference {
			assert_eq!(cache.ghost_map.get(fp), Some(slot), "lookup mismatch for fp {fp}");
		}
		for k in 0..10_000i64 {
			assert!(cache.ghost_map.contains_key(&fingerprint(&k)));
		}
		// Removal round-trip.
		for k in (0..10_000i64).step_by(2) {
			let fp = fingerprint(&k);
			assert_eq!(cache.ghost_map.remove(&fp), reference.remove(&fp));
		}
		assert_eq!(cache.ghost_map.len(), reference.len());
		for (fp, slot) in &reference {
			assert_eq!(cache.ghost_map.get(fp), Some(slot));
		}
	}

	// ---- Reference: cycle/0003/v003 linear implementation (MAX_FREQ = 7),
	// ---- plus H03 (main victims ghosted) and H02 ring-with-holes ghost.
	// ---- Cycle 7: reference main eviction changed to SIEVE (VecDeque with
	// ---- front = head, plus a hand index; survivors stay in place).

	struct RefEntry {
		key: i64,
		value: i64,
		freq: u8,
	}

	struct RefS3Fifo {
		small: VecDeque<RefEntry>,
		main: VecDeque<RefEntry>,
		hand: Option<usize>,
		ring: Vec<Option<i64>>,
		head: usize,
		small_size: usize,
		main_size: usize,
	}

	impl RefS3Fifo {
		fn new(capacity: usize) -> Self {
			let small_size = if capacity == 1 {
				1
			} else {
				(capacity / 20).max(1).min(capacity - 1)
			};
			let main_size = capacity - small_size;
			Self {
				small: VecDeque::new(),
				main: VecDeque::new(),
				hand: None,
				ring: vec![None; main_size.max(1)],
				head: 0,
				small_size,
				main_size,
			}
		}

		fn get(&mut self, key: &i64) -> Option<i64> {
			let entry = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| &e.key == key)?;
			entry.freq = (entry.freq.saturating_add(1)).min(7);
			Some(entry.value)
		}

		fn main_push_front(&mut self, e: RefEntry) {
			self.main.push_front(e);
			if let Some(h) = self.hand.as_mut() {
				*h += 1;
			}
		}

		fn put(&mut self, key: i64, value: i64) {
			if let Some(entry) = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| e.key == key) {
				entry.value = value;
				return;
			}
			if let Some(pos) = self.ring.iter().position(|g| *g == Some(key)) {
				self.ring[pos] = None;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main_push_front(RefEntry { key, value, freq: 0 });
				return;
			}
			if self.small.len() >= self.small_size {
				self.evict_small();
			}
			self.small.push_front(RefEntry { key, value, freq: 0 });
		}

		fn ghost_push(&mut self, key: i64) {
			self.ring[self.head] = Some(key);
			self.head = (self.head + 1) % self.ring.len();
		}

		fn evict_main(&mut self) {
			if self.main.is_empty() {
				return;
			}
			let mut pos = self.hand.unwrap_or(self.main.len() - 1);
			loop {
				if self.main[pos].freq > 0 {
					self.main[pos].freq -= 1;
					pos = if pos == 0 { self.main.len() - 1 } else { pos - 1 };
				} else {
					self.hand = if pos == 0 { None } else { Some(pos - 1) };
					let victim = self.main.remove(pos).unwrap();
					self.ghost_push(victim.key);
					return;
				}
			}
		}

		fn evict_small(&mut self) {
			if let Some(tail) = self.small.pop_back() {
				if tail.freq > 1 {
					if self.main.len() >= self.main_size {
						self.evict_main();
					}
					self.main_push_front(tail);
				} else {
					self.ghost_push(tail.key);
				}
			}
		}
	}

	#[test]
	fn matches_ring_reference_on_pseudo_random_streams() {
		for &capacity in &[1usize, 2, 3, 4, 8, 20, 41, 64, 100] {
			for &(key_space, skew) in &[(16u64, false), (64, true), (300, true), (1000, false)] {
				let mut cache = S3Fifo::<i64, i64>::new(capacity);
				let mut reference = RefS3Fifo::new(capacity);
				let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ (capacity as u64 * 31 + key_space);
				for step in 0..5_000 {
					state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					let r = state >> 33;
					let key = if skew {
						let a = r % key_space;
						let b = (r >> 12) % key_space;
						a.min(b) as i64
					} else {
						(r % key_space) as i64
					};

					let got = cache.get(&key).copied();
					let expected = reference.get(&key);
					assert_eq!(got, expected, "cap={capacity} step={step} key={key}");
					if got.is_none() {
						cache.put(key, key * 3);
						reference.put(key, key * 3);
					}

					let small: Vec<(i64, u8)> = cache.queue_keys(&cache.small);
					let ref_small: Vec<(i64, u8)> = reference.small.iter().map(|e| (e.key, e.freq)).collect();
					assert_eq!(small, ref_small, "small mismatch cap={capacity} step={step}");
					let main: Vec<(i64, u8)> = cache.main_queue_keys();
					let ref_main: Vec<(i64, u8)> = reference.main.iter().map(|e| (e.key, e.freq)).collect();
					assert_eq!(main, ref_main, "main mismatch cap={capacity} step={step}");
					let ref_hand = reference.hand.map(|p| reference.main[p].key);
					assert_eq!(cache.hand_key(), ref_hand, "hand mismatch cap={capacity} step={step}");

					// Ghost: ring layout and live membership match exactly.
					let ref_ring: Vec<u32> =
						reference.ring.iter().map(|g| g.map(|k| fingerprint(&k)).unwrap_or(0)).collect();
					assert_eq!(cache.ghost_ring, ref_ring, "ghost ring mismatch cap={capacity} step={step}");
					assert_eq!(cache.ghost_head, reference.head, "ghost head mismatch cap={capacity} step={step}");
					let live: usize = reference.ring.iter().filter(|g| g.is_some()).count();
					assert_eq!(cache.ghost_map.len(), live, "ghost live mismatch cap={capacity} step={step}");
					for k in reference.ring.iter().flatten() {
						assert!(cache.ghost_contains(k), "ghost missing {k} cap={capacity} step={step}");
					}
					assert!(cache.ghost_map.len() <= cache.main_size.max(1));
					assert_eq!(cache.index.len(), cache.resident_len());
				}
			}
		}
	}
}
