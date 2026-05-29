//! Universal invariant gate: runs the cross-subject
//! [`framework::test_suite::run_invariant_suite`] against a freshly constructed
//! `LruCache` through the framework `Cache` trait.

use framework::test_suite::run_invariant_suite;
use lru_cache::lru_cache::LruCache;

const CAPACITY: usize = 8;

#[test]
fn universal_invariant_suite() {
	run_invariant_suite(|| Box::new(LruCache::<i64, i64>::new(CAPACITY))).unwrap();
}
