//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! This variant applies a factor-4 wide-bucket repartition: divide the harness
//! `num_of_buckets` by 4 and multiply `bucket_size` by 4, preserving the same
//! total capacity. At the standard harness sweep point (128 × 16 = 2048) this
//! yields 32 × 64.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Factor-4 wide-bucket geometry: total capacity preserved, but 4× fewer and
/// 4× wider buckets than the harness-supplied split.
#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Seed a geometry from the harness-provided settings, applying the factor-4
    /// repartition. The construction glue and the invariant gate both build the
    /// geometry this way, so a variant that overlays this file inherits the same
    /// entry point.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        let total = settings.num_of_buckets * settings.bucket_size;
        let target_num_buckets = (settings.num_of_buckets / 4).max(1);
        // Capacity-conservation guard: total must divide evenly into the new
        // bucket count so no capacity is lost or invented.
        assert_eq!(
            total % target_num_buckets,
            0,
            "total capacity {total} must be divisible by target_num_buckets {target_num_buckets}"
        );
        let target_bucket_size = total / target_num_buckets;
        BaselineGeometry {
            num_of_buckets: target_num_buckets,
            bucket_size: target_bucket_size,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    fn layout(&self, _total_capacity: usize) -> (usize, usize) {
        (self.num_of_buckets, self.bucket_size)
    }
}
