use crate::eviction_policy::EvictionPolicy;

#[derive(Debug, Clone)]
pub struct AssociativeCacheSettings {
	pub num_of_buckets: usize,
	pub bucket_size: usize,
	pub eviction_policy: EvictionPolicy,
}

impl Default for AssociativeCacheSettings {
	fn default() -> Self {
		AssociativeCacheSettings {
			num_of_buckets: 1,
			bucket_size: 8,
			eviction_policy: EvictionPolicy::ClassicLRU,
		}
	}
}
