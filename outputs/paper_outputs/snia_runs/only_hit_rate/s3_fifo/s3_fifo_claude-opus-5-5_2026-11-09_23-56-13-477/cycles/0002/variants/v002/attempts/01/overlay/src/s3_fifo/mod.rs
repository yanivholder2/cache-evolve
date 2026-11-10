use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// Variant v001 (H01): a small-queue entry graduates to main after a single
// reuse (freq >= 1), and its frequency is reset to 0 on entering main so that
// small-queue hits do not also buy extra second-chance passes in main.
//
// Variant v002 (H02): the small/main split adapts online, ARC-style. A hit on
// the small-eviction ghost means small was too short, so the small target
// grows. A hit on the new main-eviction ghost means main was too short, so the
// small target shrinks. Because the targets move, eviction runs against the
// combined capacity budget. It evicts from small while small is over its
// target, and from main otherwise.
const MAX_FREQ: u8 = 3;

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
	main_ghost: VecDeque<K>,
	capacity: usize,
	/// Adaptive target for the small queue.
	small_size: usize,
	/// Always `capacity - small_size`.
	main_size: usize,
}

impl<K: PartialEq + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let (lower, upper) = Self::small_bounds(capacity);
		let initial = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let small_size = initial.clamp(lower, upper);
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			main_ghost: VecDeque::with_capacity(capacity / 2),
			capacity,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	/// Inclusive bounds `[lower, upper]` for the adaptive small-queue target.
	fn small_bounds(capacity: usize) -> (usize, usize) {
		let lower = (capacity / 20).max(1);
		let mut upper = (capacity / 2).max(1);
		if capacity > 1 {
			upper = upper.min(capacity - 1);
		}
		(lower.min(upper), upper)
	}

	fn grow_small(&mut self) {
		let (_, upper) = Self::small_bounds(self.capacity);
		let delta = (self.small_size / 16).max(1);
		self.small_size = (self.small_size + delta).min(upper);
		self.main_size = self.capacity - self.small_size;
	}

	fn shrink_small(&mut self) {
		let (lower, _) = Self::small_bounds(self.capacity);
		let delta = (self.small_size / 16).max(1);
		self.small_size = self.small_size.saturating_sub(delta).max(lower);
		self.main_size = self.capacity - self.small_size;
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

		if let Some(pos) = self.ghost.iter().position(|ghost_key| ghost_key == &key) {
			// Small was too short: grow its target, then take the ghost path.
			self.ghost.remove(pos);
			self.grow_small();
			self.insert_main(key, value);
			return;
		}

		if let Some(pos) = self.main_ghost.iter().position(|ghost_key| ghost_key == &key) {
			// Main was too short: shrink the small target, then insert into main.
			self.main_ghost.remove(pos);
			self.shrink_small();
			self.insert_main(key, value);
			return;
		}

		self.make_room();
		self.small.push_front(Entry::new(key, value));
	}

	fn insert_main(&mut self, key: K, value: V) {
		if self.main.len() >= self.main_size && !self.main.is_empty() {
			self.evict_main();
		}
		self.make_room();
		self.main.push_front(Entry::new(key, value));
	}

	/// Evict until there is room for one more resident under the combined budget.
	fn make_room(&mut self) {
		while self.small.len() + self.main.len() >= self.capacity {
			if self.small.len() > self.small_size || self.main.is_empty() {
				if self.small.is_empty() {
					break;
				}
				self.evict_small();
			} else {
				self.evict_main();
			}
		}
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				let bound = self.capacity / 2;
				if bound > 0 {
					while self.main_ghost.len() >= bound {
						self.main_ghost.pop_back();
					}
					self.main_ghost.push_front(tail.key);
				}
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(mut tail) = self.small.pop_back() {
			if tail.freq >= 1 {
				if self.small.len() + self.main.len() + 1 > self.capacity {
					self.evict_main();
				}
				tail.freq = 0;
				self.main.push_front(tail);
			} else {
				if self.ghost.len() >= self.main_size {
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

	#[test]
	fn small_eviction_promotes_to_main_after_multiple_hits() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(5, 50);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn small_eviction_promotes_to_main_after_single_hit_with_reset_freq() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(5, 50);
		let promoted = cache.main.iter().find(|entry| entry.key == 1).expect("key 1 graduated");
		assert_eq!(promoted.freq, 0);
		assert!(!cache.ghost.contains(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main_and_grows_small() {
		let mut cache = S3Fifo::new(4);
		for key in 1..=5 {
			cache.put(key, key * 10);
		}
		assert!(cache.ghost.contains(&1));
		let before = cache.small_size;
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
		assert!(cache.small_size > before);
		assert_eq!(cache.small_size + cache.main_size, 4);
	}

	#[test]
	fn main_ghost_hit_shrinks_small_and_inserts_into_main() {
		let mut cache = S3Fifo::new(20);
		cache.main_ghost.push_front(99);
		let before = cache.small_size;
		cache.put(99, 990);
		assert!(cache.main.iter().any(|entry| entry.key == 99));
		assert!(!cache.main_ghost.contains(&99));
		assert!(cache.small_size < before);
		assert_eq!(cache.small_size + cache.main_size, 20);
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
		cache.make_room();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
		assert!(cache.main_ghost.contains(&3));
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
	}

	fn lcg(state: &mut u64) -> u64 {
		*state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
		*state >> 33
	}

	#[test]
	fn capacity_bound_holds_across_long_mixed_sequence() {
		for capacity in [1usize, 2, 3, 8, 37, 64] {
			let mut cache = S3Fifo::<i64, i64>::new(capacity);
			let mut state = capacity as u64;
			for _ in 0..20_000 {
				let r = lcg(&mut state);
				// Mix a hot set with a wide scan range.
				let key = if r % 3 == 0 { (r % 16) as i64 } else { (r % 500) as i64 };
				if cache.get(&key).is_none() {
					cache.put(key, key * 7);
				}
				assert!(cache.small.len() + cache.main.len() <= capacity);
			}
		}
	}

	#[test]
	fn small_target_stays_within_bounds() {
		for capacity in [1usize, 2, 3, 8, 40, 64, 200] {
			let (lower, upper) = S3Fifo::<i64, i64>::small_bounds(capacity);
			let mut cache = S3Fifo::<i64, i64>::new(capacity);
			let mut state = 7 + capacity as u64;
			for _ in 0..20_000 {
				let r = lcg(&mut state);
				let key = if r % 2 == 0 { (r % (capacity as u64 * 2 + 1)) as i64 } else { (r % 1000) as i64 };
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
				assert!(cache.small_size >= lower && cache.small_size <= upper);
				assert_eq!(cache.small_size + cache.main_size, capacity);
				if capacity > 1 {
					assert!(cache.small_size <= capacity - 1);
				}
			}
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
}
