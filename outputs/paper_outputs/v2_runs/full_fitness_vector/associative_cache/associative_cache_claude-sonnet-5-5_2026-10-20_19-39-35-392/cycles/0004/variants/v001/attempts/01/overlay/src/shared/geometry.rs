//! Geometry variant: fewer, wider buckets (target 32 buckets).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

#[derive(Debug)]
pub struct BaselineGeometry {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
}

impl BaselineGeometry {
	pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
		BaselineGeometry {
			num_of_buckets: settings.num_of_buckets,
			bucket_size: settings.bucket_size,
		}
	}
}

const TARGET_BUCKETS: usize = 32;

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		if total_capacity < TARGET_BUCKETS {
			return (1, total_capacity);
		}
		// Nearest divisor of total_capacity to TARGET_BUCKETS.
		let mut n = TARGET_BUCKETS;
		for d in 0..total_capacity {
			let lo = TARGET_BUCKETS.saturating_sub(d);
			if lo >= 1 && total_capacity % lo == 0 {
				n = lo;
				break;
			}
			let hi = TARGET_BUCKETS + d;
			if hi <= total_capacity && total_capacity % hi == 0 {
				n = hi;
				break;
			}
		}
		(n, total_capacity / n)
	}
}
