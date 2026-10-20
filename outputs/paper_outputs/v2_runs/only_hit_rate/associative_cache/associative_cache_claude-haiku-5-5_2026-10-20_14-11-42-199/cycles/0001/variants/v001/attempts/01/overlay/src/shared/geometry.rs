//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! This variant (H01) repartitions the fixed capacity into wide buckets of
//! `WIDE_BUCKET_SIZE` slots, ignoring the harness-provided split. Conservation
//! (`num_of_buckets * bucket_size == total_capacity`) is preserved by deriving
//! the bucket count from the budget.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Slots per bucket under this geometry.
const WIDE_BUCKET_SIZE: usize = 64;

/// Geometry seam: fixed-width buckets over the total capacity budget.
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
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		debug_assert!(
			total_capacity % WIDE_BUCKET_SIZE == 0,
			"wide-bucket geometry requires total_capacity divisible by {WIDE_BUCKET_SIZE}, got {total_capacity}"
		);
		if total_capacity == 0 || total_capacity % WIDE_BUCKET_SIZE != 0 {
			// Budget does not factor into wide buckets: keep the harness split so the
			// conservation check sees an exact factorisation of the budget.
			return (self.num_of_buckets, self.bucket_size);
		}
		(total_capacity / WIDE_BUCKET_SIZE, WIDE_BUCKET_SIZE)
	}
}
