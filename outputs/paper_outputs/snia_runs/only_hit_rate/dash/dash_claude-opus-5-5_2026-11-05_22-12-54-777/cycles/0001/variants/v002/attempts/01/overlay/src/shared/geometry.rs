//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! Variant v002 (H02): re-partition each segment's bucket budget so the normal
//! (target/probing) tier is the majority — 3/4 normal, 1/4 stash — SLRU-style.
//! Segment count, bucket size, and the per-segment total bucket count are kept
//! exactly as the harness chose, so total capacity is unchanged.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Geometry that flips the normal/stash ratio to 75% normal / 25% stash.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;
		let stash = (total_buckets / 4).max(1);
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
