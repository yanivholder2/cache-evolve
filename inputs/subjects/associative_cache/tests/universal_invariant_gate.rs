//! Universal invariant gate: runs the cross-subject
//! [`framework::test_suite::run_invariant_suite`] against the subject's
//! `make_cache` factory. Variants overlay `src/factory.rs`, so this same test
//! exercises their cache through the framework `Cache` trait.

use associative_cache::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use associative_cache::factory::make_cache;
use associative_cache::shared::eviction_policy::EvictionPolicyKind;
use framework::test_suite::run_invariant_suite;

fn default_settings() -> AssociativeCacheSettings {
	AssociativeCacheSettings {
		num_of_buckets: 4,
		bucket_size: 2,
		eviction_policy: EvictionPolicyKind::ClassicLRU,
	}
}

#[test]
fn universal_invariant_suite() {
	run_invariant_suite(|| make_cache(&default_settings())).unwrap();
}
