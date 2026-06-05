use crate::dash::dash_settings::DashSettings;
use crate::dash::segments::{DashSegment, Segment};
use crate::dash::Dash;
use crate::shared::eviction_policy::EvictionPolicyKind;
use crate::test_utils::baseline_dash_settings;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{TestError, TestRunner};

/// Generates structurally valid `DashSettings` across the topology space, so the
/// capacity-conservation property is exercised at many budgets — not just the
/// baseline. Probing needs `num_of_normal_buckets >= 2` and the stash needs
/// `num_of_stash_buckets >= 1`; `bucket_size` ranges past the swept `bucketSize`
/// the eval path uses, so a budget-dependent variant geometry is covered.
fn valid_dash_settings() -> impl Strategy<Value = DashSettings> {
	(1usize..6, 2usize..8, 1usize..8, 1usize..128).prop_map(
		|(num_of_segments, num_of_normal_buckets, num_of_stash_buckets, bucket_size)| DashSettings {
			num_of_segments,
			num_of_normal_buckets,
			num_of_stash_buckets,
			bucket_size,
			eviction_policy: EvictionPolicyKind::ClassicLRU,
			debug_mode: 0,
		},
	)
}

/// Invariant entry point for the Dash baseline. Delegates to
/// [`run_invariant_suite`], which remains public for negative fixtures and
/// Variant harnesses.
pub struct DashInvariants;

impl framework::Invariants for DashInvariants {
	fn run_invariant_suite() -> Result<(), String> {
		let settings = baseline_dash_settings();
		run_invariant_suite(|| DashSegment::new(&settings))
	}
}

/// Run all invariant properties against any [`Segment`] implementation.
///
/// This is the invariant-gate artifact. A Variant's thin harness declares:
///
/// ```toml
/// [dependencies]
/// dash = { path = "../../", features = ["test-utils"] }
/// ```
///
/// then calls:
///
/// ```rust,ignore
/// use dash::test_utils::run_invariant_suite;
/// run_invariant_suite(|| MySegment::new(&settings)).unwrap();
/// ```
///
/// Uses proptest internally — same randomized coverage as the property tests
/// in `tests/invariant_gate.rs`. Returns `Ok(())` if all invariants hold,
/// `Err(message)` naming the first violation with proptest's shrunk counterexample
/// — suitable for populating `rejection.md` on invariant-gate failure.
pub fn run_invariant_suite<S>(factory: impl Fn() -> S) -> Result<(), String>
where
    S: Segment<i64, i64>,
{
    // Each property uses a fresh `TestRunner`: a reused runner accumulates
    // `successes` across calls, so once P1 hit its case budget, P2–P5 would
    // exit their while-loop immediately and silently return Ok.

    // P1: no_phantom_items
    TestRunner::default()
        .run(&vec(0i64..200, 1..60), |ops| {
            let mut cache: Dash<i64, i64, S> = Dash::new(1, || factory());
            for &key in &ops {
                if cache.get_and_update_item(&key).is_none() {
                    cache.put(key, key);
                }
            }
            for key in 0i64..200 {
                if cache.segments[0].core().contains_key(&key) {
                    prop_assert!(
                        cache.get_and_update_item(&key).is_some(),
                        "phantom: contains_key true but get returned None for key={key}"
                    );
                }
            }
            Ok(())
        })
        .map_err(|e| format_err("P1 no_phantom_items", e))?;

    // P2: capacity_bound
    TestRunner::default()
        .run(&vec(0i64..200, 1..50), |ops| {
            let mut cache: Dash<i64, i64, S> = Dash::new(1, || factory());
            for key in ops {
                if cache.get_and_update_item(&key).is_none() {
                    cache.put(key, key);
                    let core = cache.segments[0].core();
                    prop_assert!(
                        core.check_capacity_bound(),
                        "capacity violated: {} items but max={}",
                        core.total_items(),
                        core.max_capacity()
                    );
                }
            }
            Ok(())
        })
        .map_err(|e| format_err("P2 capacity_bound", e))?;

    // P3: slot_legality
    TestRunner::default()
        .run(&vec(0i64..200, 1..80), |ops| {
            let mut cache: Dash<i64, i64, S> = Dash::new(1, || factory());
            for &key in &ops {
                if cache.get_and_update_item(&key).is_none() {
                    cache.put(key, key);
                }
            }
            for &key in &ops {
                cache.get_and_update_item(&key);
            }
            prop_assert!(
                cache.segments[0].core().check_slot_legality(),
                "slot legality violated after workload"
            );
            Ok(())
        })
        .map_err(|e| format_err("P3 slot_legality", e))?;

    // P4: single_eviction_per_put
    TestRunner::default()
        .run(&vec(0i64..200, 1..60), |ops| {
            let mut cache: Dash<i64, i64, S> = Dash::new(1, || factory());
            for key in ops {
                if cache.get_and_update_item(&key).is_none() {
                    let before = cache.segments[0].core().total_items();
                    cache.put(key, key);
                    let after = cache.segments[0].core().total_items();
                    prop_assert!(
                        after == before || after == before + 1,
                        "single_eviction_per_put violated: before={before} after={after} key={key}"
                    );
                }
            }
            Ok(())
        })
        .map_err(|e| format_err("P4 single_eviction_per_put", e))?;

    // P5: stash_terminal
    TestRunner::default()
        .run(&vec(0i64..200, 20..80), |ops| {
            let mut cache: Dash<i64, i64, S> = Dash::new(1, || factory());
            for &key in &ops {
                if cache.get_and_update_item(&key).is_none() {
                    cache.put(key, key);
                }
            }
            for key in 0i64..200 {
                if !cache.segments[0].core().contains_key(&key) {
                    prop_assert!(
                        cache.get_and_update_item(&key).is_none(),
                        "stash_terminal violated: key={key} absent but get returned Some"
                    );
                    prop_assert!(
                        cache.get_and_update_item(&key).is_none(),
                        "stash_terminal violated: key={key} reappeared on second get"
                    );
                }
            }
            Ok(())
        })
        .map_err(|e| format_err("P5 stash_terminal", e))?;

    // P6: capacity_conservation — the geometry seam may reshape the topology but
    // must preserve the total capacity budget and stay structurally valid. Guards
    // fitness fairness: a variant cannot win by quietly enlarging the cache, and a
    // geometry that yields an unbuildable topology is rejected here cleanly instead
    // of panicking deep in segment construction. Drives `factory::resolve_topology`
    // — the exact resolution the eval path runs — across a range of valid settings,
    // so a variant whose geometry depends on the budget is checked at many operating
    // points, not just the baseline.
    TestRunner::default()
        .run(&valid_dash_settings(), |settings| {
            let result = crate::factory::resolve_topology(&settings);
            prop_assert!(
                result.is_ok(),
                "P6 capacity_conservation violated: {}",
                result.unwrap_err()
            );
            Ok(())
        })
        .map_err(|e| format!("P6 capacity_conservation: {e}"))?;

    Ok(())
}

fn format_err(invariant: &str, e: TestError<Vec<i64>>) -> String {
    format!("{invariant}: {e}")
}
