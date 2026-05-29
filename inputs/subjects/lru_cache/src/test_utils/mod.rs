/// Capacity for the baseline invariant suite.
pub const BASELINE_CAPACITY: usize = 8;

#[cfg(feature = "test-utils")]
pub use invariants::{run_invariant_suite, LruInvariants};

#[cfg(feature = "test-utils")]
mod invariants;
