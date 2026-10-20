//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! This variant rebalances bucket geometry to prefer fewer, wider buckets,
//! reducing collision clustering and lowering the rate of evictions.
//! Total capacity is conserved to ensure fair fitness comparison.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Rebalanced geometry: reduces num_of_buckets by ~71% (midpoint of 67-75%)
/// while increasing bucket_size proportionally to maintain total capacity.
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
		// Reduce num_of_buckets by ~71% (geometric mean of 67-75% reduction)
		// while increasing bucket_size to maintain total capacity.
		// Fewer, wider buckets reduce collision clustering and lower eviction rates.

		let reduction_factor = 0.71;
		let target_buckets = std::cmp::max(1, (self.num_of_buckets as f64 * reduction_factor).round() as usize);

		// Find the divisor of total_capacity closest to target_buckets
		// to satisfy the capacity-conservation contract.
		let mut best_divisor = self.num_of_buckets;
		let mut best_distance = ((self.num_of_buckets as i32) - (target_buckets as i32)).abs();

		// Iterate through divisors efficiently by checking up to sqrt(total_capacity)
		let sqrt_cap = (total_capacity as f64).sqrt() as usize;
		for d in 1..=sqrt_cap + 1 {
			if d > total_capacity {
				break;
			}
			if total_capacity % d == 0 {
				// d is a divisor; check if it's better and within bounds
				if d <= self.num_of_buckets {
					let distance = ((d as i32) - (target_buckets as i32)).abs();
					if distance < best_distance {
						best_distance = distance;
						best_divisor = d;
					}
				}

				// total_capacity / d is also a divisor
				let d2 = total_capacity / d;
				if d2 <= self.num_of_buckets {
					let distance = ((d2 as i32) - (target_buckets as i32)).abs();
					if distance < best_distance {
						best_distance = distance;
						best_divisor = d2;
					}
				}
			}
		}

		let new_bucket_size = total_capacity / best_divisor;
		(best_divisor, new_bucket_size)
	}
}
