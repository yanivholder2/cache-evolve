//! Geometry variant: fewer, wider buckets (32 x 64) over the same capacity.

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

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, total_capacity: usize) -> (usize, usize) {
		let n = 32usize;
		if n >= 1 && total_capacity >= n && total_capacity % n == 0 {
			(n, total_capacity / n)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
