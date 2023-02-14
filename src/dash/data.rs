use std::hash::Hash;

#[derive(Debug, Clone)]
pub struct Data<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub key: K,
    pub value: V,
    pub lfu_counter: usize,
}

impl<K, V> PartialEq for Data<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    fn eq(&self, other: &Data<K, V>) -> bool {
        self.key == other.key && self.value == other.value
    }
}

impl<K, V> Eq for Data<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
}

impl<K, V> Data<K, V>
where
    K: Hash + Eq + Clone,
    V: Eq + Clone,
{
    pub fn new(key: K, value: V) -> Self {
        Self {
            key,
            value,
            lfu_counter: 0,
        }
    }
}
