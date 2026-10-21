use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// H02 (cycle 0003): behaviour-neutral port of cycle/0002/v003 onto indexed
// structures. Entries live in a slab; `small`/`main` are FIFOs of slot ids;
// `index` maps resident keys to slots; the ghost FIFO uses lazy deletion with
// per-insertion sequence numbers and a `ghost_set` for O(1) membership.
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
pub struct S3Fifo<K: Eq + Hash + Clone, V> {
	slab: Vec<Option<Entry<K, V>>>,
	free: Vec<usize>,
	index: HashMap<K, usize>,
	small: VecDeque<usize>,
	main: VecDeque<usize>,
	/// Ghost FIFO (front = newest). Elements whose seq does not match
	/// `ghost_set` are stale and skipped lazily.
	ghost: VecDeque<(K, u64)>,
	ghost_set: HashMap<K, u64>,
	ghost_seq: u64,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		// Small queue is 5% of capacity (unchanged from v003).
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			slab: Vec::with_capacity(capacity),
			free: Vec::new(),
			index: HashMap::with_capacity(capacity),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_set: HashMap::with_capacity(main_size),
			ghost_seq: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn entry(&self, slot: usize) -> &Entry<K, V> {
		self.slab[slot].as_ref().expect("live slot")
	}

	#[inline]
	fn entry_mut(&mut self, slot: usize) -> &mut Entry<K, V> {
		self.slab[slot].as_mut().expect("live slot")
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		let entry = Entry::new(key.clone(), value);
		let slot = match self.free.pop() {
			Some(slot) => {
				self.slab[slot] = Some(entry);
				slot
			}
			None => {
				self.slab.push(Some(entry));
				self.slab.len() - 1
			}
		};
		self.index.insert(key, slot);
		slot
	}

	/// Removes the entry in `slot` from the slab and index, returning its key.
	fn release(&mut self, slot: usize) -> K {
		let entry = self.slab[slot].take().expect("live slot");
		self.index.remove(&entry.key);
		self.free.push(slot);
		entry.key
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = *self.index.get(key)?;
		let entry = self.slab[slot].as_mut().expect("live slot");
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&slot) = self.index.get(&key) {
			self.entry_mut(slot).value = value;
			return;
		}

		if self.ghost_set.remove(&key).is_some() {
			// Stale deque element is skipped lazily.
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let slot = self.alloc(key, value);
			self.main.push_front(slot);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let slot = self.alloc(key, value);
		self.small.push_front(slot);
	}

	fn evict_main(&mut self) {
		while let Some(slot) = self.main.pop_back() {
			let entry = self.entry_mut(slot);
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(slot);
			} else {
				self.release(slot);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(slot) = self.small.pop_back() {
			if self.entry(slot).freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(slot);
			} else {
				let key = self.release(slot);
				self.ghost_insert(key);
			}
		}
	}

	fn ghost_insert(&mut self, key: K) {
		// Mirrors v003: `if ghost.len() >= main_size { ghost.pop_back() }`,
		// where len counts only live ghost entries.
		if self.ghost_set.len() >= self.main_size {
			while let Some((old, seq)) = self.ghost.pop_back() {
				if self.ghost_set.get(&old) == Some(&seq) {
					self.ghost_set.remove(&old);
					break;
				}
			}
		}
		let seq = self.ghost_seq;
		self.ghost_seq = self.ghost_seq.wrapping_add(1);
		self.ghost_set.insert(key.clone(), seq);
		self.ghost.push_front((key, seq));

		// Compact stale elements so the deque stays O(main_size).
		if self.ghost.len() > 2 * self.main_size.max(1) + 16 {
			let ghost_set = &self.ghost_set;
			self.ghost.retain(|(k, s)| ghost_set.get(k) == Some(s));
		}
	}

	#[cfg(test)]
	fn main_keys(&self) -> Vec<K> {
		self.main.iter().map(|&s| self.entry(s).key.clone()).collect()
	}

	#[cfg(test)]
	fn small_keys(&self) -> Vec<K> {
		self.small.iter().map(|&s| self.entry(s).key.clone()).collect()
	}

	#[cfg(test)]
	fn ghost_keys(&self) -> Vec<K> {
		self
			.ghost
			.iter()
			.filter(|(k, s)| self.ghost_set.get(k) == Some(s))
			.map(|(k, _)| k.clone())
			.collect()
	}

	#[cfg(test)]
	fn test_push_main_back(&mut self, key: K, value: V, freq: u8) {
		let slot = self.alloc(key, value);
		self.entry_mut(slot).freq = freq;
		self.main.push_back(slot);
	}

