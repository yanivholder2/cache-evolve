//! Contract gate: runs the cross-subject [`framework::test_suite::run_contract_suite`]
//! against a freshly constructed `S3Fifo` baseline.

use framework::test_suite::run_contract_suite;
use s3_fifo::s3_fifo::S3Fifo;

const CAPACITY: usize = 8;

#[test]
fn contract_suite() {
	run_contract_suite(|| Box::new(S3Fifo::<i64, i64>::new(CAPACITY)));
}
