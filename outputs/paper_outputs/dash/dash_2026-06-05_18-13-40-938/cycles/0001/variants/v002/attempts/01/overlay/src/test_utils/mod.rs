use crate::dash::dash_settings::DashSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::utils::get_index;

/// Upper bound for brute-force key searches in tests.
pub const KEY_SEARCH_RANGE: i64 = 200;

/// Returns a small DashSettings for testing. Used by Dash + segment tests.
pub fn small_dash_settings(policy: EvictionPolicyKind) -> DashSettings {
	DashSettings {
		num_of_segments: 2,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 2,
		eviction_policy: policy,
		debug_mode: 0,
	}
}

/// Single-segment baseline settings — shared by the invariant suite and
/// the `DashInvariants` framework entry point.
pub fn baseline_dash_settings() -> DashSettings {
	DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 3,
		eviction_policy: EvictionPolicyKind::Lfu,
		debug_mode: 0,
	}
}

#[cfg(feature = "test-utils")]
pub use invariants::{run_invariant_suite, DashInvariants};

#[cfg(feature = "test-utils")]
mod invariants;

/// Find `count` keys that hash to `bucket_idx` given `num_buckets` total buckets.
pub fn find_keys_for_bucket(bucket_idx: usize, num_buckets: usize, count: usize) -> Vec<i64> {
	let mut keys = vec![];
	for k in 0..KEY_SEARCH_RANGE {
		if get_index(&k, num_buckets) == bucket_idx {
			keys.push(k);
			if keys.len() == count {
				break;
			}
		}
	}
	keys
}
