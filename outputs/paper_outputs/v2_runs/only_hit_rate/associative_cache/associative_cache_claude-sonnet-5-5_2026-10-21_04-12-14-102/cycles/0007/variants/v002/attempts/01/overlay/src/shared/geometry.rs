use crate::associative_cache::associative_cache_settings::AssociativeCacheSettings;
use crate::shared::traits::geometry::BucketGeometry;

/// Geometry that doubles associativity: halves bucket count, doubles bucket
/// size, preserving total capacity.
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
		if self.num_of_buckets > 1 && self.num_of_buckets % 2 == 0 {
			(self.num_of_buckets / 2, self.bucket_size * 2)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
