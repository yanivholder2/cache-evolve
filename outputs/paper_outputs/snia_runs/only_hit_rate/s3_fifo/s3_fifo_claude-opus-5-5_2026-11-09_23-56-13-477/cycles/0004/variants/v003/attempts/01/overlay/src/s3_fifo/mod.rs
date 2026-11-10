use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Variant v001 (cycle 0001, H01): a small-queue entry's frequency is reset to 0
// on entering main so that small-queue hits do not also buy extra second-chance
// passes in main.
//
// Variant v001 (cycle 0002, H01): the graduation threshold is gated on main's
// occupancy. While main has free capacity a single reuse (freq >= 1) suffices
// to graduate (no main eviction is needed); once main is full, two reuses
// (freq >= 2, the baseline rule) are required.
//
// Variant v001 (cycle 0003, H01): identical policy, re-implemented on an O(1)
// index. Entries live in a slab (`Vec<Entry>` + free list); the small and main
// FIFOs hold slab indices; `index: HashMap<K, Loc>` locates every resident and
// ghost key. Ghost entries carry a stamp and are deleted lazily: a ghost hit
// just re-points the index, leaving the deque slot stale; stale slots are
// skipped on ghost eviction and compacted away once the deque exceeds
// `2 * main_size`.
//
// Variant v003 (cycle 0004, H03): small-queue relap. When main is full and the
// small tail has exactly one reuse (freq == 1) and has not relapped yet, it is
// given one more lap in small (freq reset to 0, `relapped = true`, pushed to the
// front of small) instead of being ghosted. A relapped entry graduates on its
// next tail visit with a single further reuse (freq >= 1); otherwise it is
// ghosted. `evict_small` loops until exactly one entry has graduated or been
// ghosted, so capacity accounting is unchanged.
const MAX_FREQ: u8 = 3;

/// Graduation threshold while main has spare room.
const GRADUATE_FREQ_MAIN_ROOM: u8 = 1;
/// Graduation threshold once main is full.
const GRADUATE_FREQ_MAIN_FULL: u8 = 2;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
	relapped: bool,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			freq: 0,
			relapped: false,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
	Small(u32),
	Main(u32),
	Ghost(u64),
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Entry<K, V>>,
	free: Vec<u32>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	ghost: VecDeque<(K, u64)>,
	index: HashMap<K, Loc>,
	ghost_live: usize,
	next_stamp: u64,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			slab: Vec::with_capacity(capacity + main_size),
			free: Vec::new(),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size.max(1)),
			ghost: VecDeque::with_capacity(2 * main_size.max(1) + 1),
			index: HashMap::with_capacity(capacity + main_size),
			ghost_live: 0,
			next_stamp: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = match self.index.get(key)? {
			Loc::Small(i) | Loc::Main(i) => *i as usize,
			Loc::Ghost(_) => return None,
		};
		let entry = &mut self.slab[idx];
		entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		match self.index.get(&key).copied() {
			Some(Loc::Small(i)) | Some(Loc::Main(i)) => {
				self.slab[i as usize].value = value;
			}
			Some(Loc::Ghost(_)) => {
				// Lazy deletion: the ghost deque slot becomes stale once the
				// index is re-pointed below.
				self.ghost_live -= 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let idx = self.alloc(key.clone(), value);
				self.main.push_front(idx);
				self.index.insert(key, Loc::Main(idx));
			}
			None => {
				if self.small.len() >= self.small_size {
					self.evict_small();
				}
				let idx = self.alloc(key.clone(), value);
				self.small.push_front(idx);
				self.index.insert(key, Loc::Small(idx));
			}
		}
	}

	fn alloc(&mut self, key: K, value: V) -> u32 {
		// `Entry::new` starts with freq 0 and relapped = false, so recycled
		// slab slots never inherit a stale lap bit.
		let entry = Entry::new(key, value);
		if let Some(idx) = self.free.pop() {
			self.slab[idx as usize] = entry;
			idx
		} else {
			let idx = self.slab.len() as u32;
			self.slab.push(entry);
			idx
		}
	}

	fn evict_main(&mut self) {
		while let Some(idx) = self.main.pop_back() {
			let entry = &mut self.slab[idx as usize];
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(idx);
			} else {
				self.index.remove(&entry.key);
				self.free.push(idx);
				break;
			}
		}
	}

	/// Graduates or ghosts exactly one small-queue entry (if small is
	/// non-empty). Single-reuse tails may first be relapped once each while
	/// main is full; each entry relaps at most once, so the loop terminates
	/// within `small.len() + 1` iterations.
	fn evict_small(&mut self) {
		while let Some(idx) = self.small.pop_back() {
			let main_full = self.main.len() >= self.main_size;
			let threshold = if main_full {
				GRADUATE_FREQ_MAIN_FULL
			} else {
				GRADUATE_FREQ_MAIN_ROOM
			};
			let (freq, relapped) = {
				let e = &self.slab[idx as usize];
				(e.freq, e.relapped)
			};
			if freq >= threshold || (relapped && freq >= 1) {
				if main_full {
					self.evict_main();
				}
				let entry = &mut self.slab[idx as usize];
				entry.freq = 0;
				if let Some(loc) = self.index.get_mut(&entry.key) {
					*loc = Loc::Main(idx);
				}
				self.main.push_front(idx);
				return;
			} else if main_full && freq == 1 && !relapped {
				let entry = &mut self.slab[idx as usize];
				entry.relapped = true;
				entry.freq = 0;
				// Loc::Small(idx) in the index is unchanged.
				self.small.push_front(idx);
				continue;
			} else {
				let key = self.slab[idx as usize].key.clone();
				self.free.push(idx);
				self.ghost_insert(key);
				return;
			}
		}
	}

	fn ghost_insert(&mut self, key: K) {
		if self.ghost_live >= self.main_size {
			while let Some((old_key, stamp)) = self.ghost.pop_back() {
				if self.index.get(&old_key) == Some(&Loc::Ghost(stamp)) {
					self.index.remove(&old_key);
					self.ghost_live -= 1;
					break;
				}
			}
		}
		let stamp = self.next_stamp;
		self.next_stamp += 1;
		self.ghost.push_front((key.clone(), stamp));
		self.index.insert(key, Loc::Ghost(stamp));
		self.ghost_live += 1;

		if self.ghost.len() > 2 * self.main_size.max(1) {
			let index = &self.index;
			self
				.ghost
				.retain(|(k, s)| index.get(k) == Some(&Loc::Ghost(*s)));
		}
	}
}

