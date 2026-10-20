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

/// Geometry with 80/20 bucket allocation: 80% normal buckets, 20% stash.
/// This variant repartitions the harness-provided total capacity into an 80/20 split
/// while keeping segment count and bucket size unchanged.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings, recomputing the bucket
	/// allocation to 80/20 (80% normal, 20% stash) while preserving the total capacity.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;
		let num_of_normal_buckets = ((total_buckets * 4) / 5).max(2);
		let num_of_stash_buckets = total_buckets - num_of_normal_buckets;

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
