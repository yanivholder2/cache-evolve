//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfSegments`, `numOfNormalBuckets`, `numOfStashBuckets`, `bucketSize`),
//! so the baseline topology is unchanged. A variant overlays this file to
//! repartition the *same* total capacity differently — for example a wider
//! stash, fewer-but-wider normal buckets, or more segments — while keeping the
//! `BaselineGeometry` type name the construction glue references.
//!
//! This variant (H03) shrinks the probing tier by 128 slots and gives them to the
//! stash: two normal buckets (of `bucket_size` slots each, 64 at the harness
//! default) move to the stash, so the budget `(normal + stash) * bucket_size`
//! is unchanged.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Slots moved from the probing tier (normal buckets) to the stash.
const PROBING_SHIFT_SLOTS: usize = 128;

/// Identity-budget geometry: returns the harness-provided split with 128 slots
/// shifted from the normal (probing) tier to the stash.
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
		// Whole buckets moved: 128 slots / bucket_size (2 at bucket_size 64).
		let shift = (PROBING_SHIFT_SLOTS / base.bucket_size.max(1)).max(1);
		// Only shift when the normal tier keeps the >= 2 buckets probing requires;
		// otherwise fall back to the unchanged split.
		if base.num_of_normal_buckets >= shift + 2 {
			DashGeometry {
				num_of_normal_buckets: base.num_of_normal_buckets - shift,
				num_of_stash_buckets: base.num_of_stash_buckets + shift,
				..base
			}
		} else {
			base
		}
	}
}
