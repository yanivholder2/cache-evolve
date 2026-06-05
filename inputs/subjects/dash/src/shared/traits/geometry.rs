//! Bucket geometry: how a fixed total capacity is factored into a Dash
//! topology — segment count, normal/stash bucket counts, and per-bucket slot
//! count. Dispatched as a trait so a variant can reshape the cache (wider
//! buckets, a different normal/stash split, more segments) without touching the
//! construction glue or the JNI surface.
//!
//! The fitness comparison is only fair when every variant holds the same number
//! of entries, so the contract is a *factorisation* of a fixed budget, not a
//! free choice of capacity:
//!
//! ```text
//! num_of_segments * (num_of_normal_buckets + num_of_stash_buckets) * bucket_size == total_capacity
//! ```
//!
//! Unlike the associative baseline, not every factorisation is legal: probing
//! needs `num_of_normal_buckets >= 2`, the stash needs `num_of_stash_buckets >= 1`,
//! and both `num_of_segments` and `bucket_size` must be `> 0`. The invariant
//! gate's `P6 capacity_conservation` enforces both the budget equality and the
//! structural validity; a geometry that inflates capacity, or that yields a
//! topology `DashSettings::validate` rejects, fails the gate.

/// A resolved Dash topology: the four numbers a geometry factors a budget into.
/// Mirrors the topology fields of `DashSettings` (which carries the eviction
/// policy and debug knobs the geometry leaves untouched).
#[derive(Debug, Clone, Copy)]
pub struct DashGeometry {
	pub num_of_segments: usize,
	pub num_of_normal_buckets: usize,
	pub num_of_stash_buckets: usize,
	pub bucket_size: usize,
}

pub trait BucketGeometry {
	/// Factor `total_capacity` into a [`DashGeometry`].
	///
	/// Postcondition the gate enforces (`P6 capacity_conservation`): the
	/// returned topology is structurally valid and its capacity
	/// (`segments * (normal + stash) * bucket_size`) equals `total_capacity`.
	fn layout(&self, total_capacity: usize) -> DashGeometry;
}
