//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` keeps the harness-provided type name, constructor and
//! `BucketGeometry` impl the construction glue and invariant gate reference, but
//! `layout` repartitions the *same* fixed budget toward **more, narrower sets**:
//! it quarters the per-bucket width (`bucket_size`) and quadruples the set count
//! (`num_of_buckets`) while conserving `num_of_buckets * bucket_size`. This tests
//! whether a finer hash partition (set diversity) lifts hit-rate above the SLRU
//! ceiling the wider-bucket sweep kept regressing against.
//!
//! The frontier had only ever pushed bucket width *up*; this is the untested
//! opposite direction. Only the buckets-vs-slots factoring moves — the bucket
//! internals (u64 reuse-mask, one-hit protection) are inherited unchanged.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Geometry seeded from the harness split, repartitioning toward narrower sets.
#[derive(Debug)]
pub struct BaselineGeometry {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way, so a variant that
	/// overlays this file inherits the same entry point.
	pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
		BaselineGeometry {
			num_of_buckets: settings.num_of_buckets,
			bucket_size: settings.bucket_size,
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	/// Repartition `total_capacity` toward more, narrower sets while conserving
	/// the total budget. Each split halves `bucket_size` and doubles
	/// `num_of_buckets`, keeping the product invariant; we take up to two splits
	/// (quarter the width, quadruple the set count) so the associativity lands
	/// genuinely narrower than the harness default.
	///
	/// A split requires an even `bucket_size > 1`. If `bucket_size` is odd (not
	/// evenly divisible), we return the harness split unchanged so
	/// `check_conservation` never rejects the variant. `bucket_size` only shrinks
	/// here, so the incumbent `u64` reuse-mask's `<= 64` slot bound still holds.
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		let mut num_of_buckets = self.num_of_buckets;
		let mut bucket_size = self.bucket_size;

		let mut splits = 0;
		while splits < 2 && bucket_size % 2 == 0 && bucket_size > 1 {
			bucket_size /= 2;
			num_of_buckets *= 2;
			splits += 1;
		}

		// Guard every invariant the seam must preserve; on any failure fall back
		// to the untouched harness split so the conservation check always passes.
		if num_of_buckets >= 1
			&& bucket_size >= 1
			&& bucket_size <= 64
			&& num_of_buckets.checked_mul(bucket_size) == Some(total_capacity)
		{
			(num_of_buckets, bucket_size)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
