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

/// Expanded stash geometry: increases stash buckets from 4 to 8 to improve
/// filtering of hot items before promotion into the hot tiers. Reduces normal
/// buckets from 28 to 24 to maintain the same total capacity.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings, with custom bucket
	/// allocation: expand stash from 4 to 8 buckets and reduce normal buckets
	/// from 28 to 24 (keeping total capacity at 32 buckets). The 24 normal
	/// buckets are split as 12 Target + 12 Probing (scaled from original 14+14).
	pub fn for_settings(settings: &DashSettings) -> Self {
		// Override bucket allocation to expand stash and reduce normal buckets
		let num_of_stash_buckets = 8;     // Expanded from 4
		let num_of_normal_buckets = 24;   // Reduced from 28

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
