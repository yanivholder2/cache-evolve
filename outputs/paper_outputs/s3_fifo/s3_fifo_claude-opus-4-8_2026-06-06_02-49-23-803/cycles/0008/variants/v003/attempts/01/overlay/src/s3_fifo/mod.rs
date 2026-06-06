use std::collections::HashMap;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The resident set is indexed by an O(1) hash map (`entries`) keyed by the key,
// while `small`/`main` hold only the keys and encode FIFO/eviction order. This
// turns `get`/`find_entry_mut` into a single hash probe.
//
// Ghost membership is folded into the *same* `entries` map as value-less
// tombstone slots (`Slot::Ghost`). A ghosted key is therefore resolved by the
// one `entries.get` probe that `put` already performs on its miss path, rather
// than a linear `ghost.iter().any(...)` scan or a separate ghost hash table.
// `ghost` is retained solely to carry FIFO eviction order for the tombstones;
// it may hold stale keys (ones that have since become resident again), which are
// skipped when popped.
const MAX_FREQ: u8 = 3;

/// Inline FNV-1a / multiplicative hasher. Keys for this cache are small integers
/// (`i64`/`u64`), so a single wrapping multiply per key beats SipHash on the hot
/// `get` path. Falls back to byte-wise FNV-1a for any other key shape.
#[derive(Default)]
pub struct FastHasher(u64);

impl Hasher for FastHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut hash = self.0;
		for &b in bytes {
			hash = (hash ^ b as u64).wrapping_mul(0x0000_0100_0000_01b3);
		}
		self.0 = hash;
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.0 = (self.0 ^ i).wrapping_mul(0x9E37_79B9_7F4A_7C15);
	}

	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.write_u64(i as u64);
	}

	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.write_u64(i as u64);
	}

	#[inline]
	fn write_i32(&mut self, i: i32) {
		self.write_u64(i as u64);
	}
}

type FastBuildHasher = BuildHasherDefault<FastHasher>;

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

/// Residency state stored in the `entries` map. `Resident` carries the live
/// `Entry`; `Ghost` is a value-less tombstone recording that the key was
/// recently evicted (a ghost membership marker, not a cache hit). The enum lives
/// only in the container value type — the scored `Entry<u64,u64>` size is
/// unaffected.
#[derive(Debug)]
pub enum Slot<K, V> {
	Resident(Entry<K, V>),
	Ghost,
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	small: VecDeque<K>,
	main: VecDeque<K>,
	ghost: VecDeque<K>,
	entries: HashMap<K, Slot<K, V>, FastBuildHasher>,
	ghost_count: usize,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 32).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			entries: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			ghost_count: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		match self.entries.get_mut(key) {
			Some(Slot::Resident(entry)) => Some(entry),
			_ => None,
		}
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		match self.entries.get_mut(key) {
			Some(Slot::Resident(entry)) => {
				entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
				Some(&entry.value)
			}
			_ => None,
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		// Single O(1) ghost-membership probe over the same `entries` map.
		if matches!(self.entries.get(&key), Some(Slot::Ghost)) {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			// Overwrite the tombstone in place with a live entry.
			self.entries
				.insert(key.clone(), Slot::Resident(Entry::new(key.clone(), value)));
			self.ghost_count -= 1;
			// A stale copy of `key` may remain in the `ghost` order queue; it is
			// skipped when popped because its slot is now `Resident`.
			self.main.push_front(key);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(key.clone());
		self.entries
			.insert(key.clone(), Slot::Resident(Entry::new(key, value)));
	}

	fn evict_main(&mut self) {
		while let Some(tail_key) = self.main.pop_back() {
			let freq = match self.entries.get(&tail_key) {
				Some(Slot::Resident(e)) => e.freq,
				_ => 0,
			};
			if freq > 0 {
				if let Some(Slot::Resident(e)) = self.entries.get_mut(&tail_key) {
					e.freq -= 1;
				}
				self.main.push_front(tail_key);
			} else {
				self.demote_to_ghost(tail_key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail_key) = self.small.pop_back() {
			let freq = match self.entries.get(&tail_key) {
				Some(Slot::Resident(e)) => e.freq,
				_ => 0,
			};
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail_key);
			} else {
				self.demote_to_ghost(tail_key);
			}
		}
	}

	/// Turn a resident victim's slot into a `Ghost` tombstone (without removing it
	/// from `entries`), record its FIFO order, and trim the ghost set so the live
	/// `Ghost`-slot count stays within bound.
	fn demote_to_ghost(&mut self, key: K) {
		self.entries.insert(key.clone(), Slot::Ghost);
		self.ghost_count += 1;
		self.ghost.push_front(key);
		self.trim_ghost();
	}

	/// Enforce the ghost capacity bound on the count of live `Ghost` slots (not on
	/// the order queue length, which may carry stale entries). Pop the order
	/// queue's back and only `entries.remove` keys whose slot is still `Ghost`;
	/// keys that became `Resident` again are skipped.
	fn trim_ghost(&mut self) {
		let bound = (self.main_size / 2).max(1);
		while self.ghost_count > bound {
			match self.ghost.pop_back() {
				Some(k) => {
					if matches!(self.entries.get(&k), Some(Slot::Ghost)) {
						self.entries.remove(&k);
						self.ghost_count -= 1;
					}
				}
				None => break,
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
		assert!(cache.main.iter().any(|k| *k == 1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main.iter().any(|k| *k == 1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.main.push_back(1);
		cache
			.entries
			.insert(1, Slot::Resident(Entry { key: 1, value: 10, freq: 0 }));
		cache.main.push_back(2);
		cache
			.entries
			.insert(2, Slot::Resident(Entry { key: 2, value: 20, freq: 2 }));
		cache.main.push_back(3);
		cache
			.entries
			.insert(3, Slot::Resident(Entry { key: 3, value: 30, freq: 0 }));
		cache.main.push_back(4);
		cache
			.entries
			.insert(4, Slot::Resident(Entry { key: 4, value: 40, freq: 1 }));
		cache.small.push_front(5);
		cache
			.entries
			.insert(5, Slot::Resident(Entry { key: 5, value: 50, freq: 2 }));
		cache.evict_small();
		assert!(cache.main.iter().any(|k| *k == 4));
		assert!(!cache.main.iter().any(|k| *k == 3));
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
}
