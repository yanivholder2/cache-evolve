use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The small-queue size adapts online (ARC-style) from the ghost-hit ratio
// measured over windows of `capacity` puts. The ghost bound stays fixed.
//
// `resident` indexes small/main only. Ghost membership lives in a separate
// table (`ghost_map`: key -> ring sequence number), pre-sized to the ghost
// bound. The ghost ring stores (key, seq); entries whose seq no longer matches
// the table are stale (promoted out) and are skipped lazily on ring eviction.
const MAX_FREQ: u8 = 3;

const FX_SEED: u64 = 0x517cc1b727220a95;

/// Cheap multiplicative (Fx-style) hasher.
#[derive(Default, Clone, Copy)]
pub struct FxLikeHasher(u64);

impl FxLikeHasher {
	#[inline]
	fn fold(&mut self, x: u64) {
		self.0 = (self.0.rotate_left(5) ^ x).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxLikeHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut b = [0u8; 8];
			b.copy_from_slice(c);
			self.fold(u64::from_le_bytes(b));
		}
		for &byte in chunks.remainder() {
			self.fold(byte as u64);
		}
	}

	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.fold(i as u64);
	}
	#[inline]
	fn write_u16(&mut self, i: u16) {
		self.fold(i as u64);
	}
	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.fold(i as u64);
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.fold(i);
	}
	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.fold(i as u64);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.fold(i as u64);
	}
}

type FxBuild = BuildHasherDefault<FxLikeHasher>;
type FxSet<K> = HashSet<K, FxBuild>;
type FxMap<K, V> = HashMap<K, V, FxBuild>;

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
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<(K, u64)>,
	resident: FxSet<K>,
	ghost_map: FxMap<K, u64>,
	ghost_seq: u64,
	capacity: usize,
	small_size: usize,
	main_size: usize,
	// Fixed ghost bound (initial main_size).
	ghost_bound: usize,
	min_small: usize,
	max_small: usize,
	step: usize,
	window_puts: usize,
	ghost_hits: usize,
	small_evict_to_ghost: usize,
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
			(capacity * 3 / 100).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		let (min_small, max_small) = if capacity == 1 {
			(1, 1)
		} else {
			let lo = (capacity / 100).max(1).min(capacity - 1);
			let hi = (capacity * 10 / 100).max(lo).min(capacity - 1);
			(lo.min(small_size), hi.max(small_size))
		};
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			resident: FxSet::with_capacity_and_hasher(capacity, FxBuild::default()),
			ghost_map: FxMap::with_capacity_and_hasher(main_size, FxBuild::default()),
			ghost_seq: 0,
			capacity,
			small_size,
			main_size,
			ghost_bound: main_size,
			min_small,
			max_small,
			step: (capacity / 200).max(1),
			window_puts: 0,
			ghost_hits: 0,
			small_evict_to_ghost: 0,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		if !self.resident.contains(key) {
			return None;
		}
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let entry = self.find_entry_mut(key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	/// Close a measurement window: adapt `small_size` from the ghost-hit ratio.
	fn adapt(&mut self) {
		let hits = self.ghost_hits;
		let inserts = self.small_evict_to_ghost;
		self.window_puts = 0;
		self.ghost_hits = 0;
		self.small_evict_to_ghost = 0;
		if inserts == 0 || self.min_small == self.max_small {
			return;
		}
		// ratio > 0.5 -> grow; ratio < 0.1 -> shrink.
		if hits * 2 > inserts {
			self.small_size = (self.small_size + self.step).min(self.max_small);
			self.main_size = self.capacity - self.small_size;
		} else if hits * 10 < inserts {
			self.small_size = self.small_size.saturating_sub(self.step).max(self.min_small);
			self.main_size = self.capacity - self.small_size;
			while self.small.len() > self.small_size {
				self.evict_small();
			}
		}
	}

	/// Make room in main so that one more entry fits.
	fn make_room_main(&mut self) {
		while self.main.len() >= self.main_size && !self.main.is_empty() {
			self.evict_main();
		}
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		self.window_puts += 1;
		if self.window_puts >= self.capacity {
			self.adapt();
		}

		// Miss in the resident index: single probe of the ghost table.
		if self.ghost_map.remove(&key).is_some() {
			self.ghost_hits += 1;
			// Ring entry becomes stale (no live seq); skipped lazily.
			self.make_room_main();
			self.resident.insert(key.clone());
			self.main.push_front(Entry::new(key, value));
			return;
		}

		while self.small.len() >= self.small_size && !self.small.is_empty() {
			self.evict_small();
		}
		// After growing small, main may still be over its new bound: drain it.
		while self.small.len() + self.main.len() >= self.capacity && !self.main.is_empty() {
			self.evict_main();
		}
		self.resident.insert(key.clone());
		self.small.push_front(Entry::new(key, value));
	}

	fn evict_main(&mut self) {
		while let Some(mut tail) = self.main.pop_back() {
			if tail.freq > 0 {
				tail.freq -= 1;
				self.main.push_front(tail);
			} else {
				self.resident.remove(&tail.key);
				break;
			}
		}
	}

	/// Drop the oldest live ghost (skipping stale ring slots).
	fn evict_ghost_oldest(&mut self) {
		while let Some((k, seq)) = self.ghost.pop_back() {
			if self.ghost_map.get(&k) == Some(&seq) {
				self.ghost_map.remove(&k);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			if tail.freq > 1 {
				self.make_room_main();
				self.main.push_front(tail);
			} else {
				self.resident.remove(&tail.key);
				self.small_evict_to_ghost += 1;
				// Live ghost count == ghost_map.len(); matches the exact-order ring bound.
				if self.ghost_map.len() >= self.ghost_bound {
					self.evict_ghost_oldest();
				}
				// Bound stale slots so the ring stays O(ghost bound).
				if self.ghost.len() >= 2 * self.ghost_bound + 8 {
					let map = &self.ghost_map;
					self.ghost.retain(|(k, s)| map.get(k) == Some(s));
				}
				let seq = self.ghost_seq;
				self.ghost_seq = seq.wrapping_add(1);
				self.ghost_map.insert(tail.key.clone(), seq);
				self.ghost.push_front((tail.key, seq));
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
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
		}
		assert!(cache.small.len() + cache.main.len() <= 4);
		assert_eq!(cache.resident.len(), cache.small.len() + cache.main.len());
	}
}
