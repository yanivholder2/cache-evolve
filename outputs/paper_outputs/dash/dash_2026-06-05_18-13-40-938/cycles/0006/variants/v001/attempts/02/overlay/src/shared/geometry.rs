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

/// Identity geometry: returns the harness-provided split unchanged.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way, so a variant that
	/// overlays this file inherits the same entry point.
	///
	/// **H01 modification**: Expand stash bucket capacity while reducing normal
	/// buckets proportionally to maintain total capacity. Increase stash by 2 buckets,
	/// decrease normal by 2 buckets to conserve the total budget.
	///
	/// For example: settings with 4 stash + 28 normal → 6 stash + 26 normal (total 32→32).
	/// For test settings with 2 stash + 4 normal → 4 stash + 2 normal (total 6→6).
	pub fn for_settings(settings: &DashSettings) -> Self {
		let new_stash = settings.num_of_stash_buckets.saturating_add(2);
		let new_normal = settings.num_of_normal_buckets.saturating_sub(2);

		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_stash_buckets: new_stash,
				num_of_normal_buckets: new_normal,
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
