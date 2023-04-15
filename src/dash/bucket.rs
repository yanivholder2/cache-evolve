use super::data::Data;
use crate::dash_settings::EvictionPolicy;
use std::{
    fmt::{Display, Formatter},
    hash::Hash,
};

#[derive(Debug)]
pub struct Bucket<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    // TODO: consider using a linked list for O(1) changes
    data_vec: Vec<Data<K, V>>,
    max_size: usize,
    // TODO: make this a reference with a lifetime
    eviction_policy: EvictionPolicy,
}

impl<K, V> Bucket<K, V>
where
    K: Hash + Eq + Clone + Copy,
    V: Eq + Clone + Copy,
{
    pub fn new(max_size: usize, eviction_policy: EvictionPolicy) -> Self {
        Bucket {
            data_vec: Vec::new(),
            max_size,
            eviction_policy,
        }
    }

    pub fn remove(&mut self, key: &K) {
        if self.data_vec.is_empty() {
            return;
        }
        self.data_vec.retain(|d| d.key != *key)
    }

    /// This function updates the data if it already exists
    pub fn put(&mut self, key: K, val: V) {
        if let Some(position) = self.get_position(&key) {
            self.update_key_in_index(position);
        } else {
            if self.is_full() {
                self.evict_item();
            }
            self.data_vec.push(Data::new(key, val));
        }
    }

    pub fn put_data(&mut self, data: Data<K, V>) -> Option<&Data<K, V>> {
        if let Some(position) = self.get_position(&data.key) {
            return self.update_key_in_index(position);
        } else {
            if self.is_full() {
                self.evict_item();
            }
            self.data_vec.push(data);
            Some(&self.data_vec[self.data_vec.len() - 1])
        }
    }

    pub fn get(&self, key: &K) -> Option<&Data<K, V>> {
        if self.data_vec.is_empty() {
            return None;
        }
        let position = self.get_position(key)?;
        Some(&self.data_vec[position])
    }

    pub fn get_from_position(&mut self, position: usize) -> Option<&Data<K, V>> {
        if self.data_vec.is_empty() {
            return None;
        }
        Some(&self.data_vec[position])
    }

    pub fn get_and_update(&mut self, key: &K) -> Option<&Data<K, V>> {
        if self.data_vec.is_empty() {
            return None;
        }
        let position = self.get_position(key)?;
        self.update_key_in_index(position)
    }

    /// Updates the data in the bucket according to the eviction policy.
    ///
    /// Returns the updated data and None if the data is not in the bucket.
    pub fn update(&mut self, key: &K) -> Option<&Data<K, V>> {
        let key_index = self.get_position(key)?;
        self.update_key_in_index(key_index)
    }

    pub fn update_key_in_index(&mut self, key_index: usize) -> Option<&Data<K, V>> {
        match self.eviction_policy {
            EvictionPolicy::FIFO | EvictionPolicy::LIFO => Some(&self.data_vec[key_index]),
            EvictionPolicy::LRU => {
                let data = self.data_vec.remove(key_index);
                self.data_vec.push(data);
                self.data_vec.last()
            }
            EvictionPolicy::LFU => {
                self.data_vec[key_index].lfu_counter += 1;
                Some(&self.data_vec[key_index])
            }
        }
    }

    fn evict_item(&mut self) {
        if self.data_vec.is_empty() {
            return;
        }
        match self.eviction_policy {
            EvictionPolicy::FIFO | EvictionPolicy::LRU => {
                // TODO: this is in O(n). there could be a more performant way to do that
                self.data_vec.remove(0);
            }
            EvictionPolicy::LIFO => {
                self.data_vec.pop();
            }
            // TODO: implement better
            EvictionPolicy::LFU => {
                let mut min_lfu_counter = self.data_vec[0].lfu_counter;
                let mut min_lfu_counter_index = 0;
                for (i, data) in self.data_vec.iter().enumerate() {
                    if data.lfu_counter < min_lfu_counter {
                        min_lfu_counter = data.lfu_counter;
                        min_lfu_counter_index = i;
                    }
                }
                self.data_vec.remove(min_lfu_counter_index);
            }
        }
    }

    pub fn contains(&self, key: &K) -> bool {
        self.get_position(key).is_some()
    }

    pub fn get_position(&self, key: &K) -> Option<usize> {
        self.data_vec.iter().position(|d| d.key == *key)
    }

    pub fn is_full(&self) -> bool {
        self.size() == self.max_size
    }

    pub fn size(&self) -> usize {
        self.data_vec.len()
    }
}

impl<K, V> Display for Bucket<K, V>
where
    K: Hash + Eq + Clone + Copy + Display,
    V: Eq + Clone + Copy + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for data in &self.data_vec {
            writeln!(f, "    {}", data)?;
        }
        Ok(())
    }
}

