//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! H03 variant: shifts one normal bucket's worth of slots per segment into the
//! stash, at a constant total slot count. With the default eval topology
//! (8 normal / 24 stash / 64 slots) this yields 7 normal / 25 stash / 64 slots:
//! same bucket width, so the moved capacity is exactly one more stash bucket.
//!
//! Normal-bucket, probing and stash indices are all derived with `hash % n`
//! (`shared::utils::get_index`) and `(target + 1) % n`, so a non-power-of-two
//! normal-bucket count is handled correctly. The ghost table is sized from the
//! segment's total capacity, which is unchanged.
//!
//! The shift is only applied when at least 3 normal buckets exist, so the
//! result always keeps the >= 2 normal buckets probing requires; otherwise the
//! harness split is returned unchanged.

use crate::dash::dash_settings::DashSettings;
use crate::shared::traits::geometry::{BucketGeometry, DashGeometry};

/// Number of normal buckets per segment moved into the stash.
const NORMAL_TO_STASH_SHIFT: usize = 1;

#[derive(Debug)]
pub struct BaselineGeometry {
	topology: DashGeometry,
}

impl BaselineGeometry {
	/// Seed a geometry from the harness-provided settings. The construction glue
	/// and the invariant gate both build the geometry this way.
	pub fn for_settings(settings: &DashSettings) -> Self {
		let mut normal = settings.num_of_normal_buckets;
		let mut stash = settings.num_of_stash_buckets;
		if normal >= 2 + NORMAL_TO_STASH_SHIFT {
			normal -= NORMAL_TO_STASH_SHIFT;
			stash += NORMAL_TO_STASH_SHIFT;
		}
		BaselineGeometry {
			topology: DashGeometry {
				num_of_segments: settings.num_of_segments,
				num_of_normal_buckets: normal,
				num_of_stash_buckets: stash,
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
