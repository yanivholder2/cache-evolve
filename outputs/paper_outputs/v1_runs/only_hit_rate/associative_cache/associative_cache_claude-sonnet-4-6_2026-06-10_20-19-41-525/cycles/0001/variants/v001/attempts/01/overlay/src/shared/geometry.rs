//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! **v001 hypothesis**: divide bucket count by 4, multiply bucket size by 4.
//! With the standard 128 × 16 = 2048 configuration this yields 32 × 64 = 2048,
//! preserving total capacity while giving each bucket's LRU a 4× larger eviction
//! pool, which should raise hit rate on LIRS workload-a.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Wider-bucket geometry: reduces bucket count by `factor` and widens each
/// bucket by `factor`, keeping total capacity constant.
#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Seed a geometry from the harness-provided settings, applying a factor-4
    /// repartition: fewer, wider buckets with the same aggregate capacity.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        let factor = 4;
        BaselineGeometry {
            num_of_buckets: (settings.num_of_buckets / factor).max(1),
            bucket_size: settings.bucket_size * factor,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    fn layout(&self, _total_capacity: usize) -> (usize, usize) {
        (self.num_of_buckets, self.bucket_size)
    }
}
