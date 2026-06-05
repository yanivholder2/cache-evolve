//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! This variant implements a 75/25 normal/stash split: normal buckets get
//! 75 % of the total bucket count, stash gets the remaining 25 %. Both sides
//! are clamped to a minimum of 2 (normal) and 1 (stash) to satisfy the
//! SegmentCore validity constraints.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// 75/25 geometry: routes 75 % of the total bucket budget to normal buckets
/// and 25 % to stash, keeping all other topology dimensions unchanged.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way, so a variant that
	/// overlays this file inherits the same entry point.
	pub fn for_settings(settings: &DashSettings) -> Self {
		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_normal_buckets: settings.num_of_normal_buckets,
				num_of_stash_buckets: settings.num_of_stash_buckets,
				bucket_size: settings.bucket_size,
			},
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, _total_capacity: usize) -> DashGeometry {
		let total_buckets =
			self.topology.num_of_normal_buckets + self.topology.num_of_stash_buckets;
		let normal = (total_buckets * 3 / 4).max(2);
		let stash = (total_buckets - normal).max(1);
		DashGeometry {
			num_of_segments:       self.topology.num_of_segments,
			num_of_normal_buckets: normal,
			num_of_stash_buckets:  stash,
			bucket_size:           self.topology.bucket_size,
		}
	}
}
