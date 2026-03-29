use super::dash_bucket::DashBucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::get_index;
use log::debug;
use std::fmt::{Debug, Display};
use std::hash::Hash;

#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub buckets: Vec<DashBucket<K, V>>,
	pub segment_size: usize,
	pub stash_buckets: Vec<DashBucket<K, V>>,
	pub stash_size: usize,
}

impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub fn new(settings: &DashSettings) -> Self {
		let mut buckets: Vec<DashBucket<K, V>> = Vec::with_capacity(settings.num_of_normal_buckets);
		for _ in 0..settings.num_of_normal_buckets {
			buckets.push(DashBucket::new(settings.bucket_size, settings.eviction_policy.clone()));
		}

		let mut stash_buckets: Vec<DashBucket<K, V>> = Vec::with_capacity(settings.num_of_stash_buckets);
		for _ in 0..settings.num_of_stash_buckets {
			stash_buckets.push(DashBucket::new(settings.bucket_size, settings.eviction_policy.clone()));
		}
		DashSegment {
			buckets,
			stash_buckets,
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
		}
	}

	/// Returns a reference to the item with `key`.
	///
	/// As a side effect makes updates according to the eviction policy.
	/// Lookup order: stash bucket → target bucket → probing bucket (target + 1).
	pub fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let target_bucket_index = get_index(key, self.segment_size);
		let stash_bucket_index = get_index(key, self.stash_size);

		// 1. Check stash bucket — on hit, move item to target bucket
		if let Some(position) = self.stash_buckets[stash_bucket_index].get_position(key) {
			debug!("DashSegment.get: key={key:?} found_in=stash stash_bucket={stash_bucket_index} -> target_bucket={target_bucket_index}");
			let data = self.stash_buckets[stash_bucket_index].get_from_position(position).clone();
			self.stash_buckets[stash_bucket_index].remove(key);

			let (pushed_data, evicted_data) = self.buckets[target_bucket_index].put(data);
			if let Some(evicted) = evicted_data {
				debug!("DashSegment.get: key={key:?} migration_evicted={:?} target_bucket={target_bucket_index} -> stash_bucket={stash_bucket_index}", evicted.key);
				self.stash_buckets[stash_bucket_index].put(evicted);
			}
			return Some(pushed_data);
		}

		// 2. Check target bucket
		if let Some(position) = self.buckets[target_bucket_index].get_position(key) {
			debug!("DashSegment.get: key={key:?} found_in=target bucket={target_bucket_index}");
			return Some(self.buckets[target_bucket_index].get_from_position(position));
		}

		// 3. Check probing bucket (adjacent)
		let probing_bucket_index = target_bucket_index + 1;
		if probing_bucket_index >= self.buckets.len() {
			debug!("DashSegment.get: key={key:?} miss bucket={target_bucket_index} (last, no probing)");
			return None;
		}
		if let Some(position) = self.buckets[probing_bucket_index].get_position(key) {
			debug!("DashSegment.get: key={key:?} found_in=probing bucket={probing_bucket_index}");
			return Some(self.buckets[probing_bucket_index].get_from_position(position));
		}

		debug!("DashSegment.get: key={key:?} miss (stash={stash_bucket_index} target={target_bucket_index} probing={probing_bucket_index})");
		None
	}

	/// Insert the key, value pair into the segment.
	/// Assumes the key is not already in the segment — enforced by the Java caller
	/// (`DashRustPolicy.record()`) which does get-then-put: only calls `putToCache` on a miss.
	pub fn put(&mut self, item: Item<K, V>) {
		let idx = get_index(&item.key, self.stash_size);
		debug!("DashSegment.put: key={:?} stash_bucket={idx}", item.key);
		let stash_bucket = &mut self.stash_buckets[idx];
		stash_bucket.put(item);
	}
}

