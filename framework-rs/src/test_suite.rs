//! Cross-subject contract test suite. A subject's contract-gate test target
//! calls [`run_contract_suite`] with a factory that produces a fresh cache
//! instance per assertion; failures panic so the surrounding `#[test]` fails.
//!
//! [`run_invariant_suite`] is the universal (subject-agnostic) invariant suite:
//! it knows only the [`Cache`] surface and returns the first violation it finds,
//! so a subject's universal-invariant-gate target can `.unwrap()` it.

use crate::Cache;
use std::collections::HashMap;

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

/// Run the universal (subject-agnostic) invariant suite against `factory`.
///
/// Knows only the [`Cache`] surface (`put`/`get`), so it asserts only laws that
/// hold for every cache regardless of internal structure. Returns
/// `universal invariant violated: U<n> <name>` on the first violation; `Ok(())`
/// when every law holds. Black-box, so it sees only *observable* violations
/// (a value surfacing through `get`), never purely structural ones.
pub fn run_invariant_suite<F>(factory: F) -> Result<(), String>
where
	F: Fn() -> Box<dyn Cache<i64, i64>>,
{
	// Key span swept for never-put reads. A subject planting a construction-time
	// phantom must place it in this span for the universal gate to observe it.
	const SWEEP: i64 = 64;

	// U1 no_observable_phantom: a freshly built cache stored nothing, so every
	// key reads empty. A key returning a value here is an observable phantom.
	{
		let mut cache = factory();
		for key in 0..SWEEP {
			if cache.get(&key).is_some() {
				return Err(format!(
					"universal invariant violated: U1 no_observable_phantom (get(&{key}) returned a value on a freshly constructed cache)"
				));
			}
		}
	}

	// U2 put_get_round_trip: a freshly stored value is immediately readable.
	// `put`'s cross-subject precondition is that the key is not resident (a cache
	// only puts after a get-miss), so the suite never re-puts a live key.
	{
		let mut cache = factory();
		cache.put(1, 100);
		match cache.get(&1).copied() {
			Some(100) => {}
			other => {
				return Err(format!(
					"universal invariant violated: U2 put_get_round_trip (get(&1) returned {other:?}, expected Some(100))"
				))
			}
		}
	}

	// U3 workload_consistency: under a distinct-key put workload, a put key reads
	// back its value or None (legal eviction); a never-put key always reads None.
	{
		let mut cache = factory();
		let mut reference: HashMap<i64, i64> = HashMap::new();
		for key in (10i64..=130).step_by(10) {
			let value = key * 10;
			cache.put(key, value);
			reference.insert(key, value);
		}
		for (key, stored) in &reference {
			match cache.get(key).copied() {
				None => {}
				Some(value) if value == *stored => {}
				Some(value) => {
					return Err(format!(
						"universal invariant violated: U3 workload_consistency (get(&{key}) returned {value}, expected None or {stored})"
					))
				}
			}
		}
		for key in [3i64, 5, 7, 9, 11, 999] {
			if cache.get(&key).is_some() {
				return Err(format!(
					"universal invariant violated: U3 workload_consistency (never-put key {key} returned a value)"
				));
			}
		}
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::run_invariant_suite;
	use crate::Cache;
	use std::collections::HashMap;

	struct MapCache {
		map: HashMap<i64, i64>,
	}

	impl MapCache {
		fn new() -> Self {
			Self {
				map: HashMap::new(),
			}
		}
	}

	impl Cache<i64, i64> for MapCache {
		fn put(&mut self, key: i64, value: i64) {
			self.map.insert(key, value);
		}
		fn get(&mut self, key: &i64) -> Option<&i64> {
			self.map.get(key)
		}
	}

	/// A cache with a never-put item present from construction — the observable
	/// phantom the universal suite must catch.
	struct PhantomCache {
		map: HashMap<i64, i64>,
	}

	impl PhantomCache {
		fn new() -> Self {
			let mut map = HashMap::new();
			map.insert(7, 7);
			Self { map }
		}
	}

	impl Cache<i64, i64> for PhantomCache {
		fn put(&mut self, key: i64, value: i64) {
			self.map.insert(key, value);
		}
		fn get(&mut self, key: &i64) -> Option<&i64> {
			self.map.get(key)
		}
	}

	#[test]
	fn good_cache_passes() {
		assert!(run_invariant_suite(|| Box::new(MapCache::new())).is_ok());
	}

	#[test]
	fn phantom_cache_violates_no_observable_phantom() {
		let message = run_invariant_suite(|| Box::new(PhantomCache::new()))
			.expect_err("phantom cache must violate the universal suite");
		assert!(
			message.contains("U1 no_observable_phantom"),
			"unexpected violation message: {message}"
		);
	}
}
