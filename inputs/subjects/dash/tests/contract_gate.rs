//! Contract gate: runs the cross-subject [`framework::test_suite::run_contract_suite`]
//! against the subject's `make_cache` factory. Variants overlay `src/factory.rs`,
//! so this same test exercises their cache through the framework `Cache` trait.

use dash::dash::dash_settings::DashSettings;
use dash::factory::make_cache;
use dash::shared::eviction_policy::EvictionPolicyKind;
use framework::test_suite::run_contract_suite;

fn default_settings() -> DashSettings {
	DashSettings {
		num_of_segments: 1,
		num_of_normal_buckets: 4,
		num_of_stash_buckets: 2,
		bucket_size: 2,
		eviction_policy: EvictionPolicyKind::ClassicLRU,
		debug_mode: 0,
	}
}

#[test]
fn contract_suite() {
	run_contract_suite(|| make_cache(&default_settings()));
}
