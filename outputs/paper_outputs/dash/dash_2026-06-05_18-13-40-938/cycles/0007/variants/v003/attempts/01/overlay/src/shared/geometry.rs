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

/// Rebalanced geometry: shifts capacity toward the stash tier.
/// Expands stash buckets by 2 and reduces normal buckets by 2 to extend the
/// filtering window while maintaining total bucket count and ensuring normal
/// buckets stay >= 2 (probing invariant).
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings, rebalancing bucket
	/// allocation toward the stash tier. The construction glue and the invariant
	/// gate both build the geometry this way, so a variant that overlays this
	/// file inherits the same entry point.
	pub fn for_settings(settings: &DashSettings) -> Self {
		// Rebalance: shift 2 buckets from normal to stash.
		// Only apply if the adjustment maintains the invariant num_of_normal_buckets >= 2.
		let (num_of_stash_buckets, num_of_normal_buckets) = if settings.num_of_normal_buckets >= 4 {
			// Safe to subtract 2 from normal and add 2 to stash
			(settings.num_of_stash_buckets + 2, settings.num_of_normal_buckets - 2)
		} else {
			// Keep original if adjustment would violate the invariant
			(settings.num_of_stash_buckets, settings.num_of_normal_buckets)
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
