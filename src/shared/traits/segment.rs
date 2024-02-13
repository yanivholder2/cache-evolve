use crate::shared::item::Item;
use crate::shared::utils::{get_index, hash};
use std::hash::{self, Hash};

use super::bucket::Bucket;

pub trait Segment<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	// ------------ struct expected fields ----------------------------------------------

	// TODO: Check if using Box and dyn is the best approach

	/// Returns the buckets vector.
	fn get_buckets(&self) -> &Vec<Box<dyn Bucket<K, V>>>;

	/// Returns the buckets vector as mutable.
	fn get_buckets_mut(&mut self) -> &mut Vec<Box<dyn Bucket<K, V>>>;

	/// Returns each segment size.
	fn get_segment_size(&self) -> usize;

	// ----------------------------------------------------------------------------------

	/// Insert the key, value pair into the segment.
	/// This function assumes that the key is not already in the segment.
	fn put(&mut self, item: Item<K, V>) {
		let bucket = self.get_mut_bucket(&item.key);
		bucket.put(item);
	}

	/// Returns the value of `key` if exists, and None otherwise.
	///
	/// As a side effect makes updates according to the eviction policy.
	fn get(&mut self, key: &K) -> Option<&Item<K, V>> {
		let bucket = self.get_mut_bucket(key);
		bucket.get(key)
	}

	/// Returns a mutable reference to the bucket with the given key.
	fn get_mut_bucket(&mut self, key: &K) -> &mut Box<dyn Bucket<K, V>> {
		let hash = hash(key);
		let index = get_index(hash, self.get_segment_size());
		&mut self.get_buckets_mut()[index]
	}

	/// Returns the number of buckets in the segment.
	fn size(&self) -> usize {
		self.get_buckets().len()
	}
}

// TODO: Use blanket implementation for Display trait
