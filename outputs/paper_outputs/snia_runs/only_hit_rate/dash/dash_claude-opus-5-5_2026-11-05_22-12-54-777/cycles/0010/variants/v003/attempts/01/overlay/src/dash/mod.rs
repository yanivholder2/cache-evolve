mod dash_bucket;
pub mod dash_settings;
pub mod jni;
pub mod segments;

use log::{debug, info};
use std::any::Any;
use std::fmt::Debug;
use std::hash::Hash;
use std::marker::PhantomData;

use crate::dash::segments::Segment;
use crate::shared::item::Item;
use crate::shared::utils::segment_index;

/// Variant cycle/0010/v003 (H03): a miss-insert is flagged as a scan member
/// once `run_len` (the number of immediately preceding miss-inserts that each
/// extended the `key == last_key + 1` run) reaches this value, i.e. from the
/// 4th consecutive key onward.
pub const SCAN_RUN_MIN: u8 = 3;

/// Reads an integer key as `u64` for sequential-run detection. Keys of any
/// other type are never treated as sequential.
#[inline]
fn key_as_u64<K: 'static>(key: &K) -> Option<u64> {
	let any = key as &dyn Any;
	if let Some(k) = any.downcast_ref::<i64>() {
		Some(*k as u64)
	} else if let Some(k) = any.downcast_ref::<u64>() {
		Some(*k)
	} else if let Some(k) = any.downcast_ref::<i32>() {
		Some(*k as i64 as u64)
	} else if let Some(k) = any.downcast_ref::<u32>() {
		Some(*k as u64)
	} else if let Some(k) = any.downcast_ref::<usize>() {
		Some(*k as u64)
	} else if let Some(k) = any.downcast_ref::<isize>() {
		Some(*k as i64 as u64)
	} else {
		None
	}
}

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
///
/// Variant cycle/0010/v003 (H03): the table keeps a tiny sequential-run
/// detector over miss-inserts (`put` is only called for non-resident keys).
/// A put whose key is `last_key + 1` (wrapping) extends the run; any other key
/// resets it. Once the run length reaches `SCAN_RUN_MIN`, the insert is passed
/// to the segment as a scan member via `Segment::put_scan`.
#[derive(Debug)]
pub struct Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
	S: Segment<K, V>,
{
	pub segments: Vec<S>,
	/// Key of the previous miss-insert (valid only when `has_last`).
	last_key: u64,
	/// Whether `last_key` holds a real key yet.
	has_last: bool,
	/// Length of the current `+1` run, saturating at 255.
	run_len: u8,
	/// Number of inserts flagged as scan members (diagnostic).
	scan_flagged: u64,
	/// Total miss-inserts seen (diagnostic).
	total_puts: u64,
	_phantom: PhantomData<(K, V)>,
}

impl<K, V, S> Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
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

		info!("Dash.try_new: segments={num_segments} scan_run_min={SCAN_RUN_MIN}");
		Some(Self {
			segments: segments.unwrap(),
			last_key: 0,
			has_last: false,
			run_len: 0,
			scan_flagged: 0,
			total_puts: 0,
			_phantom: PhantomData,
		})
	}

	/// Creates a new Dash cache.
	/// # Panics
	/// Panics if `num_segments == 0` or if initialization fails.
	pub fn new(num_segments: usize, mut segment_factory: impl FnMut() -> S) -> Self {
		Self::try_new(num_segments, || Some(segment_factory())).expect("Dash::new: initialization failed")
	}

	/// Updates the run detector for a miss-insert of `key` and returns whether
	/// the insert is a scan member.
	#[inline]
	fn observe_insert(&mut self, key: &K) -> bool {
		let Some(k) = key_as_u64(key) else {
			return false;
		};
		if self.has_last && k == self.last_key.wrapping_add(1) {
			self.run_len = self.run_len.saturating_add(1);
		} else {
			self.run_len = 0;
		}
		self.last_key = k;
		self.has_last = true;
		self.run_len >= SCAN_RUN_MIN
	}

	/// Number of inserts flagged as scan members so far.
	pub fn scan_flagged(&self) -> u64 {
		self.scan_flagged
	}

	pub fn put(&mut self, key: K, value: V) {
		let scan = self.observe_insert(&key);
		self.total_puts += 1;
		if scan {
			self.scan_flagged += 1;
		}
		let idx = segment_index(&key, self.segments.len());
		debug!("Dash.put: key={key:?} segment={idx} scan={scan}");
		self.segments[idx].put_scan(Item::new(key, value), scan);
	}

	pub fn get_and_update_item(&mut self, key: &K) -> Option<&V> {
		let idx = segment_index(key, self.segments.len());
		debug!("Dash.get_and_update_item: key={key:?} segment={idx}");
		let item = self.segments[idx].get(key)?;
		Some(&item.value)
	}
}

