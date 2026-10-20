use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The per-entry frequency counter lives in a parallel array (`small_freq` /
// `main_freq`) kept in lockstep with its queue: index i of the freq deque is the
// counter for index i of the entry deque. Entries therefore hold only key and
// value, which keeps `Entry<u64, u64>` at 16 bytes.
//
// Resident lookup goes through `index`, a map from key to a slot holding the
// queue it lives in and a monotonically increasing id. Each queue has a head
// counter that rises on every push to its front, so a resident at deque position
// `i` has `id = head - i` and its position is `head - id`. Every push and pop
// goes through the four helpers below so the index and head counters stay in
// step with the deques.
//
// The ghost is a map from 64-bit key fingerprint to the value of `ghost_seq` at
// the fingerprint's last push. A fingerprint is live while it was pushed within
// the last `ghost_cap` pushes. Expired entries are swept in batches when the map
// grows past twice `ghost_cap`, so eviction never removes a ghost entry one at a
// time and the ghost population stays bounded near `ghost_cap`.
//
// Frequency aging: every `age_period` (the cache capacity) inserts of a new key,
// every main-queue frequency counter is halved. The pass runs on the insert path
// only, so `get` is unchanged and the cost amortises to O(1) per insert.
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

#[derive(Debug, Clone, Copy)]
struct Slot {
	in_small: bool,
	id: u64,
}

#[inline]
fn bump_freq(freq: &mut u8) {
	*freq = (freq.saturating_add(1)).min(MAX_FREQ);
}

#[inline]
fn fingerprint<K: Hash>(key: &K) -> u64 {
	let mut hasher = DefaultHasher::new();
	key.hash(&mut hasher);
	hasher.finish()
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	small_freq: VecDeque<u8>,
	main: VecDeque<Entry<K, V>>,
	main_freq: VecDeque<u8>,
	index: HashMap<K, Slot>,
	small_head: u64,
	main_head: u64,
	ghost: HashMap<u64, u64>,
	ghost_seq: u64,
	ghost_cap: usize,
	small_size: usize,
	main_size: usize,
	ops_since_age: u32,
	age_period: u32,
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
			(capacity / 5).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let ghost_cap = (main_size / 2).max(1);
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			small_freq: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			main_freq: VecDeque::with_capacity(main_size),
			index: HashMap::with_capacity(capacity),
			small_head: 0,
			main_head: 0,
			ghost: HashMap::with_capacity(2 * ghost_cap + 1),
			ghost_seq: 0,
			ghost_cap,
			small_size,
			main_size,
			ops_since_age: 0,
			age_period: capacity.min(u32::MAX as usize) as u32,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	// Records an evicted fingerprint at the front of the ghost. Expired entries are
	// swept in one batch once the map exceeds twice the window, which amortises to
	// O(1) per push.
	fn ghost_push(&mut self, fp: u64) {
		self.ghost_seq += 1;
		self.ghost.insert(fp, self.ghost_seq);
		if self.ghost.len() > 2 * self.ghost_cap {
			let cap = self.ghost_cap as u64;
			let now = self.ghost_seq;
			self.ghost.retain(|_, seq| *seq + cap > now);
		}
	}

	// Counts one insert of a new key. Once `age_period` inserts have accumulated,
	// every main-queue frequency counter is halved so that stale hot entries can
	// become evictable again. Small-queue counters are left alone.
	fn age_after_insert(&mut self) {
		self.ops_since_age += 1;
		if self.ops_since_age >= self.age_period {
			self.ops_since_age = 0;
			for freq in self.main_freq.iter_mut() {
				*freq >>= 1;
			}
		}
	}
}

impl<K: PartialEq + Eq + Clone + Hash, V> S3Fifo<K, V> {
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = *self.index.get(key)?;
		if slot.in_small {
			let pos = (self.small_head - slot.id) as usize;
			bump_freq(&mut self.small_freq[pos]);
			Some(&self.small[pos].value)
		} else {
			let pos = (self.main_head - slot.id) as usize;
			bump_freq(&mut self.main_freq[pos]);
			Some(&self.main[pos].value)
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&slot) = self.index.get(&key) {
			if slot.in_small {
				let pos = (self.small_head - slot.id) as usize;
				self.small[pos].value = value;
			} else {
				let pos = (self.main_head - slot.id) as usize;
				self.main[pos].value = value;
			}
			return;
		}

		let fp = fingerprint(&key);
		// The fingerprint is removed either way; it counts as a hit only if it was
		// pushed within the last `ghost_cap` pushes.
		let ghost_hit = match self.ghost.remove(&fp) {
			Some(seq) => seq + self.ghost_cap as u64 > self.ghost_seq,
			None => false,
		};
		if ghost_hit {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.push_main(Entry::new(key, value), 0);
			self.age_after_insert();
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.push_small(Entry::new(key, value), 0);
		self.age_after_insert();
	}

	fn push_small(&mut self, entry: Entry<K, V>, freq: u8) {
		self.small_head += 1;
		self.index.insert(
			entry.key.clone(),
			Slot {
				in_small: true,
				id: self.small_head,
			},
		);
		self.small.push_front(entry);
		self.small_freq.push_front(freq);
	}

	fn push_main(&mut self, entry: Entry<K, V>, freq: u8) {
		self.main_head += 1;
		self.index.insert(
			entry.key.clone(),
			Slot {
				in_small: false,
				id: self.main_head,
			},
		);
		self.main.push_front(entry);
		self.main_freq.push_front(freq);
	}

	fn pop_small(&mut self) -> Option<(Entry<K, V>, u8)> {
		let entry = self.small.pop_back()?;
		let freq = self.small_freq.pop_back()?;
		self.index.remove(&entry.key);
		Some((entry, freq))
	}

	fn pop_main(&mut self) -> Option<(Entry<K, V>, u8)> {
		let entry = self.main.pop_back()?;
		let freq = self.main_freq.pop_back()?;
		self.index.remove(&entry.key);
		Some((entry, freq))
	}

	fn evict_main(&mut self) {
		while let Some((entry, freq)) = self.pop_main() {
			if freq > 0 {
				// Second chance: re-pushing refreshes the slot's id to the new head.
				self.push_main(entry, freq - 1);
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

	#[cfg(test)]
	fn check_indexes(&self) {
		assert_eq!(self.small.len(), self.small_freq.len());
		assert_eq!(self.main.len(), self.main_freq.len());
		assert_eq!(self.index.len(), self.small.len() + self.main.len());
		for (pos, entry) in self.small.iter().enumerate() {
			let slot = self.index.get(&entry.key).expect("small key missing from index");
			assert!(slot.in_small, "small key indexed as main");
			assert_eq!(self.small_head - slot.id, pos as u64, "small slot position mismatch");
		}
		for (pos, entry) in self.main.iter().enumerate() {
			let slot = self.index.get(&entry.key).expect("main key missing from index");
			assert!(!slot.in_small, "main key indexed as small");
			assert_eq!(self.main_head - slot.id, pos as u64, "main slot position mismatch");
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

	// Capacity 10 gives small_size 2 and ghost_cap 4, so a key evicted two pushes
	// ago is still inside the ghost window.
	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(10);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|entry| entry.key == 1));
	}

	// Key 1 is evicted to the ghost, then six more evictions push it out of the
	// window of ghost_cap = 4, so re-putting it must not promote it to main.
	#[test]
	fn ghost_hit_expires_after_window() {
		let mut cache = S3Fifo::new(10);
		for key in 1..=8 {
			cache.put(key, key);
		}
		cache.put(1, 10);
		assert!(!cache.main.iter().any(|entry| entry.key == 1));
		assert!(cache.small.iter().any(|entry| entry.key == 1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		// Pushed in reverse so the back of `main` (the eviction end) is key 1, then 2, 3, 4.
		cache.push_main(Entry::new(4, 40), 1);
		cache.push_main(Entry::new(3, 30), 0);
		cache.push_main(Entry::new(2, 20), 2);
		cache.push_main(Entry::new(1, 10), 0);
		cache.push_small(Entry::new(5, 50), 2);
		cache.check_indexes();
		cache.evict_small();
		cache.check_indexes();
		assert!(cache.main.iter().any(|entry| entry.key == 4));
		assert!(!cache.main.iter().any(|entry| entry.key == 3));
	}

	// Capacity 4 gives small_size 1, main_size 3 and age_period 4. Key 1 reaches
	// main with freq 3; the fourth new-key insert triggers aging, halving it to 1.
	#[test]
	fn aging_halves_main_freq_every_capacity_inserts() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.get(&1);
		cache.get(&1);
		cache.get(&1);
		cache.put(2, 20);
		assert_eq!(cache.main_freq.front(), Some(&3));
		cache.put(3, 30);
		cache.put(4, 40);
		assert_eq!(cache.main_freq.front(), Some(&1));
		assert_eq!(cache.ops_since_age, 0);
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
	fn freq_deques_stay_in_lockstep() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			if cache.get(&(key % 6)).is_none() {
				cache.put(key % 6, key);
			}
			assert_eq!(cache.small.len(), cache.small_freq.len());
			assert_eq!(cache.main.len(), cache.main_freq.len());
		}
	}

	#[test]
	fn index_stays_in_sync_on_seeded_trace() {
		let mut cache = S3Fifo::new(16);
		let mut state: u64 = 0x9E37_79B9_7F4A_7C15;
		for _ in 0..5_000 {
			state ^= state << 13;
			state ^= state >> 7;
			state ^= state << 17;
			let key = (state % 40) as i64;
			match cache.get(&key).copied() {
				Some(value) => assert_eq!(value, key),
				None => cache.put(key, key),
			}
			cache.check_indexes();
		}
	}

	#[test]
	fn ghost_stays_bounded() {
		let mut cache = S3Fifo::new(64);
		for key in 0..10_000 {
			cache.put(key, key);
		}
		assert!(cache.ghost.len() <= 2 * cache.ghost_cap + 1);
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
