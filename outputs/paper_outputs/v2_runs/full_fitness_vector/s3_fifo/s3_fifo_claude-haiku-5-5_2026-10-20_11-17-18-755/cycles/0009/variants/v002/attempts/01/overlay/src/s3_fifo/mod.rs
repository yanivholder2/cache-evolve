use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The per-entry frequency counter lives in a parallel array (`small_freq` /
// `main_freq`) kept in lockstep with its queue: index i of the freq deque is the
// counter for index i of the entry deque. Entries therefore hold only key and
// value, which keeps `Entry<u64, u64>` at 16 bytes.
//
// The ghost queue stores 64-bit key fingerprints rather than keys, so eviction
// never clones a key. `ghost_set` holds the live fingerprints in a flat
// open-addressing table (`FpSet`); `ghost` may hold stale copies of fingerprints
// that were promoted out of the ghost, and `ghost_copies` counts the queue copies
// of each fingerprint so stale ones can be discarded when they reach the back of
// the queue.
//
// A single `index` maps every resident key to its queue and a stable id. Each
// queue has a head counter that rises on every push_front, and a resident at
// deque position i has `id = head - i`, so its position is `head - id`. All
// pushes and pops go through the queue helpers so the index and heads stay in
// step with the deques.
const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value }
	}
}

#[derive(Debug, Clone, Copy)]
struct Slot {
	in_small: bool,
	id: u64,
}

#[inline]
fn bump_freq(freq: &mut u8) {
	*freq = (freq.saturating_add(1)).min(MAX_FREQ);
}

#[inline]
fn fingerprint<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

// Set of ghost fingerprints stored in a flat power-of-two table with linear
// probing. Slot value 0 marks an empty slot, so a stored fingerprint of 0 is
// tracked by a separate flag. Removal uses backward-shift deletion, so no
// tombstones accumulate. Sized so the load factor never exceeds one half.
#[derive(Debug)]
struct FpSet {
	slots: Vec<u64>,
	mask: usize,
	len: usize,
	has_zero: bool,
}

impl FpSet {
	// Creates a set that can hold `max_live` fingerprints at once.
	fn new(max_live: usize) -> Self {
		let cap = (2 * max_live.max(1)).next_power_of_two();
		Self {
			slots: vec![0; cap],
			mask: cap - 1,
			len: 0,
			has_zero: false,
		}
	}

	#[inline]
	fn len(&self) -> usize {
		self.len
	}

	#[inline]
	fn home(&self, fp: u64) -> usize {
		(fp as usize) & self.mask
	}

	// Returns the slot holding a nonzero `fp`, if present.
	fn find(&self, fp: u64) -> Option<usize> {
		let mut i = self.home(fp);
		for _ in 0..self.slots.len() {
			let s = self.slots[i];
			if s == fp {
				return Some(i);
			}
			if s == 0 {
				return None;
			}
			i = (i + 1) & self.mask;
		}
		None
	}

	fn contains(&self, fp: u64) -> bool {
		if fp == 0 {
			self.has_zero
		} else {
			self.find(fp).is_some()
		}
	}

	fn insert(&mut self, fp: u64) {
		if fp == 0 {
			if !self.has_zero {
				self.has_zero = true;
				self.len += 1;
			}
			return;
		}
		let mut i = self.home(fp);
		for _ in 0..self.slots.len() {
			let s = self.slots[i];
			if s == fp {
				return;
			}
			if s == 0 {
				self.slots[i] = fp;
				self.len += 1;
				return;
			}
			i = (i + 1) & self.mask;
		}
		debug_assert!(false, "ghost fingerprint table is full");
	}

