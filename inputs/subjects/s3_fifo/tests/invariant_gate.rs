//! Invariant gate — invariant-gate artifact.
//!
//! The universal invariant suite is cross-subject and black-box; this gate drives
//! the baseline `S3Fifo` implementation directly.

use s3_fifo::s3_fifo::S3Fifo;
use s3_fifo::test_utils::{run_invariant_suite, S3FifoInvariants};

const CAPACITY: usize = 8;

#[test]
fn capacity_bound_holds_under_overfill() {
	let mut cache: S3Fifo<i64, i64> = S3Fifo::new(CAPACITY);
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
	let mut cache: S3Fifo<i64, i64> = S3Fifo::new(CAPACITY);
	for k in 0..CAPACITY as i64 {
		assert!(cache.get(&k).is_none(), "fresh cache returned a value for key {k}");
	}
}

#[test]
fn baseline_passes_all_invariants() {
	run_invariant_suite(CAPACITY).unwrap();
}

#[test]
fn s3_fifo_invariants_runs_baseline_suite() {
	use framework::Invariants;
	S3FifoInvariants::run_invariant_suite().unwrap();
}
