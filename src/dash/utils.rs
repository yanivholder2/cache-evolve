use std::collections::hash_map::DefaultHasher;
use std::hash::Hasher;

pub fn get_index(hash: usize, size: usize) -> usize {
    hash % size
}

pub fn hash<K>(key: &K) -> usize
where
    K: std::hash::Hash,
{
    let mut hasher = DefaultHasher::new();
    key.hash(&mut hasher);
    hasher.finish() as usize
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_index() {
        let hash = 123;
        let size = 10;
        let index = get_index(hash, size);
        // 123 % 10 = 3
        assert_eq!(index, 3);
    }
}