impl<K, V, S> Drop for Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
	S: Segment<K, V>,
{
	fn drop(&mut self) {
		info!(
			"Dash.drop: scan_flagged_inserts={} total_inserts={}",
			self.scan_flagged, self.total_puts
		);
	}
}

impl<K, V, S> framework::Cache<K, V> for Dash<K, V, S>
where
	K: Hash + Eq + Copy + Debug + 'static,
	V: Eq + Copy + Debug + 'static,
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
	use segments::DashSegment;

	#[test]
	fn put_then_get() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		cache.put(1_i64, 10_i64);
		assert_eq!(*cache.get_and_update_item(&1).unwrap(), 10);
	}

	#[test]
	fn miss_returns_none() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		assert!(cache.get_and_update_item(&42_i64).is_none());
	}

	#[test]
	fn try_new_rejects_zero_segments() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let res = Dash::<i64, i64, DashSegment<i64, i64>>::try_new(0, || {
			DashSegment::try_new(&settings)
		});
		assert!(res.is_none());
	}

	#[test]
	fn dash_implements_framework_cache_trait() {
		use framework::Cache;
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		Cache::put(&mut cache, 1_i64, 10_i64);
		assert_eq!(Cache::get(&mut cache, &1_i64), Some(&10_i64));
		assert_eq!(Cache::get(&mut cache, &999_i64), None);
	}

	#[test]
	fn run_detector_flags_from_fourth_consecutive_key() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		let flags: Vec<bool> = [10_i64, 11, 12, 13, 14, 50, 51, 52, 53].iter().map(|k| cache.observe_insert(k)).collect();
		assert_eq!(flags, vec![false, false, false, true, true, false, false, false, true]);
	}

	#[test]
	fn run_detector_wraps_and_saturates() {
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		for k in 0..400_i64 {
			cache.observe_insert(&k);
		}
		assert_eq!(cache.run_len, u8::MAX);
		let mut c2: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || DashSegment::new(&settings));
		for k in [i64::MAX - 1, i64::MAX, i64::MIN, i64::MIN + 1] {
			c2.observe_insert(&k);
		}
		assert_eq!(c2.run_len, 3, "i64::MAX -> i64::MIN is +1 wrapping");
	}

	#[test]
	fn sequential_scan_round_trips() {
		use framework::Cache;
		let settings = small_dash_settings(EvictionPolicyKind::ClassicLRU);
		let mut cache: Dash<i64, i64, _> = Dash::new(settings.num_of_segments, || {
			DashSegment::new(&settings)
		});
		for k in 0..500_i64 {
			if Cache::get(&mut cache, &k).is_none() {
				Cache::put(&mut cache, k, k * 3);
			}
			if let Some(v) = Cache::get(&mut cache, &k) {
				assert_eq!(*v, k * 3);
			}
		}
		assert!(cache.scan_flagged() > 0);
		for seg in &cache.segments {
			assert!(seg.core().check_slot_legality());
			assert!(seg.core().check_capacity_bound());
		}
	}
}
