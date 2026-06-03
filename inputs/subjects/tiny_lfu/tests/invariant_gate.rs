//! Invariant gate — invariant-gate artifact.
//!
//! The universal invariant suite is cross-subject and black-box; this gate drives
//! the baseline `TinyLfuCache` implementation directly.

use tiny_lfu::test_utils::{run_invariant_suite, TinyLfuInvariants};
use tiny_lfu::tiny_lfu::TinyLfuCache;

const CAPACITY: usize = 8;

#[test]
fn capacity_bound_holds_under_overfill() {
	let mut cache: TinyLfuCache<i64, i64> = TinyLfuCache::new(CAPACITY);
	for k in 0..100i64 {
		if cache.get(&k).is_none() {
			cache.put(k, k);
		}
	}
	let resident = (0..100i64).filter(|k| cache.get(k).is_some()).count();
	assert!(resident <= CAPACITY, "residents {resident} exceeds capacity {CAPACITY}");
}

#[test]
fn fresh_cache_has_no_phantoms() {
	let mut cache: TinyLfuCache<i64, i64> = TinyLfuCache::new(CAPACITY);
	for k in 0..CAPACITY as i64 {
		assert!(cache.get(&k).is_none(), "fresh cache returned a value for key {k}");
	}
}

#[test]
fn baseline_passes_all_invariants() {
	run_invariant_suite(CAPACITY).unwrap();
}

#[test]
fn tiny_lfu_invariants_runs_baseline_suite() {
	use framework::Invariants;
	TinyLfuInvariants::run_invariant_suite().unwrap();
}
