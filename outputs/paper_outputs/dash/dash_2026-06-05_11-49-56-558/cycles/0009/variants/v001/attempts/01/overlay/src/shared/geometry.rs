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
		// Baseline harness ships 8 normal + 24 stash = 32 total buckets.
		// Rebalance to a 15/16 split so the hot tier (target + probing) holds
		// fifteen sixteenths of the slots, keeping total capacity constant.
		//
		// For the production settings (32 total buckets): 30 normal + 2 stash,
		// compared to cycle/0008/v001's 28/4 (7/8 split). This tests whether
		// expanding the hot tier further continues to improve hit-rate at the
		// cost of a very shallow stash (2 slots).
		//
		// Edge-case guard: when total_buckets is odd or very small (proptest
		// exercises settings as small as normal=2, stash=1 → total=3), integer
		// division can yield normal=1 which fails the `num_of_normal_buckets >= 2`
		// validity check. Clamp normal to 2 so the constraint is always satisfied.
		// Clamp stash to 1 to ensure at least one stash slot exists.
		//
		// Example: total=3 → normal = max(2, 2) = 2, stash = max(1, 1) = 1. Valid.
		let total_buckets =
			self.topology.num_of_normal_buckets + self.topology.num_of_stash_buckets;
		let normal = (total_buckets * 15 / 16).max(2); // >= 2 required for probing
		let stash = (total_buckets - normal).max(1);   // >= 1 required for stash
		DashGeometry {
			num_of_segments:       self.topology.num_of_segments,
			num_of_normal_buckets: normal,
			num_of_stash_buckets:  stash,
			bucket_size:           self.topology.bucket_size,
		}
	}
}
