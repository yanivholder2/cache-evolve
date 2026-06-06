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

/// Aggressively reshaped geometry: allocates more normal buckets and further reduces stash size.
/// This variant reshapes the bucket distribution to split at 62.5%/37.5% between
/// normal and stash buckets, pushing the optimization from v002's 50/50 split even further.
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
		// Reshape to allocate even more normal buckets and further reduce stash size.
		// The baseline allocates 8 normal + 24 stash = 32 buckets (25%/75%).
		// v002 reshaped to 16 normal + 16 stash = 32 buckets (50%/50%).
		// This variant further reshapes to 20 normal + 12 stash = 32 buckets (62.5%/37.5%).
		// For any given total bucket count, we allocate 5/8 to normal and the remainder to stash,
		// while preserving the bucket_size and segment count from settings.
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;
		let num_of_normal_buckets = (total_buckets * 5) / 8;  // 62.5% to normal buckets
		let num_of_stash_buckets = total_buckets - num_of_normal_buckets;  // Remainder to stash

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
