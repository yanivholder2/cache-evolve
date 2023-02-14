#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    LRU,
    LIFO,
    LFU,
    FIFO,
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
    segment_size: 3,
    stash_size: 1,
    bucket_size: 128,
    eviction_policy: EvictionPolicy::LRU,
    debug_mode: 0,
};
