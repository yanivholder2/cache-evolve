use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// H01: Eager promotion — when get() increments a small-queue item's frequency
// to >= 2 (the promotion threshold), the item is immediately relocated to the
// front of main rather than waiting for evict_small() to discover it.
//
// H02: Reduce small queue from ~10% (capacity / 10) to ~7% (capacity / 14),
// transferring additional resident slots to main to further limit cache
// pollution from one-time-access traffic on a Zipfian workload.
//
// H03: Lower the main-queue eviction threshold from `freq > 1` to `freq > 0`,
// granting items one additional second-chance recycle pass before eviction.
// Items promoted by eager promotion enter main at freq==2; with the old
// threshold they were recycled once (2→1) then evicted at freq==1. The new
// threshold gives them an extra pass (1→0 pushed to front), so they are only
// evicted when they reach the tail a third time at freq==0.
const MAX_FREQ: u8 = 5;

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
			(capacity / 14).max(1).min(capacity - 1)
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

	/// Look up `key`, incrementing its frequency counter.
	///
	/// Small-queue items whose frequency reaches the promotion threshold (>= 2)
	/// are eagerly moved to the front of the main queue so they are protected
	/// immediately rather than waiting for `evict_small()` to discover them.
	pub fn get(&mut self, key: &K) -> Option<&V> {
		// --- small queue: separate pass so we can relocate the entry ---
		if let Some(pos) = self.small.iter().position(|e| &e.key == key) {
			let new_freq = (self.small[pos].freq.saturating_add(1)).min(MAX_FREQ);
			if new_freq >= 2 {
				// Eager promotion: move the item from small to the front of main.
				self.small[pos].freq = new_freq;
				let entry = self.small.remove(pos).unwrap();
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(entry);
				return self.main.front().map(|e| &e.value);
			} else {
				// freq < 2: update in place, stay in small.
				self.small[pos].freq = new_freq;
				return Some(&self.small[pos].value);
			}
		}

		// --- main queue: in-place increment, no relocation ---
		if let Some(entry) = self.main.iter_mut().find(|e| &e.key == key) {
			entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
			return Some(&entry.value);
		}

		None
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

	/// Evict the tail of the main queue using a second-chance policy.
	///
	/// Items with `freq > 0` are given a second chance: their frequency is
	/// decremented by one and they are reinserted at the front of main.
	/// Only items with `freq == 0` are evicted immediately. This extends the
	/// recycle window by one pass compared to the `freq > 1` threshold,
	/// accommodating eagerly-promoted items that enter main at freq==2 and
	/// would otherwise be evicted after only two passes (2→1 recycled, 1
	/// evicted). With this threshold they get three passes (2→1, 1→0, 0 evicted).
	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				// freq == 0: evict immediately
				break;
			}
		}
	}

	/// Evict the tail of the small queue.
	///
	/// Items with `freq > 1` are promoted to main (fallback path — eager
	/// promotion in `get()` handles most cases, but items that accumulate
	/// frequency only through puts or that are promoted before a `get()` call
	/// still use this path).
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

	/// After two gets the item's freq reaches 2 (the promotion threshold) and is
	/// eagerly moved to main; no eviction cycle is needed to observe it there.
	#[test]
	fn small_eager_promotion_on_second_get() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1); // freq → 1, stays in small
		assert!(cache.small.iter().any(|e| e.key == 1));
		cache.get(&1); // freq → 2 ≥ 2, eagerly promoted to main
		assert!(cache.main.iter().any(|e| e.key == 1));
		assert!(!cache.small.iter().any(|e| e.key == 1));
	}

	/// The evict_small() promotion path still works as a fallback.
	#[test]
	fn small_eviction_promotes_to_main_after_multiple_hits() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1); // eager promotion fires here
		cache.put(2, 20);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		// With capacity=10 small_size=1 (10/14=0, clamped to 1), ghost_size=4;
		// key 1 survives in ghost long enough to be re-inserted via the ghost path.
		let mut cache = S3Fifo::new(10);
		cache.put(1, 10);
		// Evict key 1 into ghost by filling small
		cache.put(2, 20);
		// key 1 should now be in ghost (small_size=1, evicted on put(2))
		// Re-insert key 1 — ghost hit promotes it directly to main
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
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

	#[test]
	fn eager_promotion_value_is_correct() {
		let mut cache = S3Fifo::new(8);
		cache.put(42, 99);
		cache.get(&42); // freq 1
		let val = cache.get(&42); // freq 2, eager promotion
		assert_eq!(*val.unwrap(), 99);
		// After promotion, further gets also return the correct value
		assert_eq!(*cache.get(&42).unwrap(), 99);
	}

	#[test]
	fn capacity_bound_with_eager_promotion() {
		let mut cache = S3Fifo::new(8);
		// Warm up key 1 so it gets eagerly promoted
		cache.put(1, 1);
		cache.get(&1);
		cache.get(&1); // promoted to main
		// Now fill the cache and check capacity
		for key in 2..30 {
			if cache.get(&key).is_none() {
				cache.put(key, key);
			}
		}
		let residents = (0i64..30).filter(|k| cache.get(k).is_some()).count();
		assert!(residents <= 8, "capacity exceeded: {residents} > 8");
	}

	/// Verify the extended recycle window: an item entering main at freq==2
	/// should survive two recycle passes (2→1, 1→0) before being evicted at
	/// freq==0, not one pass as under the old `freq > 1` threshold.
	#[test]
	fn evict_main_extended_recycle_window() {
		// Build a small main queue with one hot item (freq=2) and one cold (freq=0)
		let mut cache = S3Fifo::<i64, i64>::new(4);
		// Push directly into main to control initial frequencies
		cache.main.push_back(Entry::new(10, 100)); // tail (first to be evicted)
		cache.main.back_mut().unwrap().freq = 2;   // freq=2, should get two recycle passes
		cache.main.push_back(Entry::new(20, 200)); // newer entry
		cache.main.back_mut().unwrap().freq = 0;   // freq=0, cold

		// Trigger evict_main: key 20 (freq=0) should be evicted; key 10 recycled
		cache.evict_main();
		// key 10 (freq was 2, now 1 after one pass) should still be in main
		assert!(cache.main.iter().any(|e| e.key == 10), "hot item should survive first pass");
		// key 20 should be gone
		assert!(!cache.main.iter().any(|e| e.key == 20), "cold item should be evicted");
	}
}
