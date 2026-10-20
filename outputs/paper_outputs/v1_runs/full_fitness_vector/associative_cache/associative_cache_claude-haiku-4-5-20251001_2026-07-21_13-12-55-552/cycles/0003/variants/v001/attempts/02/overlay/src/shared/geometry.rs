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
		// Attempt to narrow buckets by doubling bucket count and halving bucket size,
		// while preserving total capacity.
		//
		// For the baseline (128 × 16 = 2048), this yields 256 × 8 = 2048.
		// For settings where doubling buckets doesn't divide evenly, we reduce the
		// multiplication until we find a factor that preserves capacity exactly.

		let mut multiplier = 2;
		let max_multiplier = 10; // Arbitrary upper bound to avoid infinite loops

		loop {
			let new_num_buckets = self.num_of_buckets * multiplier;

			// Check if total_capacity divides evenly by new_num_buckets
			if total_capacity % new_num_buckets == 0 {
				let new_bucket_size = total_capacity / new_num_buckets;
				return (new_num_buckets, new_bucket_size);
			}

			// Didn't divide evenly; try a smaller multiplier
			multiplier -= 1;

			if multiplier == 0 {
				// Shouldn't happen with valid input, but fall back to identity
				return (self.num_of_buckets, self.bucket_size);
			}
		}
	}
}
