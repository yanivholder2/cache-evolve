// Copied into $WORK_DIR/tests/g3_invariant.rs at build time — not compiled from here.
// Kept as a real .rs file so rustfmt and IDEs can target it.
//
// Runs the Invariant suite against the Variant's `VariantSegment` via
// `dash::test_utils::run_invariant_suite`. A failure panics with a message
// prefixed by the invariant name (e.g. "P3 slot_legality"), which the G3 gate
// parses into the variant's rejection.md.
use dash::dash::dash_settings::DashSettings;
use dash::shared::eviction_policy::EvictionPolicy;
use dash::test_utils::run_invariant_suite;
use variant_check::VariantSegment;

fn default_settings() -> DashSettings {
    DashSettings {
        num_of_segments: 1,
        num_of_normal_buckets: 4,
        num_of_stash_buckets: 2,
        bucket_size: 3,
        eviction_policy: EvictionPolicy::ClassicLRU,
        debug_mode: 0,
    }
}

#[test]
fn g3_invariant_suite() {
    let settings = default_settings();
    if let Err(msg) = run_invariant_suite(|| VariantSegment::<i64, i64>::new(&settings)) {
        panic!("G3 invariant violated: {msg}");
    }
}
