//! Contract gate: runs the cross-subject [`framework::test_suite::run_contract_suite`]
//! against a freshly constructed `TinyLfuCache` baseline.

use framework::test_suite::run_contract_suite;
use tiny_lfu::tiny_lfu::TinyLfuCache;

const CAPACITY: usize = 8;

#[test]
fn contract_suite() {
	run_contract_suite(|| Box::new(TinyLfuCache::<i64, i64>::new(CAPACITY)));
}
