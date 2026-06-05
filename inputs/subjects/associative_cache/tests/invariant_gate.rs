// Invariant gate — invariant-gate artifact
//
// The invariant properties live in `run_invariant_suite`
// (src/test_utils/invariants.rs), which is the single source of truth. The
// negative fixtures below confirm each property catches the violation it claims.
//
// ## Invariant-gate check mechanism
//
// Against the subject crate:
//   cargo test --features test-utils --test invariant_gate

use associative_cache::associative_cache::AssociativeCache;
use associative_cache::get_index;
use associative_cache::shared::traits::bucket::Bucket;
use associative_cache::test_utils::{baseline_settings, run_invariant_suite, AssociativeCacheInvariants};

// ── P1: capacity_bound ──────────────────────────────────────────────────────

#[test]
fn capacity_bound_holds_under_overfill() {
	let s = baseline_settings();
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(s);
	for k in 0..500i64 {
		if cache.get_and_update_item(&k).is_none() {
			cache.put(k, k);
		}
	}
	for bucket in &cache.buckets {
		assert!(
			bucket.size() <= bucket.get_max_size(),
			"bucket holds {} items, max {}",
			bucket.size(),
			bucket.get_max_size()
		);
	}
}

// ── P2: slot_legality ───────────────────────────────────────────────────────

#[test]
fn slot_legality_holds_under_workload() {
	let s = baseline_settings();
	let mut cache: AssociativeCache<i64, i64> = AssociativeCache::new(s);
	let num_buckets = cache.buckets.len();
	for k in 0..500i64 {
		if cache.get_and_update_item(&k).is_none() {
			cache.put(k, k);
		}
	}
	for (idx, bucket) in cache.buckets.iter().enumerate() {
		for item in bucket.get_items() {
			assert_eq!(
				get_index(&item.key, num_buckets),
				idx,
				"key {} sits in bucket {idx} but hashes elsewhere",
				item.key
			);
		}
	}
}

// ── P4: capacity_conservation ───────────────────────────────────────────────

#[test]
fn capacity_conservation_holds_for_baseline_geometry() {
	// The baseline geometry resolves through the same path the eval path uses and
	// conserves the budget.
	let s = baseline_settings();
	let resolved = associative_cache::factory::resolve_geometry(&s).expect("baseline geometry must resolve");
	assert_eq!(resolved.capacity(), s.capacity(), "baseline geometry must conserve the capacity budget");
}

#[test]
fn capacity_conservation_detects_inflated_geometry() {
	// An inflating geometry (extra bucket) must be rejected by the conservation
	// check the eval path and the gate both run — proving the check itself rejects
	// inflation, not just that the fixture inflates.
	let s = baseline_settings();
	let budget = s.capacity();
	let inflated = s.with_geometry(s.num_of_buckets + 1, s.bucket_size);
	assert!(
		associative_cache::factory::check_conservation(budget, &inflated).is_err(),
		"conservation check must reject an inflated layout"
	);
}

// ── run_invariant_suite: drives the baseline cache ──────────────────────────
// Requires --features test-utils.

#[test]
fn baseline_passes_all_invariants() {
	run_invariant_suite(baseline_settings()).unwrap();
}

#[test]
fn associative_invariants_runs_baseline_suite() {
	use framework::Invariants;
	AssociativeCacheInvariants::run_invariant_suite().unwrap();
}
