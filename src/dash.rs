use std::fmt::{Debug, Display, Formatter};
use std::hash::Hash;

use crate::shared::item::Item;
use crate::shared::utils::get_index;
use dash_segment::DashSegment;
use dash_settings::DashSettings;
use log::{debug, info};

mod dash_bucket;
pub mod dash_jni;
mod dash_segment;
pub mod dash_settings;

#[derive(Debug)]
pub struct Dash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	pub segments: Vec<DashSegment<K, V>>,
}

impl<K, V> Dash<K, V>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
{
	/// Creates a new Dash instance with the given settings.
	pub fn new(settings: &DashSettings) -> Self {
		info!("Dash.new: segments={} normal_buckets={} stash_buckets={} bucket_size={} policy={:?}", settings.num_of_segments, settings.num_of_normal_buckets, settings.num_of_stash_buckets, settings.bucket_size, settings.eviction_policy);
		let mut segments = Vec::with_capacity(settings.num_of_segments);
		for _ in 0..settings.num_of_segments {
			segments.push(DashSegment::new(&settings));
		}
		Self { segments }
	}

	/// Insert a key-value pair into Dash
	pub fn put(&mut self, key: K, value: V) {
		debug!("Dash.put: key={key:?} segment={}", get_index(&key, self.segments.len()));
		let segment = self.get_mut_segment(&key);
		segment.put(Item::new(key, value));
	}

	/// Returns the value of key if exists wrapper in Some ans None otherwise
	///
	/// As a side effect makes updates according to the eviction policy.
	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		debug!("Dash.get_and_update_item: key={key:?} segment={}", get_index(key, self.segments.len()));
		let segment = self.get_mut_segment(key);
		let data = segment.get(key)?;
		Some(&data.value)
	}

	fn get_mut_segment(&mut self, key: &K) -> &mut DashSegment<K, V> {
		let segment_index = get_index(key, self.segments.len());
		&mut self.segments[segment_index]
	}
}

impl<K, V> Display for Dash<K, V>
where
	K: Hash + Eq + Copy + Debug + Display,
	V: Eq + Copy + Debug + Display,
{
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		for segment in &self.segments {
			writeln!(f, "{}", segment)?;
		}
		Ok(())
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::eviction_policy::EvictionPolicy;
	use crate::test_utils::small_dash_settings;

	fn small_dash(policy: EvictionPolicy) -> Dash<i64, i64> {
		Dash::new(&small_dash_settings(policy))
	}

	#[test]
	fn get_miss_returns_none() {
		let mut dash = small_dash(EvictionPolicy::Fifo);
		assert!(dash.get_and_update_item(&42).is_none());
	}

	#[test]
	fn put_then_get() {
		let mut dash = small_dash(EvictionPolicy::Fifo);
		dash.put(1, 10);
		assert_eq!(*dash.get_and_update_item(&1).unwrap(), 10);
	}

	#[test]
	fn multiple_keys() {
		let mut dash = small_dash(EvictionPolicy::Fifo);
		for i in 0..4 {
			dash.put(i, i * 10);
		}
		for i in 0..4 {
			assert_eq!(*dash.get_and_update_item(&i).unwrap(), i * 10);
		}
	}

	#[test]
	fn put_overwrite() {
		let mut dash = small_dash(EvictionPolicy::ClassicLRU);
		dash.put(1, 10);
		dash.put(1, 20);
		assert_eq!(*dash.get_and_update_item(&1).unwrap(), 20);
	}
}
