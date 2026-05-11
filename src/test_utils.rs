use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::EvictionPolicy;
use crate::shared::utils::get_index;

/// Upper bound for brute-force key searches in tests.
pub const KEY_SEARCH_RANGE: i64 = 200;

/// Returns a small DashSettings for testing. Used by Dash + segment tests.
pub fn small_dash_settings(policy: EvictionPolicy) -> DashSettings {
	DashSettings {
		num_of_segments: 2,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 2,
		eviction_policy: policy,
		debug_mode: 0,
	}
}

/// Find `count` keys that hash to `bucket_idx` given `num_buckets` total buckets.
/// Panics with a clear message when the search range is exhausted before
/// `count` matching keys are found, so callers don't index into a short `Vec`
/// and panic without context downstream.
pub fn find_keys_for_bucket(bucket_idx: usize, num_buckets: usize, count: usize) -> Vec<i64> {
	let mut keys = vec![];
	for k in 0..KEY_SEARCH_RANGE {
		if get_index(&k, num_buckets) == bucket_idx {
			keys.push(k);
			if keys.len() == count {
				return keys;
			}
		}
	}
	panic!(
		"find_keys_for_bucket: exhausted KEY_SEARCH_RANGE={KEY_SEARCH_RANGE} looking for {count} keys hashing to bucket {bucket_idx}/{num_buckets} (found only {})",
		keys.len()
	);
}
