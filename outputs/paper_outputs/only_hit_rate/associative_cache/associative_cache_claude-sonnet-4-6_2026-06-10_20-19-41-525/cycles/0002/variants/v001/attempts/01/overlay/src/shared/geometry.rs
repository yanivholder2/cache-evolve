//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! **v002 hypothesis**: divide bucket count by 16, multiply bucket size by 16.
//! With the standard 128 × 16 = 2048 configuration this yields 8 × 256 = 2048,
//! preserving total capacity while giving each bucket's LRU a 16× larger eviction
//! pool (vs 4× in v001), which should further raise hit rate on LIRS workload-a
//! if the gain from the 4× widening is not yet saturated.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Wider-bucket geometry: reduces bucket count by `factor` and widens each
/// bucket by `factor`, keeping total capacity exactly constant.
///
/// The repartition is applied only when integer division can be done without
/// remainder (i.e. `total % target_num_buckets == 0`); otherwise the identity
/// split is preserved so capacity conservation holds for all valid settings.
#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Seed a geometry from the harness-provided settings, applying a factor-16
    /// repartition when the total capacity is exactly divisible by the target
    /// bucket count.  Falls back to the identity split when integer division
    /// would lose capacity.
    ///
    /// # Capacity conservation
    /// For all valid settings `s`: `new_num_of_buckets * new_bucket_size ==
    /// s.num_of_buckets * s.bucket_size`.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        let total = settings.num_of_buckets * settings.bucket_size;
        let factor = 16usize;
        let target_num_buckets = (settings.num_of_buckets / factor).max(1);

        // Only repartition when the total capacity is exactly divisible by the
        // new bucket count; otherwise fall back to the identity split so the
        // capacity-conservation invariant holds for all proptest inputs.
        let (num_of_buckets, bucket_size) = if total % target_num_buckets == 0 {
            (target_num_buckets, total / target_num_buckets)
        } else {
            // Identity: capacity conservation holds trivially.
            (settings.num_of_buckets, settings.bucket_size)
        };

        BaselineGeometry {
            num_of_buckets,
            bucket_size,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    fn layout(&self, _total_capacity: usize) -> (usize, usize) {
        (self.num_of_buckets, self.bucket_size)
    }
}