impl<K: Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
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

	impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
		fn in_main(&self, key: &K) -> bool {
			matches!(self.index.get(key), Some(Loc::Main(_)))
		}

		fn in_small(&self, key: &K) -> bool {
			matches!(self.index.get(key), Some(Loc::Small(_)))
		}

		fn in_ghost(&self, key: &K) -> bool {
			matches!(self.index.get(key), Some(Loc::Ghost(_)))
		}

		fn freq_of(&self, key: &K) -> Option<u8> {
			match self.index.get(key)? {
				Loc::Small(i) | Loc::Main(i) => Some(self.slab[*i as usize].freq),
				Loc::Ghost(_) => None,
			}
		}

		fn relapped_of(&self, key: &K) -> Option<bool> {
			match self.index.get(key)? {
				Loc::Small(i) | Loc::Main(i) => Some(self.slab[*i as usize].relapped),
				Loc::Ghost(_) => None,
			}
		}

		fn push_main_back(&mut self, key: K, value: V, freq: u8) {
			let idx = self.alloc(key.clone(), value);
			self.slab[idx as usize].freq = freq;
			self.main.push_back(idx);
			self.index.insert(key, Loc::Main(idx));
		}

		fn push_small_front(&mut self, key: K, value: V, freq: u8) {
			let idx = self.alloc(key.clone(), value);
			self.slab[idx as usize].freq = freq;
			self.small.push_front(idx);
			self.index.insert(key, Loc::Small(idx));
		}

		fn resident(&self) -> usize {
			self.small.len() + self.main.len()
		}

		fn live_ghosts_in_deque(&self) -> usize {
			self
				.ghost
				.iter()
				.filter(|(k, s)| self.index.get(k) == Some(&Loc::Ghost(*s)))
				.count()
		}
	}

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
		assert!(cache.in_main(&1));
	}

	#[test]
	fn small_eviction_promotes_to_main_after_single_hit_with_reset_freq() {
		// Main is empty here, so the single-reuse threshold applies.
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20);
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert!(!cache.in_ghost(&1));
	}

	#[test]
	fn small_tail_with_single_hit_goes_to_ghost_when_main_full() {
		// capacity 4 -> small_size 1, main_size 3.
		let mut cache = S3Fifo::new(4);
		cache.push_main_back(10, 100, 0);
		cache.push_main_back(11, 110, 0);
		cache.push_main_back(12, 120, 0);
		assert_eq!(cache.main.len(), cache.main_size);
		cache.push_small_front(1, 10, 1);
		// Small holds only this entry: the first visit relaps it (freq -> 0),
		// the second visit finds it relapped with no hit and ghosts it.
		cache.evict_small();
		assert!(!cache.in_main(&1));
		assert!(cache.in_ghost(&1));
		assert!(cache.small.is_empty());
		assert_eq!(cache.main.len(), 3);
		// A later re-access re-admits it into main via the ghost-hit path.
		cache.put(1, 10);
		assert!(cache.in_main(&1));
		assert!(!cache.in_ghost(&1));
		assert!(cache.resident() <= 4);
	}

	#[test]
	fn single_hit_tail_relaps_and_next_tail_is_examined() {
		// capacity 8 -> small_size 2, main_size 6.
		let mut cache = S3Fifo::new(8);
		for k in 10..16 {
			cache.push_main_back(k, k, 0);
		}
		cache.push_small_front(1, 10, 1); // tail
		cache.push_small_front(2, 20, 0);
		cache.evict_small();
		// 1 relapped and stayed in small; 2 was ghosted.
		assert!(cache.in_small(&1));
		assert_eq!(cache.relapped_of(&1), Some(true));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert!(cache.in_ghost(&2));
		assert_eq!(cache.small.len(), 1);
		assert_eq!(cache.main.len(), 6);
	}

	#[test]
	fn relapped_key_with_one_hit_graduates() {
		// capacity 8 -> small_size 2, main_size 6.
		let mut cache = S3Fifo::new(8);
		for k in 10..16 {
			cache.push_main_back(k, k, 0);
		}
		cache.push_small_front(1, 10, 1);
		cache.push_small_front(2, 20, 0);
		cache.evict_small();
		assert!(cache.in_small(&1));
		assert_eq!(cache.relapped_of(&1), Some(true));
		// One further reuse during the extra lap.
		assert_eq!(*cache.get(&1).unwrap(), 10);
		cache.push_small_front(3, 30, 0);
		cache.evict_small();
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert_eq!(cache.main.len(), 6);
		assert!(cache.in_small(&3));
		assert!(cache.resident() <= 8);
	}

	#[test]
	fn relapped_key_without_hit_is_ghosted() {
		let mut cache = S3Fifo::new(8);
		for k in 10..16 {
			cache.push_main_back(k, k, 0);
		}
		cache.push_small_front(1, 10, 1);
		cache.push_small_front(2, 20, 0);
		cache.evict_small();
		assert!(cache.in_small(&1));
		cache.push_small_front(3, 30, 0);
		cache.evict_small();
		assert!(cache.in_ghost(&1));
		assert!(cache.in_small(&3));
		assert_eq!(cache.main.len(), 6);
	}

	#[test]
	fn recycled_slot_clears_relapped() {
		let mut cache = S3Fifo::new(4);
		for k in 10..13 {
			cache.push_main_back(k, k, 0);
		}
		cache.push_small_front(1, 10, 1);
		cache.evict_small();
		assert!(cache.in_ghost(&1));
		cache.put(5, 50);
		assert_eq!(cache.relapped_of(&5), Some(false));
	}

	#[test]
	fn small_tail_with_two_hits_graduates_when_main_full() {
		let mut cache = S3Fifo::new(4);
		cache.push_main_back(10, 100, 0);
		cache.push_main_back(11, 110, 0);
		cache.push_main_back(12, 120, 0);
		cache.push_small_front(1, 10, 2);
		cache.evict_small();
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert_eq!(cache.main.len(), 3);
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.in_main(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.push_main_back(1, 10, 0);
		cache.push_main_back(2, 20, 2);
		cache.push_main_back(3, 30, 0);
		cache.push_main_back(4, 40, 1);
		cache.push_small_front(5, 50, 2);
		cache.evict_small();
		assert!(cache.in_main(&4));
		assert!(!cache.in_main(&3));
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.resident() <= 4);
	}

	#[test]
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert_eq!(cache.freq_of(&1), Some(MAX_FREQ));
	}

	#[test]
	fn ghost_compaction_bound_holds() {
		let capacity = 8;
		let mut cache = S3Fifo::<i64, i64>::new(capacity);
		let bound = 2 * cache.main_size.max(1);
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..20_000 {
			state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
			let key = ((state >> 33) % 24) as i64;
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
			assert!(cache.ghost.len() <= bound, "ghost deque {} > {bound}", cache.ghost.len());
			assert!(cache.ghost_live <= cache.main_size);
			assert_eq!(cache.ghost_live, cache.live_ghosts_in_deque());
			assert!(cache.resident() <= capacity);
			assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
			assert_eq!(cache.slab.len() - cache.free.len(), cache.resident());
		}
	}

	/// Linear-scan reference model of the policy (cycle-0002 v001 plus the
	/// cycle-0004 H03 relap rule), used as an oracle for hit/miss equivalence.
	struct Reference {
		small: VecDeque<Entry<i64, i64>>,
		main: VecDeque<Entry<i64, i64>>,
		ghost: VecDeque<i64>,
		small_size: usize,
		main_size: usize,
	}

	impl Reference {
		fn new(capacity: usize) -> Self {
			let small_size = if capacity == 1 {
				1
			} else {
				(capacity / 4).max(1).min(capacity - 1)
			};
			Self {
				small: VecDeque::new(),
				main: VecDeque::new(),
				ghost: VecDeque::new(),
				small_size,
				main_size: capacity - small_size,
			}
		}

		fn get(&mut self, key: &i64) -> Option<i64> {
			let entry = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| &e.key == key)?;
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			Some(entry.value)
		}

		fn put(&mut self, key: i64, value: i64) {
			if let Some(e) = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| e.key == key) {
				e.value = value;
				return;
			}
			if self.ghost.iter().any(|g| g == &key) {
				self.ghost.retain(|g| g != &key);
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(Entry::new(key, value));
				return;
			}
			if self.small.len() >= self.small_size {
				self.evict_small();
			}
			self.small.push_front(Entry::new(key, value));
		}

		fn evict_main(&mut self) {
			while let Some(mut tail) = self.main.pop_back() {
				if tail.freq > 0 {
					tail.freq -= 1;
					self.main.push_front(tail);
				} else {
					break;
				}
			}
		}

		fn evict_small(&mut self) {
			while let Some(mut tail) = self.small.pop_back() {
				let main_full = self.main.len() >= self.main_size;
				let threshold = if main_full { GRADUATE_FREQ_MAIN_FULL } else { GRADUATE_FREQ_MAIN_ROOM };
				if tail.freq >= threshold || (tail.relapped && tail.freq >= 1) {
					if main_full {
						self.evict_main();
					}
					tail.freq = 0;
					self.main.push_front(tail);
					return;
				} else if main_full && tail.freq == 1 && !tail.relapped {
					tail.relapped = true;
					tail.freq = 0;
					self.small.push_front(tail);
				} else {
					if self.ghost.len() >= self.main_size {
						self.ghost.pop_back();
					}
					self.ghost.push_front(tail.key);
					return;
				}
			}
		}
	}

	#[test]
	fn hit_miss_identical_to_linear_reference() {
		for &capacity in &[1usize, 2, 3, 4, 5, 8, 16, 33] {
			for &range in &[capacity as u64 + 1, 2 * capacity as u64 + 3, 8 * capacity as u64] {
				let mut fast = S3Fifo::<i64, i64>::new(capacity);
				let mut slow = Reference::new(capacity);
				let mut state: u64 = 0xDEAD_BEEF ^ (capacity as u64 * 131 + range);
				for step in 0..5_000 {
					state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					// Skewed keys: square a uniform draw to favour small keys.
					let u = (state >> 33) % range;
					let key = ((u * u) / range) as i64;
					let a = fast.get(&key).copied();
					let b = slow.get(&key);
					assert_eq!(a, b, "divergence cap={capacity} range={range} step={step} key={key}");
					if a.is_none() {
						fast.put(key, key);
						slow.put(key, key);
					}
				}
			}
		}
	}
}
