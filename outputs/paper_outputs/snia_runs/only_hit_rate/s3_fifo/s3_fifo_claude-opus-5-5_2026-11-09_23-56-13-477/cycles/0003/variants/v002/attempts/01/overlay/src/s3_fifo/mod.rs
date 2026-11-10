use std::collections::VecDeque;
use std::fmt::Debug;

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
// Variant v001 (cycle 0002, H01): while main has free capacity (cold start) a
// single reuse (freq >= 1) suffices to graduate.
//
// Variant v002 (cycle 0003, H02): in steady state the graduation threshold
// adapts between 1 and 2 using a windowed ghost-return rate. Every `win_len`
// small tails sent to ghost, the fraction of ghost hits observed in `put`
// during that window is computed: above 0.30 the threshold drops to 1 (being
// strict is costing ghost misses), below 0.15 it returns to 2; in between it is
// left unchanged (hysteresis).
const MAX_FREQ: u8 = 3;

/// Graduation threshold while main has spare room (cold start).
const GRADUATE_FREQ_MAIN_ROOM: u8 = 1;
/// Lenient adaptive threshold.
const THRESHOLD_LOW: u8 = 1;
/// Strict adaptive threshold (baseline rule) and initial value.
const THRESHOLD_HIGH: u8 = 2;
/// Ghost-return rate above which the threshold drops to `THRESHOLD_LOW`.
const RATE_TO_LOW: f32 = 0.30;
/// Ghost-return rate below which the threshold returns to `THRESHOLD_HIGH`.
const RATE_TO_HIGH: f32 = 0.15;

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
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<K>,
	small_size: usize,
	main_size: usize,
	/// Adaptive steady-state graduation threshold; always in {1, 2}.
	threshold: u8,
	/// Small tails sent to ghost in the current window.
	win_ghosted: u32,
	/// Ghost hits in `put` in the current window.
	win_ghost_hits: u32,
	/// Window length, in ghosted small tails.
	win_len: u32,
	/// Completed windows spent at threshold 1 and threshold 2 respectively.
	windows_at_threshold: [u64; 2],
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
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			small_size,
			main_size,
			threshold: THRESHOLD_HIGH,
			win_ghosted: 0,
			win_ghost_hits: 0,
			win_len: small_size.max(16) as u32,
			windows_at_threshold: [0; 2],
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	/// Current adaptive steady-state graduation threshold.
	pub fn threshold(&self) -> u8 {
		self.threshold
	}

	/// Number of completed windows spent at (threshold 1, threshold 2).
	pub fn threshold_stats(&self) -> (u64, u64) {
		(self.windows_at_threshold[0], self.windows_at_threshold[1])
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
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
			self.win_ghost_hits = self.win_ghost_hits.saturating_add(1);
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
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				break;
			}
		}
	}

	/// Closes the current window if it is complete and adapts `threshold`.
	fn maybe_close_window(&mut self) {
		if self.win_ghosted < self.win_len {
			return;
		}
		self.windows_at_threshold[(self.threshold - 1) as usize] += 1;
		let r = self.win_ghost_hits as f32 / self.win_ghosted as f32;
		if r > RATE_TO_LOW {
			self.threshold = THRESHOLD_LOW;
		} else if r < RATE_TO_HIGH {
			self.threshold = THRESHOLD_HIGH;
		}
		self.win_ghosted = 0;
		self.win_ghost_hits = 0;
	}

	fn evict_small(&mut self) {
		if let Some(mut tail) = self.small.pop_back() {
			let main_full = self.main.len() >= self.main_size;
			let threshold = if main_full {
				self.threshold
			} else {
				GRADUATE_FREQ_MAIN_ROOM
			};
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
				self.ghost.push_front(tail.key);
				self.win_ghosted = self.win_ghosted.saturating_add(1);
				self.maybe_close_window();
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
	fn small_eviction_promotes_to_main_after_single_hit_with_reset_freq() {
		// Main is empty here, so the cold-start single-reuse threshold applies.
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20);
		let promoted = cache.main.iter().find(|entry| entry.key == 1).expect("key 1 graduated");
		assert_eq!(promoted.freq, 0);
		assert!(!cache.ghost.contains(&1));
	}

	#[test]
	fn small_tail_with_single_hit_goes_to_ghost_when_main_full_and_strict() {
		// capacity 4 -> small_size 1, main_size 3. Initial threshold is 2.
		let mut cache = S3Fifo::new(4);
		assert_eq!(cache.threshold(), 2);
		cache.main.push_back(Entry::new(10, 100));
		cache.main.push_back(Entry::new(11, 110));
		cache.main.push_back(Entry::new(12, 120));
		assert_eq!(cache.main.len(), cache.main_size);
		cache.small.push_front(Entry::new(1, 10));
		cache.small.front_mut().unwrap().freq = 1;
		cache.evict_small();
		assert!(!cache.main.iter().any(|entry| entry.key == 1));
		assert!(cache.ghost.contains(&1));
		assert_eq!(cache.main.len(), 3);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
		assert!(!cache.ghost.contains(&1));
		assert!(cache.small.len() + cache.main.len() <= 4);
	}

	#[test]
	fn small_tail_with_single_hit_graduates_when_threshold_low() {
		let mut cache = S3Fifo::new(4);
		cache.threshold = 1;
		cache.main.push_back(Entry::new(10, 100));
		cache.main.push_back(Entry::new(11, 110));
		cache.main.push_back(Entry::new(12, 120));
		cache.small.push_front(Entry::new(1, 10));
		cache.small.front_mut().unwrap().freq = 1;
		cache.evict_small();
		let promoted = cache.main.iter().find(|entry| entry.key == 1).expect("key 1 graduated");
		assert_eq!(promoted.freq, 0);
		assert_eq!(cache.main.len(), 3);
	}

	#[test]
	fn small_tail_with_two_hits_graduates_when_main_full() {
		let mut cache = S3Fifo::new(4);
		cache.main.push_back(Entry::new(10, 100));
		cache.main.push_back(Entry::new(11, 110));
		cache.main.push_back(Entry::new(12, 120));
		cache.small.push_front(Entry::new(1, 10));
		cache.small.front_mut().unwrap().freq = 2;
		cache.evict_small();
		let promoted = cache.main.iter().find(|entry| entry.key == 1).expect("key 1 graduated");
		assert_eq!(promoted.freq, 0);
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

	fn access(cache: &mut S3Fifo<i64, i64>, key: i64) {
		if cache.get(&key).is_none() {
			cache.put(key, key);
		}
	}

	#[test]
	fn always_returning_ghosted_keys_flip_threshold_to_one() {
		// capacity 100 -> small 25, main 75, win_len 25.
		let mut cache = S3Fifo::new(100);
		let small = cache.small_size as i64;
		assert_eq!(cache.threshold(), 2);
		for i in 0..1000i64 {
			access(&mut cache, i);
			if i >= small {
				// Key i - small was just pushed out of small into ghost; it returns.
				let returning = i - small;
				assert!(cache.ghost.contains(&returning));
				access(&mut cache, returning);
			}
		}
		assert_eq!(cache.threshold(), 1);
		let (at_one, at_two) = cache.threshold_stats();
		assert!(at_two >= 1);
		assert!(at_one >= 1);
	}

	#[test]
	fn scan_only_trace_keeps_threshold_at_two() {
		let mut cache = S3Fifo::new(100);
		for i in 0..10_000i64 {
			access(&mut cache, i);
			assert_eq!(cache.threshold(), 2);
		}
		let (at_one, at_two) = cache.threshold_stats();
		assert_eq!(at_one, 0);
		assert!(at_two > 0);
	}

	#[test]
	fn adaptive_invariants_hold_under_mixed_workload() {
		for &capacity in &[2usize, 4, 8, 64, 100] {
			let mut cache = S3Fifo::new(capacity);
			let mut x: u64 = 0x9E37_79B9_7F4A_7C15;
			for _ in 0..20_000 {
				x ^= x << 13;
				x ^= x >> 7;
				x ^= x << 17;
				// Mix of a hot set and a long tail.
				let key = if x % 3 == 0 { (x >> 8) % 16 } else { (x >> 8) % 400 } as i64;
				access(&mut cache, key);
				assert!(cache.small.len() + cache.main.len() <= capacity);
				assert!(cache.ghost.len() <= cache.main_size);
				assert!(cache.threshold == 1 || cache.threshold == 2);
			}
		}
	}
}
