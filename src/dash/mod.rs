use crate::shared::item::Item;
use crate::shared::utils::get_index;
use log::{debug, info};
use segments::Segment;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

mod dash_bucket;
pub mod dash_settings;
pub mod segments;
pub mod jni;

/// Multi-segment associative cache. Each segment owns its own normal buckets and stash
/// and implements a probing strategy via `Segment`. `Dash` shards keys across segments
/// by hash.
#[derive(Debug)]
pub struct Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
	S: Segment<K, V>,
{
	segments: Vec<S>,
	_phantom: PhantomData<(K, V)>,
}

impl<K, V, S> Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
	S: Segment<K, V>,
{
	pub fn new(num_segments: usize, segment_factory: impl Fn() -> S) -> Self {
		assert!(num_segments > 0, "num_segments must be > 0");
		info!("Dash.new: segments={num_segments}");
		let segments: Vec<_> = (0..num_segments).map(|_| segment_factory()).collect();
		Self { segments, _phantom: PhantomData }
	}

	pub fn put(&mut self, key: K, value: V) {
		let idx = get_index(&key, self.segments.len());
		debug!("Dash.put: key={key:?} segment={idx}");
		self.segments[idx].put(Item::new(key, value));
	}

	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let idx = get_index(key, self.segments.len());
		debug!("Dash.get_and_update_item: key={key:?} segment={idx}");
		let item = self.segments[idx].get(key)?;
		Some(&item.value)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicy;
	use crate::test_utils::small_dash_settings;
	use segments::PromotionColdSegment;

	#[test]
	fn put_then_get() {
		let settings = small_dash_settings(EvictionPolicy::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			PromotionColdSegment::new(&settings)
		});
		cache.put(1_i64, 10_i64);
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}

	#[test]
	fn miss_returns_none() {
		let settings = small_dash_settings(EvictionPolicy::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			PromotionColdSegment::new(&settings)
		});
		assert!(cache.get_and_update_item(&42_i64).is_none());
	}
}
