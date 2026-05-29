//! Contract gate: runs the cross-subject [`framework::test_suite::run_contract_suite`]
//! against a freshly constructed `LruCache`. LruCache has no extension points and
//! no factory seam — its only knob is capacity — so the gate builds it directly.

use framework::test_suite::run_contract_suite;
use lru_cache::lru_cache::LruCache;

const CAPACITY: usize = 8;

#[test]
fn contract_suite() {
	run_contract_suite(|| Box::new(LruCache::<i64, i64>::new(CAPACITY)));
}
