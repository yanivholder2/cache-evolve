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
// This variant builds on cycle/0005/v001 and lowers the small-to-main promotion
// threshold from freq > 1 to freq >= 1, so that any entry re-accessed at least
// once while in small is promoted directly to main instead of going to ghost.
//
// All other v001 invariants are preserved:
//   1. small_size = capacity/40 — even smaller admission queue.
//   2. O(1) ghost membership + lazy deletion — ghost_set HashSet.
//   3. MAX_FREQ = 3.
//   4. evict_main() unbounded second-chance loop.
//   5. evict_small() freq>=1 promotion threshold (changed from freq>1).
//   6. HashMap-based data and freq_map for O(1) access.
const MAX_FREQ: u8 = 3;

/// Entry stores only the key; value and frequency are kept in side HashMaps.
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
    /// Side map: resident key → value (O(1) access, replaces Entry.value).
    data: HashMap<K, V>,
    /// Side map: resident key → frequency (O(1) access, replaces parallel freq deques).
    freq_map: HashMap<K, u8>,
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
        let small_size = if capacity == 1 {
            1
        } else {
            (capacity / 40).max(1).min(capacity - 1)
        };
        let main_size = capacity - small_size;
        Some(Self {
            small: VecDeque::with_capacity(small_size),
            main: VecDeque::with_capacity(main_size),
            data: HashMap::with_capacity(capacity),
            freq_map: HashMap::with_capacity(capacity),
            ghost: VecDeque::with_capacity(main_size),
            ghost_set: HashSet::with_capacity(main_size),
            small_size,
            main_size,
        })
    }

    pub fn new(capacity: usize) -> Self {
        Self::try_new(capacity).expect("S3Fifo::new: initialization failed")
    }

    /// Look up a key and increment its frequency. O(1) via freq_map and data.
    pub fn get(&mut self, key: &K) -> Option<&V> {
        // Update frequency first; end the mutable borrow before borrowing data.
        if let Some(freq) = self.freq_map.get_mut(key) {
            *freq = freq.saturating_add(1).min(MAX_FREQ);
        } else {
            return None;
        }
        self.data.get(key)
    }

    pub fn put(&mut self, key: K, value: V) {
        // O(1) update path: key already resident.
        if self.data.contains_key(&key) {
            self.data.insert(key, value);
            return;
        }

        // Ghost hit: O(1) via ghost_set; lazy deletion — stale key stays in
        // the ghost deque and is cleaned up by the capacity guard in evict_small.
        if self.ghost_set.contains(&key) {
            self.ghost_set.remove(&key);
            if self.main.len() >= self.main_size {
                self.evict_main();
            }
            // Insert value into data, then push key-only Entry into main.
            self.data.insert(key.clone(), value);
            self.freq_map.insert(key.clone(), 0);
            self.main.push_front(Entry { key, _phantom: std::marker::PhantomData });
            return;
        }

        // Normal admission into the small queue.
        if self.small.len() >= self.small_size {
            self.evict_small();
        }
        // Insert value into data, then push key-only Entry into small.
        self.data.insert(key.clone(), value);
        self.freq_map.insert(key.clone(), 0);
        self.small.push_front(Entry { key, _phantom: std::marker::PhantomData });
    }

    /// Evict from the main queue: second-chance via frequency decrement. O(1) per step.
    fn evict_main(&mut self) {
        while let Some(tail) = self.main.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq > 0 {
                // Give this entry a second chance: decrement freq, re-admit at front.
                self.freq_map.insert(tail.key.clone(), freq - 1);
                self.main.push_front(tail);
            } else {
                // freq == 0: evict for real.
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
                break;
            }
        }
    }

    /// Evict from the small queue: promote to main if freq >= 1, else ghost.
    fn evict_small(&mut self) {
        if let Some(tail) = self.small.pop_back() {
            let freq = self.freq_map.get(&tail.key).copied().unwrap_or(0);
            if freq >= 1 {
                // Warm entry: any re-access while in small is sufficient to
                // promote directly to main; entry stays in data and freq_map.
                if self.main.len() >= self.main_size {
                    self.evict_main();
                }
                self.main.push_front(tail);
            } else {
                // freq == 0: no re-access at all — move key to ghost; remove
                // from data and freq_map.  Ghost now acts as a filter for true
                // one-timers (zero demand while in small).
                self.data.remove(&tail.key);
                self.freq_map.remove(&tail.key);
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

    /// With freq>=1 threshold, a single re-access is enough to promote to main.
    #[test]
    fn small_eviction_promotes_to_main_after_single_hit() {
        // capacity=4 → small_size=1 (capacity/40 = 0, max(0,1)=1), main_size=3
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10);
        cache.get(&1); // freq=1 → should promote (freq>=1)
        cache.put(2, 20); // triggers evict_small for key 1
        assert!(
            cache.main.iter().any(|entry| entry.key == 1),
            "key 1 with freq=1 should be promoted to main under freq>=1 threshold"
        );
    }

    /// With freq>=1, an entry with freq=0 still goes to ghost.
    #[test]
    fn small_eviction_sends_zero_freq_to_ghost() {
        // capacity=4 → small_size=1, main_size=3
        let mut cache = S3Fifo::new(4);
        cache.put(1, 10); // freq=0, no gets
        cache.put(2, 20); // triggers evict_small for key 1 → should ghost
        assert!(
            cache.ghost_set.contains(&1),
            "key 1 with freq=0 should go to ghost"
        );
        assert!(
            !cache.main.iter().any(|entry| entry.key == 1),
            "key 1 with freq=0 should not be in main"
        );
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
    /// Updated for HashMap-based layout: freq values and data are pushed into
    /// freq_map and data alongside their entries in small/main.
    #[test]
    fn main_eviction_uses_frequency_second_chance() {
        let mut cache = S3Fifo::<u64, u64>::new(5);
        // Push entries directly into main, data, and freq_map.
        cache.main.push_back(Entry::new(1, 10)); cache.data.insert(1u64, 10u64); cache.freq_map.insert(1u64, 0u8);
        cache.main.push_back(Entry::new(2, 20)); cache.data.insert(2u64, 20u64); cache.freq_map.insert(2u64, 2u8);
        cache.main.push_back(Entry::new(3, 30)); cache.data.insert(3u64, 30u64); cache.freq_map.insert(3u64, 0u8);
        cache.main.push_back(Entry::new(4, 40)); cache.data.insert(4u64, 40u64); cache.freq_map.insert(4u64, 1u8);
        // Seed small with a hot entry so evict_small triggers evict_main.
        // freq=2 >= 1 so this entry will be promoted to main.
        cache.small.push_front(Entry::new(5, 50)); cache.data.insert(5u64, 50u64); cache.freq_map.insert(5u64, 2u8);
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
        // Fill ghost with entries 1, 2, 3 by pushing 4 entries through small
        // without any gets (freq=0 → goes to ghost under freq>=1 threshold too).
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