impl<K, V> Display for DashSegment<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		writeln!(f, "Segment {{")?;
		for bucket in &self.buckets {
			writeln!(f, "  Bucket {{")?;
			write!(f, "{}", bucket)?;
			writeln!(f, "  }}")?;
		}
		for bucket in &self.stash_buckets {
			writeln!(f, "  Stash Bucket {{")?;
			write!(f, "{}", bucket)?;
			writeln!(f, "  }}")?;
		}
		write!(f, "}}")
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::eviction_policy::EvictionPolicy;

	use crate::test_utils::{find_keys_for_bucket, small_dash_settings};

	/// Small segment for deterministic testing.
	fn small_segment(policy: EvictionPolicy) -> DashSegment<i64, i64> {
		DashSegment::new(&small_dash_settings(policy))
	}

	// ==================== Basic put/get ====================

	#[test]
	fn get_miss_returns_none() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		assert!(segment.get(&42).is_none());
	}

	#[test]
	fn put_then_get_returns_value() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		segment.put(Item::new(1, 10));
		assert_eq!(segment.get(&1).unwrap().value, 10);
	}

	#[test]
	fn put_multiple_then_get_each() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		for i in 0..4 {
			segment.put(Item::new(i, i * 10));
		}
		for i in 0..4 {
			assert_eq!(segment.get(&i).unwrap().value, i * 10);
		}
	}

	// ==================== Stash → target movement ====================

	#[test]
	fn put_goes_to_stash() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		segment.put(Item::new(1, 10));

		// Item should be in some stash bucket
		let stash_idx = get_index(&1_i64, segment.stash_size);
		assert_eq!(segment.stash_buckets[stash_idx].get_items().len(), 1);

		// Not in any normal bucket
		let total_normal: usize = segment.buckets.iter().map(|b| b.size()).sum();
		assert_eq!(total_normal, 0);
	}

	#[test]
	fn get_moves_item_from_stash_to_target() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		segment.put(Item::new(1, 10));

		let stash_idx = get_index(&1_i64, segment.stash_size);
		let target_idx = get_index(&1_i64, segment.segment_size);

		// Before get: in stash, not in target
		assert!(segment.stash_buckets[stash_idx].get_position(&1).is_some());
		assert!(segment.buckets[target_idx].get_position(&1).is_none());

		segment.get(&1);

		// After get: in target, not in stash
		assert!(segment.stash_buckets[stash_idx].get_position(&1).is_none());
		assert!(segment.buckets[target_idx].get_position(&1).is_some());
	}

	#[test]
	fn second_get_finds_item_in_target() {
		let mut segment = small_segment(EvictionPolicy::Fifo);
		segment.put(Item::new(1, 10));
		segment.get(&1); // moves to target

		// Second get should still find it
		assert_eq!(segment.get(&1).unwrap().value, 10);
	}

	// ==================== Eviction from target back to stash ====================

	#[test]
	fn target_overflow_evicts_back_to_stash() {
		// bucket_size=2: filling a target bucket beyond 2 triggers eviction back to stash.
		// Put-then-get one at a time to avoid stash overflow.
		let mut segment = small_segment(EvictionPolicy::Fifo);

		// Find 3 keys that map to the same target bucket (bucket_size=2, so 3rd overflows)
		let target_idx = 0;
		let same_bucket_keys = find_keys_for_bucket(target_idx, segment.segment_size, 3);

		// Put then get each, so stash holds at most 1 at a time
		segment.put(Item::new(same_bucket_keys[0], 10));
		segment.get(&same_bucket_keys[0]); // target: [k0]
		segment.put(Item::new(same_bucket_keys[1], 20));
		segment.get(&same_bucket_keys[1]); // target: [k0, k1] — full
		segment.put(Item::new(same_bucket_keys[2], 30));
		segment.get(&same_bucket_keys[2]); // moves k2 to target, evicts k0 (FIFO) back to stash

		assert!(segment.buckets[target_idx].get_position(&same_bucket_keys[2]).is_some());
		// k0 should be back in stash
		let stash_idx = get_index(&same_bucket_keys[0], segment.stash_size);
		assert!(segment.stash_buckets[stash_idx]
			.get_position(&same_bucket_keys[0])
			.is_some());
	}

	// ==================== Probing bucket ====================

	#[test]
	fn get_finds_item_in_probing_bucket() {
		let mut segment = small_segment(EvictionPolicy::Fifo);

		// Find a key whose target is NOT the last bucket (so it has a probing neighbor)
		// Any bucket except the last one works — use bucket 0
		let key = find_keys_for_bucket(0, segment.segment_size, 1)[0];
		let target_idx = 0;

		// Manually place the item in the probing bucket (target + 1)
		let probing_idx = target_idx + 1;
		segment.buckets[probing_idx].put(Item::new(key, 99));

		// get should find it via probing
		assert_eq!(segment.get(&key).unwrap().value, 99);
	}

	#[test]
	fn get_returns_none_when_target_is_last_bucket_and_not_found() {
		let mut segment = small_segment(EvictionPolicy::Fifo);

		// Find a key whose target is the last bucket
		let last_bucket_idx = segment.segment_size - 1;
		let key = find_keys_for_bucket(last_bucket_idx, segment.segment_size, 1)[0];

		// Item not in stash, not in target (last bucket), no probing available
		assert!(segment.get(&key).is_none());
	}
}
