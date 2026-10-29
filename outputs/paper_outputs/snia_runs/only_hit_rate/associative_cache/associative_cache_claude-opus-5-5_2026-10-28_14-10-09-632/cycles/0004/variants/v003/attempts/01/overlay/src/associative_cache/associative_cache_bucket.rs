use super::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::traits::eviction_policy::EvictionPolicy;
use log::trace;

use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

/// Newly admitted items are inserted at `len / INSERT_FRACTION_DIV` (measured
/// from the LRU end, index 0) instead of the MRU tail. LRU-insertion-point
/// scheme: a new item must be hit again before it drifts the remaining
/// ~1/INSERT_FRACTION_DIV of the bucket down to the eviction end.
const INSERT_FRACTION_DIV: usize = 4;

/// Bimodal insertion (BIP): every `BIMODAL_PERIOD`-th new item admitted to a
/// bucket is inserted at the MRU tail instead of the probation point, giving a
/// deterministic sample of long-reuse-distance items a full bucket lifetime.
const BIMODAL_PERIOD: u32 = 32;

/// Graded promotion: the protected region starts at `len / PROTECT_FRACTION_DIV`.
/// A hit on an item below it moves the item only to the bottom of the protected
/// region (index `entry - 1` after the removal shift); a hit on an item already
/// at or above that slot moves it to the MRU tail.
const PROTECT_FRACTION_DIV: usize = 2;

/// Buckets smaller than this keep plain policy behaviour on hits.
const GRADED_MIN_LEN: usize = 4;

#[derive(Debug)]
pub struct AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	items: Vec<Item<K, V>>,
	max_size: usize,
	eviction_policy: Box<dyn EvictionPolicy<K, V>>,
	/// Per-bucket count of new-item insertions (drives the bimodal choice).
	insert_count: u32,
	/// Graded promotion applies only to the position-ordered ClassicLRU policy;
	/// other policies keep their own hit semantics.
	graded: bool,
}

impl<K, V> AssociativeCacheBucket<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &AssociativeCacheSettings) -> Self {
		let eviction_policy: Box<dyn EvictionPolicy<K, V>> = settings.eviction_policy.build();
		let graded = eviction_policy.name() == "ClassicLRU";
		Self {
			items: Vec::with_capacity(settings.bucket_size),
			max_size: settings.bucket_size,
			eviction_policy,
			insert_count: 0,
			graded,
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

	/// Graded promotion (ClassicLRU only, `len >= 4`): a hit below the
	/// protected-region bottom slot (`len/2 - 1`) moves the item to that slot;
	/// a hit at or above it moves the item to the MRU tail.
	fn get_from_position(&mut self, position: usize) -> &Item<K, V> {
		let len = self.items.len();
		if self.graded && len >= GRADED_MIN_LEN {
			let entry = len / PROTECT_FRACTION_DIV;
			// Bottom slot of the protected region after the removal shift.
			let target = entry - 1;
			if position < target {
				let it = self.items.remove(position);
				self.items.insert(target, it);
				return &self.items[target];
			}
		}
		let index = self.eviction_policy.get_from_position(&mut self.items, position);
		&self.items[index]
	}

	/// Bimodal LRU-insertion-point put: hits on existing keys go through the
	/// graded `get_from_position`; new items are inserted at the bucket's 25%
	/// position (from the LRU end), except every `BIMODAL_PERIOD`-th new item,
	/// which goes to the MRU tail.
	fn put(&mut self, item: Item<K, V>) -> (&Item<K, V>, Option<Item<K, V>>) {
		if let Some(position) = self.get_position(&item.key) {
			trace!("Bucket.put: key={:?} update_existing", item.key);
			self.items[position].value = item.value;
			let pushed_item = self.get_from_position(position);
			return (pushed_item, None);
		}
		let evicted_item = if self.is_full() { self.evict_item() } else { None };
		self.insert_count = self.insert_count.wrapping_add(1);
		let idx = if self.insert_count % BIMODAL_PERIOD == 0 {
			self.items.push(item);
			self.items.len() - 1
		} else {
			let ins = self.items.len() / INSERT_FRACTION_DIV;
			if ins == 0 {
				self.items.push(item);
				self.items.len() - 1
			} else {
				self.items.insert(ins, item);
				ins
			}
		};
		(&self.items[idx], evicted_item)
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;

	fn bucket_with_keys(n: i64) -> AssociativeCacheBucket<i64, i64> {
		let settings = AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: n as usize,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
		};
		let mut b = AssociativeCacheBucket::new(&settings);
		for k in 0..n {
			b.get_items_mut().push(Item::new(k, k * 10));
		}
		b
	}

	fn keys(b: &AssociativeCacheBucket<i64, i64>) -> Vec<i64> {
		b.get_items().iter().map(|i| i.key).collect()
	}

	#[test]
	fn probation_hit_lands_below_midpoint() {
		let mut b = bucket_with_keys(8);
		let hit = b.get(&1).unwrap();
		assert_eq!(hit.key, 1);
		assert_eq!(hit.value, 10);
		assert_eq!(b.get_position(&1), Some(8 / 2 - 1));
		assert_eq!(keys(&b), vec![0, 2, 3, 1, 4, 5, 6, 7]);
		assert_eq!(b.size(), 8);
	}

	#[test]
	fn second_hit_moves_to_mru() {
		let mut b = bucket_with_keys(8);
		b.get(&1);
		let hit = b.get(&1).unwrap();
		assert_eq!(hit.key, 1);
		assert_eq!(b.get_position(&1), Some(7));
		assert_eq!(b.size(), 8);
	}

	#[test]
	fn upper_half_hit_moves_to_mru() {
		let mut b = bucket_with_keys(8);
		b.get(&5);
		assert_eq!(keys(&b), vec![0, 1, 2, 3, 4, 6, 7, 5]);
	}

	#[test]
	fn small_bucket_keeps_lru() {
		let mut b = bucket_with_keys(3);
		b.get(&0);
		assert_eq!(keys(&b), vec![1, 2, 0]);
	}

	#[test]
	fn put_existing_uses_graded_promotion() {
		let mut b = bucket_with_keys(8);
		let (it, ev) = b.put(Item::new(0, 99));
		assert_eq!(it.key, 0);
		assert_eq!(it.value, 99);
		assert!(ev.is_none());
		assert_eq!(b.get_position(&0), Some(3));
		assert_eq!(b.size(), 8);
	}
}
