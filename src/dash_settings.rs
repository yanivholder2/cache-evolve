#[derive(Debug, Clone)]
pub enum EvictionPolicy {
    LRU,
    LIFO,
    LFU,
    FIFO
}

#[derive(Debug, Clone)]
pub struct DashSettings {
    pub dash_size: usize,
    pub segment_size: usize,
    pub bucket_size: usize,
    pub eviction_policy: EvictionPolicy,
    pub debug_mode: u8
}