//! Bucket-geometry overlay — factor-2 repartition on top of v003's
//! cold-first eviction + midpoint-insertion combination.
//!
//! The baseline 128 × 16 = 2048 configuration is repartitioned to
//! 64 × 32, halving the bucket count and doubling each bucket's
//! capacity while preserving total capacity exactly. This tests whether
//! the narrow-path (mid-point insertion + cold-first eviction) gains as
//! much from modest geometry widening as the pure-LRU path did in cycle 0001.
//!
//! The `BaselineGeometry` type name is fixed — the protected construction glue
//! (`src/factory.rs`) calls `BaselineGeometry::for_settings(settings).layout(budget)`.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Factor-2 geometry: halves the bucket count, doubles per-bucket capacity.
#[derive(Debug)]
pub struct BaselineGeometry {
    /// Original bucket count from settings (fallback if division is uneven).
    pub num_of_buckets: usize,
    /// Original bucket size from settings (fallback if division is uneven).
    pub bucket_size: usize,
    /// Repartitioned target bucket count (settings.num_of_buckets / 2, clamped to ≥ 1).
    pub target_num_buckets: usize,
}

impl BaselineGeometry {
    /// Build geometry from the harness-provided settings, applying the factor-2
    /// repartition. The construction glue and the invariant gate both call this.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        let factor = 2usize;
        let target_num_buckets = (settings.num_of_buckets / factor).max(1);
        BaselineGeometry {
            num_of_buckets: settings.num_of_buckets,
            bucket_size: settings.bucket_size,
            target_num_buckets,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    /// Factor `total_capacity` into `(num_of_buckets, bucket_size)`.
    ///
    /// When `total_capacity` divides evenly by `target_num_buckets` the
    /// repartitioned layout is used (fewer, wider buckets). Otherwise the
    /// original harness-provided split is returned unchanged, preserving the
    /// capacity-conservation invariant in all cases.
    fn layout(&self, total_capacity: usize) -> (usize, usize) {
        if total_capacity % self.target_num_buckets == 0 {
            (self.target_num_buckets, total_capacity / self.target_num_buckets)
        } else {
            (self.num_of_buckets, self.bucket_size)
        }
    }
}
