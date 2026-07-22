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
		// Repartition to narrower buckets with higher bucket count.
		// Preserve total capacity: num_buckets * bucket_size = total_capacity.
		// Attempt to double the number of buckets and halve the bucket size.
		// When bucket_size is odd or doubling doesn't preserve capacity exactly,
		// find the closest divisor of total_capacity that increases bucket count.

		let target_num_buckets = self.num_of_buckets * 2;

		// Check if we can achieve the target exactly
		if total_capacity % target_num_buckets == 0 {
			// Perfect! Use the target split.
			(target_num_buckets, total_capacity / target_num_buckets)
		} else {
			// Doubling doesn't preserve capacity with even bucket distribution.
			// Find the first divisor of total_capacity that's >= target_num_buckets.
			// This increases bucket count as close to doubling as possible while
			// preserving capacity exactly.
			let mut found = false;
			let mut best_num_buckets = self.num_of_buckets; // Fallback

			for candidate in target_num_buckets..=total_capacity {
				if total_capacity % candidate == 0 {
					best_num_buckets = candidate;
					found = true;
					break;
				}
			}

			if found {
				(best_num_buckets, total_capacity / best_num_buckets)
			} else {
				// Fallback (shouldn't happen for valid budgets)
				(self.num_of_buckets, self.bucket_size)
			}
		}
	}
}
