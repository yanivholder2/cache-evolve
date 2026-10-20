use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The ghost queue stores 64-bit fingerprints of evicted keys instead of the keys
// themselves. Ghost membership is a hash set of live fingerprints. A ghost hit
// removes the fingerprint from the set and leaves its queue entry in place as a
// stale copy; stale copies are drained lazily when the ghost overflows.
//
// Resident lookup goes through two key-to-record indexes, one per resident queue.
// Each queue has a head counter that is bumped on every push_front, and every
// entry is indexed with the id it received at that push. For a queue whose front
// is the newest entry, the entry at deque index `i` has id `head - i`, so its
// position is `head - id`. Entries only leave a queue from the back, so their
// ids never need rewriting. The frequency counter lives in the same index record
// as the id, so an entry's frequency is read and written through its index.
const MAX_FREQ: u8 = 3;

#[derive(Debug)]
pub struct Entry<K, V> {
	key: K,
	value: V,
}

impl<K, V> Entry<K, V> {
	fn new(key: K, value: V) -> Self {
		Self { key, value }
	}
}

// Per-resident index record: the push id locating the entry in its queue, and
// the entry's frequency counter.
#[derive(Debug)]
struct IndexRec {
	id: u64,
	freq: u8,
}

fn fingerprint<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	// Resident key -> index record of its entry in `small` / `main`.
	small_index: HashMap<K, IndexRec>,
	main_index: HashMap<K, IndexRec>,
	// Id of the most recent push_front into each queue.
	small_head: u64,
	main_head: u64,
	// Oldest-first queue of ghost fingerprints. May contain stale copies.
	ghost: VecDeque<u64>,
	// Live ghost fingerprints; its length is the ghost population.
	ghost_set: HashSet<u64>,
	// Number of entries for each fingerprint currently held in `ghost`.
	ghost_copies: HashMap<u64, u32>,
	// Maximum live ghost population, fixed at construction: half the main
	// queue's slot budget, never more than the main queue itself.
	ghost_cap: usize,
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 5).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			small_index: HashMap::with_capacity(small_size),
			main_index: HashMap::with_capacity(main_size),
			small_head: 0,
			main_head: 0,
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: HashSet::with_capacity(main_size),
			ghost_copies: HashMap::with_capacity(main_size),
			ghost_cap: main_size.min(main_size / 2),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	// Every push to a resident queue goes through these two helpers so the index
	// and the head counter stay in step with the deque.
	fn push_small(&mut self, entry: Entry<K, V>, freq: u8) {
		self.small_head += 1;
		self
			.small_index
			.insert(entry.key.clone(), IndexRec { id: self.small_head, freq });
		self.small.push_front(entry);
	}

	fn push_main(&mut self, entry: Entry<K, V>, freq: u8) {
		self.main_head += 1;
		self
			.main_index
			.insert(entry.key.clone(), IndexRec { id: self.main_head, freq });
		self.main.push_front(entry);
	}

	// Every pop from a resident queue goes through these two helpers so the index
	// never keeps a key that has left its queue. The popped entry's frequency is
	// read from its index record before the record is removed.
	fn pop_small(&mut self) -> Option<(Entry<K, V>, u8)> {
		let tail = self.small.pop_back()?;
		let freq = self.small_index.remove(&tail.key).map_or(0, |rec| rec.freq);
		Some((tail, freq))
	}

	fn pop_main(&mut self) -> Option<(Entry<K, V>, u8)> {
		let tail = self.main.pop_back()?;
		let freq = self.main_index.remove(&tail.key).map_or(0, |rec| rec.freq);
		Some((tail, freq))
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		if let Some(rec) = self.small_index.get_mut(key) {
			rec.freq = rec.freq.saturating_add(1).min(MAX_FREQ);
			let idx = (self.small_head - rec.id) as usize;
			return self.small.get(idx).map(|entry| &entry.value);
		}
		let rec = self.main_index.get_mut(key)?;
		rec.freq = rec.freq.saturating_add(1).min(MAX_FREQ);
		let idx = (self.main_head - rec.id) as usize;
		self.main.get(idx).map(|entry| &entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(rec) = self.small_index.get(&key) {
			let idx = (self.small_head - rec.id) as usize;
			self.small[idx].value = value;
			return;
		}
		if let Some(rec) = self.main_index.get(&key) {
			let idx = (self.main_head - rec.id) as usize;
			self.main[idx].value = value;
			return;
		}

		let fp = fingerprint(&key);
		if self.ghost_set.remove(&fp) {
			// Ghost hit: the queue entry stays behind as a stale copy.
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.push_main(Entry::new(key, value), 0);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.push_small(Entry::new(key, value), 0);
	}

	fn evict_main(&mut self) {
		while let Some((tail, freq)) = self.pop_main() {
			if freq > 0 {
				self.push_main(tail, freq - 1);
			} else {
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some((tail, freq)) = self.pop_small() {
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.push_main(tail, freq);
			} else {
				let fp = fingerprint(&tail.key);
				self.ghost_push(fp);
			}
		}
	}

	fn ghost_push(&mut self, fp: u64) {
		if self.ghost_cap == 0 {
			return;
		}
		// Expire oldest fingerprints until the new one fits under the cap.
		while self.ghost_set.len() >= self.ghost_cap {
			self.ghost_pop_oldest();
		}
		self.ghost.push_back(fp);
		*self.ghost_copies.entry(fp).or_insert(0) += 1;
		self.ghost_set.insert(fp);
	}

	// Drops queue entries from the oldest end until one live fingerprint has
	// been removed. Stale copies met on the way are discarded. A fingerprint's
	// live copy is always its newest queue entry, so it is the last copy of that
	// fingerprint to leave the queue.
	fn ghost_pop_oldest(&mut self) {
		while let Some(fp) = self.ghost.pop_front() {
			let remaining = match self.ghost_copies.get_mut(&fp) {
				Some(copies) => {
					*copies -= 1;
					*copies
				}
				None => 0,
			};
			if remaining == 0 {
				self.ghost_copies.remove(&fp);
				if self.ghost_set.remove(&fp) {
					return;
				}
			}
		}
	}

	// Test-only: every resident key is indexed exactly once, and its id maps to
	// its actual deque position.
	#[cfg(test)]
	fn check_indexes(&self) {
		assert_eq!(self.small_index.len(), self.small.len(), "small index size");
		assert_eq!(self.main_index.len(), self.main.len(), "main index size");
		for (i, entry) in self.small.iter().enumerate() {
			assert_eq!(
				self.small_index.get(&entry.key).map(|rec| rec.id),
				Some(self.small_head - (i as u64)),
				"small index mismatch at position {i}"
			);
		}
		for (i, entry) in self.main.iter().enumerate() {
			assert_eq!(
				self.main_index.get(&entry.key).map(|rec| rec.id),
				Some(self.main_head - (i as u64)),
				"main index mismatch at position {i}"
			);
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
		// Build main front-to-back as 1, 2, 3, 4 so that 4 is the tail.
		cache.push_main(Entry::new(4, 40), 1);
		cache.push_main(Entry::new(3, 30), 0);
		cache.push_main(Entry::new(2, 20), 2);
		cache.push_main(Entry::new(1, 10), 0);
		cache.push_small(Entry::new(5, 50), 2);
		cache.evict_small();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
		cache.check_indexes();
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
	fn index_matches_linear_scan_over_trace() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		let mut state: u64 = 0x9e37_79b9_7f4a_7c15;
		for step in 0..5000 {
			state = state
				.wrapping_mul(6364136223846793005)
				.wrapping_add(1442695040888963407);
			let key = ((state >> 33) % 40) as i64;
			let op = (state >> 20) % 4;
			let value = key * 1000 + step as i64;

			// Index lookup must agree with a linear scan of both queues.
			let linear = cache
				.small
				.iter()
				.chain(cache.main.iter())
				.find(|entry| entry.key == key)
				.map(|entry| entry.value);
			let got = cache.get(&key).copied();
			assert_eq!(got, linear, "step {step}: get({key}) diverged from linear scan");

			if got.is_none() {
				cache.put(key, value);
			} else if op == 0 {
				cache.put(key, value);
				assert_eq!(cache.get(&key).copied(), Some(value), "step {step}: resident put did not update");
			}
			cache.check_indexes();
		}
	}
}
