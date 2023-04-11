use std::fmt::{Display, Formatter};
use std::hash::Hash;

use crate::dash::segment::Segment;
use crate::dash::utils::{get_index, hash};
use crate::dash_settings::DashSettings;

mod bucket;
mod data;
mod segment;
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

impl<K, V> Display for Dash<K, V>
where
    K: Hash + Eq + Clone + Copy + Display,
    V: Eq + Clone + Copy + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for segment in &self.segments {
            writeln!(f, "{}", segment)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::dash_settings::DEFAULT_SETTINGS;

    #[test]
    fn get_without_put() {
        let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
        let key: i64 = 0;

        assert_eq!(dash.get_and_update(&key), None);
    }

    #[test]
    fn get_after_different_value_put() {
        let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
        let key: i64 = 0;

        dash.put(key + 1, key + 1);

        assert_eq!(dash.get_and_update(&key), None);
    }

    #[test]
    fn get_after_same_value_put() {
        let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
        let key: i64 = 0;

        dash.put(key, key);

        assert_eq!(dash.get_and_update(&key), Some(&key));
    }

    #[test]
    fn print_dash() {
        let mut dash: Dash<i64, i64> = Dash::new(DEFAULT_SETTINGS);
        let key: i64 = 0;

        dash.put(key, key);

        println!("{}", dash);
    }
}
