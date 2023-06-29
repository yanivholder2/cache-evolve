#[derive(Debug, Clone)]
pub enum EvictionPolicy {
	Lru,
	Lifo,
	Lfu,
	Fifo,
}

#[derive(Debug, Clone)]
pub struct DashSettings {
	pub dash_size: usize,
	pub segment_size: usize,
	pub stash_size: usize,
	pub bucket_size: usize,
	pub eviction_policy: EvictionPolicy,
	pub debug_mode: u8,
}

pub const DEFAULT_SETTINGS: DashSettings = DashSettings {
	dash_size: 1,
	segment_size: 28,
	stash_size: 4,
	bucket_size: 16,
	eviction_policy: EvictionPolicy::Lru,
	debug_mode: 0,
};

pub const DRAGON_SETTINGS: DashSettings = DashSettings {
	dash_size: 1,
	segment_size: 56,
	stash_size: 4,
	bucket_size: 14,
	eviction_policy: EvictionPolicy::Lru,
	debug_mode: 0,
};
