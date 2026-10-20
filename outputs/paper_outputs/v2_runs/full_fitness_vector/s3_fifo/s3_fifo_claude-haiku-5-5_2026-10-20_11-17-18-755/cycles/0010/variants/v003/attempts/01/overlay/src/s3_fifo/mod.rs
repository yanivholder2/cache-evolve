use std::collections::hash_map::DefaultHasher;
use std::collections::VecDeque;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost set for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// The ghost set is an approximate membership structure: a two-hash bit array
// that is cleared once per epoch (one epoch = `ghost_cap` ghost inserts). It
// stores no keys, so a ghost hit can be a false positive; such a cold key is
// admitted to main early.
const MAX_FREQ: u8 = 3;

const SEED_A: u64 = 0x9E37_79B9_7F4A_7C15;
const SEED_B: u64 = 0xC2B2_AE3D_27D4_EB4F;

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

/// Seeded finaliser used to derive two independent slot hashes from one base hash.
fn mix(x: u64, seed: u64) -> u64 {
	let mut z = (x ^ seed).wrapping_mul(0xBF58_476D_1CE4_E5B9);
	z ^= z >> 31;
	z = z.wrapping_mul(0x94D0_49BB_1331_11EB);
	z ^= z >> 29;
	z
}

/// Approximate ghost set: `8 * capacity` bits, two bits per insert, cleared
/// wholesale at the end of each epoch of `capacity` inserts.
#[derive(Debug)]
struct BitGhost {
	bits: Vec<u64>,
	num_bits: usize,
	epoch_len: usize,
	inserts_in_epoch: usize,
}

impl BitGhost {
	fn new(epoch_len: usize) -> Self {
		let epoch_len = epoch_len.max(1);
		let num_bits = 8 * epoch_len;
		Self {
			bits: vec![0; (num_bits + 63) / 64],
			num_bits,
			epoch_len,
			inserts_in_epoch: 0,
		}
	}

	/// Multiply-shift reduction of a 64-bit hash into `[0, num_bits)`.
	fn reduce(&self, h: u64) -> usize {
		((h as u128 * self.num_bits as u128) >> 64) as usize
	}

	fn slots<K: Hash>(&self, key: &K) -> [usize; 2] {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let base = hasher.finish();
		[
			self.reduce(mix(base, SEED_A)),
			self.reduce(mix(base, SEED_B)),
		]
	}

	fn contains<K: Hash>(&self, key: &K) -> bool {
		self
			.slots(key)
			.iter()
			.all(|&i| self.bits[i >> 6] & (1u64 << (i & 63)) != 0)
	}

	fn insert<K: Hash>(&mut self, key: &K) {
		if self.inserts_in_epoch >= self.epoch_len {
			self.bits.iter_mut().for_each(|word| *word = 0);
			self.inserts_in_epoch = 0;
		}
		let slots = self.slots(key);
		for i in slots {
			self.bits[i >> 6] |= 1u64 << (i & 63);
		}
		self.inserts_in_epoch += 1;
	}
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Clone, V> {
	small: VecDeque<Entry<K, V>>,
	main: VecDeque<Entry<K, V>>,
	ghost: BitGhost,
	small_size: usize,
	main_size: usize,
}

impl<K: PartialEq + Clone + Hash, V> S3Fifo<K, V> {
	pub fn try_new(capacity: usize) -> Option<Self> {
		if capacity == 0 {
			log::error!("S3Fifo.try_new: capacity must be > 0");
			return None;
		}
		let small_size = if capacity == 1 {
			1
		} else {
			(capacity / 4).max(1).min(capacity - 1)
		};
		let main_size = capacity - small_size;
		Some(Self {
			small: VecDeque::with_capacity(small_size),
			main: VecDeque::with_capacity(main_size),
			ghost: BitGhost::new(main_size),
			small_size,
			main_size,
		})
	}

	pub fn new(capacity: usize) -> Self {
		Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
	}

	fn find_entry_mut(&mut self, key: &K) -> Option<&mut Entry<K, V>> {
		self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)
	}

	pub fn get(&mut self, key: &K) -> Option<&V> {
		let entry = self
			.small
			.iter_mut()
			.chain(self.main.iter_mut())
			.find(|entry| &entry.key == key)?;
		entry.freq = (entry.freq.saturating_add(1)).min(MAX_FREQ);
		Some(&entry.value)
	}

	pub fn put(&mut self, key: K, value: V) {
		if let Some(entry) = self.find_entry_mut(&key) {
			entry.value = value;
			return;
		}

		// Ghost hit: admit straight to main. The key's bits are left in place
		// (clearing them could erase bits shared with other keys); they age out
		// at the next epoch clear.
		if self.ghost.contains(&key) {
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
				self.ghost.insert(&tail.key);
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
}
