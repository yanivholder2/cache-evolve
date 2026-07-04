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

/// Wider-bucket geometry: halves the bucket count and doubles the bucket size,
/// preserving the fixed total capacity `num_of_buckets * bucket_size`.
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
		// Widen the buckets: fewer, larger buckets over the same total capacity.
		// Only valid when the count is even and > 1, so the product is preserved
		// and at least one bucket remains; otherwise keep the identity split.
		if self.num_of_buckets > 1 && self.num_of_buckets % 2 == 0 {
			(self.num_of_buckets / 2, self.bucket_size * 2)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
