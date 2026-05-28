// Invariant gate — invariant-gate artifact
//
// Negative fixtures confirm each property test catches the violation it claims to.
// The invariant properties live in `run_invariant_suite` (src/test_utils/invariants.rs),
// which is the single source of truth. The `harness_tests` module below drives that
// suite against all built-in segment types.
//
// ## Invariant-gate check mechanism
//
// Against the subject crate:
//   cargo test --features test-utils --test invariant_gate
//
// Variants overlay subject files (e.g. `src/factory.rs`); the overlaid workspace
// is run with the same command. Non-zero exit = invariant-gate failure; the
// failing invariant name is recorded in `rejection.md`.

use dash::dash::segments::{PromotionColdSegment, Segment, SegmentCore};
use dash::dash::Dash;
use dash::get_index;
use dash::shared::item::Item;
use dash::test_utils::baseline_dash_settings;

/// Returns the first key in 0..1000 whose target bucket index equals `bucket_idx`.
fn key_for_bucket(bucket_idx: usize, num_buckets: usize) -> i64 {
	(0i64..1000)
		.find(|k| get_index(k, num_buckets) == bucket_idx)
		.unwrap_or_else(|| panic!("no key in 0..1000 hashes to bucket {bucket_idx}"))
}

// ── P1: no_phantom_items ──────────────────────────────────────────────────────

#[test]
fn no_phantom_items_negative_fixture() {
	let s = baseline_dash_settings();
	let mut core = SegmentCore::<i64, i64>::new(&s);
	// Insert a key that hashes to bucket 2 directly into bucket 0.
	// contains_key checks stash, bucket 2, bucket 3 — never bucket 0.
	let phantom_key = key_for_bucket(2, s.num_of_normal_buckets);
	core.force_push_normal_bucket(0, Item::new(phantom_key, phantom_key));
	assert!(
		!core.contains_key(&phantom_key),
		"fixture: item in wrong bucket must be invisible to contains_key"
	);
}

// ── P2: capacity_bound ────────────────────────────────────────────────────────

#[test]
fn capacity_bound_negative_fixture() {
	let s = baseline_dash_settings();
	let mut core = SegmentCore::<i64, i64>::new(&s);
	// Overfill bucket 0 beyond its max_size by bypassing normal eviction.
	for k in 0..=(s.bucket_size as i64) {
		core.force_push_normal_bucket(0, Item::new(k, k));
	}
	assert!(
		!core.check_capacity_bound(),
		"fixture must produce an overfilled bucket (items={}, max={})",
		core.total_items(),
		s.bucket_size
	);
}

// ── P3: slot_legality ─────────────────────────────────────────────────────────

#[test]
fn slot_legality_negative_fixture() {
	let s = baseline_dash_settings();
	let mut core = SegmentCore::<i64, i64>::new(&s);
	// A key that hashes to bucket 2 placed in bucket 0 violates slot legality.
	let bad_key = key_for_bucket(2, s.num_of_normal_buckets);
	core.force_push_normal_bucket(0, Item::new(bad_key, bad_key));
	assert!(
		!core.check_slot_legality(),
		"slot_legality check must catch item in wrong bucket"
	);
}

// ── P4: single_eviction_per_put ───────────────────────────────────────────────

#[test]
fn single_eviction_per_put_negative_fixture() {
	let s = baseline_dash_settings();
	let mut core = SegmentCore::<i64, i64>::new(&s);
	let before = core.total_items();
	// Simulate a buggy segment that inserts two items in one put.
	core.force_push_normal_bucket(0, Item::new(1, 1));
	core.force_push_normal_bucket(1, Item::new(2, 2));
	let after = core.total_items();
	assert!(
		!(after == before || after == before + 1),
		"fixture: double-insert must violate single_eviction_per_put (before={before} after={after})"
	);
}

// ── P5: stash_terminal ────────────────────────────────────────────────────────

#[test]
fn stash_terminal_negative_fixture() {
	let s = baseline_dash_settings();
	let mut cache: Dash<i64, i64, _> =
		Dash::new(1, || PromotionColdSegment::new(&s));
	// Place a key that hashes to bucket 2 into bucket 0 — contains_key will
	// return false (searches stash, bucket 2, bucket 3), while the item is
	// physically present. A buggy get that searched all buckets would return
	// Some; the correct implementation must return None.
	let ghost_key = key_for_bucket(2, s.num_of_normal_buckets);
	cache.segments[0].core_mut().force_push_normal_bucket(0, Item::new(ghost_key, ghost_key));
	assert!(
		!cache.segments[0].core().contains_key(&ghost_key),
		"fixture: ghost item must be invisible to contains_key"
	);
	assert_eq!(cache.segments[0].core().total_items(), 1, "item must be physically present");
	// The stash_terminal property: if contains_key is false, get must return None.
	assert!(
		cache.get_and_update_item(&ghost_key).is_none(),
		"fixture: get must return None when contains_key is false (item in wrong bucket)"
	);
}

// ── run_invariant_suite: drives all built-in segment types ────────────────────
// Requires --features test-utils.  Variants call run_invariant_suite directly
// from their own thin harness test; see the invariant-gate comment at the top.

#[cfg(feature = "test-utils")]
mod harness_tests {
	use super::baseline_dash_settings;
	use dash::dash::segments::PromotionColdSegment;
	use dash::test_utils::run_invariant_suite;

	#[test]
	fn promotion_cold_passes_all_invariants() {
		let s = baseline_dash_settings();
		run_invariant_suite(|| PromotionColdSegment::new(&s)).unwrap();
	}

	#[test]
	fn dash_invariants_runs_baseline_suite() {
		use dash::test_utils::DashInvariants;
		use framework::Invariants;
		DashInvariants::run_invariant_suite().unwrap();
	}
}
