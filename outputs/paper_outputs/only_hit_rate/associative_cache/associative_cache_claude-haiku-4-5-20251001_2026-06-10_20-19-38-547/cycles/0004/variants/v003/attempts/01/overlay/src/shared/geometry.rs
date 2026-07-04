//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! This variant implements a geometry optimization that uses fewer, wider buckets
//! to reduce collisions. The total capacity is conserved, but the bucket layout is
//! reshaped from many narrow buckets to fewer wide buckets.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Optimized geometry: uses fewer, wider buckets.
///
/// This reduces the bucket count by approximately 0.67-0.75 factor while
/// increasing bucket width proportionally, keeping total capacity fixed.
/// This geometry optimization aims to reduce collision rates, which in turn
/// should enable the cache to use a smaller stash (15% instead of 25%)
/// while maintaining competitive hit rates.
#[derive(Debug)]
pub struct BaselineGeometry {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings.
	pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
		BaselineGeometry {
			num_of_buckets: settings.num_of_buckets,
			bucket_size: settings.bucket_size,
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		// Reduce bucket count to approximately 0.33 of the original (a ~0.67 factor reduction).
		// This favors fewer, wider buckets. The exact factor (0.33) is chosen to balance
		// between collision reduction and practical bucket sizes.
		let reduction_factor = 0.33;
		let new_bucket_count = (self.num_of_buckets as f64 * reduction_factor).ceil() as usize;
		let new_bucket_count = new_bucket_count.max(1); // ensure at least 1 bucket

		// Compute the new bucket size to maintain total capacity.
		let new_bucket_size = if new_bucket_count > 0 {
			(total_capacity + new_bucket_count - 1) / new_bucket_count // ceiling division
		} else {
			total_capacity
		};

		(new_bucket_count, new_bucket_size)
	}
}
