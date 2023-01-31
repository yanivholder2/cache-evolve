use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use crate::dash::bucket::Bucket;

use crate::dash::data::Data;
use crate::dash::segment::Segment;
use crate::dash::utils::{get_index, hash};
use crate::dash_settings::DashSettings;

#[cfg(test)]
mod tests;
mod segment;
mod bucket;
mod data;
mod utils;

#[derive(Debug)]
pub struct Dash<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub settings: DashSettings,
    pub segments: Vec<Segment<K, V>>,
}

impl<K, V> Dash<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub fn new(settings: DashSettings) -> Self {
        // TODO: think about maybe using Vec::with_capacity
        let mut segments = Vec::new();
        for _ in 0..settings.dash_size {
            segments.push(Segment::new(settings.segment_size, settings.bucket_size, settings.clone()));
        }
        Self { settings, segments }
    }

    pub fn put(&mut self, key: K, value: V) {
        let bucket = self.get_bucket_mut(&key);
        bucket.insert(key, value);
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let data = self.get_data(&key);
        match data {
            Some(data) => Some(&data.value),
            None => None,
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        let data = self.get_data(&key);
        match data {
            Some(d) => d.key == *key,
            None => false,
        }
    }

    fn get_data(&self, key: &K) -> Option<&Data<K, V>> {
        let bucket = self.get_bucket(&key);
        return bucket.get(key);
    }

    fn get_bucket(&self, key: &K) -> &Bucket<K, V> {
        let hash = hash(&key);
        let segment_index = get_index(hash, self.settings.dash_size);
        let bucket_index = get_index(hash, self.settings.segment_size);
        return &self.segments[segment_index].buckets[bucket_index];
    }

    // TODO: think about a better way to combine this logic with get_bucket
    fn get_bucket_mut(&mut self, key: &K) -> &mut Bucket<K, V> {
        let hash = hash(&key);
        let segment_index = get_index(hash, self.settings.dash_size);
        let bucket_index = get_index(hash, self.settings.segment_size);
        return &mut self.segments[segment_index].buckets[bucket_index];
    }
}