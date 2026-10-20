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

/// Geometry seam: repartitions the fixed total capacity into wider buckets when
/// the budget is a multiple of 32, otherwise returns the harness-provided split.
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
		if total_capacity % 32 == 0 {
			(total_capacity / 32, 32)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
