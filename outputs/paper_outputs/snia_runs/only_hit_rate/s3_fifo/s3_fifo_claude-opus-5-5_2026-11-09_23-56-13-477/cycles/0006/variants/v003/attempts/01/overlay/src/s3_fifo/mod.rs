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
// Variant v001 (cycle 0004, H01): each ghost remembers the small-queue freq the
// key had when it was ghosted. A ghost hit with freq >= 1 (the key was reused in
// small) is admitted straight into main as before; a ghost hit with freq == 0
// (never reused) goes back into small with freq 1, so one further reuse
// graduates it, and if it is ghosted again it carries freq 1.
//
// Variant v002 (cycle 0005, H02): a freq-0 ghost returnee whose own ghost age
// (`next_stamp - stamp`, i.e. small->ghost evictions since it was ghosted) is
// within about one small-queue turnover (`small_size / GHOST_RECENT_AGE_DIVISOR`)
// is treated as a strong returnee and admitted straight into main. Older freq-0
// returnees keep the cycle-0004 path back through small.
//
// Variant v003 (cycle 0006, H03): mirror-image age ceiling on the freq >= 1
// branch. A freq >= 1 ghost returnee older than half the ghost window
// (`main_size / GHOST_STALE_AGE_DIVISOR`) is no longer admitted to main on its
// freq alone; unless it is also within the recent window it takes the weak arm
// back into small with freq 1.
const MAX_FREQ: u8 = 3;

/// Graduation threshold while main has spare room.
const GRADUATE_FREQ_MAIN_ROOM: u8 = 1;
/// Graduation threshold once main is full.
const GRADUATE_FREQ_MAIN_FULL: u8 = 2;
/// Minimum ghosted freq for a ghost hit to be admitted directly into main.
const GHOST_TO_MAIN_FREQ: u8 = 1;
/// Freq given to a weak ghost returnee re-admitted into small.
const GHOST_RETURN_SMALL_FREQ: u8 = 1;
/// Recency window for freq-0 ghost returnees is `small_size / this`.
const GHOST_RECENT_AGE_DIVISOR: usize = 1;
/// Age ceiling for freq >= 1 ghost returnees is `main_size / this`.
const GHOST_STALE_AGE_DIVISOR: usize = 2;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value, freq: 0 }
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
	Small(u32),
	Main(u32),
	Ghost { stamp: u64, freq: u8 },
}

