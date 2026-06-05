//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfSegments`, `numOfNormalBuckets`, `numOfStashBuckets`, `bucketSize`),
//! so the baseline topology is unchanged. A variant overlays this file to
//! repartition the *same* total capacity differently — for example a wider
//! stash, fewer-but-wider normal buckets, or more segments — while keeping the
//! `BaselineGeometry` type name the construction glue references.
//!
//! This variant (H01) inherits the cycle-0008 normal-dominant 3:1 split but,
//! before applying it, trades **depth for addressability**: it halves the
//! per-bucket slot count and doubles the bucket count under a conserved budget.
//! Narrower-but-more-numerous buckets spread colliding hot keys across more
//! addressable target buckets, attacking the small-bucket occupancy ceiling the
//! winner saturated against ("addressability beats depth"; the reverse trade has
//! been tested and regressed).
//!
//! Concretely:
//!   1. recover the seeded capacity `cap = segments * (normal + stash) * bucket_size`
//!      from `self.topology` (the `total_capacity` arg is ignored on purpose);
//!   2. choose `bucket_size' = max(1, bucket_size / 2)` and re-derive the per-segment
//!      bucket count as `total_buckets' = total_buckets * (bucket_size / bucket_size')`;
//!   3. apply the inherited 3:1 normal/stash clamp to `total_buckets'`.
//!
//! Halving is only exact for an even `bucket_size >= 2`. When `bucket_size` is
//! odd or 1 (or any drift would leave a remainder), the rewrite is abandoned and
//! the seeded `DashGeometry` is returned verbatim, so
//! `num_of_segments * (normal + stash) * bucket_size'` always equals the seeded
//! total capacity exactly and `check_conservation` / `P6 capacity_conservation`
//! still pass.
//!
//! The split is clamped to keep `num_of_normal_buckets >= 2` (required for
//! probing) and `num_of_stash_buckets >= 1`. After doubling, `total_buckets'`
//! is at least `2 * 3 == 6`, so the clamp range `[2, total_buckets' - 1]` is
//! always valid.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Addressability-trading geometry: conserves the budget but halves bucket width
/// and doubles bucket count, then applies the inherited normal-dominant 3:1 split.
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
		// Recover the seeded budget from the topology itself; do not trust the arg.
		let segments = self.topology.num_of_segments;
		let total_buckets =
			self.topology.num_of_normal_buckets + self.topology.num_of_stash_buckets;
		let bucket_size = self.topology.bucket_size;
		let cap = segments * total_buckets * bucket_size;

		// Halving is only exact for an even bucket_size >= 2. For an odd or unit
		// bucket_size, abandon the rewrite and return the seeded geometry verbatim
		// so capacity conservation cannot drift.
		if bucket_size < 2 || bucket_size % 2 != 0 {
			return self.topology;
		}

		// Trade depth for addressability: halve the bucket width, double the count.
		let bucket_size_new = bucket_size / 2; // == max(1, bucket_size / 2) since bucket_size >= 2
		let ratio = bucket_size / bucket_size_new; // == 2 for an even bucket_size
		let total_buckets_new = total_buckets * ratio;

		// Capacity-conservation guard: bail to the seeded geometry on any drift so
		// P6 capacity_conservation cannot fail.
		if segments * total_buckets_new * bucket_size_new != cap {
			return self.topology;
		}

		// Inherited normal-dominant 3:1 split, clamped to keep the topology valid:
		//   num_of_normal_buckets >= 2 (probing requires >= 2 normal buckets)
		//   num_of_stash_buckets  >= 1 (stash must keep at least one bucket)
		// total_buckets_new >= 2 * 3 == 6, so [2, total_buckets_new - 1] is valid.
		let num_of_normal_buckets = (total_buckets_new * 3 / 4).clamp(2, total_buckets_new - 1);
		let num_of_stash_buckets = total_buckets_new - num_of_normal_buckets;

		DashGeometry {
			num_of_segments: segments,
			num_of_normal_buckets,
			num_of_stash_buckets,
			bucket_size: bucket_size_new,
		}
	}
}
