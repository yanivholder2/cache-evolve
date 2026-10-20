use std::collections::{HashMap, VecDeque};
use std::fmt::Debug;
use std::hash::{BuildHasherDefault, Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The main queue is a CLOCK-style ring: its front is the eviction head, new
// entries are pushed to the back, and second-chance entries are rotated from
// the front to the back (pop_front/push_back only).
//
// A single index `HashMap<K, Loc>` records where each key lives (small, main,
// or ghost). Resident entries are stored in a slab; the queues hold slab slots.
// The ghost queue holds (key, seq) pairs; a ghost hit leaves a tombstone that
// is recognised by the index no longer holding `Ghost(seq)` for that key.
//
// The index uses an inline Fx-style multiplicative hasher and is pre-sized so
// that no rehash happens during a run.
const MAX_FREQ: u8 = 3;

const FX_SEED: u64 = 0x51_7c_c1_b7_27_22_0a_95;

#[derive(Default, Clone, Copy)]
pub struct FxHasher {
	hash: u64,
}

impl FxHasher {
	#[inline]
	fn add(&mut self, word: u64) {
		self.hash = (self.hash.rotate_left(5) ^ word).wrapping_mul(FX_SEED);
	}
}

impl Hasher for FxHasher {
	#[inline]
	fn write(&mut self, bytes: &[u8]) {
		let mut chunks = bytes.chunks_exact(8);
		for c in &mut chunks {
			let mut b = [0u8; 8];
			b.copy_from_slice(c);
			self.add(u64::from_le_bytes(b));
		}
		for &b in chunks.remainder() {
			self.add(b as u64);
		}
	}
	#[inline]
	fn write_u8(&mut self, i: u8) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u32(&mut self, i: u32) {
		self.add(i as u64);
	}
	#[inline]
	fn write_u64(&mut self, i: u64) {
		self.add(i);
	}
	#[inline]
	fn write_i64(&mut self, i: i64) {
		self.add(i as u64);
	}
	#[inline]
	fn write_usize(&mut self, i: usize) {
		self.add(i as u64);
	}
	#[inline]
	fn finish(&self) -> u64 {
		// Fold high bits down: hashbrown uses both top 7 bits and low bits.
		self.hash ^ (self.hash >> 29)
	}
}

type FxBuild = BuildHasherDefault<FxHasher>;

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

#[derive(Debug, Clone, Copy)]
enum Loc {
	Small(usize),
	Main(usize),
	Ghost(u64),
}

#[derive(Debug)]
pub struct S3Fifo<K: Hash + Eq + Clone, V> {
	index: HashMap<K, Loc, FxBuild>,
	slab: Vec<Option<Entry<K, V>>>,
	free: Vec<usize>,
	small: VecDeque<usize>,
	main: VecDeque<usize>,
	ghost: VecDeque<(K, u64)>,
	ghost_live: usize,
	ghost_seq: u64,
	small_size: usize,
	main_size: usize,
}

impl<K: Hash + Eq + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		// (capacity + ghost bound) * 1.25, plus slack for transient states.
		let idx_cap = (capacity + main_size) * 5 / 4 + 8;
		Some(Self {
			index: HashMap::with_capacity_and_hasher(idx_cap, FxBuild::default()),
			slab: Vec::with_capacity(capacity + 1),
			free: Vec::new(),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: VecDeque::with_capacity(main_size),
			ghost_live: 0,
			ghost_seq: 0,
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn slot_of(loc: Loc) -> Option<usize> {
		match loc {
			Loc::Small(s) | Loc::Main(s) => Some(s),
			Loc::Ghost(_) => None,
		}
	}

	#[inline]
	fn alloc(&mut self, entry: Entry<K, V>) -> usize {
		if let Some(s) = self.free.pop() {
			self.slab[s] = Some(entry);
			s
		} else {
			self.slab.push(Some(entry));
			self.slab.len() - 1
		}
	}

	#[inline]
	pub fn get(&mut self, key: &K) -> Option<&V> {
		let slot = Self::slot_of(*self.index.get(key)?)?;
		let entry = self.slab[slot].as_mut()?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	#[inline]
	pub fn put(&mut self, key: K, value: V) {
		match self.index.get(&key).copied() {
			Some(Loc::Small(s)) | Some(Loc::Main(s)) => {
				if let Some(entry) = self.slab[s].as_mut() {
					entry.value = value;
				}
				return;
			}
			Some(Loc::Ghost(_)) => {
				// Tombstone the ghost entry (its queue slot is skipped later).
				self.ghost_live -= 1;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				let slot = self.alloc(Entry::new(key.clone(), value));
				self.main.push_back(slot);
				self.index.insert(key, Loc::Main(slot));
				return;
			}
			None => {}
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let slot = self.alloc(Entry::new(key.clone(), value));
		self.small.push_front(slot);
		self.index.insert(key, Loc::Small(slot));
	}

	fn evict_main(&mut self) {
		while let Some(head) = self.main.pop_front() {
			let freq = self.slab[head].as_ref().map_or(0, |e| e.freq);
			if freq > 0 {
				if let Some(e) = self.slab[head].as_mut() {
					e.freq -= 1;
				}
				self.main.push_back(head);
			} else {
				if let Some(e) = self.slab[head].take() {
					self.index.remove(&e.key);
				}
				self.free.push(head);
				break;
			}
		}
	}

	fn pop_ghost_oldest(&mut self) {
		while let Some((k, seq)) = self.ghost.pop_back() {
			if matches!(self.index.get(&k), Some(Loc::Ghost(s)) if *s == seq) {
				self.index.remove(&k);
				self.ghost_live -= 1;
				return;
			}
		}
	}

	fn compact_ghost(&mut self) {
		let index = &self.index;
		self.ghost
			.retain(|(k, seq)| matches!(index.get(k), Some(Loc::Ghost(s)) if *s == *seq));
	}

	fn evict_small(&mut self) {
		if let Some(tail) = self.small.pop_back() {
			let freq = self.slab[tail].as_ref().map_or(0, |e| e.freq);
			if freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_back(tail);
				if let Some(e) = self.slab[tail].as_ref() {
					if let Some(loc) = self.index.get_mut(&e.key) {
						*loc = Loc::Main(tail);
					}
				}
			} else {
				if self.ghost_live >= self.main_size {
					self.pop_ghost_oldest();
				}
				if self.ghost.len() > 2 * self.main_size + 8 {
					self.compact_ghost();
				}
				if let Some(e) = self.slab[tail].take() {
					let seq = self.ghost_seq;
					self.ghost_seq += 1;
					if let Some(loc) = self.index.get_mut(&e.key) {
						*loc = Loc::Ghost(seq);
					}
					self.ghost.push_front((e.key, seq));
					self.ghost_live += 1;
				}
				self.free.push(tail);
			}
		}
	}
}

impl<K: PartialEq + Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V> for S3Fifo<K, V> {
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

	fn in_main(c: &S3Fifo<i64, i64>, k: i64) -> bool {
		matches!(c.index.get(&k), Some(Loc::Main(_)))
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
		assert!(in_main(&cache, 1));
	}

	#[test]
	fn ghost_hit_promotes_back_to_main() {
		let mut cache = S3Fifo::new(4);
		cache.put(1, 10);
		cache.put(2, 20);
		cache.put(3, 30);
		cache.put(4, 40);
		cache.put(1, 10);
		assert!(in_main(&cache, 1));
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
