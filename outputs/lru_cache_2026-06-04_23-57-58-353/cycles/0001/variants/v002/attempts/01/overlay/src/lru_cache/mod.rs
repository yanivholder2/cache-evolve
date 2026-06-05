// CLOCK second-chance cache using HashMap + arena Vec.
// Each slot carries a `referenced` bit; the clock hand sweeps the arena during
// eviction, clearing bits on first encounter and evicting the first unreferenced
// slot it reaches.  At most one full revolution is needed because in the worst
// case every bit is cleared, guaranteeing a victim on the second pass.

use std::collections::HashMap;
use std::fmt::Debug;
use std::hash::Hash;

pub mod lru_cache_jni;

#[derive(Debug)]
struct Node<K, V> {
    key: K,
    value: V,
    /// Set to `true` on any `get` or `put`-update after insertion.
    /// Cleared to `false` on insertion and when the clock hand sweeps past.
    referenced: bool,
}

#[derive(Debug)]
pub struct LruCache<K, V> {
    map: HashMap<K, usize>, // key -> node index in arena
    nodes: Vec<Node<K, V>>, // fixed-capacity arena
    capacity: usize,
    len: usize,        // number of resident entries
    clock_hand: usize, // next candidate slot for eviction
}

impl<K, V> LruCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("LruCache.try_new: capacity must be > 0");
            return None;
        }
        log::info!("LruCache.try_new: capacity={capacity}");
        Some(Self {
            map: HashMap::with_capacity(capacity),
            nodes: Vec::with_capacity(capacity),
            capacity,
            len: 0,
            clock_hand: 0,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("LruCache::new: initialization failed")
    }

    /// Returns the value for `key` if present, setting its referenced bit.
    /// No structural movement is performed.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        let &idx = self.map.get(key)?;
        self.nodes[idx].referenced = true;
        Some(&self.nodes[idx].value)
    }

    /// Inserts or updates `key`.  If at capacity, evicts via the CLOCK algorithm.
    pub fn put(&mut self, key: K, value: V) {
        // Key already exists: update value in-place and mark referenced.
        if let Some(&idx) = self.map.get(&key) {
            self.nodes[idx].value = value;
            self.nodes[idx].referenced = true;
            return;
        }
        // New key — need an arena slot.
        if self.len < self.capacity {
            // Arena has room: append a new slot.
            let idx = self.nodes.len();
            self.nodes.push(Node {
                key: key.clone(),
                value,
                referenced: false,
            });
            self.map.insert(key, idx);
            self.len += 1;
        } else {
            // Arena full: sweep clock hand to find an unreferenced slot.
            // At most one full revolution is required.
            loop {
                if self.nodes[self.clock_hand].referenced {
                    // Give a second chance: clear the bit and advance.
                    self.nodes[self.clock_hand].referenced = false;
                    self.clock_hand = (self.clock_hand + 1) % self.capacity;
                } else {
                    // Evict this slot and reuse it for the new entry.
                    let evicted_key = self.nodes[self.clock_hand].key.clone();
                    self.map.remove(&evicted_key);
                    self.nodes[self.clock_hand] = Node {
                        key: key.clone(),
                        value,
                        referenced: false,
                    };
                    self.map.insert(key, self.clock_hand);
                    self.clock_hand = (self.clock_hand + 1) % self.capacity;
                    break;
                }
            }
        }
        debug_assert!(self.map.len() <= self.capacity);
    }
}

