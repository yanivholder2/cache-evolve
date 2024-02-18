use crate::shared::item::Item;
use crate::shared::utils::{get_index, hash};
use std::hash::Hash;

use super::bucket::Bucket;

pub trait Segment<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	type B: Bucket<K, V>;
	// ------------ struct expected fields ----------------------------------------------

	/// Returns the buckets vector.
	fn get_buckets(&self) -> &Vec<Self::B>;

	/// Returns the buckets vector as mutable.
	fn get_buckets_mut(&mut self) -> &mut Vec<Self::B>;

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
	fn get_mut_bucket(&mut self, key: &K) -> &mut Self::B {
		let hash = hash(key);
		let index = get_index(hash, self.get_segment_size());
		&mut self.get_buckets_mut()[index]
	}
}

// TODO: Use blanket implementation for Display trait
