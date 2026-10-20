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
// `resident` indexes small/main only. Ghost membership lives in a separate
// table (`ghost_map`: key -> ring sequence number), pre-sized to the ghost
// bound. The ghost ring stores (key, seq); entries whose seq no longer matches
// the table are stale (promoted out) and are skipped lazily on ring eviction.
//
// Keys are wrapped in `HKey`, which carries the key's hash computed exactly
// once. The maps use an identity hasher that just replays the stored hash, so
// resident / ghost / insert / eviction probes never rehash.
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

/// Identity hasher: replays a precomputed hash passed via `write_u64`.
#[derive(Default, Clone, Copy)]
pub struct IdentityHasher(u64);

impl Hasher for IdentityHasher {
	#[inline]
	fn finish(&self) -> u64 {
		self.0
	}

	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		for &b in bytes {
			self.0 = (self.0.rotate_left(8)) ^ b as u64;
		}
	}

	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.0 = i;
	}
}

type IdBuild = BuildHasherDefault<IdentityHasher>;
type HSet<K> = HashSet<HKey<K>, IdBuild>;
type HMap<K, V> = HashMap<HKey<K>, V, IdBuild>;

/// Key paired with its once-computed hash.
#[derive(Debug, Clone)]
pub struct HKey<K> {
	hash: u64,
	key: K,
}

impl<K: Hash> HKey<K> {
	#[inline]
	fn new(key: K) -> Self {
		let mut h = FxLikeHasher::default();
		key.hash(&mut h);
		Self { hash: h.finish(), key }
	}
}

impl<K: PartialEq> PartialEq for HKey<K> {
	#[inline]
	fn eq(&self, other: &Self) -> bool {
		self.hash == other.hash && self.key == other.key
	}
}
impl<K: Eq> Eq for HKey<K> {}

impl<K> Hash for HKey<K> {
	#[inline]
	fn hash<H: Hasher>(&self, state: &mut H) {
		state.write_u64(self.hash);
	}
}

#[derive(Debug)]
pub struct Entry<K, V> {
	key: HKey<K>,
	value: V,
	freq: u8,
}

impl<K, V> Entry<K, V> {
	fn new(key: HKey<K>, value: V) -> Self {
		Self { key, value, freq: 0 }
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: VecDeque<(HKey<K>, u64)>,
	resident: HSet<K>,
	ghost_map: HMap<K, u64>,
	ghost_seq: u64,
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
			(capacity * 3 / 100).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			resident: HSet::with_capacity_and_hasher(capacity, IdBuild::default()),
			ghost_map: HMap::with_capacity_and_hasher(main_size, IdBuild::default()),
			ghost_seq: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &HKey<K>) -> Option<&mut Entry<K, V>> {
		if !self.resident.contains(key) {
			return None;
		}
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| entry.key.key == key.key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		// Hash once; clone is avoided by building a temporary wrapper around a clone
		// only on the (cheap for the workload) lookup path.
		let hk = HKey::new(key.clone());
		let entry = self.find_entry_mut(&hk)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		// Single hash computation for the whole put.
		let hk = HKey::new(key);

		if let Some(entry) = self.find_entry_mut(&hk) {
			entry.value = value;
			return;
		}

		// Miss in the resident index: single probe of the ghost table.
		if self.ghost_map.remove(&hk).is_some() {
			// Ring entry becomes stale (no live seq); skipped lazily.
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			self.resident.insert(hk.clone());
			self.main.push_front(Entry::new(hk, value));
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		self.resident.insert(hk.clone());
		self.small.push_front(Entry::new(hk, value));
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
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(tail);
			} else {
				self.resident.remove(&tail.key);
				// Live ghost count == ghost_map.len(); matches the exact-order ring bound.
				if self.ghost_map.len() >= self.main_size {
					self.evict_ghost_oldest();
				}
				// Bound stale slots so the ring stays O(ghost bound).
				if self.ghost.len() >= 2 * self.main_size + 8 {
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
