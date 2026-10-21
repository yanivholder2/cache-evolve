//! Geometry: double the ways per bucket, halve the bucket count (capacity conserved).

use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

#[derive(Debug)]
pub struct BaselineGeometry {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
}

impl BaselineGeometry {
	pub fn for_settings(settings: &AssociativeCacheSettings) -> Self {
		if settings.num_of_buckets % 2 == 0 && settings.num_of_buckets >= 2 {
			BaselineGeometry {
				num_of_buckets: settings.num_of_buckets / 2,
				bucket_size: settings.bucket_size * 2,
			}
		} else {
			BaselineGeometry {
				num_of_buckets: settings.num_of_buckets,
				bucket_size: settings.bucket_size,
			}
		}
	}
}

impl BucketGeometry for BaselineGeometry {
	fn layout(&self, _total_capacity: usize) -> (usize, usize) {
		(self.num_of_buckets, self.bucket_size)
	}
}
