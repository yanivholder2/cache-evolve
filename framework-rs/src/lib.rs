//! Cross-subject cache contract: runtime behaviour ([`Cache`]) and
//! correctness gate ([`Invariants`]).

use std::hash::Hash;

/// A key-value cache.
pub trait Cache<K: Hash + Eq, V> {
	fn put(&mut self, key: K, value: V);
	fn get(&mut self, key: &K) -> Option<&V>;
}

/// Subject-level invariant suite. Implementors expose a single entry point
/// that the invariant gate runs against the baseline Subject.
pub trait Invariants {
	fn run_invariant_suite() -> Result<(), String>;
}
