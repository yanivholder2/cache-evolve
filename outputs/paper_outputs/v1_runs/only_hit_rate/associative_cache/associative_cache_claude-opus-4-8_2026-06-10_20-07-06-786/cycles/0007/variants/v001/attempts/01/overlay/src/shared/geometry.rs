//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Coarsening geometry: applies up to two capacity-conserving widening steps to
/// the harness-provided split (fewer, wider buckets).
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
	fn layout(&self, _total_capacity: usize) -> (usize, usize) {
		// H01: apply the capacity-conserving widening step up to TWO times —
		// each step halves the bucket count and doubles the bucket size, giving
		// each collision set still more associativity to retain hot keys. The
		// parent confirmed one halving helps; this overshoots by one step to
		// locate the associativity sweet spot.
		//
		// Each iteration preserves the product exactly (`nb * bs` unchanged), so
		// the capacity invariant `nb * bs == self.num_of_buckets * self.bucket_size`
		// always holds. The loop only widens while `nb` is even and halving keeps
		// `nb >= 1` (`nb / 2 >= 1`), so odd or already-minimal bucket counts fall
		// back to the identity/one-step split.
		let mut nb = self.num_of_buckets;
		let mut bs = self.bucket_size;
		let mut steps = 0;
		while steps < 2 && nb % 2 == 0 && nb / 2 >= 1 {
			nb /= 2;
			bs *= 2;
			steps += 1;
		}
		(nb, bs)
	}
}
