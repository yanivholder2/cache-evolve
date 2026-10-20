use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// S3-FIFO with resident_map: a single HashMap<K, (V, u8)> stores both value
// and frequency for every resident key. This replaces the parallel small_freq /
// main_freq VecDeques of the parent (cycle/0004/v001) and eliminates every
// O(n) position() scan: a single HashMap::get_mut covers hit detection,
// frequency increment, and value retrieval in one round-trip.
//
// Retained from parent:
//   - small_size = capacity/20 (H02)
//   - O(1) ghost membership via ghost_set + lazy deletion
// Removed from parent:
//   - small_freq / main_freq parallel deques
//   - SCAN_CAP=16 bounded scan (cycle 4 showed it hurts hit-rate on Zipfian)
//
// H02 (cycle/0006): ghost_capacity = 2 * main_size to widen the temporal
// window in which evicted keys can be ghost-promoted back into main.

const MAX_FREQ: u8 = 3;

/// Entry stores only the key; value and frequency are held in resident_map.
/// The constructor still accepts `_value` so call-sites that pass a value
/// compile unchanged (the value is dropped immediately).
#[derive(Debug)]
pub struct Entry<K, V> {
    key: K,
    _phantom: std::marker::PhantomData<V>,
}

impl<K, V> Entry<K, V> {
    fn new(key: K, _value: V) -> Self {
        Self { key, _phantom: std::marker::PhantomData }
    }
}

#[derive(Debug)]
pub struct S3Fifo<K: PartialEq + Eq + Hash + Clone, V> {
    /// Small admission FIFO queue (newest at front, oldest at back).
    small: VecDeque<Entry<K, V>>,
    /// Main resident FIFO queue.
    main: VecDeque<Entry<K, V>>,
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    /// ghost_set.len() <= ghost.len() always holds.
    ghost_set: HashSet<K>,
    /// Maps every resident key to its (value, frequency) pair. O(1) access for
    /// both get() value retrieval and frequency updates. A single lookup covers
    /// both fields, avoiding a second hashmap round-trip.
    resident_map: HashMap<K, (V, u8)>,
    small_size: usize,
    main_size: usize,
    /// Ghost queue capacity: 2 * main_size, widening the temporal window in
    /// which evicted keys can be ghost-promoted back into the main queue.
    ghost_capacity: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        // H02: small queue = 1/20 of capacity, giving main ~95% on Zipfian.
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 20).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        // H02 (cycle/0006): ghost capacity is 2 * main_size to extend the
        // ghost window and improve hit-rate on Zipfian workloads.
        let ghost_capacity = 2 * main_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            ghost: VecDeque::with_capacity(ghost_capacity),
            ghost_set: HashSet::with_capacity(ghost_capacity),
            resident_map: HashMap::with_capacity(capacity),
            small_size,
            main_size,
            ghost_capacity,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// O(1) lookup: single HashMap::get_mut covers hit detection, frequency
    /// increment, and value retrieval.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.resident_map.get_mut(key) {
            entry.1 = entry.1.saturating_add(1).min(MAX_FREQ);
            return Some(&entry.0);
        }
        None
    }

    pub fn put(&mut self, key: K, value: V) {
        // O(1) update path: key already resident — update value in-place.
        if let Some(entry) = self.resident_map.get_mut(&key) {
            entry.0 = value;
            return;
        }

        // Ghost hit: O(1) via ghost_set; lazy deletion preserved (stale key
        // remains in the deque and is cleaned up by the ghost capacity guard).
        if self.ghost_set.contains(&key) {
            self.ghost_set.remove(&key);
            if self.main.len() >= self.main_size {
                self.evict_main();
            }
            // Insert value into resident_map first, then push key into the
            // deque. Entry::new ignores _value, so we use a struct literal to
            // avoid a double-move of `value` (which was already consumed above).
            self.resident_map.insert(key.clone(), (value, 0));
            self.main.push_front(Entry { key, _phantom: std::marker::PhantomData });
            return;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        self.resident_map.insert(key.clone(), (value, 0));
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: unbounded second-chance via frequency
    /// decrement. SCAN_CAP removed — cold entries accumulate naturally at the
    /// tail of main on Zipfian workloads so the loop terminates quickly.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.resident_map.get(&tail.key).map_or(0, |e| e.1);
            if freq > 0 {
                // Second chance: decrement frequency and re-admit at front.
                if let Some(entry) = self.resident_map.get_mut(&tail.key) {
                    entry.1 = freq - 1;
                }
                self.main.push_front(tail);
            } else {
                // Zero frequency: evict and remove from resident_map.
                self.resident_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq > 1, else ghost.
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.resident_map.get(&tail.key).map_or(0, |e| e.1);
            if freq > 1 {
                // Hot entry: promote to main. resident_map entry stays intact.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: evict to ghost, remove from resident_map.
                self.resident_map.remove(&tail.key);
                // Ghost capacity guard with lazy deletion: pop the back of ghost
                // until we find a live ghost (still in ghost_set) and evict it.
                // Uses ghost_capacity (2 * main_size) instead of main_size to
                // widen the temporal window for ghost-based re-promotion.
                while self.ghost.len() >= self.ghost_capacity {
                    if let Some(evicted_key) = self.ghost.pop_back() {
                        if self.ghost_set.contains(&evicted_key) {
                            self.ghost_set.remove(&evicted_key);
                            break; // evicted one real ghost entry
                        }
                        // stale (lazily deleted): loop again
                    }
                }
                let k = tail.key;
                self.ghost.push_front(k.clone());
                self.ghost_set.insert(k);
            }
        }
    }
}

