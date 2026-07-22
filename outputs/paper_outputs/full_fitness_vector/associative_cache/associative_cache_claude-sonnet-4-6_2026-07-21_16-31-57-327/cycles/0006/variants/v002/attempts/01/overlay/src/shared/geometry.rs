//! In-crate bucket-geometry implementation — the overlayable extension point.
//!
//! `BaselineGeometry` honours the split the harness chose (the swept
//! `numOfBuckets` and the derived `bucket_size`), so the baseline topology is
//! unchanged. A variant overlays this file to repartition the *same* total
//! capacity differently — for example fewer, wider buckets to escape the
//! narrow-bucket regime — while keeping the `BaselineGeometry` type name the
//! construction glue references.
//!
//! H02: Halve the number of buckets and double the bucket size. Each
//! hash-collision class can now hold a larger working-set slice, giving popular
//! items more breathing room before being evicted and raising hit rate under
//! Zipfian access patterns.

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Wider-bucket geometry: half as many buckets, each twice as deep.
///
/// Total capacity is conserved: `new_num * new_bsz == original_num * original_bsz`.
#[derive(Debug)]
pub struct BaselineGeometry {
    pub num_of_buckets: usize,
    pub bucket_size: usize,
}

impl BaselineGeometry {
    /// Seed a geometry from the harness-provided settings, halving the bucket
    /// count and doubling the slot depth so that total capacity is preserved.
    ///
    /// Integer division is safe: if `num_of_buckets` is odd, the `.max(1)` guard
    /// keeps `new_num` non-zero, and `total / new_num` absorbs any slack into
    /// `bucket_size`. The factory's capacity-conservation check (`new_num *
    /// new_bsz == budget`) will validate correctness at build time.
    pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
        // Halve the number of buckets, double the bucket size.
        let new_num = (settings.num_of_buckets / 2).max(1);
        let total = settings.num_of_buckets * settings.bucket_size;
        let new_bsz = total / new_num;
        BaselineGeometry {
            num_of_buckets: new_num,
            bucket_size: new_bsz,
        }
    }
}

impl BucketGeometry for BaselineGeometry {
    fn layout(&self, _total_capacity: usize) -> (usize, usize) {
        (self.num_of_buckets, self.bucket_size)
    }
}
