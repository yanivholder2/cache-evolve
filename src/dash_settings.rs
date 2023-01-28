#[derive(Debug)]
pub enum EvictionPolicy {
    MoveAhead,
    LRU,
    LIFO,
    LFU,
    LFUDisp,
    FIFO
}

#[derive(Debug)]
pub struct DashSettings {
    pub dash_size: usize,
    pub segment_size: usize,
    pub bucket_size: usize,
    pub eviction_policy: EvictionPolicy,
    pub debug_mode: u8
}