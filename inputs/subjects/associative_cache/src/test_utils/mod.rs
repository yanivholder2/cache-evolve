use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::shared::utils::get_index;

/// Upper bound for brute-force key searches in tests.
pub const KEY_SEARCH_RANGE: i64 = 200;

/// Returns small AssociativeCacheSettings for testing.
pub fn small_settings(policy: EvictionPolicyKind) -> AssociativeCacheSettings {
	AssociativeCacheSettings {
		num_of_buckets: 4,
		bucket_size: 2,
		eviction_policy: policy,
	}
}

/// Baseline settings — shared by the invariant suite and the
/// `AssociativeCacheInvariants` framework entry point.
pub fn baseline_settings() -> AssociativeCacheSettings {
	AssociativeCacheSettings {
		num_of_buckets: 4,
		bucket_size: 3,
		eviction_policy: EvictionPolicyKind::ClassicLRU,
	}
}

#[cfg(feature = "test-utils")]
pub use invariants::{run_invariant_suite, AssociativeCacheInvariants};

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
