//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfSegments`, `numOfNormalBuckets`, `numOfStashBuckets`, `bucketSize`),
//! so the baseline topology is unchanged. A variant overlays this file to
//! repartition the *same* total capacity differently — for example a wider
//! stash, fewer-but-wider normal buckets, or more segments — while keeping the
//! `BaselineGeometry` type name the construction glue references.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Reshaped geometry: allocates more normal buckets and reduces stash size.
/// This variant increases normal buckets from 8 to 16 and decreases stash buckets
/// from 24 to 16, keeping total capacity constant at 2048 slots.
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
				num_of_normal_buckets: 16,
				num_of_stash_buckets: 16,
				bucket_size: 64,
			},
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, _total_capacity: usize) -> DashGeometry {
		self.topology
	}
}
