//! 4x-wider bucket geometry: divides the number of buckets by 4 to create
//! wider buckets, improving associativity and working-set capture for SLRU.
//!
//! For the standard 128×16=2048 configuration this yields 32×64.
//! The total capacity budget is always conserved: `num_of_buckets * bucket_size`
//! is invariant. When the 4x reduction does not divide the budget evenly,
//! the original layout is used as a fallback (also capacity-conserving).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// 4x-wider bucket geometry: one quarter as many buckets, four times as many
/// slots per bucket. The `BaselineGeometry` name is kept so the construction
/// glue and invariant gate can reference it without change.
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
		// Target: 1/4 as many buckets (4x wider each).
		// max(…, 1) ensures we never produce zero buckets.
		let target_num_buckets = (self.num_of_buckets / 4).max(1);

		// Use the wider layout only when it evenly divides the budget.
		if total_capacity % target_num_buckets == 0 {
			let bucket_size = total_capacity / target_num_buckets;
			(target_num_buckets, bucket_size)
		} else {
			// Fallback: preserve the original layout (always capacity-conserving).
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
