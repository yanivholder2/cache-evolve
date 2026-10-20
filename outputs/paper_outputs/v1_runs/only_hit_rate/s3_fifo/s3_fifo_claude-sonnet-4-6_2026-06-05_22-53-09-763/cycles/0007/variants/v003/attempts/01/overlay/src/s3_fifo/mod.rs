use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// H03: Dynamically adjusts the main-queue second-chance threshold (`evict_thresh`)
// based on the observed main-hit ratio over a sliding window of ADAPT_PERIOD
// accesses. This allows the cache to self-tune across workload phases beyond
// the static `freq > 1` gate from cycle/0006/v001.
const MAX_FREQ: u8 = 5;

/// Number of cache accesses (gets) between threshold adaptations.
/// Aligned to cache capacity (2048) so the mechanism has enough signal
/// to detect phase shifts without reacting to noise.
const ADAPT_PERIOD: u64 = 2048;

#[derive(Debug)]
struct Entry<K, V> {
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
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
	ghost_size: usize,
	/// H03: count of main-queue hits in the current adaptation window
	main_hits: u64,
	/// H03: total get() calls (drives the adaptation period check)
	total_gets: u64,
	/// H03: adaptive eviction threshold; starts at 1 (matching v001 static gate).
	/// Updated by adapt_threshold() every ADAPT_PERIOD gets.
	evict_thresh: u8,
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
			(capacity / 10).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let ghost_size = (main_size / 2).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(ghost_size),
			small_size,
			main_size,
			ghost_size,
			main_hits: 0,
			total_gets: 0,
			evict_thresh: 1, // same as v001's static threshold until first adaptation
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	/// Recompute `evict_thresh` from the main-hit ratio observed in the last
	/// ADAPT_PERIOD gets, then reset the window counter.
	///
	/// Three-level control surface (ratios match the H03 specification):
	/// - ratio >= 0.60 → thresh 0  (lenient: freq > 0 reprieved)
	/// - ratio >= 0.45 → thresh 1  (default: freq > 1, same as v001)
	/// - ratio <  0.45 → thresh 2  (aggressive: freq > 2 only reprieved)
	fn adapt_threshold(&mut self) {
		let ratio = self.main_hits as f64 / ADAPT_PERIOD as f64;
		// Reset window counter for the next period
		self.main_hits = 0;
		self.evict_thresh = if ratio >= 0.60 {
			0 // lenient: warm workload — let freq=1 items survive
		} else if ratio >= 0.45 {
			1 // default: matches v001 behaviour
		} else {
			2 // aggressive: scan-heavy phase — flush cold items quickly
		};
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		self.total_gets += 1;

		// Trigger threshold adaptation at the start of each new window.
		// We do this before touching the entry so all mutable borrows of `self`
		// are finished before we borrow into the queues.
		if self.total_gets % ADAPT_PERIOD == 0 {
			self.adapt_threshold();
		}

		// Determine whether the key is resident in main (shared borrow,
		// released before the mutable find below).
		let in_main = self.main.iter().any(|e| &e.key == key);
		if in_main {
			self.main_hits += 1;
		}

		// Find the entry in small or main and bump its frequency.
		let entry = self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		let in_ghost = self.ghost.iter().any(|ghost_key| ghost_key == &key);
		if in_ghost {
			self.ghost.retain(|ghost_key| ghost_key != &key);
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
			if tail.freq > self.evict_thresh {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				// freq <= evict_thresh: evict immediately
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
				if self.ghost.len() >= self.ghost_size {
					self.ghost.pop_back();
				}
				self.ghost.push_front(tail.key);
			}
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

	// ── v001 tests preserved ────────────────────────────────────────────────

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
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().freq = 2;
		cache.main.push_back(Entry::new(3, 30));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(4, 40));
		cache.main.back_mut().unwrap().freq = 1;
		cache.small.push_front(Entry::new(5, 50));
		cache.small.front_mut().unwrap().freq = 2;
		cache.evict_small();
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
	fn repeated_get_clamps_frequency() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
	}

	// ── H03 adaptive-threshold tests ────────────────────────────────────────

	/// evict_thresh=0: freq=1 items get a second chance; only freq=0 items are
	/// evicted immediately.
	#[test]
	fn adaptive_thresh_zero_reprieves_freq_one_items() {
		let mut cache = S3Fifo::<i64, i64>::new(5);
		cache.evict_thresh = 0;
		// Build main queue: front=key=1(f=0), back=key=2(f=1)
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().freq = 1;
		// evict_main: pop key=2(f=1). 1 > 0 → second chance (push front, f→0).
		//             pop key=1(f=0). 0 > 0 = false → evict key=1.
		cache.evict_main();
		assert!(
			cache.main.iter().any(|e| e.key == 2),
			"key=2 (freq was 1) should survive evict_thresh=0"
		);
		assert!(
			!cache.main.iter().any(|e| e.key == 1),
			"key=1 (freq was 0) should be evicted"
		);
	}

	/// evict_thresh=2: items with freq<=2 are evicted immediately; only freq>=3
	/// items receive a second chance.
	#[test]
	fn adaptive_thresh_two_evicts_freq_two_items() {
		let mut cache = S3Fifo::<i64, i64>::new(5);
		cache.evict_thresh = 2;
		// Build main queue: front=key=1(f=2), back=key=2(f=3)
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 2;
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().freq = 3;
		// evict_main: pop key=2(f=3). 3 > 2 → second chance (push front, f→2).
		//             pop key=1(f=2). 2 > 2 = false → evict key=1.
		cache.evict_main();
		assert!(
			cache.main.iter().any(|e| e.key == 2),
			"key=2 (freq was 3) should survive evict_thresh=2"
		);
		assert!(
			!cache.main.iter().any(|e| e.key == 1),
			"key=1 (freq was 2) should be evicted with evict_thresh=2"
		);
	}
}
