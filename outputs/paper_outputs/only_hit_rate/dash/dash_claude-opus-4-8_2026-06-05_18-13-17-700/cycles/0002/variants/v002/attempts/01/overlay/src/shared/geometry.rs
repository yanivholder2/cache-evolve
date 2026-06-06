//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfSegments`, `numOfNormalBuckets`, `numOfStashBuckets`, `bucketSize`),
//! so the baseline topology is unchanged. A variant overlays this file to
//! repartition the *same* total capacity differently — for example a wider
//! stash, fewer-but-wider normal buckets, or more segments — while keeping the
//! `BaselineGeometry` type name the construction glue references.
//!
//! This variant (H02) rebalances the normal/stash split toward more normal
//! buckets while conserving the total budget: instead of echoing the harness
//! split (8 normal / 24 stash), it splits the bucket budget evenly (16 / 16).
//! `num_of_segments` and `bucket_size` are left exactly as seeded, so
//! `num_of_segments * (normal + stash) * bucket_size` is unchanged and
//! `check_conservation` / `P6 capacity_conservation` still pass.
//!
//! The even split is clamped to keep `num_of_normal_buckets >= 2` (required for
//! probing) and `num_of_stash_buckets >= 1`, so even tiny topologies (e.g. the
//! proptest minimal `2 normal / 1 stash`) resolve to a valid topology.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Rebalancing geometry: conserves total buckets but shifts the normal/stash
/// split toward the promotion-protected normal tier.
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
		// Conserve the total bucket budget, but rebalance the normal/stash
		// split evenly so the promotion-protected normal tier gets more room
		// and target-bucket hash collisions drop.
		let total_buckets =
			self.topology.num_of_normal_buckets + self.topology.num_of_stash_buckets;

		// Even split, but clamp so the topology stays valid:
		//   num_of_normal_buckets >= 2 (probing requires >= 2 normal buckets)
		//   num_of_stash_buckets  >= 1 (stash must keep at least one bucket)
		// The seed already satisfies normal >= 2 && stash >= 1, so
		// total_buckets >= 3 and the clamp range [2, total_buckets - 1] is valid.
		let num_of_normal_buckets = (total_buckets / 2).clamp(2, total_buckets - 1);
		let num_of_stash_buckets = total_buckets - num_of_normal_buckets;

		DashGeometry {
			num_of_segments: self.topology.num_of_segments,
			num_of_normal_buckets,
			num_of_stash_buckets,
			bucket_size: self.topology.bucket_size,
		}
	}
}