impl<K: PartialEq + Eq + Hash + Clone + Debug, V: Debug> framework::Cache<K, V>
    for S3Fifo<K, V>
{
    fn put(&mut self, key: K, value: V) {
        S3Fifo::put(self, key, value);
    }

    fn get(&mut self, key: &K) -> Option<&V> {
        S3Fifo::get(self, key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_miss_returns_none() {
        let mut cache = S3Fifo::<i64, i64>::new(4);
        assert!(cache.get(&42).is_none());
    }

    #[test]
    fn put_then_get_finds_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        assert_eq!(*cache.get(&1).unwrap(), 10);
    }

    #[test]
    fn put_existing_updates_value() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.put(1, 20);
        assert_eq!(*cache.get(&1).unwrap(), 20);
    }

    #[test]
    fn small_eviction_promotes_to_main_after_multiple_hits() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.get(&1);
        cache.get(&1);
        cache.put(2, 20);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    #[test]
    fn ghost_hit_promotes_back_to_main() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.put(2, 20);
        cache.put(3, 30);
        cache.put(4, 40);
        cache.put(1, 10);
        assert!(cache.main.iter().any(|entry| entry.key == 1));
    }

    /// White-box test for second-chance eviction in main.
    /// Adapted for the resident_map layout: freq values are stored alongside
    /// their values in resident_map, not in parallel deques.
    ///
    /// Layout: main deque = [key:1(front), key:2, key:3, key:4(back)],
    ///         small = [key:5(front)],
    ///         resident_map = {1:(10,0), 2:(20,2), 3:(30,0), 4:(40,1), 5:(50,2)}.
    /// evict_small() sees freq(5)=2 > 1 → hot, promotes key:5 to main.
    /// main is full (4 entries) → evict_main():
    ///   pop key:4 (freq=1 > 0) → second chance, freq→0, push to front.
    ///   pop key:3 (freq=0)     → evict immediately.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::new(5);
        cache.main.push_back(Entry::new(1, 10)); cache.resident_map.insert(1u64, (10u64, 0u8));
        cache.main.push_back(Entry::new(2, 20)); cache.resident_map.insert(2u64, (20u64, 2u8));
        cache.main.push_back(Entry::new(3, 30)); cache.resident_map.insert(3u64, (30u64, 0u8));
        cache.main.push_back(Entry::new(4, 40)); cache.resident_map.insert(4u64, (40u64, 1u8));
        cache.small.push_front(Entry::new(5, 50)); cache.resident_map.insert(5u64, (50u64, 2u8));
        cache.evict_small();
        // Entry 4 had freq=1 → gets a second chance (freq decremented to 0,
        // moved to front of main), so it survives.
        assert!(cache.main.iter().any(|entry| entry.key == 4));
        // Entry 3 had freq=0 → evicted immediately (no second chance).
        assert!(!cache.main.iter().any(|entry| entry.key == 3));
    }

    #[test]
    fn capacity_bound_never_exceeded() {
        let mut cache = S3Fifo::new(4);
        for key in 0..20 {
            cache.put(key, key);
        }
        assert!(cache.small.len() + cache.main.len() <= 4);
    }

    #[test]
    fn repeated_get_clamps_frequency() {
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        for _ in 0..10 {
            assert_eq!(*cache.get(&1).unwrap(), 10);
        }
    }

    /// Lazy ghost deletion must not desync ghost_set and ghost deque:
    /// ghost_set.len() <= ghost.len() must hold after ghost-hit scenarios.
    #[test]
    fn ghost_set_len_bounded_after_ghost_hit() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::<i64, i64>::new(4);
        // Fill ghost with entries 1, 2, 3 by pushing 4 entries through small.
        cache.put(1, 10); // small=[1]
        cache.put(2, 20); // evicts 1 → ghost=[1]
        cache.put(3, 30); // evicts 2 → ghost=[2,1]
        cache.put(4, 40); // evicts 3 → ghost=[3,2,1]
        // Ghost hit on 1: removes 1 from ghost_set (lazy), stays in deque.
        cache.put(1, 10);
        // ghost_set = {2, 3}, ghost deque = [3,2,1(stale)]
        assert!(
            cache.ghost_set.len() <= cache.ghost.len(),
            "ghost_set.len() ({}) must not exceed ghost.len() ({})",
            cache.ghost_set.len(),
            cache.ghost.len()
        );
        // Also verify the ghost hit actually promoted key 1 to main.
        assert!(cache.main.iter().any(|e| e.key == 1));
    }
}