// TODO: implement more tests
#[cfg(test)]
mod tests {
    use super::*;
    use crate::dash_settings::DEFAULT_SETTINGS;

    mod insert {
        use super::*;

        mod get {
            use super::*;

            // #[test]
            // fn get_one_item() {
            //     let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            //     let value = 1;
            //     bucket.insert(value, value);
            //     assert_eq!(bucket.get(&value).unwrap().value, value);
            // }
            //
            // #[test]
            // fn get_multiple_items() {
            //     let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            //     let num_of_bucket_items = 5;
            //
            //     for i in 0..num_of_bucket_items {
            //         bucket.insert(i, i);
            //     }
            //     for i in 0..num_of_bucket_items {
            //         assert_eq!(bucket.get(&i).unwrap().value, i);
            //     }
            // }
            //
            // #[test]
            // fn get_duplicate_items() {
            //     let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            //     let num_of_bucket_items = 5;
            //
            //     for i in 0..num_of_bucket_items {
            //         bucket.insert(i, i);
            //     }
            //     for i in 0..num_of_bucket_items {
            //         bucket.insert(i, i);
            //     }
            //     for i in 0..num_of_bucket_items {
            //         assert_eq!(bucket.get(&i).unwrap().value, i);
            //     }
            // }
            //
            // #[test]
            // fn get_more_items_than_bucket_size() {
            //     let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            //     let num_of_bucket_items = 15;
            //
            //     for i in 0..num_of_bucket_items {
            //         bucket.insert(i, i);
            //     }
            //     for i in 0..num_of_bucket_items {
            //         assert_eq!(bucket.get(&i).unwrap().value, i);
            //     }
            // }
        }

        #[test]
        fn insert_one_item() {
            let mut bucket = Bucket::new(
                DEFAULT_SETTINGS.bucket_size,
                DEFAULT_SETTINGS.eviction_policy,
            );
            let value = 1;
            bucket.put(value, value);
            assert_eq!(bucket.size(), 1);
            assert_eq!(bucket.get_and_update(&value).unwrap().value, value);
        }

        #[test]
        fn insert_multiple_items() {
            let mut bucket = Bucket::new(
                DEFAULT_SETTINGS.bucket_size,
                DEFAULT_SETTINGS.eviction_policy,
            );
            let num_of_bucket_items = 5;

            for i in 0..num_of_bucket_items {
                bucket.put(i, i);
            }
            assert_eq!(bucket.size(), num_of_bucket_items);
        }

        #[test]
        fn insert_duplicate_items() {
            let mut bucket = Bucket::new(
                DEFAULT_SETTINGS.bucket_size,
                DEFAULT_SETTINGS.eviction_policy,
            );
            let num_of_bucket_items = 5;

            for i in 0..num_of_bucket_items {
                bucket.put(i, i);
            }
            for i in 0..num_of_bucket_items {
                bucket.put(i, i);
            }
            assert_eq!(bucket.size(), num_of_bucket_items);
        }

        #[test]
        fn insert_more_items_than_bucket_size() {
            let bucket_size = DEFAULT_SETTINGS.bucket_size;

            let mut bucket = Bucket::new(bucket_size, DEFAULT_SETTINGS.eviction_policy);
            let num_of_bucket_items = bucket_size + 1;

            for i in 0..num_of_bucket_items {
                bucket.put(i, i);
            }
            assert_eq!(bucket.size(), DEFAULT_SETTINGS.bucket_size);
        }
    }

    #[test]
    fn is_full() {
        let bucket_size = 10;
        let mut bucket = Bucket::new(bucket_size, DEFAULT_SETTINGS.eviction_policy);
        for i in 0..bucket_size {
            assert_eq!(bucket.is_full(), false);
            bucket.put(i, i);
        }
        assert_eq!(bucket.is_full(), true);
    }

    #[test]
    fn size() {
        let mut bucket = Bucket::new(
            DEFAULT_SETTINGS.bucket_size,
            DEFAULT_SETTINGS.eviction_policy,
        );
        let num_of_bucket_items = 5;

        for i in 0..num_of_bucket_items {
            bucket.put(i, i);
        }
        assert_eq!(bucket.size(), num_of_bucket_items);
    }

    mod evict_item {
        use super::*;

        #[test]
        fn evict_item_fifo() {
            // TODO: implement
        }

        #[test]
        fn evict_item_lifo() {
            // TODO: implement
        }

        #[test]
        fn evict_item_lfu() {
            // TODO: implement
        }

        #[test]
        fn evict_item_lru() {
            // TODO: implement
        }
    }
}
