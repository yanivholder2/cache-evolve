//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfSegments`, `numOfNormalBuckets`, `numOfStashBuckets`, `bucketSize`),
//! then moves a fixed slot budget from the stash into the probing (normal) tier.
//! The total capacity is unchanged: the normal tier gains exactly the slots the
//! stash loses, so `segments * (normal + stash) * bucket_size` is conserved.
//!
//! The shift is expressed in slots (`PROBING_SHIFT_SLOTS`) and converted to whole
//! buckets. The stash always keeps at least one bucket so the topology stays valid
//! (e.g. the tiny invariant-suite settings, which have only two stash buckets).

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Slots moved from the stash tier to the probing (normal) tier.
const PROBING_SHIFT_SLOTS: usize = 256;

/// Geometry that shifts `PROBING_SHIFT_SLOTS` of stash capacity into normal buckets.
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
		let base = self.topology;
		// Whole buckets moved from stash to normal; never drain the stash below one bucket.
		let shift = (PROBING_SHIFT_SLOTS / base.bucket_size.max(1)).min(base.num_of_stash_buckets.saturating_sub(1));
		DashGeometry {
			num_of_segments: base.num_of_segments,
			num_of_normal_buckets: base.num_of_normal_buckets + shift,
			num_of_stash_buckets: base.num_of_stash_buckets - shift,
			bucket_size: base.bucket_size,
		}
	}
}
