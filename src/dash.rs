use std::hash::Hash;

use crate::dash::segment::Segment;
use crate::dash::utils::{get_index, hash};
use crate::dash_settings::DashSettings;

mod bucket;
mod data;
mod segment;
#[cfg(test)]
mod tests;
mod utils;

#[derive(Debug)]
pub struct Dash<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    pub segments: Vec<Segment<K, V>>,
}

impl<K, V> Dash<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    pub fn new(settings: DashSettings) -> Self {
        // TODO: think about maybe using Vec::with_capacity
        let mut segments = Vec::new();
        for _ in 0..settings.dash_size {
            // TODO: pass the settings as a reference
            segments.push(Segment::new(settings.clone()));
        }
        Self { segments }
    }

    pub fn put(&mut self, key: K, value: V) {
        let segment = self.get_mut_segment(&key);
        segment.put(key, value);
    }

    pub fn get_and_update(&mut self, key: &K) -> Option<&V> {
        let segment = self.get_mut_segment(&key);
        let data = segment.get_and_update(&key)?;
        Some(&data.value)
    }

    fn get_mut_segment(&mut self, key: &K) -> &mut Segment<K, V> {
        let hash = hash(&key);
        let segment_index = get_index(hash, self.segments.len());
        return &mut self.segments[segment_index];
    }
}
