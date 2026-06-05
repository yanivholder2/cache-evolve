//! Universal invariant gate: runs the cross-subject
//! [`framework::test_suite::run_invariant_suite`] against a freshly constructed
//! `TinyLfuCache` through the framework `Cache` trait.

use framework::test_suite::run_invariant_suite;
use tiny_lfu::tiny_lfu::TinyLfuCache;

const CAPACITY: usize = 8;

#[test]
fn universal_invariant_suite() {
	run_invariant_suite(|| Box::new(TinyLfuCache::<i64, i64>::new(CAPACITY))).unwrap();
}
