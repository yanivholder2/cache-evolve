//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! This variant repartitions the *same* total capacity into fewer, wider
//! buckets: where the harness picks `num_of_buckets` buckets, this geometry
//! halves the bucket count and doubles the bucket size (when that stays
//! capacity-exact), doubling associativity so hot keys collide less often.
//! The `BaselineGeometry` type name and its `for_settings` constructor are
//! preserved because the factory and the invariant gate both build the
//! geometry through that entry point.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Coarsening geometry: halves the bucket count (doubling bucket size) when
/// that keeps the product capacity-exact, otherwise returns the harness split.
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
		// Halving the bucket count and doubling the bucket size conserves the
		// total capacity exactly only when the original bucket count is even and
		// at least 2. P4 capacity_conservation sweeps odd bucket counts too, so
		// the even-only guard is required to stay exact.
		if self.num_of_buckets >= 2 && self.num_of_buckets % 2 == 0 {
			(self.num_of_buckets / 2, self.bucket_size * 2)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
