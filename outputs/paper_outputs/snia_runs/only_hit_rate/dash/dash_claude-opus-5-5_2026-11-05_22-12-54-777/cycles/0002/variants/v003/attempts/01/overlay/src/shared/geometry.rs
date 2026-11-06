//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! Variant v003 (H03): child of v002. Pushes the normal/stash split from 3/4
//! normal (24/8 of 32) to 7/8 normal (28/4 of 32), SLRU-style. Segment count,
//! bucket size, and the per-segment total bucket count are kept exactly as the
//! harness chose, so total capacity is unchanged. Stash/probing indexing in the
//! segment derives from these counts, so no other file changes.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Geometry that sets the normal/stash ratio to 87.5% normal / 12.5% stash.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;
		let stash = (total_buckets / 8).max(1);
		let normal = total_buckets.saturating_sub(stash);

		let (num_of_normal_buckets, num_of_stash_buckets) = if normal < 2 {
			// Too small to re-split legally: keep the harness split unchanged.
			(settings.num_of_normal_buckets, settings.num_of_stash_buckets)
		} else {
			(normal, stash)
		};

		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_normal_buckets,
				num_of_stash_buckets,
				bucket_size: settings.bucket_size,
			},
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, _total_capacity: usize) -> DashGeometry {
		self.topology
	}
}
