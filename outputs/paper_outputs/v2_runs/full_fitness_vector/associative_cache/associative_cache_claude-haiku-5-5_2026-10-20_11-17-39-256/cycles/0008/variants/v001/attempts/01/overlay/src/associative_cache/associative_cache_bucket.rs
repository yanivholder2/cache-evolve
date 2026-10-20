use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;

use std::collections::hash_map::DefaultHasher;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};

/// Per-slot access frequency saturates here (a 2-bit-style counter).
const FREQ_SATURATION: usize = 3;
/// Every this many insertions into a bucket, all of its frequency counters are halved.
const AGING_PERIOD: usize = 256;
/// Doorkeeper bits per slot in the bucket's bitset (rounded up to a power of two).
const DOORKEEPER_BITS_PER_SLOT: usize = 4;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Count of insertions into this bucket, driving the periodic aging pass.
	insertions: usize,
	/// Heap-allocated doorkeeper bitset: one bit per hashed key slot, set on first
	/// sighting of a key that would have to evict. Cleared on each aging pass.
	doorkeeper: Vec<u64>,
	/// Bit-index mask for `doorkeeper` (the bit count is a power of two).
	doorkeeper_mask: usize,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let doorkeeper_bits = (settings.bucket_size * DOORKEEPER_BITS_PER_SLOT).max(1).next_power_of_two();
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy: settings.eviction_policy.build(),
			insertions: 0,
			doorkeeper: vec![0u64; (doorkeeper_bits + 63) / 64],
			doorkeeper_mask: doorkeeper_bits - 1,
		}
	}

	/// Locates the doorkeeper bit for `key`. Uses the upper half of the key hash so
	/// the bits are not correlated with the low bits that select the bucket.
	fn doorkeeper_slot(&self, key: &K) -> (usize, u64) {
		let mut hasher = DefaultHasher::new();
		key.hash(&mut hasher);
		let bit = ((hasher.finish() >> 32) as usize) & self.doorkeeper_mask;
		(bit >> 6, 1u64 << (bit & 63))
	}

	/// Doorkeeper admission check for a put miss. Returns `true` if the key was
	/// already seen (admit through the normal insert path). Otherwise records the
	/// first sighting and returns `false` (the caller bypasses the insert).
	pub fn doorkeeper_admit(&mut self, key: &K) -> bool {
		let (word, mask) = self.doorkeeper_slot(key);
		if self.doorkeeper[word] & mask != 0 {
			true
		} else {
			self.doorkeeper[word] |= mask;
			false
		}
	}
}

impl<K, V> Bucket<K, V> for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	fn get_items(&self) -> &Vec<Item<K, V>> {
		&self.items
	}

	fn get_items_mut(&mut self) -> &mut Vec<Item<K, V>> {
		&mut self.items
	}

	fn get_max_size(&self) -> usize {
		self.max_size
	}

	fn get_eviction_policy(&self) -> &dyn EvictionPolicy<K, V> {
		&*self.eviction_policy
	}

	/// Insert a new key (or update an existing one). Slots start with frequency 0,
	/// and the bucket ages all frequency counters every `AGING_PERIOD` insertions.
	/// The aging pass also clears the doorkeeper bitset.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.items.push(item);
		self.insertions += 1;
		if self.insertions % AGING_PERIOD == 0 {
			for slot in self.items.iter_mut() {
				slot.lfu_counter >>= 1;
			}
			self.doorkeeper.fill(0);
		}
		let pushed_item = self.items.last().expect("bucket invariant: just pushed");
		(pushed_item, evicted_item)
	}

	/// Victim selection: the slot with the smallest frequency counter. Ties go to
	/// the lowest index (`min_by_key` returns the first minimum), which is the
	/// baseline's front-of-bucket choice order.
	fn eviction_candidate_index(&self) -> Option<usize> {
		self.items
			.iter()
			.enumerate()
			.min_by_key(|(_, item)| item.lfu_counter)
			.map(|(index, _)| index)
	}

	/// Promotion is unchanged (delegated to the policy); the hit additionally bumps
	/// the slot's saturating frequency counter.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		let item = &mut self.items[index];
		item.lfu_counter = (item.lfu_counter + 1).min(FREQ_SATURATION);
		&self.items[index]
	}
}

impl<K, V> Display for AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for item in &self.items {
			writeln!(f, "    {}", item)?;
		}
		Ok(())
	}
}
