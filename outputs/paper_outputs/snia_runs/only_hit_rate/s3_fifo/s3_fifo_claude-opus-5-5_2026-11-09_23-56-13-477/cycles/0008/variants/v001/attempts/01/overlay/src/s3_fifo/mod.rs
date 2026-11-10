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
// (`next_stamp - stamp`, i.e. ghost insertions since it was ghosted) is
// within about one small-queue turnover (`small_size / GHOST_RECENT_AGE_DIVISOR`)
// is treated as a strong returnee and admitted straight into main. Older freq-0
// returnees keep the cycle-0004 path back through small.
//
// Variant v001 (cycle 0007, H01): main-queue victims that were hit at least once
// during their current main residence (`main_hit`) leave a ghost, so a return
// re-enters main directly instead of re-qualifying through small. Never-hit main
// victims still leave no trace.
//
// Variant v001 (cycle 0008, H01): main-victim ghosts live in their own FIFO
// (`main_ghost`) with their own budget (`max(1, small_size)`) and their own
// stamp counter (`main_ghost_stamp`). The small-victim ghost, its `main_size`
// bound and its `next_stamp` clock are exactly the cycle-0005 v002 ones: only
// small->ghost insertions advance `next_stamp`, so the freq-0 recency window is
// unaffected by main evictions.
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

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
	freq: u8,
	/// Set when the entry is hit while resident in main; reset on main entry.
	main_hit: bool,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self {
			key,
			value,
			freq: 0,
			main_hit: false,
		}
	}
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Loc {
	Small(u32),
	Main(u32),
	/// Small-queue victim ghost (stamped with `next_stamp`).
	Ghost { stamp: u64, freq: u8 },
	/// Hit main-queue victim ghost (stamped with `main_ghost_stamp`).
	MainGhost { stamp: u64 },
}

const _: () = assert!(std::mem::size_of::<Loc>() <= 16);

/// True iff `loc` is a live small-victim ghost with stamp `s` (freq is ignored).
#[inline]
fn ghost_stamp_matches(loc: Option<&Loc>, s: u64) -> bool {
	matches!(loc, Some(Loc::Ghost { stamp, .. }) if *stamp == s)
}