	fn remove(&mut self, fp: u64) -> bool {
		if fp == 0 {
			if self.has_zero {
				self.has_zero = false;
				self.len -= 1;
				return true;
			}
			return false;
		}
		let mut hole = match self.find(fp) {
			Some(i) => i,
			None => return false,
		};
		self.slots[hole] = 0;
		self.len -= 1;
		// Backward-shift: pull later entries of the probe run into the hole when the
		// hole lies on their probe path from home to their current slot.
		let mut j = hole;
		loop {
			j = (j + 1) & self.mask;
			let s = self.slots[j];
			if s == 0 {
				break;
			}
			let home = self.home(s);
			let dist_hole = hole.wrapping_sub(home) & self.mask;
			let dist_j = j.wrapping_sub(home) & self.mask;
			if dist_hole <= dist_j {
				self.slots[hole] = s;
				self.slots[j] = 0;
				hole = j;
			}
		}
		true
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	small_freq: VecDeque<u8>,
	small_head: u64,
	main: VecDeque<Entry<K, V>>,
	main_freq: VecDeque<u8>,
	main_head: u64,
	index: HashMap<K, Slot>,
	ghost: VecDeque<u64>,
	ghost_set: FpSet,
	ghost_copies: HashMap<u64, u32>,
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 5).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			small_freq: VecDeque::with_capacity(small_size),
			small_head: 0,
			main: VecDeque::with_capacity(main_size),
			main_freq: VecDeque::with_capacity(main_size),
			main_head: 0,
			index: HashMap::with_capacity(capacity),
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: FpSet::new(main_size),
			ghost_copies: HashMap::with_capacity(main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn ghost_push(&mut self, fp: u64) {
		if self.ghost_set.len() >= self.main_size {
			self.ghost_pop_oldest();
		}
		self.ghost.push_front(fp);
		self.ghost_set.insert(fp);
		*self.ghost_copies.entry(fp).or_insert(0) += 1;
		if self.ghost.len() > 2 * (self.main_size + 1) {
			self.ghost_compact();
		}
	}

	// Drops the oldest live ghost fingerprint, discarding stale copies on the way.
	// The live copy of a fingerprint is always its newest queue copy, so it is the
	// last copy reached from the back.
	fn ghost_pop_oldest(&mut self) {
		while let Some(fp) = self.ghost.pop_back() {
			let remaining = match self.ghost_copies.get_mut(&fp) {
				Some(c) => {
					*c -= 1;
					*c
				}
				None => 0,
			};
			if remaining == 0 {
				self.ghost_copies.remove(&fp);
				if self.ghost_set.remove(fp) {
					return;
				}
			}
		}
	}

	// Rebuilds the ghost queue with only the live copy of each fingerprint, keeping
	// recency order. Bounds the queue's length to the live ghost count.
	fn ghost_compact(&mut self) {
		let mut seen: HashSet<u64> = HashSet::with_capacity(self.ghost_set.len());
		let mut kept: VecDeque<u64> = VecDeque::with_capacity(self.ghost_set.len());
		for &fp in self.ghost.iter() {
			if self.ghost_set.contains(fp) && seen.insert(fp) {
				kept.push_back(fp);
			}
		}
		self.ghost = kept;
		self.ghost_copies = self.ghost.iter().map(|&fp| (fp, 1)).collect();
	}
}

impl<K: Eq + Clone + Hash, V> S3Fifo<K, V> {
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = *self.index.get(key)?;
		if slot.in_small {
			let i = (self.small_head - slot.id) as usize;
			bump_freq(&mut self.small_freq[i]);
			Some(&self.small[i].value)
		} else {
			let i = (self.main_head - slot.id) as usize;
			bump_freq(&mut self.main_freq[i]);
			Some(&self.main[i].value)
		}
	}

	fn push_small(&mut self, entry: Entry<K, V>, freq: u8) {
		self.small_head += 1;
		self.index.insert(
			entry.key.clone(),
			Slot {
				in_small: true,
				id: self.small_head,
			},
		);
		self.small.push_front(entry);
		self.small_freq.push_front(freq);
	}

	fn pop_small(&mut self) -> Option<(Entry<K, V>, u8)> {
		let entry = self.small.pop_back()?;
		let freq = self.small_freq.pop_back()?;
		self.index.remove(&entry.key);
		Some((entry, freq))
	}

	fn push_main(&mut self, entry: Entry<K, V>, freq: u8) {
		self.main_head += 1;
		self.index.insert(
			entry.key.clone(),
			Slot {
				in_small: false,
				id: self.main_head,
			},
		);
		self.main.push_front(entry);
		self.main_freq.push_front(freq);
	}

