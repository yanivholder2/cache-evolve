use crate::dash::bucket::Bucket;
use crate::dash::data::Data;
use crate::dash::utils::{get_index, hash};
use crate::dash_settings::DashSettings;
use std::hash::Hash;

#[derive(Debug)]
pub struct Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub buckets: Vec<Bucket<K, V>>,
    pub segment_size: usize,
    pub stash_buckets: Vec<Bucket<K, V>>,
    pub stash_size: usize,
}

impl<K, V> Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub fn new(size: usize, bucket_size: usize, settings: DashSettings) -> Self {
        let mut buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..size {
            // TODO: pass the settings as a reference
            buckets.push(Bucket::new(bucket_size, settings.clone()));
        }

        let mut stash_buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..settings.stash_size {
            // TODO: pass the settings as a reference
            stash_buckets.push(Bucket::new(bucket_size, settings.clone()));
        }
        Segment {
            buckets,
            stash_buckets,
            segment_size: size,
            stash_size: settings.stash_size,
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&Data<K, V>> {
        let hash = hash(&key);
        let stash_bucket = &mut self.stash_buckets[get_index(hash, self.stash_size)];

        // The order assumes that the data is more likely to be in the stash bucket, this
        // assumption should be tested
        match stash_bucket.get(&key) {
            Some(data) => Some(data),
            None => {
                let bucket = &mut self.buckets[get_index(hash, self.segment_size)];
                match bucket.get(&key) {
                    Some(data) => Some(data),
                    None => None,
                }
            }
        }
    }

    /// Insert the key, value pair into the segment.
    /// If the key already exists in the segment, the value and position will be updated.
    pub fn insert(&mut self, key: K, val: V) {
        let hash = hash(&key);
        let stash_bucket = &mut self.stash_buckets[get_index(hash, self.stash_size)];
        let bucket = &mut self.buckets[get_index(hash, self.segment_size)];
        // The order assumes that the data is more likely to be in the stash bucket, this
        // assumption should be tested
        match stash_bucket.get(&key) {
            Some(data) => {
                if data.value != val {
                    // TODO: consider what to do if data.val != val
                }
                stash_bucket.remove(&key);
                bucket.insert(key, val);
            }
            None => match bucket.get(&key) {
                Some(_) => {
                    bucket.insert(key, val);
                    return;
                }
                None => {
                    stash_bucket.insert(key, val);
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dash_settings::{EvictionPolicy, DEFAULT_SETTINGS};

    #[test]
    fn test_segment_length() {
        let segment: Segment<i32, i32> = Segment::new(10, 10, DEFAULT_SETTINGS);
        assert_eq!(segment.buckets.len(), 10);
    }
}