/// True iff `loc` is a live main-victim ghost with stamp `s`.
#[inline]
fn main_ghost_stamp_matches(loc: Option<&Loc>, s: u64) -> bool {
	matches!(loc, Some(Loc::MainGhost { stamp }) if *stamp == s)
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Entry<K, V>>,
	free: Vec<u32>,
	small: VecDeque<u32>,
	main: VecDeque<u32>,
	ghost: VecDeque<(K, u64)>,
	main_ghost: VecDeque<(K, u64)>,
	index: HashMap<K, Loc>,
	ghost_live: usize,
	main_ghost_live: usize,
	next_stamp: u64,
	main_ghost_stamp: u64,
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
		let main_ghost_budget = small_size.max(1);
		Some(Self {
			slab: Vec::with_capacity(capacity + main_size),
			free: Vec::new(),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size.max(1)),
			ghost: VecDeque::with_capacity(2 * main_size.max(1) + 1),
			main_ghost: VecDeque::with_capacity(2 * main_ghost_budget + 1),
			index: HashMap::with_capacity(capacity + main_size + main_ghost_budget),
			ghost_live: 0,
			main_ghost_live: 0,
			next_stamp: 0,
			main_ghost_stamp: 0,
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

	/// Live-entry budget of the main-victim ghost FIFO.
	#[inline]
	fn main_ghost_budget(&self) -> usize {
		self.small_size.max(1)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let (idx, in_main) = match self.index.get(key)? {
			Loc::Small(i) => (*i as usize, false),
			Loc::Main(i) => (*i as usize, true),
			Loc::Ghost { .. } | Loc::MainGhost { .. } => return None,
		};
		let entry = &mut self.slab[idx];
		entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
		if in_main {
			entry.main_hit = true;
		}
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		let recent = self.ghost_recent_window();
		match self.index.get(&key).copied() {
			Some(Loc::Small(i)) | Some(Loc::Main(i)) => {
				self.slab[i as usize].value = value;
			}
			Some(Loc::MainGhost { .. }) => {
				// Returning hit main victim. Drop the record *before* evict_main:
				// evict_main may insert into main_ghost, and its eviction loop must
				// not mistake this key's (now stale) deque slot for a live entry.
				self.index.remove(&key);
				self.main_ghost_live -= 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let idx = self.alloc(key.clone(), value);
				self.main.push_front(idx);
				self.index.insert(key, Loc::Main(idx));
			}
			Some(Loc::Ghost { stamp, freq })
				if freq >= GHOST_TO_MAIN_FREQ || self.next_stamp - stamp <= recent =>
			{
				// Strong returnee (reused in small, or returned within about one
				// small-queue turnover). Drop the ghost record before evict_main.
				self.index.remove(&key);
				self.ghost_live -= 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let idx = self.alloc(key.clone(), value);
				self.main.push_front(idx);
				self.index.insert(key, Loc::Main(idx));
			}
			Some(Loc::Ghost { .. }) => {
				// Weak, old returnee: drop the ghost record *before* evict_small
				// so its (now stale) deque slot can't be mistaken for a live
				// ghost by ghost_insert's eviction loop.
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
				if entry.main_hit {
					// Hit during its main residence: leave a main-victim ghost so a
					// return re-enters main directly. main_ghost_insert overwrites
					// the index entry with Loc::MainGhost.
					let key = entry.key.clone();
					self.free.push(idx);
					self.main_ghost_insert(key);
				} else {
					self.index.remove(&entry.key);
					self.free.push(idx);
				}
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
				entry.main_hit = false;
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

	/// Small-victim ghost insertion; the only place `next_stamp` advances.
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

	/// Main-victim ghost insertion. Independent of `ghost`, `ghost_live` and
	/// `next_stamp`.
	fn main_ghost_insert(&mut self, key: K) {
		let budget = self.main_ghost_budget();
		if self.main_ghost_live >= budget {
			while let Some((old_key, stamp)) = self.main_ghost.pop_back() {
				if main_ghost_stamp_matches(self.index.get(&old_key), stamp) {
					self.index.remove(&old_key);
					self.main_ghost_live -= 1;
					break;
				}
			}
		}
		let stamp = self.main_ghost_stamp;
		self.main_ghost_stamp += 1;
		self.main_ghost.push_front((key.clone(), stamp));
		self.index.insert(key, Loc::MainGhost { stamp });
		self.main_ghost_live += 1;

		if self.main_ghost.len() > 2 * budget {
			let index = &self.index;
			self
				.main_ghost
				.retain(|(k, s)| main_ghost_stamp_matches(index.get(k), *s));
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

		fn in_main_ghost(&self, key: &K) -> bool {
			matches!(self.index.get(key), Some(Loc::MainGhost { .. }))
		}

		fn ghost_freq_of(&self, key: &K) -> Option<u8> {
			match self.index.get(key)? {
				Loc::Ghost { freq, .. } => Some(*freq),
				_ => None,
			}
		}

		fn freq_of(&self, key: &K) -> Option<u8> {
			match self.index.get(key)? {
				Loc::Small(i) | Loc::Main(i) => Some(self.slab[*i as usize].freq),
				Loc::Ghost { .. } | Loc::MainGhost { .. } => None,
			}
		}

		fn main_hit_of(&self, key: &K) -> Option<bool> {
			match self.index.get(key)? {
				Loc::Small(i) | Loc::Main(i) => Some(self.slab[*i as usize].main_hit),
				Loc::Ghost { .. } | Loc::MainGhost { .. } => None,
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

		fn live_main_ghosts_in_deque(&self) -> usize {
			self
				.main_ghost
				.iter()
				.filter(|(k, s)| main_ghost_stamp_matches(self.index.get(k), *s))
				.count()
		}

		fn assert_accounting(&self) {
			assert_eq!(
				self.index.len(),
				self.resident() + self.ghost_live + self.main_ghost_live
			);
			assert!(self.ghost_live <= self.main_size);
			assert!(self.main_ghost_live <= self.main_ghost_budget());
			assert_eq!(self.ghost_live, self.live_ghosts_in_deque());
			assert_eq!(self.main_ghost_live, self.live_main_ghosts_in_deque());
			assert_eq!(self.slab.len() - self.free.len(), self.resident());
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
	fn loc_fits_in_16_bytes() {
		assert!(std::mem::size_of::<Loc>() <= 16);
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
		assert_eq!(cache.main_hit_of(&1), Some(false));
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
		// Ghosted with freq 1 -> a later re-access re-admits it into main.
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
		cache.put(1, 10);
		assert!(cache.in_main(&1));
		assert_eq!(cache.freq_of(&1), Some(0));
		assert!(cache.in_small(&2));
		assert_eq!(cache.main.len(), 3);
		cache.assert_accounting();
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
		cache.assert_accounting();
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
		cache.assert_accounting();
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
		cache.assert_accounting();
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
		cache.assert_accounting();
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
	fn small_hit_does_not_set_main_hit() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		assert!(cache.in_small(&1));
		assert_eq!(cache.main_hit_of(&1), Some(false));
		fill_main(&mut cache);
		cache.get(&10);
		assert_eq!(cache.main_hit_of(&10), Some(true));
	}

	#[test]
	fn hit_main_victim_becomes_main_ghost_and_returns_to_main() {
		// capacity 4 -> main_size 3. Tail order (evicted first): 12, 11, 10.
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		assert!(cache.get(&12).is_some()); // freq 1, main_hit.
		cache.evict_main(); // 12 decays to 0 and moves to front; 11 dropped.
		assert!(cache.index.get(&11).is_none(), "never-hit main victim must leave no ghost");
		assert_eq!(cache.main_ghost_live, 0);
		cache.evict_main(); // 10 dropped, no ghost.
		assert!(cache.index.get(&10).is_none());
		assert_eq!(cache.main_ghost_live, 0);
		cache.evict_main(); // 12: freq 0 but main_hit -> main ghost.
		assert!(cache.in_main_ghost(&12));
		assert!(!cache.in_ghost(&12));
		assert!(cache.get(&12).is_none());
		assert_eq!(cache.main_ghost_live, 1);
		assert_eq!(cache.ghost_live, 0);
		assert_eq!(cache.next_stamp, 0, "main ghosts must not advance next_stamp");
		assert_eq!(cache.main.len(), 0);
		cache.assert_accounting();
		cache.put(12, 120);
		assert!(cache.in_main(&12));
		assert_eq!(cache.freq_of(&12), Some(0));
		assert_eq!(cache.main_hit_of(&12), Some(false));
		assert_eq!(cache.main_ghost_live, 0);
		cache.assert_accounting();
	}

	#[test]
	fn main_ghost_budget_evicts_oldest_main_ghost_only() {
		// capacity 4 -> budget 1.
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		cache.push_small_front(1, 10, 0);
		cache.evict_small(); // small ghost 1 (stamp 0).
		assert!(cache.in_ghost(&1));
		cache.get(&12);
		cache.get(&11);
		let i12 = match cache.index.get(&12) {
			Some(Loc::Main(i)) => *i as usize,
			_ => unreachable!(),
		};
		let i11 = match cache.index.get(&11) {
			Some(Loc::Main(i)) => *i as usize,
			_ => unreachable!(),
		};
		cache.slab[i12].freq = 0;
		cache.slab[i11].freq = 0;
		cache.evict_main(); // 12 -> main ghost.
		assert!(cache.in_main_ghost(&12));
		cache.evict_main(); // 11 -> main ghost; budget 1 evicts 12.
		assert!(cache.in_main_ghost(&11));
		assert!(cache.index.get(&12).is_none());
		assert!(cache.in_ghost(&1), "small ghost must be untouched");
		assert_eq!(cache.ghost_live, 1);
		assert_eq!(cache.next_stamp, 1);
		cache.assert_accounting();
	}

	#[test]
	fn main_ghost_returnee_into_full_main_with_full_main_ghost_keeps_accounting() {
		// Re-entrancy: a main-ghost returnee triggers evict_main, which inserts a
		// hit main victim into the main ghost while it is at budget.
		let mut cache = S3Fifo::new(8); // small 2, main 6, budget 2.
		for k in 0..6 {
			cache.push_main_back(100 + k, k, 0);
		}
		// Ghost two hit main victims: tails are 105, 104.
		for k in [105, 104] {
			cache.get(&k);
			let i = match cache.index.get(&k) {
				Some(Loc::Main(i)) => *i as usize,
				_ => unreachable!(),
			};
			cache.slab[i].freq = 0;
			cache.evict_main();
			assert!(cache.in_main_ghost(&k));
		}
		assert_eq!(cache.main_ghost_live, 2);
		// Refill main; make the tail a hit victim.
		cache.push_main_back(200, 0, 0);
		cache.push_main_back(201, 0, 0);
		cache.get(&201);
		let i = match cache.index.get(&201) {
			Some(Loc::Main(i)) => *i as usize,
			_ => unreachable!(),
		};
		cache.slab[i].freq = 0;
		assert_eq!(cache.main.len(), cache.main_size);
		cache.put(105, 1050); // returns -> evicts 201 into main ghost.
		assert!(cache.in_main(&105));
		assert!(cache.in_main_ghost(&201));
		assert!(cache.in_main_ghost(&104));
		assert_eq!(cache.next_stamp, 0);
		cache.assert_accounting();
	}

	#[test]
	fn ghost_returnee_into_full_main_with_full_ghost_keeps_accounting() {
		// Re-entrancy: a strong ghost returnee triggers evict_main, which ghosts
		// a hit main victim into the main ghost while the small ghost is full.
		let mut cache = S3Fifo::new(4);
		fill_main(&mut cache);
		// Fill ghost to main_size (3) with freq-1 small victims.
		for k in 1..=3 {
			cache.push_small_front(k, k * 10, 1);
			cache.evict_small();
		}
		assert_eq!(cache.ghost_live, cache.main_size);
		cache.get(&12); // main tail is hit.
		cache.evict_main(); // 12 -> front with freq 0; 11 dropped (no ghost).
		assert_eq!(cache.main.len(), 2);
		cache.push_main_back(13, 130, 0);
		cache.get(&13);
		let i13 = match cache.index.get(&13) {
			Some(Loc::Main(i)) => *i as usize,
			_ => unreachable!(),
		};
		cache.slab[i13].freq = 0;
		assert!(cache.slab[i13].main_hit);
		// Main full: [12, 10, 13(tail, freq 0, main_hit)].
		cache.put(3, 30); // ghost freq 1 -> main; evicts 13 into the main ghost.
		assert!(cache.in_main(&3));
		assert!(cache.in_main_ghost(&13));
		assert_eq!(cache.ghost_live, 2);
		assert!(cache.in_ghost(&1));
		assert!(cache.in_ghost(&2));
		cache.assert_accounting();
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
			let mg_bound = 2 * cache.main_ghost_budget();
			let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ capacity as u64;
			for _ in 0..20_000 {
				state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
				let key = ((state >> 33) % (3 * capacity as u64)) as i64;
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
				assert!(cache.ghost.len() <= bound, "ghost deque {} > {bound}", cache.ghost.len());
				assert!(
					cache.main_ghost.len() <= mg_bound,
					"main ghost deque {} > {mg_bound}",
					cache.main_ghost.len()
				);
				assert!(cache.resident() <= capacity);
				cache.assert_accounting();
			}
		}
	}

	/// Linear-scan reference model of the policy (cycle-0002 v001 plus the
	/// cycle-0004 freq-gated ghost return plus the cycle-0005 ghost-age rule
	/// plus the cycle-0008 separate main-victim ghost), used as a hit/miss oracle.
	struct Reference {
		small: VecDeque<Entry<i64, i64>>,
		main: VecDeque<Entry<i64, i64>>,
		ghost: VecDeque<(i64, u8, u64)>,
		main_ghost: VecDeque<i64>,
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
				main_ghost: VecDeque::new(),
				next_stamp: 0,
				small_size,
				main_size: capacity - small_size,
			}
		}

		fn get(&mut self, key: &i64) -> Option<i64> {
			if let Some(entry) = self.small.iter_mut().find(|e| &e.key == key) {
				entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
				return Some(entry.value);
			}
			let entry = self.main.iter_mut().find(|e| &e.key == key)?;
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			entry.main_hit = true;
			Some(entry.value)
		}

		fn put(&mut self, key: i64, value: i64) {
			if let Some(e) = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| e.key == key) {
				e.value = value;
				return;
			}
			if self.main_ghost.iter().any(|g| g == &key) {
				self.main_ghost.retain(|g| g != &key);
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(Entry::new(key, value));
				return;
			}
			if let Some(&(_, gfreq, gstamp)) = self.ghost.iter().find(|(g, _, _)| g == &key) {
				self.ghost.retain(|(g, _, _)| g != &key);
				let recent = (self.small_size / GHOST_RECENT_AGE_DIVISOR).max(1) as u64;
				if gfreq >= GHOST_TO_MAIN_FREQ || self.next_stamp - gstamp <= recent {
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

		/// Small-victim ghost push; the only place stamps advance.
		fn ghost_push(&mut self, key: i64, freq: u8) {
			if self.ghost.len() >= self.main_size {
				self.ghost.pop_back();
			}
			let stamp = self.next_stamp;
			self.next_stamp += 1;
			self.ghost.push_front((key, freq, stamp));
		}

		fn main_ghost_push(&mut self, key: i64) {
			if self.main_ghost.len() >= self.small_size.max(1) {
				self.main_ghost.pop_back();
			}
			self.main_ghost.push_front(key);
		}

		fn evict_main(&mut self) {
			while let Some(mut tail) = self.main.pop_back() {
				if tail.freq > 0 {
					tail.freq -= 1;
					self.main.push_front(tail);
				} else {
					if tail.main_hit {
						self.main_ghost_push(tail.key);
					}
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
					tail.main_hit = false;
					self.main.push_front(tail);
				} else {
					self.ghost_push(tail.key, tail.freq);
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
