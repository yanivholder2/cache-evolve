//! Bucket geometry: how a fixed total capacity is factored into a bucket count
//! and a per-bucket slot count. Dispatched as a trait so a variant can reshape
//! the cache (wider/narrower buckets) without touching construction glue or the
//! JNI surface.
//!
//! The fitness comparison is only fair when every variant holds the same number
//! of entries, so the contract is a *factorisation* of a fixed budget, not a
//! free choice of capacity: `num_of_buckets * bucket_size == total_capacity`.
//! The invariant gate's `P4 capacity_conservation` enforces this; a geometry
//! that inflates capacity fails the gate.

pub trait BucketGeometry {
	/// Factor `total_capacity` into `(num_of_buckets, bucket_size)`.
	///
	/// Postcondition the gate enforces: both values are `> 0` and their product
	/// equals `total_capacity`.
	fn layout(&self, total_capacity: usize) -> (usize, usize);
}
