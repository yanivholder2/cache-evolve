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

/// Proportional rebalancing of stash and normal buckets while preserving
/// total capacity: expands stash to 25% of bucket count (from 12.5%) and
/// reduces normal buckets to 75% (from 87.5%). This widens the stash tier
/// for better characterization of hot items before promotion into competitive
/// hot tiers.
#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Rebalance the topology to expand stash and reduce normal buckets while
	/// preserving total capacity. The new ratio is 25% stash : 75% normal,
	/// applied proportionally to whatever bucket count the settings provide.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let total_buckets = settings.num_of_normal_buckets + settings.num_of_stash_buckets;

		// Apply the new ratio: stash should be 25% of total buckets.
		// The original baseline is 4 stash out of 32 = 12.5%.
		// The new ratio is 8 stash out of 32 = 25%.
		// We apply this ratio proportionally to any input budget.
		let new_stash_buckets = (total_buckets as f64 * 0.25).round() as usize;
		let new_stash_buckets = new_stash_buckets.max(1); // Ensure at least 1 stash bucket

		let new_normal_buckets = total_buckets - new_stash_buckets;
		// Ensure normal buckets meet the minimum (>= 2 for probing), if possible
		let new_normal_buckets = if new_normal_buckets < 2 && total_buckets >= 3 {
			// If normal is below minimum, give one less to stash (if possible)
			total_buckets - 2
		} else {
			new_normal_buckets
		};

		let final_stash_buckets = total_buckets - new_normal_buckets;

		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_normal_buckets: new_normal_buckets,
				num_of_stash_buckets: final_stash_buckets,
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
