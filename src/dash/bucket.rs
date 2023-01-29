use std::hash::Hash;
use crate::dash_settings::{DashSettings, EvictionPolicy};
use super::data::Data;


#[derive(Debug)]
pub struct Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    datas: Vec<Data<K, V>>,
    size: usize,
    // TODO: make this a reference with a lifetime
    settings: DashSettings
}

impl<K, V> Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone
{
    pub fn new(size: usize, settings: DashSettings) -> Self {
        Bucket {
            datas: Vec::new(),
            size,
            settings
        }
    }

    pub fn is_full(&self) -> bool {
        self.datas.len() == self.size
    }

    pub fn size(&self) -> usize {
        self.datas.len()
    }

    pub fn insert(&mut self, key: K, val: V) {
        let new_data: Data<K, V> = Data::new(key, val);
        if self.is_full() {
            return
        } else if self.datas.contains(&new_data) {
            match self.settings.eviction_policy {
                EvictionPolicy::LRU => {
                    self.datas.retain(|d| d.key != new_data.key);
                    self.datas.push(new_data);
                }
                EvictionPolicy::LFU => {
                    let mut data_in_vec = self.datas.iter_mut().find(|d| d.key == new_data.key).unwrap();
                    data_in_vec.lfu_counter += 1;
                }
                EvictionPolicy::FIFO | EvictionPolicy::LIFO => {}
            }
        } else {
            self.datas.push(new_data)
        }
    }

    pub fn get(&self, key: &K) -> Option<&Data<K, V>> {
        if self.datas.is_empty() {
            None
        } else {
            let data_in_vec = self.datas.iter().find(|&d| d.key == *key);
            match data_in_vec {
                Some(d) => Some(d),
                None => None
            }
        }

    }

    pub fn remove(&mut self, key: &K) {
        if self.datas.is_empty() {
            return;
        }
        self.datas.retain(|d| d.key != *key)
    }

    pub fn evict_item(&mut self, eviction_policy: &EvictionPolicy) {
        if self.datas.is_empty() {
            return;
        }
        match eviction_policy {
            EvictionPolicy::FIFO | EvictionPolicy::LRU => {
                // TODO: this is in O(n). there could be a more performant way to do that
                self.datas.remove(0);
            }
            EvictionPolicy::LIFO => { self.datas.pop(); }
            // TODO: implement better
            EvictionPolicy::LFU => {
                let mut min_lfu_counter = self.datas[0].lfu_counter;
                let mut min_lfu_counter_index = 0;
                for (i, data) in self.datas.iter().enumerate() {
                    if data.lfu_counter < min_lfu_counter {
                        min_lfu_counter = data.lfu_counter;
                        min_lfu_counter_index = i;
                    }
                }
                self.datas.remove(min_lfu_counter_index);
            }
        }
    }
}



// TODO: implement more tests
#[cfg(test)]
mod tests {
    use super::*;
    const SETTINGS: DashSettings = DashSettings {
        dash_size: 1,
        segment_size: 1,
        bucket_size: 100,
        eviction_policy: EvictionPolicy::LRU,
        debug_mode: 0,
    };

    mod insert {
        use super::*;

        #[test]
        fn insert_one_item() {
            let mut bucket = Bucket::new(10, SETTINGS);
            let value = 1;
            bucket.insert(value, value);
            assert_eq!(bucket.size(), 1);
            assert_eq!(bucket.get(&value).unwrap().value, value);
        }

        #[test]
        fn insert_multiple_items() {
            let mut bucket = Bucket::new(10, SETTINGS);
            let num_of_bucket_items = 5;

            for i in 0..num_of_bucket_items {
                bucket.insert(i, i);
            }
            assert_eq!(bucket.size(), num_of_bucket_items);
        }

        #[test]
        fn insert_duplicate_items() {
            let mut bucket = Bucket::new(10, SETTINGS);
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
            let mut bucket = Bucket::new(10, SETTINGS);
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
        let mut bucket = Bucket::new(bucket_size, SETTINGS);
        for i in 0..bucket_size {
            assert_eq!(bucket.is_full(), false);
            bucket.insert(i,i);
        }
        assert_eq!(bucket.is_full(), true);
    }

    #[test]
    fn size() {
        let mut bucket = Bucket::new(10, SETTINGS);
        let num_of_bucket_items = 5;

        for i in 0..num_of_bucket_items {
            bucket.insert(i,i);
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
