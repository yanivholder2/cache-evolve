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
	/// buckets proportionally to maintain total capacity budget (add 2 stash,
	/// subtract 2 normal).
	///
	/// For example: settings with 28 normal + 4 stash → 26 normal + 6 stash
	/// (total buckets unchanged, total slots unchanged: 32 buckets → 32 buckets).
	///
	/// The transformation applies only when it maintains structural validity
	/// (num_of_normal_buckets >= 2 for probing). For edge-case settings where
	/// subtracting 2 from normal_buckets would violate this constraint, the
	/// settings pass through unchanged to preserve invariant compliance.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let new_stash = settings.num_of_stash_buckets.saturating_add(2);
		let new_normal = settings.num_of_normal_buckets.saturating_sub(2);

		// Apply the transformation only if it maintains validity
		// (num_of_normal_buckets >= 2 is required for probing to work).
		let (final_stash, final_normal) = if new_normal >= 2 {
			(new_stash, new_normal)
		} else {
			// Transformation would violate invariants; pass through unchanged
			(settings.num_of_stash_buckets, settings.num_of_normal_buckets)
		};

		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_stash_buckets: final_stash,
				num_of_normal_buckets: final_normal,
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
