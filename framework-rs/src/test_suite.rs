//! Cross-subject contract test suite. A subject's contract-gate test target
//! calls [`run_contract_suite`] with a factory that produces a fresh cache
//! instance per assertion; failures panic so the surrounding `#[test]` fails.

use crate::Cache;

/// Run the cross-subject contract suite against `factory`.
///
/// Panics on the first contract violation. Currently checks:
///   - constructor succeeds (the factory call returns a usable cache)
///   - put-then-get round-trip preserves the stored value
pub fn run_contract_suite<F>(factory: F)
where
	F: Fn() -> Box<dyn Cache<i64, i64>>,
{
	// constructor_succeeds: factory must produce a cache without panicking.
	let _cache = factory();

	// put_get_round_trip: stored value must be readable through `get`.
	let mut cache = factory();
	cache.put(1, 100);
	let value = cache
		.get(&1)
		.copied()
		.expect("contract gate: get(&1) returned None immediately after put(1, 100)");
	assert_eq!(
		value, 100,
		"contract gate: get(&1) returned {value}, expected 100"
	);
}
