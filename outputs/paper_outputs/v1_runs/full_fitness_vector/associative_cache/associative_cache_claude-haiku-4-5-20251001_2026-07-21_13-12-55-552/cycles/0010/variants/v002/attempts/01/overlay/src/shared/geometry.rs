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

/// Identity geometry: returns the harness-provided split unchanged.
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
		// Wider buckets: halve buckets twice, double items twice, relative to baseline.
		// Preserve total capacity: num_buckets * bucket_size = total_capacity.
		// From 128×16 → 64×32 → 32×64
		let new_num_buckets = self.num_of_buckets / 4;
		if new_num_buckets == 0 {
			return (self.num_of_buckets, self.bucket_size);
		}
		if total_capacity % new_num_buckets == 0 {
			let new_bucket_size = total_capacity / new_num_buckets;
			(new_num_buckets, new_bucket_size)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
