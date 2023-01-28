use std::hash::Hash;
use crate::dash::bucket::Bucket;

#[derive(Debug)]
pub struct Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub buckets: Vec<Bucket<K, V>>,
    pub size: usize,
}

impl<K, V> Segment<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone
{
    pub fn new(size: usize, bucket_size: usize) -> Self {
        let mut buckets: Vec<Bucket<K, V>> = Vec::new();
        for _ in 0..size {
            buckets.push(Bucket::new(bucket_size));
        }
        Segment {
            buckets,
            size,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_segment_length() {
        let segment: Segment<i32, i32> = Segment::new(10, 10);
        assert_eq!(segment.buckets.len(), 10);
    }
}





