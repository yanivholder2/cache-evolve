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
// Policy (unchanged from cycle/0003/v003): 5% small queue, MAX_FREQ = 7,
// graduation from small at freq > 1, ghost re-admission at the front of main
// with freq 0, CLOCK-style rotation in main.
//
// H01 (cycle 4): structure only. Entries live in a slab addressed by slot id;
// a HashMap index gives O(1) residency lookup; the small and main FIFOs hold
// slot ids; the ghost is a FIFO of (key, seq) with a HashMap of live ghost keys
// and lazy deletion of stale deque elements.
//
// H03 (cycle 5): main-queue eviction victims are also recorded in the ghost
// (same main_size cap), so a returning main victim re-enters main directly.
const MAX_FREQ: u8 = 7;

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

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Option<Entry<K, V>>>,
	free: Vec<usize>,
	index: HashMap<K, usize>,
	small: VecDeque<usize>,
	main: VecDeque<usize>,
	ghost: VecDeque<(K, u64)>,
	ghost_set: HashMap<K, u64>,
	ghost_seq: u64,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		// Small queue is 5% of capacity (inherited from cycle/0002/v003).
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			slab: Vec::with_capacity(capacity + 1),
			free: Vec::with_capacity(capacity + 1),
			index: HashMap::with_capacity(capacity + 1),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size + 1),
			ghost: VecDeque::with_capacity(2 * main_size + 17),
			ghost_set: HashMap::with_capacity(main_size + 1),
			ghost_seq: 0,
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

		if self.ghost_set.remove(&key).is_some() {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let idx = self.alloc(key, value);
			self.main.push_front(idx);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let idx = self.alloc(key, value);
		self.small.push_front(idx);
	}

	fn evict_main(&mut self) {
		while let Some(idx) = self.main.pop_back() {
			let entry = self.entry_mut(idx);
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(idx);
			} else {
				// H03: remember main victims in the ghost so a return
				// re-enters main directly. The victim was just removed from
				// `index` and cannot equal any key currently being admitted.
				let entry = self.release(idx);
				self.ghost_insert(entry.key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(idx) = self.small.pop_back() {
			if self.entry_mut(idx).freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(idx);
			} else {
				let entry = self.release(idx);
				self.ghost_insert(entry.key);
			}
		}
	}

	#[inline]
	fn ghost_is_live(&self, key: &K, seq: u64) -> bool {
		self.ghost_set.get(key) == Some(&seq)
	}

	fn ghost_insert(&mut self, key: K) {
		// Cap live ghost entries at main_size: drop stale elements and the
		// oldest live element from the back (matches v003's pop_back when full).
		while self.ghost_set.len() >= self.main_size {
			match self.ghost.pop_back() {
				Some((old_key, seq)) => {
					if self.ghost_is_live(&old_key, seq) {
						self.ghost_set.remove(&old_key);
					}
				}
				None => break,
			}
		}
		let seq = self.ghost_seq;
		self.ghost_seq = self.ghost_seq.wrapping_add(1);
		self.ghost_set.insert(key.clone(), seq);
		self.ghost.push_front((key, seq));

		if self.ghost.len() > 2 * self.main_size + 16 {
			let ghost_set = &self.ghost_set;
			self.ghost.retain(|(k, s)| ghost_set.get(k) == Some(s));
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
	fn test_push_main_back(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.main.push_back(idx);
	}

	fn test_push_main_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.main.push_front(idx);
	}

	fn test_push_small_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.small.push_front(idx);
	}

	fn queue_keys(&self, q: &VecDeque<usize>) -> Vec<(K, u8)> {
		q.iter()
			.map(|&idx| {
				let e = self.slab[idx].as_ref().unwrap();
				(e.key.clone(), e.freq)
			})
			.collect()
	}

	fn main_keys(&self) -> Vec<K> {
		self.queue_keys(&self.main).into_iter().map(|(k, _)| k).collect()
	}

	fn small_keys(&self) -> Vec<K> {
		self.queue_keys(&self.small).into_iter().map(|(k, _)| k).collect()
	}

	fn freq_of(&self, key: &K) -> Option<u8> {
		self.index.get(key).map(|&idx| self.slab[idx].as_ref().unwrap().freq)
	}

	fn ghost_keys(&self) -> Vec<K> {
		self.ghost
			.iter()
			.filter(|(k, s)| self.ghost_is_live(k, *s))
			.map(|(k, _)| k.clone())
			.collect()
	}

	fn resident_len(&self) -> usize {
		self.small.len() + self.main.len()
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
		assert!(!cache.ghost_keys().contains(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
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
	}

	#[test]
	fn main_victim_is_ghosted_and_returns_to_main() {
		// capacity 20 -> small_size 1, main_size 19.
		let mut cache = S3Fifo::<i64, i64>::new(20);
		assert_eq!(cache.main_size, 19);
		// Fill main; key 7 sits at the tail with freq 0, others are hot.
		cache.test_push_main_back(7, 70, 0);
		for k in 100..118i64 {
			cache.test_push_main_front(k, k, 1);
		}
		assert_eq!(cache.main.len(), cache.main_size);
		assert_eq!(*cache.main_keys().last().unwrap(), 7);

		// Force a main eviction via small-queue graduation.
		cache.test_push_small_front(50, 50, 2);
		cache.evict_small();
		assert!(cache.get(&7).is_none());
		assert!(cache.ghost_keys().contains(&7));
		assert!(cache.main_keys().contains(&50));

		// Returning main victim re-enters main directly, not small.
		cache.put(7, 71);
		assert!(cache.main_keys().contains(&7));
		assert!(!cache.small_keys().contains(&7));
		assert!(!cache.ghost_keys().contains(&7));
		assert_eq!(cache.freq_of(&7), Some(0));
		assert!(cache.resident_len() <= 20);
		assert_eq!(*cache.get(&7).unwrap(), 71);
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.resident_len() <= 4);
		assert_eq!(cache.index.len(), cache.resident_len());
	}

	#[test]
	fn small_queue_is_five_percent_of_capacity() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_size, 102);
		assert_eq!(cache.main_size, 1946);
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
	fn ghost_stays_bounded_with_lazy_deletion() {
		let mut cache = S3Fifo::<i64, i64>::new(40);
		for key in 0..10_000i64 {
			let k = key % 500;
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert!(cache.ghost_set.len() <= cache.main_size.max(1));
			assert!(cache.ghost.len() <= 2 * cache.main_size + 17);
		}
	}

	// ---- Reference: cycle/0003/v003 linear implementation (MAX_FREQ = 7),
	// ---- plus H03: main victims are ghosted with the same main_size cap.

	struct RefEntry {
		key: i64,
		value: i64,
		freq: u8,
	}

	struct RefS3Fifo {
		small: VecDeque<RefEntry>,
		main: VecDeque<RefEntry>,
		ghost: VecDeque<i64>,
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
				ghost: VecDeque::new(),
				small_size,
				main_size,
			}
		}

		fn get(&mut self, key: &i64) -> Option<i64> {
			let entry = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| &e.key == key)?;
			entry.freq = (entry.freq.saturating_add(1)).min(7);
			Some(entry.value)
		}

		fn put(&mut self, key: i64, value: i64) {
			if let Some(entry) = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| e.key == key) {
				entry.value = value;
				return;
			}
			if self.ghost.iter().any(|g| g == &key) {
				self.ghost.retain(|g| g != &key);
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(RefEntry { key, value, freq: 0 });
				return;
			}
			if self.small.len() >= self.small_size {
				self.evict_small();
			}
			self.small.push_front(RefEntry { key, value, freq: 0 });
		}

		fn ghost_push(&mut self, key: i64) {
			if self.ghost.len() >= self.main_size {
				self.ghost.pop_back();
			}
			self.ghost.push_front(key);
		}

		fn evict_main(&mut self) {
			while let Some(mut tail) = self.main.pop_back() {
				if tail.freq > 0 {
					tail.freq -= 1;
					self.main.push_front(tail);
				} else {
					self.ghost_push(tail.key);
					break;
				}
			}
		}

		fn evict_small(&mut self) {
			if let Some(tail) = self.small.pop_back() {
				if tail.freq > 1 {
					if self.main.len() >= self.main_size {
						self.evict_main();
					}
					self.main.push_front(tail);
				} else {
					self.ghost_push(tail.key);
				}
			}
		}
	}

	#[test]
	fn matches_v003_reference_on_pseudo_random_streams() {
		for &capacity in &[1usize, 2, 3, 4, 8, 20, 41, 64, 100] {
			for &(key_space, skew) in &[(16u64, false), (64, true), (300, true), (1000, false)] {
				let mut cache = S3Fifo::<i64, i64>::new(capacity);
				let mut reference = RefS3Fifo::new(capacity);
				let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ (capacity as u64 * 31 + key_space);
				for step in 0..5_000 {
					state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					let r = state >> 33;
					let key = if skew {
						// Mix of a hot set and a long tail.
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
					let main: Vec<(i64, u8)> = cache.queue_keys(&cache.main);
					let ref_main: Vec<(i64, u8)> = reference.main.iter().map(|e| (e.key, e.freq)).collect();
					assert_eq!(main, ref_main, "main mismatch cap={capacity} step={step}");
					let ghost: Vec<i64> = cache.ghost_keys();
					let ref_ghost: Vec<i64> = reference.ghost.iter().copied().collect();
					assert_eq!(ghost, ref_ghost, "ghost mismatch cap={capacity} step={step}");
					assert_eq!(cache.index.len(), cache.resident_len());
				}
			}
		}
	}
}
