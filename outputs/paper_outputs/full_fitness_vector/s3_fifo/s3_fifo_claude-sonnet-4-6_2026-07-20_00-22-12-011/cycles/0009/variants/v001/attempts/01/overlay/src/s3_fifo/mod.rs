use std::collections::{HashMap, HashSet, VecDeque};
use std::fmt::Debug;
use std::hash::Hash;

pub mod s3_fifo_jni;

// Simple S3-FIFO implementation derived from "FIFO Queues are ALL You Need for
// Cache Eviction" by Juncheng Yang et al. The policy uses a small FIFO admission
// queue, a main FIFO resident queue, and a ghost queue for recently evicted
// items. Reaccesses to ghosted keys promote them into the main queue, and main
// eviction uses a second-chance-style frequency decrement.
//
// This variant builds on cycle/0007/v003 and merges the two side HashMaps
// (data: HashMap<K,V> and freq_map: HashMap<K,u8>) into a single
// resident_map: HashMap<K,(V,u8)>. This halves the number of hash probes on
// the get() hot path: one probe reads the value AND updates the frequency
// in a single operation, consolidating hot data onto fewer cache lines.
//
// All other v003 invariants are preserved:
//   1. small_size = capacity/40 — even smaller admission queue.
//   2. O(1) ghost membership + lazy deletion — ghost_set HashSet.
//   3. MAX_FREQ = 5 (raised from 3 to extend second-chance window for hot entries).
//   4. evict_main() unbounded second-chance loop.
//   5. evict_small() freq>1 promotion threshold.
const MAX_FREQ: u8 = 5;

/// Entry stores only the key; value and frequency are kept in resident_map.
/// The constructor still accepts a value argument (ignored) so white-box test
/// call-sites compile unchanged.
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
    /// Combined side map: resident key → (value, frequency).
    /// Merges the two separate data and freq_map lookups into one O(1) probe.
    resident_map: HashMap<K, (V, u8)>,
    /// Ghost deque of recently evicted keys; may contain stale entries after
    /// lazy deletion.
    ghost: VecDeque<K>,
    /// Set of keys currently live in the ghost deque (O(1) membership test).
    /// ghost_set.len() <= ghost.len() always holds.
    ghost_set: HashSet<K>,
    small_size: usize,
    main_size: usize,
}

impl<K: PartialEq + Eq + Hash + Clone, V> S3Fifo<K, V> {
    pub fn try_new(capacity: usize) -> Option<Self> {
        if capacity == 0 {
            log::error!("S3Fifo.try_new: capacity must be > 0");
            return None;
        }
        // H02: use 1/40 instead of 1/20 for the small queue.
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 40).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            resident_map: HashMap::with_capacity(capacity),
            ghost: VecDeque::with_capacity(main_size),
            ghost_set: HashSet::with_capacity(main_size),
            small_size,
            main_size,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// Look up a key and increment its frequency. O(1) via a single probe
    /// into resident_map — reads value and updates frequency in one operation.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        if let Some(entry) = self.resident_map.get_mut(key) {
            entry.1 = entry.1.saturating_add(1).min(MAX_FREQ);
            Some(&entry.0)
        } else {
            None
        }
    }

    pub fn put(&mut self, key: K, value: V) {
        // O(1) update path: key already resident — update value in resident_map.
        if let Some(entry) = self.resident_map.get_mut(&key) {
            entry.0 = value;
            return;
        }

        // Ghost hit: O(1) via ghost_set; lazy deletion — stale key stays in
        // the ghost deque and is cleaned up by the capacity guard in evict_small.
        if self.ghost_set.contains(&key) {
            self.ghost_set.remove(&key);
            if self.main.len() >= self.main_size {
                self.evict_main();
            }
            // Insert (value, freq=0) into resident_map, push key-only Entry into main.
            self.resident_map.insert(key.clone(), (value, 0));
            self.main.push_front(Entry { key, _phantom: std::marker::PhantomData });
            return;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        // Insert (value, freq=0) into resident_map, push key-only Entry into small.
        self.resident_map.insert(key.clone(), (value, 0));
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: second-chance via frequency decrement. O(1) per step.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.resident_map.get(&tail.key).map_or(0, |e| e.1);
            if freq > 0 {
                // Give this entry a second chance: decrement freq, re-admit at front.
                if let Some(e) = self.resident_map.get_mut(&tail.key) {
                    e.1 -= 1;
                }
                self.main.push_front(tail);
            } else {
                // freq == 0: evict for real.
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
                // Hot entry: promote to main; entry stays in resident_map.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // Cold entry: move key to ghost; remove from resident_map.
                self.resident_map.remove(&tail.key);
                // Capacity guard with lazy deletion:
                //   Pop from the back of the ghost deque while it is at capacity.
                //   If the popped key is still in ghost_set it is a live ghost —
                //   remove it from ghost_set (normal aged eviction) and break.
                //   If not, it was already lazily deleted; continue popping.
                while self.ghost.len() >= self.main_size {
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
    /// Updated for resident_map layout: (value, freq) tuples stored together.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::<u64, u64>::new(5);
        // Push entries directly into main and resident_map.
        cache.main.push_back(Entry::new(1, 10)); cache.resident_map.insert(1u64, (10u64, 0u8));
        cache.main.push_back(Entry::new(2, 20)); cache.resident_map.insert(2u64, (20u64, 2u8));
        cache.main.push_back(Entry::new(3, 30)); cache.resident_map.insert(3u64, (30u64, 0u8));
        cache.main.push_back(Entry::new(4, 40)); cache.resident_map.insert(4u64, (40u64, 1u8));
        // Seed small with a hot entry so evict_small triggers evict_main.
        cache.small.push_front(Entry::new(5, 50)); cache.resident_map.insert(5u64, (50u64, 2u8));
        cache.evict_small();
        // Entry 4 had freq=1 → gets a second chance (freq decremented to 0,
        //   moved to front of main), so it survives.
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
