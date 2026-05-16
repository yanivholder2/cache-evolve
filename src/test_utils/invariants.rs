use crate::dash::segments::Segment;
use crate::dash::Dash;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{TestError, TestRunner};

/// Run all invariant properties against any [`Segment`] implementation.
///
/// This is the G3 gate artifact. A Variant's thin harness declares:
///
/// ```toml
/// [dev-dependencies]
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
/// in `tests/invariant_suite.rs`. Returns `Ok(())` if all invariants hold,
/// `Err(message)` naming the first violation with proptest's shrunk counterexample
/// — suitable for populating `rejection-log.md` on G3 failure.
pub fn run_invariant_suite<S>(factory: impl Fn() -> S) -> Result<(), String>
where
    S: Segment<i64, i64>,
{
    // A fresh `TestRunner` per property: a reused runner accumulates `successes`
    // across calls, so once P1 reached its case budget, subsequent properties would
    // exit their while-loop immediately and trivially return Ok — silently skipping
    // P2–P5.
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

    Ok(())
}

fn format_err(invariant: &str, e: TestError<Vec<i64>>) -> String {
    format!("{invariant}: {e}")
}
