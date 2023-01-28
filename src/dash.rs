use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use crate::dash::segment::Segment;
use crate::dash::utils::get_index;
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
            segments.push(Segment::new(settings.segment_size, settings.bucket_size));
        }
        Self { settings, segments }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        let hash = self.hash(key);
        let segment_index = get_index(hash, self.settings.dash_size);
        let bucket_index = get_index(hash, self.settings.segment_size);
        let bucket = &self.segments[segment_index].buckets[bucket_index];
        let data = bucket.get(key);
        match data {
            Some(data) => Some(&data.value),
            None => None,
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        let hash = self.hash(&key);
        let segment_index = get_index(hash, self.settings.dash_size);
        let bucket_index = get_index(hash, self.settings.segment_size);
        let bucket = &mut self.segments[segment_index].buckets[bucket_index];
        let data = bucket.get(&key);
        if data.is_some() {
            // TODO: need to evict here
            return;
        }
        bucket.insert(key, value);
    }

    pub fn contains(&self, key: &K) -> bool {
        let hash = self.hash(key);
        let segment_index = get_index(hash, self.settings.dash_size);
        let bucket_index = get_index(hash, self.settings.segment_size);
        let bucket = &self.segments[segment_index].buckets[bucket_index];
        let data = bucket.get(key);
        match data {
            Some(d) => d.key == *key,
            None => false,
        }
    }

    fn hash(&self, key: &K) -> u64 {
        let mut hasher = DefaultHasher::new();
        key.hash(&mut hasher);
        hasher.finish()
    }
}