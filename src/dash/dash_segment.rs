use super::dash_bucket::DashBucket;
use super::dash_settings::DashSettings;
use crate::shared::item::Item;
use crate::shared::settings::EvictionPolicy;
use crate::shared::traits::bucket::Bucket;
use crate::shared::utils::{get_index, hash};
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug)]
pub struct DashSegment<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	pub buckets: Vec<DashBucket<K, V>>,
	pub segment_size: usize,
	pub stash_buckets: Vec<DashBucket<K, V>>,
	pub stash_size: usize,
}

// TODO: use the Segment trait
impl<K, V> DashSegment<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	pub fn new(settings: DashSettings) -> Self {
		let mut buckets: Vec<DashBucket<K, V>> = Vec::new();
		for _ in 0..settings.num_of_normal_buckets {
			// TODO: pass the settings as a reference
			buckets.push(DashBucket::new(settings.bucket_size, settings.eviction_policy.clone()));
		}

		let mut stash_buckets: Vec<DashBucket<K, V>> = Vec::new();
		for _ in 0..settings.num_of_stash_buckets {
			stash_buckets.push(DashBucket::new(settings.bucket_size, EvictionPolicy::Fifo));
		}
		DashSegment {
			buckets,
			stash_buckets,
			segment_size: settings.num_of_normal_buckets,
			stash_size: settings.num_of_stash_buckets,
		}
	}

	// TODO: could be written better
	/// Returns a reference to the item with `key`.
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let hash = hash(&key);
		let stash_bucket_index = get_index(hash, self.stash_size);
		let stash_bucket = &self.stash_buckets[stash_bucket_index];
		let target_bucket_index = get_index(hash, self.segment_size);

		// The order assumes that the data is more likely to be in the stash bucket,
		// this assumption should be tested
		if let Some(position) = stash_bucket.get_position(key) {
			// If the key is in the stash bucket, we need to move it to the target bucket
			let mut_stash_bucket = &mut self.stash_buckets[stash_bucket_index];
			let data = mut_stash_bucket.get_from_position(position)?.clone();
			mut_stash_bucket.remove(key);

			let mut_target_bucket = &mut self.buckets[target_bucket_index];
			let (pushed_data, evicted_data) = mut_target_bucket.put(data);
			if let Some(data) = evicted_data {
				mut_stash_bucket.put(data);
			}
			Some(pushed_data)
		} else {
			// If the key is not in the stash bucket, we need to check the target bucket
			let target_bucket_index = get_index(hash, self.segment_size);
			let target_bucket = &self.buckets[target_bucket_index];

			if let Some(position) = target_bucket.get_position(key) {
				// If the key is in the target bucket, we need to update the position
				let mut_target_bucket = &mut self.buckets[target_bucket_index];
				Some(mut_target_bucket.get_from_position(position)?)
			} else {
				// If the key is not in the target bucket, we need to check the probing bucket

				if target_bucket_index == self.buckets.len() - 1 {
					// If the target bucket is the last bucket, there is not probing bucket
					return None;
				}
				let probing_bucket_index = target_bucket_index + 1;
				let probing_bucket = &self.buckets[probing_bucket_index];
				if let Some(position) = probing_bucket.get_position(key) {
					// If the key is in the probing bucket, we need to update the position

					let mut_probing_bucket = &mut self.buckets[probing_bucket_index];
					Some(mut_probing_bucket.get_from_position(position)?)
				} else {
					None
				}
			}
		}
	}

	/// Insert the key, value pair into the segment.
	/// This function assumes that the key is not already in the segment.
	pub fn put(&mut self, item: Item<K, V>) {
		let hash = hash(&item.key);
		let stash_bucket = &mut self.stash_buckets[get_index(hash, self.stash_size)];
		stash_bucket.put(item);
	}
}

impl<K, V> Display for DashSegment<K, V>
where
	K: Hash + Eq + Copy + Display,
	V: Eq + Copy + Display,
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

// TODO: implement
/*
#[cfg(test)]
mod tests {
}
*/
