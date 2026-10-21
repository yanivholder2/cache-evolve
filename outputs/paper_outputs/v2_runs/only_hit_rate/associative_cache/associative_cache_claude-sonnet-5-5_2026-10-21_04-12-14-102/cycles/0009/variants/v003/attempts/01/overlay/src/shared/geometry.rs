//! Geometry: 4x wider buckets (ways x4, bucket count /4), same total capacity.

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
		const WIDEN: usize = 4;
		if self.num_of_buckets % WIDEN == 0 && self.num_of_buckets >= WIDEN {
			(self.num_of_buckets / WIDEN, self.bucket_size * WIDEN)
		} else {
			(self.num_of_buckets, self.bucket_size)
		}
	}
}
