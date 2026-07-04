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

/// Absolute-associativity geometry: factors the fixed budget so each bucket lands
/// on (or as close as the budget's divisors allow to) a target collision-set
/// width, rather than halving the harness-provided split relatively.
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
		// H03: derive the split from an *absolute* target associativity instead of
		// halving the harness split relatively. Choose `bucket_size` as the divisor
		// of `total_capacity` closest to the target `T`, so the collision-set width
		// can land on non-power-of-two values the halving sequence skips over.
		//
		// `T` is the tunable under test.
		const TARGET_ASSOCIATIVITY: usize = 8;

		// Degenerate budget: nothing to factor — fall back to the identity split.
		if total_capacity == 0 {
			return (self.num_of_buckets, self.bucket_size);
		}

		// Pick the divisor `d` of `total_capacity` minimising `|d - T|`. `d == 1`
		// always divides, so `best_bs` is always set; because `d` divides exactly,
		// `num_of_buckets * bucket_size == total_capacity` holds for every budget,
		// keeping P4 capacity_conservation satisfied across the full sweep.
		let mut best_bs = 1usize;
		let mut best_dist = usize::MAX;
		let mut d = 1usize;
		while d <= total_capacity {
			if total_capacity % d == 0 {
				let dist = if d > TARGET_ASSOCIATIVITY {
					d - TARGET_ASSOCIATIVITY
				} else {
					TARGET_ASSOCIATIVITY - d
				};
				if dist < best_dist {
					best_dist = dist;
					best_bs = d;
				}
			}
			d += 1;
		}

		let bucket_size = best_bs;
		let num_of_buckets = total_capacity / bucket_size;
		(num_of_buckets, bucket_size)
	}
}