	#[cfg(test)]
	fn test_push_small_front(&mut self, key: K, value: V, freq: u8) {
		let slot = self.alloc(key, value);
		self.entry_mut(slot).freq = freq;
		self.small.push_front(slot);
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

	/// Verbatim reference copy of cycle/0002/v003's linear-scan logic.
	mod reference {
		use std::collections::VecDeque;

		const MAX_FREQ: u8 = 3;

		pub struct Entry {
			pub key: i64,
			pub value: i64,
			pub freq: u8,
		}

		pub struct RefS3Fifo {
			pub small: VecDeque<Entry>,
			pub main: VecDeque<Entry>,
			pub ghost: VecDeque<i64>,
			small_size: usize,
			main_size: usize,
		}

		impl RefS3Fifo {
			pub fn new(capacity: usize) -> Self {
				let small_size = if capacity == 1 {
					1
				} else {
					(capacity / 20).max(1).min(capacity - 1)
				};
				let main_size = capacity - small_size;
				Self {
					small: VecDeque::new(),
					main: VecDeque::new(),
					ghost: VecDeque::new(),
					small_size,
					main_size,
				}
			}

			pub fn get(&mut self, key: &i64) -> Option<&i64> {
				let entry = self
					.small
					.iter_mut()
					.chain(self.main.iter_mut())
					.find(|entry| &entry.key == key)?;
				entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
				Some(&entry.value)
			}

			pub fn put(&mut self, key: i64, value: i64) {
				if let Some(entry) = self
					.small
					.iter_mut()
					.chain(self.main.iter_mut())
					.find(|entry| entry.key == key)
				{
					entry.value = value;
					return;
				}
				if self.ghost.iter().any(|g| g == &key) {
					self.ghost.retain(|g| g != &key);
					if self.main.len() >= self.main_size {
						self.evict_main();
					}
					self.main.push_front(Entry { key, value, freq: 0 });
					return;
				}
				if self.small.len() >= self.small_size {
					self.evict_small();
				}
				self.small.push_front(Entry { key, value, freq: 0 });
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

			fn evict_small(&mut self) {
				if let Some(tail) = self.small.pop_back() {
					if tail.freq > 1 {
						if self.main.len() >= self.main_size {
							self.evict_main();
						}
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
	}

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
		assert!(cache.main_keys().contains(&1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(cache.main_keys().contains(&1));
	}

	#[test]
	fn main_eviction_uses_frequency_second_chance() {
		let mut cache = S3Fifo::new(5);
		cache.test_push_main_back(1, 10, 0);
		cache.test_push_main_back(2, 20, 2);
		cache.test_push_main_back(3, 30, 0);
		cache.test_push_main_back(4, 40, 1);
		cache.test_push_small_front(5, 50, 2);
		cache.evict_small();
		assert!(cache.main_keys().contains(&4));
		assert!(!cache.main_keys().contains(&3));
		assert!(cache.get(&3).is_none());
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_eq!(cache.index.len(), cache.small.len() + cache.main.len());
	}

	#[test]
	fn small_queue_is_five_percent_of_capacity() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_size, 102);
		assert_eq!(cache.main_size, 1946);
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
	fn matches_v003_reference_on_pseudo_random_streams() {
		for &(capacity, key_space, seed) in &[
			(1usize, 4u64, 1u64),
			(2, 8, 2),
			(4, 16, 3),
			(8, 40, 4),
			(20, 60, 5),
			(64, 300, 6),
			(256, 1500, 7),
		] {
			let mut new = S3Fifo::<i64, i64>::new(capacity);
			let mut old = reference::RefS3Fifo::new(capacity);
			let mut state = seed.wrapping_mul(0x9E37_79B9_7F4A_7C15) | 1;
			for step in 0..50_000 {
				state ^= state << 13;
				state ^= state >> 7;
				state ^= state << 17;
				// Skew toward low keys so there is reuse.
				let r = state % key_space;
				let key = if state & 0x100 == 0 { (r % (key_space / 4 + 1)) as i64 } else { r as i64 };
				let a = new.get(&key).copied();
				let b = old.get(&key).copied();
				assert_eq!(a, b, "hit/miss diverged cap={capacity} step={step} key={key}");
				if a.is_none() {
					new.put(key, key * 7);
					old.put(key, key * 7);
				}
			}
			assert_eq!(new.small_keys(), old.small.iter().map(|e| e.key).collect::<Vec<_>>());
			assert_eq!(new.main_keys(), old.main.iter().map(|e| e.key).collect::<Vec<_>>());
			assert_eq!(new.ghost_keys(), old.ghost.iter().copied().collect::<Vec<_>>());
		}
	}
}
