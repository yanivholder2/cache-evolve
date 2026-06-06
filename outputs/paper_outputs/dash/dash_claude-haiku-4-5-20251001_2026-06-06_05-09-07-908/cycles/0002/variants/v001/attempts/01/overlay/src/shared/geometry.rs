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

/// Reshaped geometry: allocates more normal buckets and reduces stash size.
/// This variant reshapes the bucket distribution to split equally between
/// normal and stash buckets, instead of the baseline's 25/75 split.
/// Maintains total capacity by respecting the provided settings.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way, so a variant that
	/// overlays this file inherits the same entry point.
	pub fn for_settings(settings: &DashSettings) -> Self {
		// Reshape to allocate more normal buckets and reduce stash size.
		// The baseline allocates 8 normal + 24 stash = 32 buckets (25%/75%).
		// This variant reshapes to 16 normal + 16 stash = 32 buckets (50%/50%).
		// For any given total bucket count, we split evenly while preserving
		// the bucket_size and segment count from settings.
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;
		let num_of_normal_buckets = (total_buckets + 1) / 2;  // Round up
		let num_of_stash_buckets = total_buckets / 2;         // Round down

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
