//! Universal invariant gate: runs the cross-subject
//! [`framework::test_suite::run_invariant_suite`] against the baseline `S3Fifo`.

use framework::test_suite::run_invariant_suite;
use s3_fifo::s3_fifo::S3Fifo;

const CAPACITY: usize = 8;

#[test]
fn universal_invariant_suite() {
	run_invariant_suite(|| Box::new(S3Fifo::<i64, i64>::new(CAPACITY))).unwrap();
}
