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

/// Rebalanced geometry: reduces bucket count by 50% and increases bucket size by 100%.
/// This reduces hash collision probability while maintaining total capacity.
/// However, the transformation is only applied when it conserves exact capacity.
/// For budgets that don't factor evenly with the target configuration, the original
/// geometry is preserved to ensure the invariant constraint (capacity conservation)
/// is always satisfied.
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
		let budget = self.num_of_buckets * self.bucket_size;

		// Try to reduce bucket count by 50% and increase bucket size by 100%
		// to reduce hash collision probability while maintaining total capacity.
		let new_num_buckets = self.num_of_buckets / 2;
		let new_bucket_size = self.bucket_size * 2;

		// Only apply the transformation if it conserves capacity exactly.
		// For odd bucket counts, integer division loses capacity, so we keep
		// the original configuration to satisfy the invariant constraint.
		if new_num_buckets > 0 && new_num_buckets * new_bucket_size == budget {
			(new_num_buckets, new_bucket_size)
		} else {
			// Fall back to original geometry if transformation would violate
			// capacity conservation
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
