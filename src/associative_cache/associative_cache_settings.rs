use crate::shared::settings::EvictionPolicy;

#[derive(Debug, Clone)]
pub struct AssociativeCacheSettings {
	pub size: usize,
	pub segment_size: usize,
	pub bucket_size: usize,
	pub eviction_policy: EvictionPolicy,
	pub debug_mode: u8,
}

pub const DEFAULT_SETTINGS: AssociativeCacheSettings = AssociativeCacheSettings {
	size: 1,
	segment_size: 1,
	bucket_size: 512,
	eviction_policy: EvictionPolicy::Lru,
	debug_mode: 0,
};
