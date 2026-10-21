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
// Policy (unchanged from cycle/0003/v003): 5% small queue, MAX_FREQ = 7,
// graduation from small at freq > 1, ghost re-admission at the front of main
// with freq 0, CLOCK-style rotation in main.
//
// H01 (cycle 4): structure only. Entries live in a slab addressed by slot id;
// a HashMap index gives O(1) residency lookup; the small and main FIFOs hold
// slot ids.
//
// H03 (cycle 5): main-queue eviction victims are also recorded in the ghost
// (same main_size cap), so a returning main victim re-enters main directly.
//
// H02 (cycle 6): the ghost is a fixed-size ring of key fingerprints
// (`ghost_ring`, length main_size.max(1)) plus a `ghost_map` from fingerprint
// to ring slot. No `K` is cloned into the ghost and there is no compaction.
// Behavioural notes:
//   * A ghost hit removes the entry and zeroes its slot, leaving a hole; the
//     ring does not shift. The window is therefore bounded by ring *position*
//     (the last main_size ghost inserts) rather than by live count, so the
//     effective ghost window can be slightly larger than before.
//   * Fingerprint collisions are accepted: a colliding key may be treated as a
//     ghost hit (admitted to main) — a policy decision only, never a
//     correctness issue for residency/values.
//
// H03 (cycle 7): fingerprints narrowed from u64 to u32 (64-bit DefaultHasher
// output folded as `h ^ (h >> 32)`), with a `u32 -> u32` ghost map. With at
// most ~main_size live fingerprints the false ghost-hit rate per miss is about
// main_size / 2^32.
const MAX_FREQ: u8 = 7;

/// Fixed-seed 32-bit fingerprint of a key. 0 is reserved for "empty slot".
#[inline]
fn fingerprint<K: Hash>(key: &K) -> u32 {
	let mut h = DefaultHasher::new();
	key.hash(&mut h);
	let h = h.finish();
	let fp = (h ^ (h >> 32)) as u32;
	if fp == 0 {
		1
	} else {
		fp
	}
}

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
	ghost_ring: Vec<u32>,
	ghost_head: usize,
	ghost_map: HashMap<u32, u32>,
	small_size: usize,
	main_size: usize,
}

impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		// Small queue is 5% of capacity (inherited from cycle/0002/v003).
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 20).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		if main_size.max(1) > u32::MAX as usize {
			log::error!("S3Fifo.try_new: capacity too large for ghost ring");
			return None;
		}
		Some(Self {
			slab: Vec::with_capacity(capacity + 1),
			free: Vec::with_capacity(capacity + 1),
			index: HashMap::with_capacity(capacity + 1),
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size + 1),
			ghost_ring: vec![0u32; main_size.max(1)],
			ghost_head: 0,
			ghost_map: HashMap::with_capacity(main_size + 1),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	#[inline]
	fn entry_mut(&mut self, idx: usize) -> &mut Entry<K, V> {
		self.slab[idx].as_mut().expect("S3Fifo: queued slot must be occupied")
	}

	fn alloc(&mut self, key: K, value: V) -> usize {
		let entry = Entry::new(key.clone(), value);
		let idx = match self.free.pop() {
			Some(idx) => {
				self.slab[idx] = Some(entry);
				idx
			}
			None => {
				self.slab.push(Some(entry));
				self.slab.len() - 1
			}
		};
		self.index.insert(key, idx);
		idx
	}

	fn release(&mut self, idx: usize) -> Entry<K, V> {
		let entry = self.slab[idx].take().expect("S3Fifo: released slot must be occupied");
		self.free.push(idx);
		self.index.remove(&entry.key);
		entry
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let idx = *self.index.get(key)?;
		let entry = self.slab[idx].as_mut()?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(&idx) = self.index.get(&key) {
			self.entry_mut(idx).value = value;
			return;
		}

		let fp: u32 = fingerprint(&key);
		if let Some(slot) = self.ghost_map.remove(&fp) {
			self.ghost_ring[slot as usize] = 0;
			if self.main.len() >= self.main_size {
				self.evict_main();
			}
			let idx = self.alloc(key, value);
			self.main.push_front(idx);
			return;
		}

		if self.small.len() >= self.small_size {
			self.evict_small();
		}
		let idx = self.alloc(key, value);
		self.small.push_front(idx);
	}

	fn evict_main(&mut self) {
		while let Some(idx) = self.main.pop_back() {
			let entry = self.entry_mut(idx);
			if entry.freq > 0 {
				entry.freq -= 1;
				self.main.push_front(idx);
			} else {
				// H03: remember main victims in the ghost so a return
				// re-enters main directly.
				let entry = self.release(idx);
				self.ghost_insert(&entry.key);
				break;
			}
		}
	}

	fn evict_small(&mut self) {
		if let Some(idx) = self.small.pop_back() {
			if self.entry_mut(idx).freq > 1 {
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(idx);
			} else {
				let entry = self.release(idx);
				self.ghost_insert(&entry.key);
			}
		}
	}

	fn ghost_insert(&mut self, key: &K) {
		let fp: u32 = fingerprint(key);
		let slot = self.ghost_head;
		let old: u32 = self.ghost_ring[slot];
		// Evict the oldest ring entry only if the map still points at this
		// slot (a stale slot — hole or superseded duplicate — is just overwritten).
		if old != 0 && self.ghost_map.get(&old) == Some(&(slot as u32)) {
			self.ghost_map.remove(&old);
		}
		self.ghost_ring[slot] = fp;
		self.ghost_map.insert(fp, slot as u32);
		self.ghost_head += 1;
		if self.ghost_head == self.ghost_ring.len() {
			self.ghost_head = 0;
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
impl<K: Eq + Hash + Clone, V> S3Fifo<K, V> {
	fn test_push_main_back(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.main.push_back(idx);
	}

	fn test_push_main_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.main.push_front(idx);
	}

	fn test_push_small_front(&mut self, key: K, value: V, freq: u8) {
		let idx = self.alloc(key, value);
		self.entry_mut(idx).freq = freq;
		self.small.push_front(idx);
	}

	fn queue_keys(&self, q: &VecDeque<usize>) -> Vec<(K, u8)> {
		q.iter()
			.map(|&idx| {
				let e = self.slab[idx].as_ref().unwrap();
				(e.key.clone(), e.freq)
			})
			.collect()
	}

	fn main_keys(&self) -> Vec<K> {
		self.queue_keys(&self.main).into_iter().map(|(k, _)| k).collect()
	}

	fn small_keys(&self) -> Vec<K> {
		self.queue_keys(&self.small).into_iter().map(|(k, _)| k).collect()
	}

	fn freq_of(&self, key: &K) -> Option<u8> {
		self.index.get(key).map(|&idx| self.slab[idx].as_ref().unwrap().freq)
	}

	/// Fingerprints are irreversible; ghost membership is checked by key.
	fn ghost_contains(&self, key: &K) -> bool {
		self.ghost_map.contains_key(&fingerprint(key))
	}

	fn resident_len(&self) -> usize {
		self.small.len() + self.main.len()
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
		assert!(!cache.ghost_contains(&1));
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
	fn main_victim_is_ghosted_and_returns_to_main() {
		// capacity 20 -> small_size 1, main_size 19.
		let mut cache = S3Fifo::<i64, i64>::new(20);
		assert_eq!(cache.main_size, 19);
		cache.test_push_main_back(7, 70, 0);
		for k in 100..118i64 {
			cache.test_push_main_front(k, k, 1);
		}
		assert_eq!(cache.main.len(), cache.main_size);
		assert_eq!(*cache.main_keys().last().unwrap(), 7);

		cache.test_push_small_front(50, 50, 2);
		cache.evict_small();
		assert!(cache.get(&7).is_none());
		assert!(cache.ghost_contains(&7));
		assert!(cache.main_keys().contains(&50));

		cache.put(7, 71);
		assert!(cache.main_keys().contains(&7));
		assert!(!cache.small_keys().contains(&7));
		assert!(!cache.ghost_contains(&7));
		assert_eq!(cache.freq_of(&7), Some(0));
		assert!(cache.resident_len() <= 20);
		assert_eq!(*cache.get(&7).unwrap(), 71);
	}

	#[test]
	fn capacity_bound_never_exceeded() {
		let mut cache = S3Fifo::new(4);
		for key in 0..20 {
			cache.put(key, key);
			assert!(cache.ghost_map.len() <= cache.main_size.max(1));
		}
		assert!(cache.resident_len() <= 4);
		assert_eq!(cache.index.len(), cache.resident_len());
		assert!(cache.ghost_map.len() <= cache.main_size);
	}

	#[test]
	fn small_queue_is_five_percent_of_capacity() {
		let cache = S3Fifo::<i64, i64>::new(2048);
		assert_eq!(cache.small_size, 102);
		assert_eq!(cache.main_size, 1946);
		assert_eq!(cache.ghost_ring.len(), 1946);
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
	fn main_resident_frequency_saturates_at_seven() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.test_push_main_front(1, 10, 0);
		for _ in 0..10 {
			assert_eq!(*cache.get(&1).unwrap(), 10);
		}
		assert_eq!(cache.freq_of(&1), Some(7));
		assert_eq!(MAX_FREQ, 7);
	}

	#[test]
	fn max_freq_key_survives_seven_main_passes_then_evicted_on_eighth() {
		let mut cache = S3Fifo::<i64, i64>::new(8);
		cache.test_push_main_back(1, 10, 0);
		for _ in 0..10 {
			cache.get(&1);
		}
		assert_eq!(cache.freq_of(&1), Some(MAX_FREQ));

		for pass in 0..7i64 {
			let cold = 100 + pass;
			cache.test_push_main_front(cold, cold, 0);
			cache.evict_main();
			assert!(cache.main_keys().contains(&1), "hot key evicted on pass {}", pass + 1);
			assert!(!cache.main_keys().contains(&cold));
			assert!(cache.get(&cold).is_none());
		}
		assert_eq!(cache.freq_of(&1), Some(0));

		cache.test_push_main_front(200, 200, 0);
		assert_eq!(*cache.main_keys().last().unwrap(), 1);
		cache.evict_main();
		assert!(!cache.main_keys().contains(&1));
		assert!(cache.main_keys().contains(&200));
	}

	#[test]
	fn ghost_ring_is_fixed_size_and_bounded() {
		let mut cache = S3Fifo::<i64, i64>::new(40);
		let ring_len = cache.ghost_ring.len();
		for key in 0..10_000i64 {
			let k = key % 500;
			if cache.get(&k).is_none() {
				cache.put(k, k);
			}
			assert!(cache.ghost_map.len() <= cache.main_size.max(1));
			assert_eq!(cache.ghost_ring.len(), ring_len);
			assert!(cache.ghost_head < ring_len);
		}
	}

	#[test]
	fn fingerprint_never_zero_and_deterministic() {
		for k in -1000i64..1000 {
			let fp: u32 = fingerprint(&k);
			assert_ne!(fp, 0);
			assert_eq!(fp, fingerprint(&k));
		}
	}

	// ---- Reference: cycle/0003/v003 linear implementation (MAX_FREQ = 7),
	// ---- plus H03 (main victims ghosted) and H02 ring-with-holes ghost.

	struct RefEntry {
		key: i64,
		value: i64,
		freq: u8,
	}

	struct RefS3Fifo {
		small: VecDeque<RefEntry>,
		main: VecDeque<RefEntry>,
		ring: Vec<Option<i64>>,
		head: usize,
		small_size: usize,
		main_size: usize,
	}

	impl RefS3Fifo {
		fn new(capacity: usize) -> Self {
			let small_size = if capacity == 1 {
				1
			} else {
				(capacity / 20).max(1).min(capacity - 1)
			};
			let main_size = capacity - small_size;
			Self {
				small: VecDeque::new(),
				main: VecDeque::new(),
				ring: vec![None; main_size.max(1)],
				head: 0,
				small_size,
				main_size,
			}
		}

		fn get(&mut self, key: &i64) -> Option<i64> {
			let entry = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| &e.key == key)?;
			entry.freq = (entry.freq.saturating_add(1)).min(7);
			Some(entry.value)
		}

		fn put(&mut self, key: i64, value: i64) {
			if let Some(entry) = self.small.iter_mut().chain(self.main.iter_mut()).find(|e| e.key == key) {
				entry.value = value;
				return;
			}
			if let Some(pos) = self.ring.iter().position(|g| *g == Some(key)) {
				self.ring[pos] = None;
				if self.main.len() >= self.main_size {
					self.evict_main();
				}
				self.main.push_front(RefEntry { key, value, freq: 0 });
				return;
			}
			if self.small.len() >= self.small_size {
				self.evict_small();
			}
			self.small.push_front(RefEntry { key, value, freq: 0 });
		}

		fn ghost_push(&mut self, key: i64) {
			self.ring[self.head] = Some(key);
			self.head = (self.head + 1) % self.ring.len();
		}

		fn evict_main(&mut self) {
			while let Some(mut tail) = self.main.pop_back() {
				if tail.freq > 0 {
					tail.freq -= 1;
					self.main.push_front(tail);
				} else {
					self.ghost_push(tail.key);
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
					self.ghost_push(tail.key);
				}
			}
		}
	}

	#[test]
	fn matches_ring_reference_on_pseudo_random_streams() {
		for &capacity in &[1usize, 2, 3, 4, 8, 20, 41, 64, 100] {
			for &(key_space, skew) in &[(16u64, false), (64, true), (300, true), (1000, false)] {
				let mut cache = S3Fifo::<i64, i64>::new(capacity);
				let mut reference = RefS3Fifo::new(capacity);
				let mut state: u64 = 0x9E37_79B9_7F4A_7C15 ^ (capacity as u64 * 31 + key_space);
				for step in 0..5_000 {
					state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
					let r = state >> 33;
					let key = if skew {
						let a = r % key_space;
						let b = (r >> 12) % key_space;
						a.min(b) as i64
					} else {
						(r % key_space) as i64
					};

					let got = cache.get(&key).copied();
					let expected = reference.get(&key);
					assert_eq!(got, expected, "cap={capacity} step={step} key={key}");
					if got.is_none() {
						cache.put(key, key * 3);
						reference.put(key, key * 3);
					}

					let small: Vec<(i64, u8)> = cache.queue_keys(&cache.small);
					let ref_small: Vec<(i64, u8)> = reference.small.iter().map(|e| (e.key, e.freq)).collect();
					assert_eq!(small, ref_small, "small mismatch cap={capacity} step={step}");
					let main: Vec<(i64, u8)> = cache.queue_keys(&cache.main);
					let ref_main: Vec<(i64, u8)> = reference.main.iter().map(|e| (e.key, e.freq)).collect();
					assert_eq!(main, ref_main, "main mismatch cap={capacity} step={step}");

					// Ghost: ring layout and live membership match exactly.
					let ref_ring: Vec<u32> =
						reference.ring.iter().map(|g| g.map(|k| fingerprint(&k)).unwrap_or(0)).collect();
					assert_eq!(cache.ghost_ring, ref_ring, "ghost ring mismatch cap={capacity} step={step}");
					assert_eq!(cache.ghost_head, reference.head, "ghost head mismatch cap={capacity} step={step}");
					let live: usize = reference.ring.iter().filter(|g| g.is_some()).count();
					assert_eq!(cache.ghost_map.len(), live, "ghost live mismatch cap={capacity} step={step}");
					for k in reference.ring.iter().flatten() {
						assert!(cache.ghost_contains(k), "ghost missing {k} cap={capacity} step={step}");
					}
					assert!(cache.ghost_map.len() <= cache.main_size.max(1));
					assert_eq!(cache.index.len(), cache.resident_len());
				}
			}
		}
	}
}