	fn pop_main(&mut self) -> Option<(Entry<K, V>, u8)> {
		let entry = self.main.pop_back()?;
		let freq = self.main_freq.pop_back()?;
		self.index.remove(&entry.key);
		Some((entry, freq))
	}

	fn evict_main(&mut self) {
		while let Some((tail, freq)) = self.pop_main() {
			if freq > 0 {
				self.push_main(tail, freq - 1);
			} else {
				break;
			}
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&slot) = self.index.get(&key) {
			if slot.in_small {
				let i = (self.small_head - slot.id) as usize;
				self.small[i].value = value;
			} else {
				let i = (self.main_head - slot.id) as usize;
				self.main[i].value = value;
			}
			return;
		}

		let fp = fingerprint(&key);
		if self.ghost_set.remove(fp) {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.push_main(Entry::new(key, value), 0);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.push_small(Entry::new(key, value), 0);
	}

	fn evict_small(&mut self) {
		if let Some((tail, freq)) = self.pop_small() {
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.push_main(tail, freq);
			} else {
				let fp = fingerprint(&tail.key);
				self.ghost_push(fp);
			}
		}
	}

	#[cfg(test)]
	fn check_indexes(&self) {
		assert_eq!(self.small.len(), self.small_freq.len());
		assert_eq!(self.main.len(), self.main_freq.len());
		assert_eq!(self.index.len(), self.small.len() + self.main.len());
		for (i, entry) in self.small.iter().enumerate() {
			let slot = self.index.get(&entry.key).expect("small key missing from index");
			assert!(slot.in_small);
			assert_eq!(self.small_head - slot.id, i as u64);
		}
		for (i, entry) in self.main.iter().enumerate() {
			let slot = self.index.get(&entry.key).expect("main key missing from index");
			assert!(!slot.in_small);
			assert_eq!(self.main_head - slot.id, i as u64);
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
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		// Push back-to-front so the deque reads 1, 2, 3, 4 from front to back.
		cache.push_main(Entry::new(4, 40), 1);
		cache.push_main(Entry::new(3, 30), 0);
		cache.push_main(Entry::new(2, 20), 2);
		cache.push_main(Entry::new(1, 10), 0);
		cache.push_small(Entry::new(5, 50), 2);
		cache.check_indexes();
		cache.evict_small();
		cache.check_indexes();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
	}

	#[test]
	fn freq_deques_stay_in_lockstep() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			if cache.get(&(key % 6)).is_none() {
				cache.put(key % 6, key);
			}
			assert_eq!(cache.small.len(), cache.small_freq.len());
			assert_eq!(cache.main.len(), cache.main_freq.len());
		}
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
	fn index_matches_deque_positions_on_seeded_trace() {
		let mut cache = S3Fifo::new(16);
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		for step in 0..5_000u64 {
			state = state
				.wrapping_mul(6364136223846793005)
				.wrapping_add(1442695040888963407);
			let key = (state >> 33) % 40;
			if cache.index.contains_key(&key) {
				if step % 3 == 0 {
					// Resident update path: exercises the index-based value write.
					cache.put(key, step);
				} else {
					assert!(cache.get(&key).is_some());
				}
			} else if cache.get(&key).is_none() {
				cache.put(key, step);
			}
			cache.check_indexes();
		}
	}

	#[test]
	fn fp_set_matches_hash_set() {
		let mut set = FpSet::new(8);
		let mut reference: HashSet<u64> = HashSet::new();
		let mut state: u64 = 0x2545_F491_4F6C_DD1D;
		for _ in 0..20_000 {
			state = state
				.wrapping_mul(6364136223846793005)
				.wrapping_add(1442695040888963407);
			// Small key domain forces collisions, probe wrap-around and the zero key.
			let fp = (state >> 40) % 12;
			if (state >> 20) & 1 == 0 {
				if reference.len() < 8 || reference.contains(&fp) {
					set.insert(fp);
					reference.insert(fp);
				}
			} else {
				assert_eq!(set.remove(fp), reference.remove(&fp));
			}
			assert_eq!(set.len(), reference.len());
			for k in 0..12u64 {
				assert_eq!(set.contains(k), reference.contains(&k));
			}
		}
	}
}
