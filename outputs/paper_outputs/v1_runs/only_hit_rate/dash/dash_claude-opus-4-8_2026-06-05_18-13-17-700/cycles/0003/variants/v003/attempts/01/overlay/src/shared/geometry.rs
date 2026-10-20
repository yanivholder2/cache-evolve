//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! H03 variant: trade bucket *count* for bucket *width* while conserving the
//! budget. We double `bucket_size` and halve both bucket counts (at the seeded
//! ratio), deepening each hash bucket's LRU stack so hot keys colliding in a
//! target bucket survive longer. The normal/stash split *ratio* is preserved
//! (both counts halved), isolating per-bucket associativity from the split
//! sweep H01 performs separately.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Width-trading geometry: fewer-but-wider buckets at the seeded ratio.
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
	fn layout(&self, total_capacity: usize) -> DashGeometry {
		let seeded = self.topology;

		// Double the per-bucket width, halve the bucket counts (preserving the
		// seeded normal:stash ratio). Clamp to structural minima: probing needs
		// >= 2 normal buckets, the stash needs >= 1.
		let num_of_normal_buckets = (seeded.num_of_normal_buckets / 2).max(2);
		let num_of_stash_buckets = (seeded.num_of_stash_buckets / 2).max(1);
		let bucket_size = seeded.bucket_size.saturating_mul(2);

		let candidate = DashGeometry {
			num_of_segments: seeded.num_of_segments,
			num_of_normal_buckets,
			num_of_stash_buckets,
			bucket_size,
		};

		// Conserve the budget exactly: if halving lost capacity (odd counts) or a
		// clamp kicked in, the reshaped capacity won't equal the budget — fall
		// back to the seeded topology so P6 capacity_conservation never fails.
		let candidate_capacity = candidate
			.num_of_segments
			.saturating_mul(candidate.num_of_normal_buckets + candidate.num_of_stash_buckets)
			.saturating_mul(candidate.bucket_size);

		if candidate_capacity == total_capacity
			&& candidate.num_of_segments > 0
			&& candidate.bucket_size > 0
			&& candidate.num_of_normal_buckets >= 2
			&& candidate.num_of_stash_buckets >= 1
		{
			candidate
		} else {
			seeded
		}
	}
}
