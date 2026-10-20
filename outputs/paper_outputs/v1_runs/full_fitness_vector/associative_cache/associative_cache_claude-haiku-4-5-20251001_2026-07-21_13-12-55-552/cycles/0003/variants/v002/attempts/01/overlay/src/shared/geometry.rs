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
		// Repartition to wider buckets with lower bucket count.
		// Preserve total capacity: num_buckets * bucket_size = total_capacity.
		// Attempt to halve the number of buckets and correspondingly adjust bucket size.

		// Try to halve the bucket count
		let new_num_buckets = self.num_of_buckets / 2;

		if new_num_buckets == 0 {
			// Can't halve from 1 bucket; return unchanged to preserve capacity
			return (self.num_of_buckets, self.bucket_size);
		}

		// Check if we can divide capacity evenly with the new bucket count
		if total_capacity % new_num_buckets == 0 {
			// We can divide evenly; use the halved bucket count and doubled bucket size
			let new_bucket_size = total_capacity / new_num_buckets;
			(new_num_buckets, new_bucket_size)
		} else {
			// Can't divide evenly with halved bucket count; return unchanged
			// This preserves capacity exactly when simple halving would lose capacity.
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
