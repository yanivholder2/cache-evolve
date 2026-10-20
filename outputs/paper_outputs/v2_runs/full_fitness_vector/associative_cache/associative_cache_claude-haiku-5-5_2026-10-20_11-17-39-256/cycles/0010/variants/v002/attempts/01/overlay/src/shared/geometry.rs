//! Bucket geometry: narrow-bucket variant (H02).
//!
//! The total capacity budget is factored into fixed 16-slot buckets, so the
//! bucket count is `budget / 16`. A budget that is not a multiple of 16 (or whose
//! bucket count is not a power of two) falls back to the harness-provided split,
//! which still conserves the budget. The `BaselineGeometry` type name and
//! `for_settings` entry point are kept because the construction glue and the
//! invariant gate reference them.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Slots per bucket for this variant.
const BUCKET_SLOTS: usize = 16;

#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Seed a geometry from the harness-provided settings. The construction glue
    /// and the invariant gate both build the geometry this way.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        BaselineGeometry {
            num_of_buckets: settings.num_of_buckets,
            bucket_size: settings.bucket_size,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    fn layout(&self, total_capacity: usize) -> (usize, usize) {
        if total_capacity >= BUCKET_SLOTS && total_capacity % BUCKET_SLOTS == 0 {
            let buckets = total_capacity / BUCKET_SLOTS;
            if buckets.is_power_of_two() {
                return (buckets, BUCKET_SLOTS);
            }
        }
        (self.num_of_buckets, self.bucket_size)
    }
}
