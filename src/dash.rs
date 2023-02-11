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
            // TODO: pass the settings as a reference
            segments.push(Segment::new(settings.segment_size, settings.bucket_size, settings.clone()));
        }
        Self { settings, segments }
    }

    pub fn put(&mut self, key: K, value: V) {
        let segment = self.get_mut_segment(&key);
        segment.insert(key, value);
    }

    pub fn get(&mut self, key: &K) -> Option<&V> {
        let data = self.get_data(&key);
        match data {
            Some(data) => Some(&data.value),
            None => None,
        }
    }

    fn get_data(&mut self, key: &K) -> Option<&Data<K, V>> {
        let segment = self.get_mut_segment(&key);
        segment.get(&key)
    }

    fn get_segment(&self, key: &K) -> &Segment<K, V> {
        let hash = hash(&key);
        let segment_len = self.segments.len();
        let segment_index = get_index(hash, segment_len);
        return &self.segments[segment_index];
    }

    fn get_mut_segment(&mut self, key: &K) -> &mut Segment<K, V> {
        let hash = hash(&key);
        let segment_len = self.segments.len();
        let segment_index = get_index(hash, segment_len);
        return &mut self.segments[segment_index];
    }
}