//! Geometry: octuples bucket width while keeping total capacity constant.

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
		for f in [8usize, 4, 2] {
			if self.num_of_buckets >= f && self.num_of_buckets % f == 0 {
				return (self.num_of_buckets / f, self.bucket_size * f);
			}
		}
		(self.num_of_buckets, self.bucket_size)
	}
}
