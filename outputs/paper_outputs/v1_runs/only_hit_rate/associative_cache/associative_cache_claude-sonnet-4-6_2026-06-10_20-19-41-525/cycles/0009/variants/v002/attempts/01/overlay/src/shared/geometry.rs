//! Bucket-geometry extension point — 4× wide-bucket repartition for SLRU.
//!
//! Applies the same factor-4 wide-bucket repartition used in prior winning
//! variants (cycle/0008/v003, cycle/0007/v003, cycle/0004/v001, …) to the
//! SLRU bucket implementation from cycle/0008/v002.
//!
//! For the standard 128×16 = 2048 configuration this yields 32 buckets of
//! 64 slots each, giving each SLRU bucket prob_cap = 32 / prot_cap = 32
//! instead of 8/8 — a significantly larger eviction pool per bucket.
//!
//! Invariant preserved: `num_of_buckets * bucket_size == settings.num_of_buckets
//! * settings.bucket_size` for all valid inputs (falls back to the original
//! split when the total is not divisible by the target bucket count).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Wide-bucket geometry: repartitions the fixed total capacity into ¼ as many
/// buckets, each 4× wider.
#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Build geometry from the harness-provided settings, applying a factor-4
    /// wide-bucket repartition when it divides cleanly, otherwise falling back
    /// to the original split.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        let total = settings.num_of_buckets * settings.bucket_size;
        let factor = 4usize;
        let target_num_buckets = (settings.num_of_buckets / factor).max(1);
        let (num_of_buckets, bucket_size) = if total % target_num_buckets == 0 {
            (target_num_buckets, total / target_num_buckets)
        } else {
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
