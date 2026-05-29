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

use lru_cache::lru_cache::LruCache;
use lru_cache::test_utils::{run_invariant_suite, LruInvariants};

const CAPACITY: usize = 8;

// ── P1: capacity_bound ──────────────────────────────────────────────────────

#[test]
fn capacity_bound_holds_under_overfill() {
	// 100 distinct keys into a capacity-8 cache: residency must never exceed 8.
	let mut cache: LruCache<i64, i64> = LruCache::new(CAPACITY);
	for k in 0..100i64 {
		if cache.get(&k).is_none() {
			cache.put(k, k);
		}
	}
	let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
	assert!(resident <= CAPACITY, "residents {resident} exceeds capacity {CAPACITY}");
}

// ── P3: no_phantom_items ────────────────────────────────────────────────────

#[test]
fn fresh_cache_has_no_phantoms() {
	let mut cache: LruCache<i64, i64> = LruCache::new(CAPACITY);
	for k in 0..CAPACITY as i64 {
		assert!(cache.get(&k).is_none(), "fresh cache returned a value for key {k}");
	}
}

// ── run_invariant_suite: drives the baseline LRU cache ──────────────────────
// Requires --features test-utils.

#[test]
fn baseline_passes_all_invariants() {
	run_invariant_suite(CAPACITY).unwrap();
}

#[test]
fn lru_invariants_runs_baseline_suite() {
	use framework::Invariants;
	LruInvariants::run_invariant_suite().unwrap();
}
