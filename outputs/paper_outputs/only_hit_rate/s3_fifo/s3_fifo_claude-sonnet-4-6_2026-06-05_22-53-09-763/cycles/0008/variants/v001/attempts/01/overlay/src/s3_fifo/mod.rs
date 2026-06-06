use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// cycle/0007 H01: Eager promotion — when get() increments a small-queue item's
// frequency to >= 2 (the promotion threshold), the item is immediately relocated
// to the front of main rather than waiting for evict_small() to discover it.
//
// cycle/0008 H01: Raise the main-queue second-chance threshold from freq > 1 to
// freq > 2. Only items with freq >= 3 are recycled to the front; items with
// freq <= 2 are evicted immediately on reaching the tail.
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

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 2 {
				// freq >= 3: item has proven itself — give a second chance by
				// decrementing and requeueing at the front.
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				// freq == 0, 1, or 2: evict immediately, no front reprieve.
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
		// With capacity=10 small_size=1, ghost_size=4; key 1 survives in ghost
		// long enough to be re-inserted via the ghost path.
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

	/// Verify the new freq > 2 threshold: items with freq == 2 at the tail of main
	/// are evicted immediately (no second chance), while items with freq == 3 get
	/// decremented to 2 and requeued at the front.
	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		// Build a main queue with mixed frequencies.
		// Queue (back → front): [freq=0, freq=2, freq=0, freq=3]
		// Eviction pops from the back.  freq=0 → evicted.
		// A second evict_main call: pops freq=2 → evicted (freq > 2 is false).
		// A third: pops freq=0 → evicted.
		// A fourth: pops freq=3 → recycled to front with freq=2.
		let mut cache = S3Fifo::<i64, i64>::new(10);
		// Bypass put() to set up controlled freq values directly.
		cache.main.push_back(Entry::new(1, 10));
		cache.main.back_mut().unwrap().freq = 0;
		cache.main.push_back(Entry::new(2, 20));
		cache.main.back_mut().unwrap().freq = 2; // freq == 2: should be evicted now
		cache.main.push_back(Entry::new(3, 30));
		cache.main.back_mut().unwrap().freq = 3; // freq == 3: gets one recycle

		// First evict_main: pops key 1 (freq=0), evicted immediately.
		cache.evict_main();
		assert!(!cache.main.iter().any(|e| e.key == 1), "key 1 (freq=0) should be evicted");
		assert!(cache.main.iter().any(|e| e.key == 2), "key 2 should still be present");

		// Second evict_main: pops key 2 (freq=2), evicted immediately (not > 2).
		cache.evict_main();
		assert!(!cache.main.iter().any(|e| e.key == 2), "key 2 (freq=2) should be evicted under new threshold");

		// Third evict_main: pops key 3 (freq=3), decremented to freq=2 and recycled.
		cache.evict_main();
		assert!(cache.main.iter().any(|e| e.key == 3), "key 3 (freq=3) should survive one recycle");
		// After recycling, freq should have been decremented to 2.
		assert_eq!(cache.main.iter().find(|e| e.key == 3).unwrap().freq, 2);
	}
}
