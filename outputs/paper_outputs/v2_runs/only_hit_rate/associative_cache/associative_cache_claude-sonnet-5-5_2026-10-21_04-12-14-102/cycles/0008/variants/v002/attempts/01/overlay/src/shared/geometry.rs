//! Geometry: quadruples bucket width while keeping total capacity constant.

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
	fn layout(&self, _total_capacity: usize) -> (usize, usize) {
		if self.num_of_buckets >= 4 && self.num_of_buckets % 4 == 0 {
			(self.num_of_buckets / 4, self.bucket_size * 4)
		} else if self.num_of_buckets >= 2 && self.num_of_buckets % 2 == 0 {
			(self.num_of_buckets / 2, self.bucket_size * 2)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