/// True iff `loc` is a live ghost with stamp `s` (freq is ignored).
#[inline]
fn ghost_stamp_matches(loc: Option<&Loc>, s: u64) -> bool {
	matches!(loc, Some(Loc::Ghost { stamp, .. }) if *stamp == s)
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

	#[inline]
	fn ghost_recent_window(&self) -> u64 {
		(self.small_size / GHOST_RECENT_AGE_DIVISOR).max(1) as u64
	}

	#[inline]
	fn ghost_stale_age(&self) -> u64 {
		(self.main_size / GHOST_STALE_AGE_DIVISOR).max(1) as u64
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = match self.index.get(key)? {
			Loc::Small(i) | Loc::Main(i) => *i as usize,
			Loc::Ghost { .. } => return None,
		};
		let entry = &mut self.slab[idx];
		entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let recent = self.ghost_recent_window();
		let stale = self.ghost_stale_age();
		match self.index.get(&key).copied() {
			Some(Loc::Small(i)) | Some(Loc::Main(i)) => {
				self.slab[i as usize].value = value;
			}
			Some(Loc::Ghost { stamp, freq }) => {
				let age = self.next_stamp - stamp;
				if (freq >= GHOST_TO_MAIN_FREQ && age <= stale) || age <= recent {
					// Strong returnee (reused in small and not stale, or returned
					// within about one small-queue turnover). Lazy deletion: the
					// ghost deque slot becomes stale once the index is re-pointed.
					self.ghost_live -= 1;
					if self.main.len() >= self.main_size {
						self.evict_main();
					}
					let idx = self.alloc(key.clone(), value);
					self.main.push_front(idx);
					self.index.insert(key, Loc::Main(idx));
				} else {
					// Weak or stale returnee: drop the ghost record *before*
					// evict_small so its (now stale) deque slot can't be mistaken
					// for a live ghost by ghost_insert's eviction loop.
					self.index.remove(&key);
					self.ghost_live -= 1;
					if self.small.len() >= self.small_size {
						self.evict_small();
					}
					let idx = self.alloc(key.clone(), value);
					self.slab[idx as usize].freq = GHOST_RETURN_SMALL_FREQ;
					self.small.push_front(idx);
					self.index.insert(key, Loc::Small(idx));
				}
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

	fn evict_small(&mut self) {
		if let Some(idx) = self.small.pop_back() {
			let main_full = self.main.len() >= self.main_size;
			let threshold = if main_full {
				GRADUATE_FREQ_MAIN_FULL
			} else {
				GRADUATE_FREQ_MAIN_ROOM
			};
			if self.slab[idx as usize].freq >= threshold {
				if main_full {
					self.evict_main();
				}
				let entry = &mut self.slab[idx as usize];
				entry.freq = 0;
				if let Some(loc) = self.index.get_mut(&entry.key) {
					*loc = Loc::Main(idx);
				}
				self.main.push_front(idx);
			} else {
				let entry = &self.slab[idx as usize];
				let key = entry.key.clone();
				let freq = entry.freq;
				self.free.push(idx);
				self.ghost_insert(key, freq);
			}
		}
	}

	fn ghost_insert(&mut self, key: K, freq: u8) {
		if self.ghost_live >= self.main_size {
			while let Some((old_key, stamp)) = self.ghost.pop_back() {
				if ghost_stamp_matches(self.index.get(&old_key), stamp) {
					self.index.remove(&old_key);
					self.ghost_live -= 1;
					break;
				}
			}
		}
		let stamp = self.next_stamp;
		self.next_stamp += 1;
		self.ghost.push_front((key.clone(), stamp));
		self.index.insert(key, Loc::Ghost { stamp, freq });
		self.ghost_live += 1;

		if self.ghost.len() > 2 * self.main_size.max(1) {
			let index = &self.index;
			self
				.ghost
				.retain(|(k, s)| ghost_stamp_matches(index.get(k), *s));
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
			matches!(self.index.get(key), Some(Loc::Ghost { .. }))
		}

		fn ghost_freq_of(&self, key: &K) -> Option<u8> {
			match self.index.get(key)? {
				Loc::Ghost { freq, .. } => Some(*freq),
				_ => None,
			}
		}

		fn ghost_age_of(&self, key: &K) -> Option<u64> {
			match self.index.get(key)? {
				Loc::Ghost { stamp, .. } => Some(self.next_stamp - *stamp),
				_ => None,
			}
		}

		fn freq_of(&self, key: &K) -> Option<u8> {
			match self.index.get(key)? {
				Loc::Small(i) | Loc::Main(i) => Some(self.slab[*i as usize].freq),
				Loc::Ghost { .. } => None,
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
				.filter(|(k, s)| ghost_stamp_matches(self.index.get(k), *s))
				.count()
		}
	}

	fn fill_main(cache: &mut S3Fifo<i64, i64>) {
		// capacity 4 -> small_size 1, main_size 3.
		cache.push_main_back(10, 100, 0);
		cache.push_main_back(11, 110, 0);
		cache.push_main_back(12, 120, 0);
		assert_eq!(cache.main.len(), cache.main_size);
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
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.push_small_front(1, 10, 1);
		cache.evict_small();
		assert!(!cache.in_main(&1));
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(1));
		assert_eq!(cache.main.len(), 3);
		// Ghosted with freq 1 and age 1 (<= stale age 1) -> a later re-access
		// re-admits it into main.
		assert!(cache.ghost_age_of(&1).unwrap() <= cache.ghost_stale_age());
		cache.put(1, 10);
		assert!(cache.in_main(&1));
		assert!(!cache.in_ghost(&1));
		assert!(cache.resident() <= 4);
	}

	#[test]
	fn ghosted_with_freq_one_returns_into_main() {
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20); // evicts 1 from small with freq 1 -> ghost.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(1));
		assert!(cache.ghost_age_of(&1).unwrap() <= cache.ghost_stale_age());
		cache.put(1, 10);
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert!(cache.in_small(&2));
		assert_eq!(cache.main.len(), 3);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
	}

	#[test]
	fn stale_freq_one_ghost_returnee_goes_to_small_with_freq_one() {
		// capacity 4 -> main_size 3 -> stale age 1; small_size 1 -> recent 1.
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20); // 1 ghosted with freq 1 (stamp 0).
		cache.put(3, 30); // 2 ghosted (stamp 1); 1's age is now 2.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(1));
		let age = cache.ghost_age_of(&1).unwrap();
		assert!(age > cache.ghost_stale_age());
		assert!(age > cache.ghost_recent_window());
		cache.put(1, 10); // stale freq-1 returnee -> small with freq 1.
		assert!(cache.in_small(&1));
		assert!(!cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(GHOST_RETURN_SMALL_FREQ));
		assert!(cache.in_ghost(&3));
		assert_eq!(cache.main.len(), 3);
		assert!(cache.resident() <= 4);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
		// One further reuse graduates it under the main-full freq >= 2 rule.
		cache.get(&1);
		cache.put(4, 40);
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
	}

	#[test]
	fn freq_one_ghost_returnee_within_stale_age_goes_to_main() {
		// capacity 16 -> small_size 4 (recent 4), main_size 12 (stale 6).
		let mut cache = S3Fifo::<i64, i64>::new(16);
		for k in 100..112 {
			cache.push_main_back(k, k, 0);
		}
		cache.put(1, 1);
		cache.get(&1);
		for k in 2..6 {
			cache.put(k, k); // fills small; the 4th evicts 1 with freq 1.
		}
		assert!(cache.in_ghost(&1));
		for k in 6..11 {
			cache.put(k, k); // each ghosts one more small key.
		}
		let age = cache.ghost_age_of(&1).unwrap();
		assert!(age > cache.ghost_recent_window());
		assert!(age <= cache.ghost_stale_age());
		cache.put(1, 1);
		assert!(cache.in_main(&1));
		assert!(cache.resident() <= 16);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
	}

	#[test]
	fn recent_freq_zero_ghost_returnee_goes_to_main() {
		// capacity 4 -> small_size 1 -> recent window 1.
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.put(2, 20); // 1 ghosted with freq 0; age now 1.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(0));
		assert!(cache.next_stamp - 0 <= cache.ghost_recent_window());
		cache.put(1, 10); // recent -> straight to main.
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert!(cache.in_small(&2));
		assert_eq!(cache.main.len(), 3);
		assert!(cache.resident() <= 4);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
	}

	#[test]
	fn old_freq_zero_ghost_returnee_goes_to_small_with_freq_one() {
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.put(2, 20); // 1 ghosted (stamp 0).
		cache.put(3, 30); // 2 ghosted (stamp 1); 1's age is now 2 > 1.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(0));
		cache.put(1, 10);
		assert!(cache.in_small(&1));
		assert_eq!(cache.freq_of(&1), Some(1));
		assert!(cache.in_ghost(&3));
		assert_eq!(cache.main.len(), 3);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
	}

	#[test]
	fn weak_ghost_returnee_lands_in_small_then_graduates_after_one_get() {
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.put(2, 20); // evicts 1 from small with freq 0 -> ghost.
		cache.put(3, 30); // ages 1 past the recent window.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(0));
		cache.put(1, 10); // old weak returnee -> small with freq 1; 3 is ghosted.
		assert!(cache.in_small(&1));
		assert_eq!(cache.freq_of(&1), Some(1));
		assert!(cache.in_ghost(&3));
		assert_eq!(cache.main.len(), 3);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
		cache.get(&1); // freq 2 -> meets main-full threshold.
		cache.put(4, 40);
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert_eq!(cache.main.len(), 3);
		assert!(cache.resident() <= 4);
	}

	#[test]
	fn weak_ghost_returnee_reghosted_with_freq_one() {
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(1, 10); // 1 back in small with freq 1 (old returnee).
		assert!(cache.in_small(&1));
		cache.put(4, 40); // 1 evicted without reuse -> ghost with freq 1.
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(1));
		// Quick return (age 1 <= stale age 1) -> main.
		assert!(cache.ghost_age_of(&1).unwrap() <= cache.ghost_stale_age());
		cache.put(1, 10);
		assert!(cache.in_main(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		// capacity 4: key 1 is ghosted with freq 0 and is aged 3 (> window 1)
		// when it returns, so it goes back into small.
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		assert!(cache.in_ghost(&1));
		assert_eq!(cache.ghost_freq_of(&1), Some(0));
		cache.put(1, 10);
		assert!(cache.in_small(&1));
		assert!(!cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(1));
		assert!(cache.resident() <= 4);
		assert_eq!(cache.index.len(), cache.resident() + cache.ghost_live);
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
		for &capacity in &[8usize, 16, 33] {
			let mut cache = S3Fifo::<i64, i64>::new(capacity);
			let bound = 2 * cache.main_size.max(1);
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ capacity as u64;
			for _ in 0..20_000 {
				state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let key = ((state >> 33) % (3 * capacity as u64)) as i64;
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
	}

	/// Linear-scan reference model of the policy (cycle-0002 v001 plus the
	/// cycle-0004 freq-gated ghost return plus the cycle-0005 ghost-age rule
	/// plus the cycle-0006 stale-age ceiling), used as a hit/miss oracle.
	struct Reference {
		small: VecDeque<Entry<i64, i64>>,
		main: VecDeque<Entry<i64, i64>>,
		ghost: VecDeque<(i64, u8, u64)>,
		next_stamp: u64,
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
				next_stamp: 0,
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
			if let Some(&(_, gfreq, gstamp)) = self.ghost.iter().find(|(g, _, _)| g == &key) {
				self.ghost.retain(|(g, _, _)| g != &key);
				let recent = (self.small_size / GHOST_RECENT_AGE_DIVISOR).max(1) as u64;
				let stale = (self.main_size / GHOST_STALE_AGE_DIVISOR).max(1) as u64;
				let age = self.next_stamp - gstamp;
				if (gfreq >= GHOST_TO_MAIN_FREQ && age <= stale) || age <= recent {
					if self.main.len() >= self.main_size {
						self.evict_main();
					}
					self.main.push_front(Entry::new(key, value));
				} else {
					if self.small.len() >= self.small_size {
						self.evict_small();
					}
					let mut e = Entry::new(key, value);
					e.freq = GHOST_RETURN_SMALL_FREQ;
					self.small.push_front(e);
				}
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
			if let Some(mut tail) = self.small.pop_back() {
				let main_full = self.main.len() >= self.main_size;
				let threshold = if main_full { GRADUATE_FREQ_MAIN_FULL } else { GRADUATE_FREQ_MAIN_ROOM };
				if tail.freq >= threshold {
					if main_full {
						self.evict_main();
					}
					tail.freq = 0;
					self.main.push_front(tail);
				} else {
					if self.ghost.len() >= self.main_size {
						self.ghost.pop_back();
					}
					let stamp = self.next_stamp;
					self.next_stamp += 1;
					self.ghost.push_front((tail.key, tail.freq, stamp));
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
