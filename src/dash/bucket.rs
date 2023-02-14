use super::data::Data;
use crate::dash_settings::{DashSettings, EvictionPolicy};
use std::hash::Hash;

#[derive(Debug)]
pub struct Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    data_vec: Vec<Data<K, V>>,
    max_size: usize,
    // TODO: make this a reference with a lifetime
    settings: DashSettings,
}

impl<K, V> Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub fn new(max_size: usize, settings: DashSettings) -> Self {
        Bucket {
            data_vec: Vec::new(),
            max_size,
            settings,
        }
    }

    pub fn remove(&mut self, key: &K) {
        if self.data_vec.is_empty() {
            return;
        }
        self.data_vec.retain(|d| d.key != *key)
    }

    /// This function updates the data if it already exists
    pub fn insert(&mut self, key: K, val: V) {
        if self.contains(&key) {
            if self.get(&key).unwrap().value != val {
                // TODO: what should we do if the assert fails?
            }
            self.update(&key);
        } else {
            if self.is_full() {
                self.evict_item();
            }
            self.data_vec.push(Data::new(key, val));
        }
    }

    pub fn get(&mut self, key: &K) -> Option<&Data<K, V>> {
        if self.data_vec.is_empty() {
            return None;
        }
        // TODO: this is not performant, thea any, find and the update should be done in one iteration
        let is_data_in_vec = self.contains(key);
        if !is_data_in_vec {
            return None;
        }

        return Some(self.update(&key));
    }

    /// Updates the data in the bucket according to the eviction policy.
    /// Returns the updated data.
    ///
    /// This function assumes that the key exists in the bucket
    fn update(&mut self, key: &K) -> &Data<K, V> {
        let key_index = self.data_vec.iter().position(|d| d.key == *key).unwrap();

        match self.settings.eviction_policy {
            EvictionPolicy::FIFO | EvictionPolicy::LIFO => &self.data_vec[key_index],
            EvictionPolicy::LRU => {
                let data = self.data_vec.remove(key_index);
                self.data_vec.push(data);
                self.data_vec.last().unwrap()
            }
            EvictionPolicy::LFU => {
                self.data_vec[key_index].lfu_counter += 1;
                &self.data_vec[key_index]
            }
        }
    }

    fn evict_item(&mut self) {
        if self.data_vec.is_empty() {
            return;
        }
        match self.settings.eviction_policy {
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
        self.data_vec.iter().any(|d| d.key == *key)
    }

    pub fn is_full(&self) -> bool {
        self.size() == self.max_size
    }

    pub fn size(&self) -> usize {
        self.data_vec.len()
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
            let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            let value = 1;
            bucket.insert(value, value);
            assert_eq!(bucket.size(), 1);
            assert_eq!(bucket.get(&value).unwrap().value, value);
        }

        #[test]
        fn insert_multiple_items() {
            let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            let num_of_bucket_items = 5;

            for i in 0..num_of_bucket_items {
                bucket.insert(i, i);
            }
            assert_eq!(bucket.size(), num_of_bucket_items);
        }

        #[test]
        fn insert_duplicate_items() {
            let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            let num_of_bucket_items = 5;

            for i in 0..num_of_bucket_items {
                bucket.insert(i, i);
            }
            for i in 0..num_of_bucket_items {
                bucket.insert(i, i);
            }
            assert_eq!(bucket.size(), num_of_bucket_items);
        }

        #[test]
        fn insert_more_items_than_bucket_size() {
            let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
            let num_of_bucket_items = 15;

            for i in 0..num_of_bucket_items {
                bucket.insert(i, i);
            }
            assert_eq!(bucket.size(), 10);
        }
    }

    #[test]
    fn is_full() {
        let bucket_size = 10;
        let mut bucket = Bucket::new(bucket_size, DEFAULT_SETTINGS);
        for i in 0..bucket_size {
            assert_eq!(bucket.is_full(), false);
            bucket.insert(i, i);
        }
        assert_eq!(bucket.is_full(), true);
    }

    #[test]
    fn size() {
        let mut bucket = Bucket::new(10, DEFAULT_SETTINGS);
        let num_of_bucket_items = 5;

        for i in 0..num_of_bucket_items {
            bucket.insert(i, i);
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
