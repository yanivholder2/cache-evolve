use crate::dash::bucket::Bucket;
use crate::dash::data::Data;
use crate::dash::utils::{get_index, hash};
use crate::dash_settings::{DashSettings, EvictionPolicy};
use std::fmt::Display;
use std::hash::Hash;

#[derive(Debug)]
pub struct Segment<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    pub buckets: Vec<Bucket<K, V>>,
    pub segment_size: usize,
    pub stash_buckets: Vec<Bucket<K, V>>,
    pub stash_size: usize,
}

impl<K, V> Segment<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    pub fn new(settings: DashSettings) -> Self {
        let mut buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..settings.segment_size {
            // TODO: pass the settings as a reference
            buckets.push(Bucket::new(
                settings.bucket_size.clone(),
                settings.eviction_policy.clone(),
            ));
        }

        let mut stash_buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..settings.stash_size {
            stash_buckets.push(Bucket::new(
                settings.bucket_size.clone(),
                EvictionPolicy::FIFO,
            ));
        }
        Segment {
            buckets,
            stash_buckets,
            segment_size: settings.segment_size,
            stash_size: settings.stash_size,
        }
    }

    // TODO: could be written better
    pub fn get_and_update(&mut self, key: &K) -> Option<&Data<K, V>> {
        let hash = hash(&key);
        let stash_bucket_index = get_index(hash, self.stash_size);
        let stash_bucket = &self.stash_buckets[stash_bucket_index];
        let target_bucket_index = get_index(hash, self.segment_size);

        // The order assumes that the data is more likely to be in the stash bucket,
        // this assumption should be tested
        if let Some(position) = stash_bucket.get_position(key) {
            // If the key is in the stash bucket, we need to move it to the target bucket
            let mut_stash_bucket = &mut self.stash_buckets[stash_bucket_index];
            let data = mut_stash_bucket.get_from_position(position)?.clone();
            mut_stash_bucket.remove(key);

            let mut_target_bucket = &mut self.buckets[target_bucket_index];
            return mut_target_bucket.put_data(data);
        } else {
            // If the key is not in the stash bucket, we need to check the target bucket
            let target_bucket_index = get_index(hash, self.segment_size);
            let target_bucket = &self.buckets[target_bucket_index];

            if let Some(position) = target_bucket.get_position(key) {
                // If the key is in the target bucket, we need to update the position
                let mut_target_bucket = &mut self.buckets[target_bucket_index];
                mut_target_bucket.update_key_in_index(position)
            } else {
                // If the key is not in the target bucket, we need to check the probing bucket

                if target_bucket_index == self.buckets.len() - 1 {
                    // If the target bucket is the last bucket, there is not probing bucket
                    return None;
                }
                let probing_bucket_index = target_bucket_index + 1;
                let probing_bucket = &self.buckets[probing_bucket_index];
                if let Some(position) = probing_bucket.get_position(key) {
                    // If the key is in the probing bucket, we need to update the position

                    let mut_probing_bucket = &mut self.buckets[probing_bucket_index];
                    mut_probing_bucket.update_key_in_index(position)
                } else {
                    None
                }
            }
        }
    }

    /// Insert the key, value pair into the segment.
    /// If the key already exists in the segment, the value and position will be updated.
    pub fn put(&mut self, key: K, val: V) {
        let hash = hash(&key);
        let stash_bucket = &mut self.stash_buckets[get_index(hash, self.stash_size)];
        let bucket = &mut self.buckets[get_index(hash, self.segment_size)];
        // The order assumes that the data is more likely to be in the stash bucket,
        // this assumption should be tested
        if stash_bucket.get_and_update(&key).is_some() {
            // TODO: consider what to do if data.val != val
            stash_bucket.remove(&key);
            bucket.put(key, val);
        } else if bucket.get_and_update(&key).is_some() {
            // TODO: move the data to the probing bucket
            bucket.put(key, val);
        } else {
            stash_bucket.put(key, val);
        }
    }
}

impl<K, V> Display for Segment<K, V>
where
    K: Hash + Eq + Clone + Copy + Display,
    V: Eq + Clone + Copy + Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Segment {{")?;
        for bucket in &self.buckets {
            writeln!(f, "  Bucket {{")?;
            writeln!(f, "    {}", bucket)?;
            writeln!(f, "  }}")?;
        }
        for bucket in &self.stash_buckets {
            writeln!(f, "  Stash Bucket {{")?;
            writeln!(f, "    {}", bucket)?;
            writeln!(f, "  }}")?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::dash_settings::DEFAULT_SETTINGS;

    #[test]
    fn test_segment_length() {
        let segment: Segment<i32, i32> = Segment::new(DEFAULT_SETTINGS);
        assert_eq!(segment.buckets.len(), DEFAULT_SETTINGS.segment_size);
    }
}
