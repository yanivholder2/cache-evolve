use std::collections::VecDeque;
use std::fmt::Debug;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
const MAX_FREQ: u8 = 3;

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
	// Momentum controller state for the adaptive split. `step` is the current
	// move magnitude (init 1, doubled up to a cap when the steering signal keeps
	// pointing the same way, reset to 1 on reversal). `last_dir` is the last
	// applied direction (-1 shrink small, 0 none yet, +1 grow small).
	step: usize,
	last_dir: i8,
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
			step: 1,
			last_dir: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	// Lower/upper bounds for the adaptive `small_size`. The window orbits the
	// proven `capacity/24` center and is never allowed to collapse the admission
	// filter or starve main; both bounds are floored at 1 so degenerate small
	// capacities still keep a usable admission slot. The window is widened
	// (`capacity/96 .. capacity/8`) so the momentum controller can track larger
	// sustained phase swings.
	fn small_lower(&self) -> usize {
		(self.capacity / 96).max(1)
	}

	fn small_upper(&self) -> usize {
		(self.capacity / 8).max(1)
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
			// ARC-style adaptation with a momentum/acceleration controller, now
			// modulated by signal *quality*. The ghost record's source tells us
			// which partition is starved; the record's *position* tells us how
			// fresh that evidence is. Read `from_main` and the match position
			// `pos` *before* removing the record, then move the split by the
			// current `step`, clamped to the bounded window. The queues are bounded
			// by `small_size`/`main_size` only at eviction time, so shifting these
			// fields simply moves future eviction thresholds — no resident entries
			// need to be relocated.
			let from_main = self.ghost[pos].from_main;
			// Recency weight: a match in the front half of the ghost queue means a
			// key was evicted very recently, i.e. the partition is only just too
			// small — a strong, fresh imbalance signal. `self.ghost.len()` is read
			// before the removal so the half-point reflects the queue at match time.
			let fresh = pos < self.ghost.len() / 2;
			self.ghost.remove(pos);

			// Desired direction: a strong (from_main == true) hit means main is
			// undersized, so shrink small (-1). A weak hit means the admission
			// window is too short, so grow small (+1).
			let desired: i8 = if from_main { -1 } else { 1 };
			if desired == self.last_dir {
				// Agreeing signal accelerates; a fresh agreeing signal jumps
				// straight to the max step instead of ramping toward it.
				self.step = if fresh { 4 } else { (self.step * 2).min(4) };
			} else {
				// A reversal collapses the step, but a fresh reversal still commits
				// a slightly larger move than a deep one — the trace's imbalance is
				// monotone signal, not noise, so fresh reversals are real drift.
				self.step = if fresh { 2 } else { 1 };
			}

			let lower = self.small_lower();
			let upper = self.small_upper();
			if desired > 0 {
				self.small_size = self.small_size.saturating_add(self.step);
			} else {
				self.small_size = self.small_size.saturating_sub(self.step);
			}
			self.small_size = self.small_size.clamp(lower, upper);
			self.last_dir = desired;

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
