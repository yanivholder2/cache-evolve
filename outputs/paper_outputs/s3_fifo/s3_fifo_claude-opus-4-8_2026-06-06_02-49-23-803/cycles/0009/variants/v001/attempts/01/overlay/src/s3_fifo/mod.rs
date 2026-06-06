use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};
use std::marker::PhantomData;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The resident set is indexed by an O(1) hash map (`entries`) keyed by the
// resident key, while `small`/`main` hold only the keys and continue to encode
// FIFO/eviction order exactly as before. This turns `get`/`find_entry_mut` from
// a linear queue walk into a single hash probe without changing any eviction
// decision.
//
// Ghost membership is likewise indexed by an O(1) hash set (`ghost_set`). The
// `ghost` VecDeque is retained purely to encode FIFO eviction order, and uses
// lazy deletion: a key removed from `ghost_set` (on a ghost hit) is left in the
// VecDeque as a tombstone and skipped when later popped. This removes the last
// linear scan on the hot `put` path (the old `ghost.iter().any(..)` membership
// test and `ghost.retain(..)` removal), without changing any eviction decision.
//
// Each resident `Entry` stores only `{ value, freq }`. The key is *not*
// duplicated in the entry record: the `entries` map is already keyed by the
// resident key, so the per-entry copy is dead weight. `K` is retained as a type
// parameter via a zero-sized `PhantomData<K>`, shrinking `Entry<u64,u64>` from
// 24 to 16 bytes without changing any policy decision.
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
	value: V,
	freq: u8,
	_k: PhantomData<K>,
}

impl<K, V> Entry<K, V> {
	fn new(value: V) -> Self {
		Self { value, freq: 0, _k: PhantomData }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	small: VecDeque<K>,
	main: VecDeque<K>,
	ghost: VecDeque<K>,
	ghost_set: HashSet<K, FastBuildHasher>,
	entries: HashMap<K, Entry<K, V>, FastBuildHasher>,
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
			ghost_set: HashSet::with_capacity_and_hasher(main_size, FastBuildHasher::default()),
			entries: HashMap::with_capacity_and_hasher(capacity, FastBuildHasher::default()),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self.entries.get_mut(key)
	}

	/// Push a freshly-evicted key into the ghost queue, enforcing the live-entry
	/// capacity bound `(main_size/2).max(1)` against `ghost_set.len()`. Trimming
	/// pops from the back of the FIFO order, skipping tombstones (keys no longer
	/// present in `ghost_set`) until it discards one live entry. The key is then
	/// recorded both in the order queue and the membership set.
	fn push_ghost(&mut self, key: K) {
		let bound = (self.main_size / 2).max(1);
		if self.ghost_set.len() >= bound {
			while let Some(old) = self.ghost.pop_back() {
				// `remove` returns true only for a live entry; tombstones are
				// skipped (false) and we keep popping until one live entry is
				// discarded, making room for the new key.
				if self.ghost_set.remove(&old) {
					break;
				}
			}
		}
		self.ghost.push_front(key.clone());
		self.ghost_set.insert(key);
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let entry = self.entries.get_mut(key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		// O(1) ghost membership test. On a hit, remove the live membership entry
		// (the order-queue slot is left as a tombstone and reclaimed lazily).
		if self.ghost_set.remove(&key) {
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.main.push_front(key.clone());
			self.entries.insert(key, Entry::new(value));
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.small.push_front(key.clone());
		self.entries.insert(key, Entry::new(value));
	}

	fn evict_main(&mut self) {
		while let Some(tail_key) = self.main.pop_back() {
			let freq = self.entries.get(&tail_key).map(|e| e.freq).unwrap_or(0);
			if freq > 0 {
				if let Some(entry) = self.entries.get_mut(&tail_key) {
					entry.freq -= 1;
				}
				self.main.push_front(tail_key);
			} else {
				self.entries.remove(&tail_key);
				self.push_ghost(tail_key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail_key) = self.small.pop_back() {
			let freq = self.entries.get(&tail_key).map(|e| e.freq).unwrap_or(0);
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail_key);
			} else {
				self.entries.remove(&tail_key);
				self.push_ghost(tail_key);
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
		cache.entries.insert(1, Entry { value: 10, freq: 0, _k: PhantomData });
		cache.main.push_back(2);
		cache.entries.insert(2, Entry { value: 20, freq: 2, _k: PhantomData });
		cache.main.push_back(3);
		cache.entries.insert(3, Entry { value: 30, freq: 0, _k: PhantomData });
		cache.main.push_back(4);
		cache.entries.insert(4, Entry { value: 40, freq: 1, _k: PhantomData });
		cache.small.push_front(5);
		cache.entries.insert(5, Entry { value: 50, freq: 2, _k: PhantomData });
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
