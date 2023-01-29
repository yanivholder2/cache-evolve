use std::hash::Hash;
use crate::dash::bucket::Bucket;
use crate::dash_settings::DashSettings;

#[derive(Debug)]
pub struct Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub buckets: Vec<Bucket<K, V>>,
    pub size: usize,
    // TODO: make this a reference with a lifetime
    settings: DashSettings
}

impl<K, V> Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone
{
    pub fn new(size: usize, bucket_size: usize, settings: DashSettings) -> Self {
        let mut buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..size {
            buckets.push(Bucket::new(bucket_size, settings.clone()));
        }
        Segment {
            buckets,
            size,
            settings
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::dash_settings::EvictionPolicy;
    use super::*;

    const SETTINGS: DashSettings = DashSettings {
        dash_size: 1,
        segment_size: 1,
        bucket_size: 100,
        eviction_policy: EvictionPolicy::LRU,
        debug_mode: 0,
    };

    #[test]
    fn test_segment_length() {
        let segment: Segment<i32, i32> = Segment::new(10, 10, SETTINGS);
        assert_eq!(segment.buckets.len(), 10);
    }
}





