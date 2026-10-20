//! Identity geometry: returns the harness-provided split unchanged.
//!
//! The three-tier stash is managed within AssociativeCache alongside the full
//! bucket capacity. Stash capacity is calculated as a fraction of the total bucket
//! capacity (hot: 10%, warm: 10%, cold: 5%).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Identity geometry: returns the harness-provided split unchanged.
/// Stash management (hot/warm/cold tiering) is handled within AssociativeCache.
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
		// Identity: return the harness-provided split unchanged.
		// Stash capacity is derived within AssociativeCache as a fraction of bucket capacity.
		(self.num_of_buckets, self.bucket_size)
	}
}
