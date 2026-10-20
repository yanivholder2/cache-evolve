use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
const MAX_FREQ: u8 = 3;

// Hysteresis threshold for the adaptive split controller. The signed
// `steer_accum` integrates per-event steering evidence and only commits a one-
// step move to `small_size` once agreement crosses this band, suppressing the
// single-event thrash a per-event controller suffers under mixed bursts.
const STEER_THRESHOLD: i32 = 4;

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

// A ghost record tagged by its source. `from_main == true` marks a strong
// record: a key that earned promotion to main and then cooled to freq == 0.
// `from_main == false` marks a weak record: a small-queue one-hit-wonder that
// never earned promotion. The tag drives source-aware retention priority while
// leaving the total ghost capacity unchanged. It also doubles as the steering
// signal for runtime partition adaptation (see `put`'s in-ghost branch).
#[derive(Debug)]
struct GhostKey<K> {
	key: K,
	from_main: bool,
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<GhostKey<K>>,
	small_size: usize,
	main_size: usize,
	capacity: usize,
	// Signed hysteresis accumulator for the adaptive split. Each ghost re-
	// admission adds its `from_main` evidence (+1 weak / -1 strong); the split
	// only moves once |steer_accum| crosses STEER_THRESHOLD, after which it
	// resets to 0. This denoises the per-event steering of v001.
	steer_accum: i32,
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
			(capacity / 24).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			small_size,
			main_size,
			capacity,
			steer_accum: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	// Lower/upper bounds for the adaptive `small_size`. The window orbits the
	// proven `capacity/24` center and is never allowed to collapse the admission
	// filter or starve main; both bounds are floored at 1 so degenerate small
	// capacities still keep a usable admission slot.
	fn small_lower(&self) -> usize {
		(self.capacity / 48).max(1)
	}

	fn small_upper(&self) -> usize {
		(self.capacity / 12).max(1)
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// 1. Search the small admission queue first. A hit there bumps `freq`
		//    (clamped to MAX_FREQ) without reordering small.
		if let Some(pos) = self.small.iter().position(|entry| &entry.key == key) {
			let entry = &mut self.small[pos];
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			return Some(&self.small[pos].value);
		}

		// 2. Otherwise search the main resident queue. A hit there promotes the
		//    entry to the front (most-recently-used position) in addition to the
		//    `freq` bump, making main LRU-aware while leaving the tail-based
		//    second-chance eviction discipline intact.
		if let Some(pos) = self.main.iter().position(|entry| &entry.key == key) {
			let mut entry = self
				.main
				.remove(pos)
				.expect("S3Fifo::get: position within bounds");
			entry.freq = entry.freq.saturating_add(1).min(MAX_FREQ);
			self.main.push_front(entry);
			return Some(&self.main.front().expect("S3Fifo::get: just pushed").value);
		}

		// 3. No match.
		None
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		if let Some(pos) = self.ghost.iter().position(|ghost_key| ghost_key.key == key) {
			// Hysteresis-gated adaptation: the ghost record's source is evidence
			// of which partition is starved, but a single event is noisy. Read
			// `from_main` *before* removing the record and integrate it into the
			// signed `steer_accum` instead of moving the split immediately. Only
			// once accumulated agreement crosses STEER_THRESHOLD do we commit a
			// one-step move, then reset the accumulator. The queues are bounded by
			// `small_size`/`main_size` only at eviction time, so shifting these
			// fields simply moves future eviction thresholds — no resident entries
			// need to be relocated.
			let from_main = self.ghost[pos].from_main;
			self.ghost.remove(pos);

			if from_main {
				// A proven main-evicted key cooled, was ghosted, and is now reused:
				// evidence main is undersized — push toward shrinking small.
				self.steer_accum -= 1;
			} else {
				// A small-queue one-hit-wonder ghost record is reused: evidence the
				// admission window is too short — push toward growing small.
				self.steer_accum += 1;
			}

			if self.steer_accum >= STEER_THRESHOLD {
				// Sustained weak-reuse majority: grow the admission window by one.
				if self.small_size < self.small_upper() {
					self.small_size += 1;
				}
				self.steer_accum = 0;
			} else if self.steer_accum <= -STEER_THRESHOLD {
				// Sustained strong-reuse majority: shrink small, growing main.
				if self.small_size > self.small_lower() {
					self.small_size -= 1;
				}
				self.steer_accum = 0;
			}
			// Maintain `small_size + main_size == capacity` at all times. Ghost
			// capacity continues to track `main_size`.
			self.main_size = self.capacity - self.small_size;

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

	// Make room in the full ghost queue by evicting the oldest weak
	// (from_main == false) record first, scanning from the tail (oldest end).
	// Only when no weak record exists does it fall back to dropping the tail.
	// This preserves the strong main-evicted history longer without changing
	// the ghost's total capacity.
	fn evict_ghost_one(&mut self) {
		if let Some(pos) = self.ghost.iter().rposition(|gk| !gk.from_main) {
			self.ghost.remove(pos);
		} else {
			self.ghost.pop_back();
		}
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				// Record the main-evicted victim in the ghost queue before
				// dropping it. A key that earned promotion to main and then cooled
				// to freq == 0 is a strong reuse signal: ghosting it lets a later
				// reaccess re-admit it directly to main via the put() in-ghost
				// branch. Tagged `from_main: true` so source-aware trimming keeps
				// it longer than weak small-evicted records.
				if self.ghost.len() >= self.main_size {
					self.evict_ghost_one();
				}
				self.ghost.push_front(GhostKey { key: tail.key, from_main: true });
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
				if self.ghost.len() >= self.main_size {
					self.evict_ghost_one();
				}
				self.ghost.push_front(GhostKey { key: tail.key, from_main: false });
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

	#[test]
	fn main_hit_moves_entry_to_front() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.main.push_back(Entry::new(1, 10));
		cache.main.push_back(Entry::new(2, 20));
		cache.main.push_back(Entry::new(3, 30));
		assert_eq!(*cache.get(&3).unwrap(), 30);
		assert_eq!(cache.main.front().unwrap().key, 3);
		assert_eq!(cache.small.len() + cache.main.len(), 3);
	}
}
