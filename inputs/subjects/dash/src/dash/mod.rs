mod dash_bucket;
pub mod dash_settings;
pub mod jni;
pub mod segments;

use log::{debug, info};
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::dash::segments::Segment;
use crate::shared::item::Item;
use crate::shared::utils::get_index;

/// Multi-segment associative cache. Each segment owns its own normal buckets and stash
/// and implements a probing strategy via `Segment`. `Dash` shards keys across segments
/// by hash.
///
/// # Uniqueness invariant
/// Each key resides in at most one slot (stash, target, or probing) across the
/// entire cache at any given moment.
///
/// **Why it holds under the JNI call contract:** the Caffeine caller only calls
/// `put` after a `get` miss (return value `== -1`). `get` returns on the first
/// matching slot it finds, so any resident key is always returned — never
/// re-inserted. Cross-slot duplicates are therefore structurally impossible
/// during normal operation.
///
/// **What breaks if violated:** `get` stops searching after the first match.
/// A second copy of the same key in target or probing becomes permanently
/// invisible — a phantom item that occupies a slot but can never be retrieved
/// or evicted through the normal path.
///
/// In debug builds `Segment::put` asserts this invariant before every insertion.
#[derive(Debug)]
pub struct Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
	S: Segment<K, V>,
{
	pub segments: Vec<S>,
	_phantom: PhantomData<(K, V)>,
}

impl<K, V, S> Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
	S: Segment<K, V>,
{
	/// Creates a new Dash cache with fallible segment initialization.
	/// Returns `None` if `num_segments == 0` or if `segment_factory` returns `None` for any segment.
	pub fn try_new(num_segments: usize, mut segment_factory: impl FnMut() -> Option<S>) -> Option<Self> {
		if num_segments == 0 {
			log::error!("Dash.try_new: num_segments must be > 0");
			return None;
		}
		let segments = (0..num_segments)
			.map(|_| segment_factory())
			.collect::<Option<Vec<_>>>();

		if segments.is_none() {
			log::error!("Dash.try_new: segment initialization failed");
			return None;
		}

		info!("Dash.try_new: segments={num_segments}");
		Some(Self {
			segments: segments.unwrap(),
			_phantom: PhantomData,
		})
	}

	/// Creates a new Dash cache.
	/// # Panics
	/// Panics if `num_segments == 0` or if initialization fails.
	pub fn new(num_segments: usize, mut segment_factory: impl FnMut() -> S) -> Self {
		Self::try_new(num_segments, || Some(segment_factory())).expect("Dash::new: initialization failed")
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

impl<K, V, S> framework::Cache<K, V> for Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug,
	V: Eq + Copy + Debug,
	S: Segment<K, V>,
{
	fn put(&mut self, key: K, value: V) {
		Dash::put(self, key, value);
	}

	fn get(&mut self, key: &K) -> Option<&V> {
		Dash::get_and_update_item(self, key)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::shared::eviction_policy::EvictionPolicyKind;
	use crate::test_utils::small_dash_settings;
	use segments::PromotionColdSegment;

	#[test]
	fn put_then_get() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			PromotionColdSegment::new(&settings)
		});
		cache.put(1_i64, 10_i64);
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}

	#[test]
	fn miss_returns_none() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			PromotionColdSegment::new(&settings)
		});
		assert!(cache.get_and_update_item(&42_i64).is_none());
	}

	#[test]
	fn try_new_rejects_zero_segments() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let res = Dash::<i64, i64, PromotionColdSegment<i64, i64>>::try_new(0, || {
			PromotionColdSegment::try_new(&settings)
		});
		assert!(res.is_none());
	}

	#[test]
	fn dash_implements_framework_cache_trait() {
		use framework::Cache;
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			PromotionColdSegment::new(&settings)
		});
		Cache::put(&mut cache, 1_i64, 10_i64);
		assert_eq!(Cache::get(&mut cache, &1_i64), Some(&10_i64));
		assert_eq!(Cache::get(&mut cache, &999_i64), None);
	}
}
