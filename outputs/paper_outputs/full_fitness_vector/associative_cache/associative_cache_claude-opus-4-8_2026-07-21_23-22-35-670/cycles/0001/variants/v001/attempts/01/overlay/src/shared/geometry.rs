//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! This variant (H01) repartitions the fixed total capacity into one-quarter as
//! many buckets of quadruple width, raising set-associativity so wider buckets
//! absorb more hash collisions. The `(num, size)` product is conserved, so the
//! P4 capacity-conservation check still passes and keys remain hashed modulo the
//! resolved bucket count (P2 slot-legality holds).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Identity geometry entry point, reshaped in `layout` to widen buckets.
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
		// Conserve the total budget: num_of_buckets * bucket_size is invariant
		// under this repartition, so P4 (capacity conservation) still holds.
		if self.num_of_buckets % 4 == 0 && self.num_of_buckets / 4 >= 1 {
			// Quarter as many buckets, four times as wide — for 128 x 16 this
			// yields (32, 64), lifting set-associativity to cut conflict misses.
			(self.num_of_buckets / 4, self.bucket_size * 4)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
