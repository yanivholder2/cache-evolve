use super::segment::Segment;
use crate::shared::{
	item::Item,
	utils::{get_index, hash},
};
use std::hash::Hash;

pub trait Cache<K, V>
where
	K: Hash + Eq + Copy,
	V: Eq + Copy,
{
	type S: Segment<K, V>;
	// ------------ struct expected fields ----------------------------------------------

	fn get_segments_mut(&mut self) -> &mut Vec<Self::S>;

	// ----------------------------------------------------------------------------------

	/// Insert a key-value pair into the cache
	fn put(&mut self, key: K, value: V) {
		let segment = self.get_mut_segment(&key);
		segment.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	fn get(&mut self, key: &K) -> Option<V> {
		let segment = self.get_mut_segment(key);
		let data = segment.get(key)?;
		Some(data.value.clone())
	}

	fn get_mut_segment(&mut self, key: &K) -> &mut Self::S {
		let hash = hash(key);
		let segment_index = get_index(hash, self.get_segments_mut().len());
		&mut self.get_segments_mut()[segment_index]
	}
}
