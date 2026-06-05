use crate::lru_cache::LruCache;
use crate::test_utils::BASELINE_CAPACITY;
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{TestError, TestRunner};

/// Invariant entry point for the LRU baseline. Delegates to
/// [`run_invariant_suite`] at the baseline capacity.
pub struct LruInvariants;

impl framework::Invariants for LruInvariants {
	fn run_invariant_suite() -> Result<(), String> {
		run_invariant_suite(BASELINE_CAPACITY)
	}
}

/// Run all invariant properties against an `LruCache` of the given `capacity`.
///
/// Uses proptest internally for randomized coverage. Returns `Ok(())` if all
/// invariants hold, `Err(message)` naming the first violation with proptest's
/// shrunk counterexample.
pub fn run_invariant_suite(capacity: usize) -> Result<(), String> {
	// Each property uses a fresh `TestRunner`: a reused runner accumulates
	// `successes` across calls, so later properties would exit early.

	// P1: capacity_bound — the cache never retains more than `capacity` distinct
	// keys after any workload (only-put-on-miss, mirroring the JNI call contract).
	TestRunner::default()
		.run(&vec(0i64..200, 1..200), |ops| {
			let mut cache: LruCache<i64, i64> = LruCache::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
			}
			let resident = (0i64..200).filter(|k| cache.get(k).is_some()).count();
			prop_assert!(
				resident <= capacity,
				"capacity_bound violated: {resident} residents but capacity={capacity}"
			);
			Ok(())
		})
		.map_err(|e| format_err("P1 capacity_bound", e))?;

	// P2: value_fidelity — every resident key reads back the value it was put with.
	TestRunner::default()
		.run(&vec(0i64..200, 1..200), |ops| {
			let mut cache: LruCache<i64, i64> = LruCache::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key * 10);
				}
			}
			for key in 0i64..200 {
				if let Some(&value) = cache.get(&key) {
					prop_assert!(
						value == key * 10,
						"value_fidelity violated: key={key} read {value}, expected {}",
						key * 10
					);
				}
			}
			Ok(())
		})
		.map_err(|e| format_err("P2 value_fidelity", e))?;

	// P3: no_phantom_items — a never-put key always reads None.
	TestRunner::default()
		.run(&vec(0i64..100, 1..200), |ops| {
			let mut cache: LruCache<i64, i64> = LruCache::new(capacity);
			for &key in &ops {
				if cache.get(&key).is_none() {
					cache.put(key, key);
				}
			}
			for phantom in [1000i64, 2000, 3000, -1, -50] {
				prop_assert!(
					cache.get(&phantom).is_none(),
					"no_phantom_items violated: never-put key={phantom} returned a value"
				);
			}
			Ok(())
		})
		.map_err(|e| format_err("P3 no_phantom_items", e))?;

	Ok(())
}

fn format_err(invariant: &str, e: TestError<Vec<i64>>) -> String {
	format!("invariant gate violated: {invariant}: {e}")
}
