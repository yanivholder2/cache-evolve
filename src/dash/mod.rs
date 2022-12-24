use std::collections::HashMap;
use std::hash::Hash;

#[cfg(test)]
mod tests;

#[derive(Debug)]
pub struct Dash<K, V> {
    map: HashMap<K, V>
}

impl<K, V> Dash<K, V> {
    pub fn new() -> Self {
        Dash {
            map: HashMap::new()
        }
    }

    pub fn contains(&self, key: &K) -> bool
    where
        K: Eq + Hash
    {

        self.map.contains_key(key)
    }

    pub fn get(&self, key: &K) -> Option<&V>
    where
        K: Eq + Hash
    {
        self.map.get(key)
    }

    pub fn put(&mut self, key: K, val: V) -> ()
    where
        K: Eq + Hash
    {
        self.map.insert(key, val);
    }
}