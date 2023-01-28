use std::hash::Hash;
use crate::dash_settings::EvictionPolicy;
use super::data::Data;


#[derive(Debug)]
pub struct Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    datas: Vec<Data<K, V>>,
    size: usize
}

impl<K, V> Bucket<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone
{
    pub fn new(size: usize) -> Self {
        Bucket {
            datas: Vec::new(),
            size
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
            return
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

    pub fn evict_item(&mut self, eviction_policy: EvictionPolicy) {
        if self.datas.is_empty() {
            return;
        }
        match eviction_policy {
            EvictionPolicy::MoveAhead | EvictionPolicy::FIFO | EvictionPolicy::LRU => {
                // TODO: this is in O(n). there could be a more performant way to do that
                self.datas.remove(0);
            }
            EvictionPolicy::LIFO => { self.datas.pop(); }
            // TODO: implement
            EvictionPolicy::LFU | EvictionPolicy::LFUDisp => ()
        }
    }
}



// TODO: implement more tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn is_full() {
        let mut bucket = Bucket::new(10);
        for _ in 0..10 {
            assert_eq!(bucket.is_full(), false);
            bucket.insert(0,0);
        }
        assert_eq!(bucket.is_full(), true);
    }

    #[test]
    fn size() {
        let mut bucket = Bucket::new(10);
        for _ in 0..5 {
            bucket.insert(0,0);
        }
        assert_eq!(bucket.size(), 5);
    }

    #[test]
    fn evict_item_lru() {
        let mut bucket = Bucket::new(10);
        for i in 0..5 {
            bucket.insert(i,0);

        }
        bucket.evict_item(EvictionPolicy::LRU);
        assert_eq!(bucket.size(), 5);
        // FIXME
        // assert_eq!(bucket.data_list.iter().any(|&data| data.value == 0), false);
    }
}