impl<K, V> framework::Cache<K, V> for LruCache<K, V>
where
    K: Hash + Eq + Clone + Debug,
    V: Debug,
{
    fn put(&mut self, key: K, value: V) {
        LruCache::put(self, key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        LruCache::get(self, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_miss_returns_none() {
        let mut cache = LruCache::<i64, i64>::new(4);
        assert!(cache.get(&42).is_none());
    }

    #[test]
    fn put_then_get() {
        let mut cache = LruCache::new(4);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_overwrites_existing_key() {
        let mut cache = LruCache::new(4);
        cache.put(1, 10);
        cache.put(1, 20);
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    #[test]
    fn overwrite_does_not_grow_order() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(1, 20);
        cache.put(1, 30);
        // Should still have room for a second key without eviction
        cache.put(2, 40);
        assert_eq!(*cache.get(&1).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 40);
    }

    #[test]
    fn eviction_removes_unreferenced() {
        // With clock_hand=0 and both slots unreferenced, key 1 (slot 0) is evicted first.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0, ref=false
        cache.put(2, 20); // slot 1, ref=false
        cache.put(3, 30); // clock_hand=0: slot 0 unreferenced -> evict key 1

        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn referenced_entry_gets_second_chance() {
        // key 1 is accessed (ref=true) before eviction pressure arrives.
        // CLOCK skips it and evicts key 2 instead.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.get(&1);    // nodes[0].ref = true
        cache.put(3, 30); // clock_hand=0: ref=true -> clear, advance;
                          // clock_hand=1: ref=false -> evict key 2

        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn put_update_grants_second_chance() {
        // Updating an existing key sets its referenced bit, protecting it.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(1, 11); // update: nodes[0].ref = true
        cache.put(3, 30); // clock_hand=0: ref=true -> clear, advance;
                          // clock_hand=1: ref=false -> evict key 2

        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert!(cache.get(&2).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
    }

    #[test]
    fn eviction_chain() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40); // evicts key 1 (slot 0, ref=false), clock_hand -> 1
        cache.put(5, 50); // evicts key 2 (slot 1, ref=false), clock_hand -> 2
        cache.put(6, 60); // evicts key 3 (slot 2, ref=false), clock_hand -> 0

        assert!(cache.get(&1).is_none());
        assert!(cache.get(&2).is_none());
        assert!(cache.get(&3).is_none());
        assert_eq!(*cache.get(&4).unwrap(), 40);
        assert_eq!(*cache.get(&5).unwrap(), 50);
        assert_eq!(*cache.get(&6).unwrap(), 60);
    }

    #[test]
    fn capacity_one() {
        let mut cache = LruCache::new(1);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
        cache.put(2, 20); // clock_hand=0: ref=true (from get) -> clear; ref=false -> evict key 1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn overwrite_at_full_capacity_does_not_evict() {
        let mut cache = LruCache::new(2);
        cache.put(1, 10);
        cache.put(2, 20); // cache full
        cache.put(1, 11); // overwrite, must NOT evict key 2
        assert_eq!(*cache.get(&1).unwrap(), 11);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn repeated_get_preserves_second_chance() {
        // key 1 is accessed repeatedly; its slot keeps its referenced bit set
        // so the clock skips it and evicts key 2 instead.
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // slot 2
        for _ in 0..10 {
            cache.get(&1); // nodes[0].ref stays true
        }
        cache.put(4, 40); // clock_hand=0: ref=true -> clear, advance;
                          // clock_hand=1: ref=false -> evict key 2
        assert_eq!(*cache.get(&1).unwrap(), 10);
        assert!(cache.get(&2).is_none());
    }

    #[test]
    fn non_adjacent_referenced_entry_gets_second_chance() {
        // key 2 (middle slot) is accessed, giving it a second chance.
        // clock_hand starts at 0; slot 0 (key1) is unreferenced -> evicted.
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // slot 2
        cache.get(&2);    // nodes[1].ref = true
        cache.put(4, 40); // clock_hand=0: ref=false -> evict key 1, slot 0 = key 4
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert_eq!(*cache.get(&2).unwrap(), 20);
    }

    #[test]
    fn second_chance_survives_one_eviction_round() {
        // Detailed trace of the CLOCK second-chance eviction order.
        let mut cache = LruCache::new(2);
        cache.put(1, 10); // slot 0, ref=false
        cache.put(2, 20); // slot 1, ref=false
        cache.put(3, 30); // clock_hand=0: ref=false -> evict key 1; slot 0 = key 3; clock_hand=1
        cache.get(&3);    // nodes[0].ref = true
        cache.put(4, 40); // clock_hand=1: ref=false -> evict key 2; slot 1 = key 4; clock_hand=0
        cache.put(5, 50); // clock_hand=0: ref=true  -> clear; clock_hand=1;
                          // clock_hand=1: ref=false -> evict key 4; slot 1 = key 5; clock_hand=0
        // key 3 survived because its second-chance bit was set
        assert_eq!(*cache.get(&3).unwrap(), 30);
        assert!(cache.get(&4).is_none());
        assert_eq!(*cache.get(&5).unwrap(), 50);
    }

    #[test]
    fn get_only_element() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10);
        // Single resident; get sets referenced bit
        assert_eq!(*cache.get(&1).unwrap(), 10);
        // Verify cache is still intact after filling remaining slots
        cache.put(2, 20);
        cache.put(3, 30);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn overwrite_mru_at_capacity() {
        let mut cache = LruCache::new(3);
        cache.put(1, 10); // slot 0
        cache.put(2, 20); // slot 1
        cache.put(3, 30); // slot 2; clock_hand=0
        cache.put(3, 31); // update existing: nodes[2].ref = true
                          // Eviction should take slot 0 (key 1, ref=false)
        cache.put(4, 40); // clock_hand=0: ref=false -> evict key 1
        assert!(cache.get(&1).is_none());
        assert_eq!(*cache.get(&2).unwrap(), 20);
        assert_eq!(*cache.get(&3).unwrap(), 31);
        assert_eq!(*cache.get(&4).unwrap(), 40);
    }

    #[test]
    #[should_panic(expected = "initialization failed")]
    fn capacity_zero_panics() {
        LruCache::<i64, i64>::new(0);
    }
}
